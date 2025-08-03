use shared::units::angle::AngleAny;

pub trait AngleReader {
    type Error: core::fmt::Debug;
    async fn read_angle(&mut self) -> Result<AngleAny, Self::Error>;
}
