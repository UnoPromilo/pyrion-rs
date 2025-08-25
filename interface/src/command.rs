use foc::MotorSnapshot;
use foc::state::ShaftData;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Echo,
    GetState,
    GetShaft,
    SetControlCommand(foc::state::ControlCommand),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandResult {
    Echo,
    State(MotorSnapshot),
    Shaft(Option<ShaftData>),
    Ok,
}
