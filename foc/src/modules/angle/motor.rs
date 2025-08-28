use crate::Motor;
use crate::angle::calibration_accumulator::CalibrationAccumulator;
use crate::state::ShaftCalibrationState::{MeasuringFast, MeasuringSlow};
use crate::state::MotorState::Powered;
use crate::state::Powered::ShaftCalibration;
use crate::state::{MotorState, ShaftCalibrationConstants, ShaftData};
use embassy_time::Duration;
use embassy_time::Instant;
use fixed::types::U1F15;
use hardware_abstraction::angle_sensor::AngleReader;
use shared::units::angle::{AngleAny, Electrical};
use shared::units::{Angle, Velocity};
use shared::{error, info};
use shared::units::low_pass_filter::LowPassFilter;

impl Motor {
    pub async fn update_angle_task<R: AngleReader>(
        &self,
        angle_reader: &mut R,
    ) -> Result<(), R::Error> {
        loop {
            if self.is_calibrating_shaft().await {
                self.calibrate_shaft(angle_reader).await?;
            }

            let angle = angle_reader.read_angle().await?;
            self.store_shaft_data(angle).await;
        }
    }

    async fn calibrate_shaft<R: AngleReader>(&self, angle_reader: &mut R) -> Result<(), R::Error> {
        let mut last_cmd: Option<Angle<Electrical>> = None;

        let mut accumulator_slow = CalibrationAccumulator::<16>::new();
        let mut accumulator_fast = CalibrationAccumulator::<16>::new();

        loop {
            let current_angle = angle_reader.read_angle().await?;
            self.store_shaft_data(current_angle.clone()).await;

            let current_mech = match current_angle {
                AngleAny::Mechanical(value) => value,
                AngleAny::Electrical(_) => return Ok(()),
            };

            if let Powered(ShaftCalibration(MeasuringSlow(current_cmd, _))) = self.get_state().await
            {
                if Some(current_cmd) == last_cmd {
                    embassy_futures::yield_now().await;
                    continue;
                }

                if let Some(last_cmd) = last_cmd {
                    accumulator_slow.add_sample(&last_cmd, &current_mech);
                }

                last_cmd = Some(current_cmd);
            } else if let Powered(ShaftCalibration(MeasuringFast(current_cmd, _))) =
                self.get_state().await
            {
                if Some(current_cmd) == last_cmd {
                    embassy_futures::yield_now().await;
                    continue;
                }

                if let Some(last_cmd) = last_cmd {
                    accumulator_fast.add_sample(&last_cmd, &current_mech);
                }

                last_cmd = Some(current_cmd);
            } else {
                let old_shaft_data = {
                    let shaft_data_guard = self.shaft.lock().await;
                    match &*shaft_data_guard {
                        Some(data) => data.clone(),
                        None => return Ok(()),
                    }
                };

                let result_slow = accumulator_slow.finalize();
                let result_fast = accumulator_fast.finalize();

                info!("Fast calibration offset: {}", result_fast.offset);
                info!("Slow calibration offset: {}", result_slow.offset);
                info!("Fast coherence {}", result_fast.coherence.to_num::<f32>());
                info!("Slow coherence {}", result_slow.coherence.to_num::<f32>());

                let offset_delta = result_fast.offset.checked_sub(&result_slow.offset);

                let offset_delta = if let Some(value) = offset_delta {
                    value
                } else {
                    error!("Failed to calculate latency, try again");
                    return Ok(());
                };

                // TODO have single constant for state machine and motor
                const SPEED_SLOW: u64 = 128 * 2; // 256 of 'raws' per millisecond
                const SPEED_FAST: u64 = 512 * 2; // 1024 of 'raws' per millisecond

                const SPEED_DELTA: u64 = SPEED_FAST - SPEED_SLOW;
                let latency_micro = offset_delta.raw() as u64 * 1000 / SPEED_DELTA;
                let latency = Duration::from_micros(latency_micro);

                let error_slow = Angle::from_raw((latency_micro * SPEED_SLOW / 1000) as u16);

                let offset = result_slow.offset.checked_sub(&error_slow);

                let offset = if let Some(value) = offset {
                    value
                } else {
                    error!("Failed to calculate offset, try again");
                    return Ok(());
                };

                let shaft_calibration = ShaftCalibrationConstants {
                    pole_pairs: result_slow.pole_pairs,
                    offset,
                    measurement_delay: latency,
                };

                info!("New shaft calibration: {}", shaft_calibration);

                {
                    let mut shaft_data_guard = self.shaft.lock().await;
                    *shaft_data_guard = Some(ShaftData {
                        shaft_calibration,
                        ..old_shaft_data
                    });
                }

                return Ok(());
            }
        }
    }

    async fn store_shaft_data(&self, angle: AngleAny) {
        let mut shaft_data_guard = self.shaft.lock().await;

        let calibration = shaft_data_guard
            .map(|shaft| shaft.shaft_calibration)
            .unwrap_or_default();

        // TODO now before or after reading?
        let measure_time = Instant::now();
        let electrical_angle = match angle {
            AngleAny::Electrical(value) => value,
            AngleAny::Mechanical(value) => {
                value.to_electrical(&calibration.offset, calibration.pole_pairs)
            }
        };
        let speed = match shaft_data_guard.as_ref() {
            None => Velocity::<Electrical>::ZERO,
            Some(old_data) => {
                let old_speed = old_data.filtered_velocity;
                let angle_delta = electrical_angle.overflowing_sub(&old_data.electrical_angle);
                let time_delta = measure_time - old_data.measure_time;
                let speed = angle_delta / time_delta;
                old_speed.low_pass_filter(speed, U1F15::lit("0.01"))
            }
        };

        *shaft_data_guard = Some(ShaftData {
            angle,
            electrical_angle,
            measure_time,
            shaft_calibration: calibration,
            filtered_velocity: speed,
        });
    }

    async fn get_state(&self) -> MotorState {
        self.state.lock().await.state
    }

    async fn is_calibrating_shaft(&self) -> bool {
        matches!(
            self.get_state().await,
            Powered(ShaftCalibration(MeasuringSlow(_, _)))
        )
    }
}
