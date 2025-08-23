use foc::MotorSnapshot;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Echo,
    GetState,
    SetControlCommand(foc::state::ControlCommand),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandResult {
    Echo,
    State(MotorSnapshot),
    Ok,
}
