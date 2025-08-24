use crate::Motor;
use crate::state::MeasurementState::Direction;
use crate::state::MotorState::Powered;
use crate::state::Powered::Measuring;
use crate::state::{EncoderCalibrationConstants, MotorState, ShaftData};
use embassy_time::Instant;
use hardware_abstraction::angle_sensor::AngleReader;
use shared::info;
use shared::units::Angle;
use shared::units::angle::{AngleAny, Electrical, Mechanical};

impl Motor {
    pub async fn update_angle<R: AngleReader>(&self, angle_reader: &mut R) -> Result<(), R::Error> {
        if self.is_measuring_direction().await {
            self.start_direction_calibration(angle_reader).await?;
        }

        let angle = angle_reader.read_angle().await?;
        self.store_shaft_date(angle).await;
        Ok(())
    }

    async fn start_direction_calibration<R: AngleReader>(
        &self,
        angle_reader: &mut R,
    ) -> Result<(), R::Error> {
        let mut last_mech: Option<Angle<Mechanical>> = None;
        let mut last_cmd: Option<Angle<Electrical>> = None;
        let mut correlation: i32 = 0;

        loop {
            let current_angle = angle_reader.read_angle().await?;
            self.store_shaft_date(current_angle.clone()).await;

            let current_mech = match current_angle {
                AngleAny::Mechanical(value) => value,
                AngleAny::Electrical(_) => return Ok(()),
            };

            if let Powered(Measuring(Direction(current_cmd))) = self.get_state().await {
                if Some(current_cmd) == last_cmd {
                    embassy_futures::yield_now().await;
                    continue;
                }

                if let (Some(last_mech), Some(last_cmd)) = (&last_mech, &last_cmd) {
                    let cmd_dir = last_cmd.get_direction(&current_cmd);
                    let mech_dir = last_mech.get_direction(&current_mech);
                    correlation += if mech_dir == cmd_dir { 1 } else { -1 };
                }

                last_mech = Some(current_mech);
                last_cmd = Some(current_cmd);
            } else {
                let mut shaft_data_guard = self.shaft.lock().await;

                let old_shaft_data = match &*shaft_data_guard {
                    Some(data) => data.clone(),
                    None => return Ok(()),
                };

                let encoder_calibration = EncoderCalibrationConstants {
                    direction: if correlation > 1 {
                        shared::units::Direction::Clockwise
                    } else {
                        shared::units::Direction::CounterClockwise
                    },
                    ..old_shaft_data.encoder_calibration
                };

                info!("New encoder calibration: {}", encoder_calibration);

                *shaft_data_guard = Some(ShaftData {
                    encoder_calibration,
                    ..old_shaft_data
                });

                return Ok(());
            }
        }
    }

    async fn store_shaft_date(&self, angle: AngleAny) {
        let mut shaft_data_guard = self.shaft.lock().await;

        let calibration = shaft_data_guard
            .map(|shaft| shaft.encoder_calibration)
            .unwrap_or_default();

        // TODO now before or after reading?
        let measure_time = Instant::now();
        let electrical_angle = match angle {
            AngleAny::Electrical(value) => value,
            AngleAny::Mechanical(value) => {
                let value = if calibration.direction == shared::units::Direction::Clockwise {
                    value.inverted()
                } else {
                    value
                };
                Angle::<Electrical>::from(&value, calibration.offset, calibration.pole_pairs)
            }
        };

        *shaft_data_guard = Some(ShaftData {
            angle,
            electrical_angle,
            measure_time,
            encoder_calibration: calibration,
        });
    }

    async fn get_state(&self) -> MotorState {
        self.state.lock().await.state
    }

    async fn is_measuring_direction(&self) -> bool {
        matches!(self.get_state().await, Powered(Measuring(Direction(_))))
    }
}
