use core::sync::atomic::Ordering;
use logging::info;
use transport::{Command, Event};
use transport::event::DeviceIntroduction;

pub async fn execute_command(command: Command) -> Option<Event> {
    info!("Command received: {:?}", command);
    match command {
        Command::IntroduceYourself => {
            let state = controller_shared::state::state();
            let version_major = state.version.major.load(Ordering::Relaxed);
            let version_minor = state.version.minor.load(Ordering::Relaxed);
            let version_patch = state.version.patch.load(Ordering::Relaxed);
            Some(Event::DeviceIntroduction(DeviceIntroduction {
                uid: *embassy_stm32::uid::uid(),
                firmware_version: [version_major, version_minor, version_patch],
            }))
        }
        Command::Stop => {
            // TODO stop
            None
        }
    }
}
