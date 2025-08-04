use defmt::Format;
use foc::MotorFrozen;

#[derive(Format)]
pub enum Command {
    Echo,
    GetState,
}

#[derive(Format)]
pub enum CommandResult {
    Echo,
    State(MotorFrozen),
}
