use crate::modules::models::ElectricalAngle;
use crate::modules::{clarke_transformation, park_transformation};
use defmt::Format;
use embassy_time::{Duration, Ticker};
use hardware_abstraction::angle_sensor::AngleSensor;
use hardware_abstraction::models::{Angle, Direction};
use hardware_abstraction::motor_driver::MotorDriver;

const U_CALIBRATION: i16 = i16::MAX / 4;
const MINIMAL_MOVEMENT: u16 = 360 / 10; // It will work for any motor that has at least 9 pole pairs TODO verify by changing to 7, 6 and 8

pub struct Shaft<TAngleSensor> {
    sensor: TAngleSensor,
    config: Option<Config>,
}

#[derive(Debug, Copy, Clone)]
struct Config {
    offset: u16,
    pole_pairs: u8,
    natural_direction: NaturalDirection,
}

impl<TAngleSensor: AngleSensor> Shaft<TAngleSensor> {
    pub fn new(sensor: TAngleSensor) -> Self {
        Self {
            sensor,
            config: None,
        }
    }
}

impl<TAngleSensor: AngleSensor<Error = TAngleSensorError>, TAngleSensorError: Format>
    Shaft<TAngleSensor>
{
    pub async fn init(
        &mut self,
        driver: &mut impl MotorDriver,
    ) -> Result<(), Error<TAngleSensorError>> {
        let natural_direction = self.find_natural_direction(driver).await?;
        self.config = Some(Config {
            pole_pairs: 7, // TODO calculate
            offset: 0,     // TODO calculate
            natural_direction,
        });
        Ok(())
    }

    pub async fn read_angle_async(
        &mut self,
    ) -> Result<(Angle, ElectricalAngle), Error<TAngleSensorError>> {
        let config = &self.config.ok_or_else(|| Error::NotInitialized)?;
        let angle = self.read_raw_angle().await?;
        let electrical_angle =
            ElectricalAngle::from_angle(&angle, config.offset, config.pole_pairs);
        Ok((angle, electrical_angle))
    }

    async fn find_natural_direction(
        &mut self,
        driver: &mut impl MotorDriver,
    ) -> Result<NaturalDirection, Error<TAngleSensorError>> {
        // Move backward 1 electrical revolution:
        Self::move_one_rotation(driver, true).await;
        let middle_angle = self.read_raw_angle().await?;
        // Move forward 1 electrical revolution:
        Self::move_one_rotation(driver, false).await;
        let end_angle = self.read_raw_angle().await?;
        driver.set_voltages(0, 0, 0);
        let delta = middle_angle.get_abs(&end_angle);
        Self::disable_motor(driver);

        if delta.to_degrees() < MINIMAL_MOVEMENT {
            return Err(Error::NoMovement);
        }

        match middle_angle.get_direction(&end_angle) {
            None => Err(Error::NoMovement),
            Some(Direction::CounterClockwise) => Ok(NaturalDirection::CounterClockwise),
            Some(Direction::Clockwise) => Ok(NaturalDirection::Clockwise),
        }
    }

    async fn move_one_rotation(driver: &mut impl MotorDriver, backward: bool) {
        let mut ticker = Ticker::every(Duration::from_millis(2));
        const STEPS: u16 = u8::MAX as u16;
        for index in 0..=STEPS {
            let angle = ElectricalAngle(match backward {
                true => (STEPS - index) << 8,
                false => index << 8,
            });
            let (alpha, beta) = park_transformation::inverse(U_CALIBRATION, 0, &angle);
            let (a, b, c) = clarke_transformation::inverse(alpha, beta);
            driver.set_voltages(a, b, c);
            ticker.next().await;
        }
    }

    async fn read_raw_angle(&mut self) -> Result<Angle, Error<TAngleSensorError>> {
        self.sensor
            .read_angle_u16()
            .await
            .map_err(Error::AngleSensorError)
    }

    fn disable_motor(driver: &mut impl MotorDriver) {
        driver.set_voltages(0, 0, 0);
    }
}

#[derive(Debug, Format)]
pub enum Error<TAngleSensorError: Format> {
    NotInitialized,
    NoMovement,
    AngleSensorError(TAngleSensorError),
}

#[derive(Debug, Copy, Clone)]
enum NaturalDirection {
    Clockwise,
    CounterClockwise,
}
