use crate::Motor;
use crate::current::calibration_accumulator::CalibrationAccumulator;
use crate::state::InitializationState::CalibratingCurrentSensor;
use crate::state::MotorState::Initializing;
use crate::state::PhaseCurrent;
use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::CurrentReader;
use shared::{debug, warn};

impl Motor {
    pub async fn update_current_task<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error> {
        loop {
            if self.is_calibrating().await {
                let mut calibration_accumulator = CalibrationAccumulator::default();
                let mut sample_count = 0u32;

                loop {
                    let state = { self.state.lock().await.state };
                    match state {
                        Initializing(CalibratingCurrentSensor(cal_state)) => {
                            let raw_output = current_reader.wait_for_next_raw().await?;
                            calibration_accumulator.update(cal_state, raw_output);
                            sample_count += 1;
                        }
                        _ => break,
                    }
                }
                debug!(
                    "Current sensor calibration was based on {} samples",
                    sample_count,
                );
                let (a, b, c) = calibration_accumulator.finalize();
                current_reader.calibrate_current(a, b, c).await;
            }

            let output = current_reader.wait_for_next().await?;
            let phase_current = PhaseCurrent::from_output(output);

            if let Ok(mut current_mutex) = self.current.try_lock() {
                *current_mutex = Some(phase_current);
            } else {
                warn!("Skipping the Current update because motor is busy")
            }
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
            //TODO add logic about calculating third Current if low duty cycle to improve accuracy
            current_sensor::Output::ThreePhases(a, b, c) => Self { a, b, c },
        }
    }
}
