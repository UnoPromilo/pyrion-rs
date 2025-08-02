use defmt::{Format, info};
use hardware_abstraction::angle_sensor::AngleSensor;
use hardware_abstraction::motor_driver::MotorDriver;
use shared::units::Angle;
use shared::units::angle::{Electrical, Mechanical};

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
        todo!()
    }

    #[allow(dead_code)]
    pub fn init_with_config(&mut self, config: Config) {
        self.config = Some(config);
        info!("Shaft initialized manually with config: {:#?}", self.config);
    }

    pub async fn read_angle_async(
        &mut self,
    ) -> Result<(Angle<Mechanical>, Angle<Electrical>), Error<TAngleSensorError>> {
        todo!()
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
