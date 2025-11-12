use core::sync::atomic::Ordering;
use logging::info;
use transport::event::DeviceIntroduction;
use transport::{Command, Event};
use update_manager::FirmwareUpdateManager;

pub async fn execute_command(
    command: Command,
    update_manager: &mut FirmwareUpdateManager<'_>,
) -> Option<Event> {
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
        Command::WriteFirmwareBlock(block) => {
            match update_manager
                .write_block(block.slice(), block.offset as usize)
                .await
            {
                Ok(()) => Some(Event::Success),
                Err(_) => Some(Event::Failure),
            }
        }
        Command::FinalizeFirmwareUpdate => match update_manager.finish_update().await {
            // OK is never triggered, because a successful firmware update results in a device reset.
            Ok(()) => Some(Event::Success),
            Err(_) => Some(Event::Failure),
        },
    }
}
