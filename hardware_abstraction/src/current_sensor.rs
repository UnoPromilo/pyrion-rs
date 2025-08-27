use shared::units::Current;

pub enum Output {
    TwoPhases(Current, Current),
    ThreePhases(Current, Current, Current),
}

pub enum RawOutput {
    TwoPhases(u16, u16),
    ThreePhases(u16, u16, u16),
}

pub trait CurrentReader {
    type Error: core::fmt::Debug;

    async fn wait_for_next(&mut self) -> Result<Output, Self::Error>;

    async fn wait_for_next_raw(&mut self) -> Result<RawOutput, Self::Error>;

    async fn calibrate_current(&mut self, zero_a: u16, zero_b: u16, zero_c: u16);
}
