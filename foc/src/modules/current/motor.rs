use crate::Motor;
use crate::state::InitializationState::CalibratingCurrentSensor;
use crate::state::MotorState::Initializing;
use crate::state::PhaseCurrent;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Instant;
use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::CurrentReader;
use shared::debug;
use crate::current::calibration_accumulator::CalibrationAccumulator;

impl Motor {
    pub async fn update_current_task<R: CurrentReader, M: RawMutex>(
        &self,
        pwm_wrap_signal: &Signal<M, Instant>,
        current_reader: &mut R,
    ) -> Result<(), R::Error> {
        loop {
            let time_of_wrap = pwm_wrap_signal.wait().await;
            if (time_of_wrap.elapsed().as_ticks() as u32) > 10 {
                // 10 ticks is too late for accurate measurement, it is better to skip it.
                continue;
            }
            if self.is_calibrating().await {
                let mut calibration_accumulator = CalibrationAccumulator::default();
                let mut sample_count = 0u32;

                loop {
                    let state = { self.state.lock().await.state };
                    match state {
                        Initializing(CalibratingCurrentSensor(cal_state)) => {
                            let raw_output = current_reader.read_raw().await?;
                            calibration_accumulator.update(cal_state, raw_output);
                            sample_count += 1;
                        }
                        _ => break,
                    }
                }
                debug!(
                    "Current sensor calibration was based on {} samples",
                    sample_count
                );
                let (a, b, c) = calibration_accumulator.finalize();
                current_reader.calibrate_current(a, b, c).await;
            }

            let output = current_reader.read().await?;
            let phase_current = PhaseCurrent::from_output(output);

            *self.current.lock().await = Some(phase_current);
        }
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
