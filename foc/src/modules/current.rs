use crate::Motor;
use crate::state::{CalibratingCurrentSensorState, InitializationState, MotorState, PhaseCurrent};
use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::{CurrentReader, RawOutput};

pub trait CurrentMeasurement {
    async fn update_current<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error>;
}

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

impl CurrentMeasurement for Motor {
    async fn update_current<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error> {
        let state = { self.state.lock().await.state };

        if let MotorState::Initializing(InitializationState::CalibratingCurrentSensor(_)) = state {
            let mut calibration_accumulator = CalibrationAccumulator::default();
            while let MotorState::Initializing(InitializationState::CalibratingCurrentSensor(
                cal_state,
            )) = state
            {
                let raw_output = current_reader.read_raw().await?;
                calibration_accumulator.update(cal_state, raw_output);
                // Let's run other tasks between readings
                embassy_futures::yield_now().await;
            }
            let calibration_values = calibration_accumulator.finalize();
            current_reader
                .calibrate_current(
                    calibration_values.0,
                    calibration_values.1,
                    calibration_values.2,
                )
                .await;
        }

        let result = current_reader.read().await?;
        let phase_current = match result {
            current_sensor::Output::TwoPhases(a, b) => {
                let c = -a - b;
                PhaseCurrent { a, b, c }
            }
            current_sensor::Output::ThreePhases(a, b, c) => {
                //TODO add logic about calculating third current if low duty cycle
                PhaseCurrent { a, b, c }
            }
        };
        {
            let mut power = self.current.lock().await;
            *power = Some(phase_current);
        }
        Ok(())
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
