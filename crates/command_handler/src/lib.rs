#![no_std]

use core::sync::atomic::Ordering;
use logging::info;
use transport::event::{DeviceIntroduction, Telemetry};
use transport::{Command, Event};
use units::si::thermodynamic_temperature::kelvin;

pub fn get_telemetry() -> Telemetry {
    let controller_state = controller_shared::state::state();
    Telemetry {
        cpu_temperature: controller_state
            .cpu_temp
            .load(Ordering::Relaxed)
            .get::<kelvin>(),
    }
}

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
