use shared::units::Angle;
use shared::units::angle::AngleType;

pub trait AngleSensor
where
    Self::AngleType: AngleType,
{
    type Error;
    type AngleType;

    async fn read_angle_u16(&mut self) -> Result<Angle<Self::AngleType>, Self::Error>;
}
