use crate::state::ShaftData;
use crate::Motor;
use embassy_time::Instant;
use hardware_abstraction::angle_sensor::AngleReader;
use shared::units::angle::{AngleAny, Electrical};
use shared::units::Angle;

pub trait AngleMeasurement {
    async fn update_angle<R: AngleReader>(&self, angle_reader: &mut R) -> Result<(), R::Error>;
}

impl AngleMeasurement for Motor {
    async fn update_angle<R: AngleReader>(&self, angle_reader: &mut R) -> Result<(), R::Error> {
        let angle = angle_reader.read_angle().await?;

        // TODO now before or after reading?
        let measure_time = Instant::now();
        let electrical_angle = match angle {
            AngleAny::Electrical(value) => value,
            // TODO add calibration data
            AngleAny::Mechanical(value) => Angle::<Electrical>::from(&value, 0, 1),
        };

        let mut shaft_data = self.shaft.lock().await;
        *shaft_data = Some(ShaftData {
            angle,
            electrical_angle,
            measure_time,
        });
        Ok(())
    }
}
