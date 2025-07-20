use crate::modules::clarke_transformation;
use crate::modules::park_transformation;
use crate::modules::shaft;
use crate::modules::shaft::Shaft;
use defmt::Format;
use hardware_abstraction::angle_sensor::AngleSensor;
use hardware_abstraction::motor_driver::MotorDriver;

pub struct Controller<TAngleSensor, TAngleSensorError, TMotorDriver>
where
    TAngleSensor: AngleSensor<Error = TAngleSensorError>,
    TMotorDriver: MotorDriver,
{
    shaft: Shaft<TAngleSensor>,
    motor_driver: TMotorDriver,
}

impl<TAngleSensor, TAngleSensorError, TMotorDriver>
    Controller<TAngleSensor, TAngleSensorError, TMotorDriver>
where
    TAngleSensor: AngleSensor<Error = TAngleSensorError>,
    TMotorDriver: MotorDriver,
    TAngleSensorError: Format,
{
    pub fn new(angle_sensor: TAngleSensor, motor_driver: TMotorDriver) -> Self {
        let shaft = Shaft::new(angle_sensor);
        Self {
            shaft,
            motor_driver,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error<TAngleSensorError>> {
        let result = self.run_until_error().await;
        self.disable();
        result
    }

    async fn run_until_error(&mut self) -> Result<(), Error<TAngleSensorError>> {
        self.init().await?;

        loop {
            self.on_tick().await?;
        }
    }

    async fn init(&mut self) -> Result<(), Error<TAngleSensorError>> {
        self.motor_driver.enable();
        self.shaft.init(&mut self.motor_driver).await?;
        Ok(())
    }

    async fn on_tick(&mut self) -> Result<(), Error<TAngleSensorError>> {
        let (angle, electrical_angle) = self.shaft.read_angle_async().await?;

        // TODO replace i_d and i_q with calculated value
        let (alpha, beta) = park_transformation::inverse(i16::MAX, 0, &electrical_angle);
        let (a, b, c) = clarke_transformation::inverse(alpha, beta);

        // TODO convert current into voltage with current loop

        self.motor_driver.set_voltages(a, b, c);
        Ok(())
    }

    fn disable(&mut self) {
        self.motor_driver.disable();
    }
}

#[derive(Debug, Format)]
pub enum Error<TAngleSensorError: Format> {
    ShaftError(shaft::Error<TAngleSensorError>),
}

impl<TAngleSensorError: Format> From<shaft::Error<TAngleSensorError>> for Error<TAngleSensorError> {
    fn from(value: shaft::Error<TAngleSensorError>) -> Self {
        Self::ShaftError(value)
    }
}
