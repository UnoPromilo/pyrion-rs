use crate::helpers::{clarke_transformation, park_transformation};
use crate::motor_controller::Config;
use crate::motor_controller::units::ElectricalAngle;
use hardware_abstraction::angle_sensor::AngleSensor;
use hardware_abstraction::motor_driver::MotorDriver;

pub struct MotorController<'a, TAngleSensor, TAngleSensorError, TMotorDriver>
where
    TAngleSensor: AngleSensor<Error = TAngleSensorError> + 'a,
    TMotorDriver: MotorDriver + 'a,
{
    angle_sensor: &'a mut TAngleSensor,
    motor_driver: &'a mut TMotorDriver,
    config: Config,
}

impl<'a, TAngleSensor, TAngleSensorError, TMotorDriver>
    MotorController<'a, TAngleSensor, TAngleSensorError, TMotorDriver>
where
    TAngleSensor: AngleSensor<Error = TAngleSensorError> + 'a,
    TMotorDriver: MotorDriver + 'a,
{
    pub async fn run(&mut self) -> Result<(), Error<TAngleSensorError>> {
        loop {
            self.task().await?;
        }
    }

    async fn task(&mut self) -> Result<(), Error<TAngleSensorError>> {
        // take fresh angle reading
        let angle = self
            .angle_sensor
            .read_angle_u16()
            .await
            .map_err(Error::AngleSensorError)?;

        // convert to electrical angle
        let electrical_angle =
            ElectricalAngle::from_angle(&angle, self.config.angle_offset, self.config.pole_pairs);

        // TODO replace i_d and i_q with calculated value
        let (alpha, beta) = park_transformation::inverse(i16::MAX, 0, &electrical_angle);
        let (a, b, c) = clarke_transformation::inverse(alpha, beta);

        // TODO convert current into required pwm signals
        // TODO drive motor

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error<TAngleSensorError> {
    AngleSensorError(TAngleSensorError),
}
