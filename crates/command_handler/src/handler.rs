use controller_shared::command::{ControlCommand, ControlCommandChannel};
use core::sync::atomic::Ordering;
use logging::info;
use transport::event::DeviceIntroduction;
use transport::{Command, Event};

// TODO decide if should return Option<Event> or just Event
pub async fn execute_command(
    command: Command,
    control_command_channel: &ControlCommandChannel,
) -> Option<Event> {
    info!("Command received: {:?}", command);
    match command {
        Command::IntroduceYourself => {
            let state = controller_shared::state::state();
            let version_major = state.version.major.load(Ordering::Relaxed);
            let version_minor = state.version.minor.load(Ordering::Relaxed);
            let version_patch = state.version.patch.load(Ordering::Relaxed);
            Some(Event::DeviceIntroduction(DeviceIntroduction {
                uid: embassy_stm32::uid::uid(),
                firmware_version: [version_major, version_minor, version_patch],
            }))
        }
        Command::Stop => match control_command_channel.try_send(ControlCommand::DisableMotor) {
            Ok(_) => Some(Event::Success),
            Err(_) => Some(Event::Failure),
        },
        Command::WriteFirmwareBlock(_block) => {
            Some(Event::Failure)
        }
        Command::FinalizeFirmwareUpdate => {
            Some(Event::Failure)
        }
    }
}
