use crate::modules::clarke_transformation;
use crate::modules::park_transformation;
use crate::modules::shaft;
use crate::modules::shaft::{NaturalDirection, Shaft};
use defmt::{Format, info, warn};
use embassy_time::{Instant, Ticker};
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
        if let Err(error) = &result {
            warn!("Motor controller finished with error: {}", error);
        }
        result
    }

    async fn run_until_error(&mut self) -> Result<(), Error<TAngleSensorError>> {
        self.init().await?;

        let mut last_print_time = Instant::now();
        let mut counter: u16 = 0;

        loop {
            self.on_tick().await?;
            counter+=1;
            if last_print_time.elapsed().as_secs() > 1 {
                last_print_time = Instant::now();
                info!("Main loop tick frequency: {}Hz", counter);
                counter = 0;
            }
        }
    }

    async fn init(&mut self) -> Result<(), Error<TAngleSensorError>> {
        self.shaft.init(&mut self.motor_driver).await?;
        self.motor_driver.enable();
        info!("Motor controller initialized successfully");
        Ok(())
    }

    async fn on_tick(&mut self) -> Result<(), Error<TAngleSensorError>> {
        let (angle, electrical_angle) = self.shaft.read_angle_async().await?;

        // TODO replace i_d and i_q with calculated value
        let (alpha, beta) = park_transformation::inverse( i16::MAX/4, 0, &electrical_angle);
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
