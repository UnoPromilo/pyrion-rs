use defmt::Format;
use foc::MotorSnapshot;

#[derive(Format)]
pub enum Command {
    Echo,
    GetState,
}

#[derive(Format)]
pub enum CommandResult {
    Echo,
    State(MotorSnapshot),
}
