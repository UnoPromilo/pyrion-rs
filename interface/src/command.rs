use defmt::Format;
use foc::MotorSnapshot;

#[derive(Format)]
pub enum Command {
    Echo,
    GetState,
    SetControlCommand(foc::state::ControlCommand),
}

#[derive(Format)]
pub enum CommandResult {
    Echo,
    State(MotorSnapshot),
    Ok,
}
