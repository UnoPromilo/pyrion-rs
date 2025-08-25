use crate::Motor;
use crate::angle::calibration_accumulator::CalibrationAccumulator;
use crate::state::EncoderCalibrationState::Measuring;
use crate::state::MotorState::Powered;
use crate::state::Powered::EncoderCalibration;
use crate::state::{ShaftCalibrationConstants, MotorState, ShaftData};
use embassy_time::Instant;
use hardware_abstraction::angle_sensor::AngleReader;
use shared::info;
use shared::units::Angle;
use shared::units::angle::{AngleAny, Electrical};

impl Motor {
    pub async fn update_angle_task<R: AngleReader>(
        &self,
        angle_reader: &mut R,
    ) -> Result<(), R::Error> {
        loop {
            if self.is_measuring_direction().await {
                self.start_direction_calibration(angle_reader).await?;
            }

            let angle = angle_reader.read_angle().await?;
            self.store_shaft_date(angle).await;
        }
    }

    async fn start_direction_calibration<R: AngleReader>(
        &self,
        angle_reader: &mut R,
    ) -> Result<(), R::Error> {
        let mut last_cmd: Option<Angle<Electrical>> = None;

        let mut accumulator = CalibrationAccumulator::<16>::new();

        loop {
            let current_angle = angle_reader.read_angle().await?;
            self.store_shaft_date(current_angle.clone()).await;

            let current_mech = match current_angle {
                AngleAny::Mechanical(value) => value,
                AngleAny::Electrical(_) => return Ok(()),
            };

            if let Powered(EncoderCalibration(Measuring(current_cmd, _))) = self.get_state().await {
                if Some(current_cmd) == last_cmd {
                    embassy_futures::yield_now().await;
                    continue;
                }

                if let Some(last_cmd) = last_cmd {
                    accumulator.add_sample(&last_cmd, &current_mech);
                }

                last_cmd = Some(current_cmd);
            } else {
                let mut shaft_data_guard = self.shaft.lock().await;

                let old_shaft_data = match &*shaft_data_guard {
                    Some(data) => data.clone(),
                    None => return Ok(()),
                };

                let result = accumulator.finalize();

                let shaft_calibration = ShaftCalibrationConstants {
                    pole_pairs: result.pole_pairs,
                    offset: result.offset,
                };


                info!("New shaft calibration: {}", shaft_calibration);
                info!("Coherence {}", result.coherence.to_num::<f32>());

                *shaft_data_guard = Some(ShaftData {
                    shaft_calibration,
                    ..old_shaft_data
                });

                return Ok(());
            }
        }
    }

    async fn store_shaft_date(&self, angle: AngleAny) {
        let mut shaft_data_guard = self.shaft.lock().await;

        let calibration = shaft_data_guard
            .map(|shaft| shaft.shaft_calibration)
            .unwrap_or_default();

        // TODO now before or after reading?
        let measure_time = Instant::now();
        let electrical_angle = match angle {
            AngleAny::Electrical(value) => value,
            AngleAny::Mechanical(value) => {
                Angle::<Electrical>::from_mechanical(
                    &value,
                    &calibration.offset,
                    calibration.pole_pairs,
                )
            }
        };

        *shaft_data_guard = Some(ShaftData {
            angle,
            electrical_angle,
            measure_time,
            shaft_calibration: calibration,
        });
    }

    async fn get_state(&self) -> MotorState {
        self.state.lock().await.state
    }

    async fn is_measuring_direction(&self) -> bool {
        matches!(
            self.get_state().await,
            Powered(EncoderCalibration(Measuring(_, _)))
        )
    }
}
