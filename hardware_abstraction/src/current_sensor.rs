use shared::units::Current;

pub enum Output {
    TwoPhases(Current, Current),
    ThreePhases(Current, Current, Current),
}

pub trait CurrentReader {
    type Error: core::fmt::Debug;
    
    async fn read(&mut self) -> Result<Output, Self::Error>;
}
