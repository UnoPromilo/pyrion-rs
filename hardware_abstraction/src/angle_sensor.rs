pub trait AngleSensor {
    type Error;

    async fn read_angle_u16(&mut self) -> Result<u16, Self::Error>;
}
