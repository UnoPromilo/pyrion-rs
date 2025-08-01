use crate::modules::models::ElectricalAngle;
use crate::modules::{clarke_transformation, park_transformation};
use defmt::{Format, info, warn};
use embassy_time::{Duration, Ticker, Timer};
use hardware_abstraction::angle_sensor::AngleSensor;
use hardware_abstraction::models::{Angle, Direction};
use hardware_abstraction::motor_driver::MotorDriver;

const U_CALIBRATION: i16 = i16::MAX / 4;
const MINIMAL_MOVEMENT: u16 = 360 / 6 / 10; // It will work for any motor that has at least 9 pole pairs TODO verify by changing to 7, 6 and 8
const POLE_PAIR_ESTIMATION_RANGE_FACTOR: u16 = 50; // Range is ideal value +- this const * idealValue
const MINIMAL_POLE_PAIRS: u16 = 2;
const MAXIMAL_POLE_PAIRS: u16 = 15;

pub struct Shaft<TAngleSensor> {
    sensor: TAngleSensor,
    config: Option<Config>,
}

#[derive(Debug, Copy, Clone, Format)]
pub struct Config {
    pub offset: u16,
    pub pole_pairs: u8,
    pub natural_direction: NaturalDirection,
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
    #[allow(dead_code)]

    pub async fn init(
        &mut self,
        driver: &mut impl MotorDriver,
    ) -> Result<(), Error<TAngleSensorError>> {
        // Move backward 1 electrical revolution:
        driver.enable();
        let (alpha, beta) = park_transformation::inverse(0, U_CALIBRATION, &ElectricalAngle(0));
        let (a, b, c) = clarke_transformation::inverse(alpha, beta);
        driver.set_voltages(a, b, c);
        Self::move_from_to(driver, 0, u16::MAX).await;
        let end_angle = self.read_raw_angle().await?;
        Self::move_from_to(driver, 0, u16::MAX).await;
        let middle_angle = self.read_raw_angle().await?;

        Timer::after(Duration::from_millis(100)).await;
        driver.disable();

        info!("Middle_angle: {:?}", middle_angle);
        info!("end_angle: {:?}", end_angle);

        let natural_direction = Self::find_natural_direction(&middle_angle, &end_angle)?;
        let pole_pairs = Self::estimate_pole_pairs(&middle_angle, &end_angle)?;
        let offset = Self::estimate_offset(&end_angle, pole_pairs);

        self.config = Some(Config {
            pole_pairs,
            offset,
            natural_direction,
        });

        info!(
            "Shaft initialized with discovered values: {:#?}",
            self.config
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn init_with_config(&mut self, config: Config) {
        self.config = Some(config);
        info!("Shaft initialized manually with config: {:#?}", self.config);
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

    fn find_natural_direction(
        start_angle: &Angle,
        end_angle: &Angle,
    ) -> Result<NaturalDirection, Error<TAngleSensorError>> {
        let delta = start_angle.get_abs(&end_angle);

        info!("Delta: {:?}", delta.to_degrees());

        if delta.to_degrees() < MINIMAL_MOVEMENT {
            return Err(Error::NoMovement);
        }

        match start_angle.get_direction(&end_angle) {
            None => Err(Error::NoMovement),
            Some(Direction::CounterClockwise) => Ok(NaturalDirection::CounterClockwise),
            Some(Direction::Clockwise) => Ok(NaturalDirection::Clockwise),
        }
    }

    async fn move_from_to(
        driver: &mut impl MotorDriver,
        from: u16,
        to: u16,
    ) {
        let mut ticker = Ticker::every(Duration::from_hz(2056));
        const STEPS: u16 = 256;
        let diff = to as i32 - from as i32;

        for index in 0..=STEPS {
            let angle = ElectricalAngle(if diff > 0 {
                from + diff as u16 / STEPS * index
            } else {
                from - diff.abs() as u16 / STEPS * index
            });


            let (alpha, beta) = park_transformation::inverse(0, U_CALIBRATION, &angle);
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

    fn estimate_pole_pairs(
        angle_1: &Angle,
        angle_2: &Angle,
    ) -> Result<u8, Error<TAngleSensorError>> {
        let delta = angle_1.get_abs(&angle_2);
        let raw_delta = delta.get_raw();
        for pole_pair in MINIMAL_POLE_PAIRS..=MAXIMAL_POLE_PAIRS {
            let ideal_angle = u16::MAX / pole_pair;
            let lower_limit = ideal_angle - (ideal_angle / POLE_PAIR_ESTIMATION_RANGE_FACTOR);
            let upper_limit = ideal_angle + (ideal_angle / POLE_PAIR_ESTIMATION_RANGE_FACTOR);
            if (lower_limit <= raw_delta) && (upper_limit > raw_delta) {
                return Ok(pole_pair as u8);
            }
        }

        Err(Error::UnknownCountOfPolePairs)
    }

    fn estimate_offset(start_angle: &Angle, pole_pairs: u8) -> u16 {
        let period = u16::MAX / pole_pairs as u16;
        start_angle.get_raw() % period
    }
}

#[derive(Debug, Format)]
pub enum Error<TAngleSensorError: Format> {
    NotInitialized,
    NoMovement,
    UnknownCountOfPolePairs,
    AngleSensorError(TAngleSensorError),
}

#[derive(Debug, Copy, Clone, Format)]
pub enum NaturalDirection {
    Clockwise,
    CounterClockwise,
}
