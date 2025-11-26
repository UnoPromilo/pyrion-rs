use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub type ControlCommandChannel = Channel<CriticalSectionRawMutex, ControlCommand, 10>;

pub enum ControlCommand {
    DisableMotor,
}
