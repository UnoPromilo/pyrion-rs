use crate::Motor;
use crate::state::{
    CalibratingCurrentSensorState, InitializationState::CalibratingCurrentSensor,
    MotorState::Initializing, PhaseCurrent,
};

use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::{CurrentReader, RawOutput};

#[derive(Default)]
struct ChannelAccumulator {
    sum: u64,
    count: u64,
}

#[derive(Default)]
struct CalibrationAccumulator {
    a: ChannelAccumulator,
    b: ChannelAccumulator,
    c: ChannelAccumulator,
}

impl Motor {
    pub async fn update_current<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error> {
        if self.is_calibrating().await {
            let mut calibration_accumulator = CalibrationAccumulator::default();

            loop {
                let state = { self.state.lock().await.state };
                match state {
                    Initializing(CalibratingCurrentSensor(cal_state)) => {
                        let raw_output = current_reader.read_raw().await?;
                        calibration_accumulator.update(cal_state, raw_output);
                        embassy_futures::yield_now().await;
                    }
                    _ => break,
                }
            }

            let (a, b, c) = calibration_accumulator.finalize();
            current_reader.calibrate_current(a, b, c).await;
        }

        let output = current_reader.read().await?;
        let phase_current = PhaseCurrent::from_output(output);

        *self.current.lock().await = Some(phase_current);

        Ok(())
    }

    async fn is_calibrating(&self) -> bool {
        matches!(
            self.state.lock().await.state,
            Initializing(CalibratingCurrentSensor(_))
        )
    }
}

impl PhaseCurrent {
    fn from_output(output: current_sensor::Output) -> Self {
        match output {
            current_sensor::Output::TwoPhases(a, b) => Self { a, b, c: -a - b },
            //TODO add logic about calculating third current if low duty cycle to improve accuracy
            current_sensor::Output::ThreePhases(a, b, c) => Self { a, b, c },
        }
    }
}

impl CalibrationAccumulator {
    pub fn update(&mut self, state: CalibratingCurrentSensorState, raw_output: RawOutput) {
        match state {
            CalibratingCurrentSensorState::PhaseAPowered => {
                let a = match raw_output {
                    RawOutput::TwoPhases(a, _) => a,
                    RawOutput::ThreePhases(a, _, _) => a,
                };
                self.a.add(a);
            }
            CalibratingCurrentSensorState::PhaseBPowered => {
                let b = match raw_output {
                    RawOutput::TwoPhases(_, b) => b,
                    RawOutput::ThreePhases(_, b, _) => b,
                };
                self.b.add(b);
            }
            CalibratingCurrentSensorState::PhaseCPowered => {
                let c = match raw_output {
                    RawOutput::ThreePhases(_, _, c) => c,
                    _ => 0,
                };
                self.c.add(c);
            }
        }
    }

    pub fn finalize(self) -> (u16, u16, u16) {
        (self.a.average(), self.b.average(), self.c.average())
    }
}

impl ChannelAccumulator {
    fn add(&mut self, value: u16) {
        self.sum += value as u64;
        self.count += 1;
    }

    fn average(&self) -> u16 {
        if self.count > 0 {
            (self.sum / self.count) as u16
        } else {
            u16::MAX / 2
        }
    }
}
