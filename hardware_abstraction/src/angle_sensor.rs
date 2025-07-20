use crate::models::Angle;

pub trait AngleSensor {
    type Error;

    async fn read_angle_u16(&mut self) -> Result<Angle, Self::Error>;
}
