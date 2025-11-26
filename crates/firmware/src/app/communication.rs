use crate::board::{BoardCrc, BoardFlash};

use communication::channel_types::{CommandChannel, EventChannel};
use controller_shared::command::ControlCommandChannel;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;
use static_cell::StaticCell;
use update_manager::{FirmwareUpdateManager, FwBuffer, new_fw_buffer};

static FLASH: StaticCell<BoardFlash> = StaticCell::new();
static FW_BUFFER: StaticCell<FwBuffer> = StaticCell::new();
pub static COMMAND_CHANNEL: CommandChannel = Channel::new();
pub static EVENT_CHANNEL: EventChannel = PubSubChannel::new();
pub static CONTROL_COMMAND_CHANNEL: ControlCommandChannel = ControlCommandChannel::new();

#[embassy_executor::task]
pub async fn task_communication(mut crc: BoardCrc<'static>, flash: BoardFlash<'static>) {
    let flash = FLASH.init(flash);
    let fw_buffer = FW_BUFFER.init(new_fw_buffer());
    let mut update_manager = FirmwareUpdateManager::new(flash, fw_buffer);
    communication::run(
        &COMMAND_CHANNEL,
        &EVENT_CHANNEL,
        &CONTROL_COMMAND_CHANNEL,
        &mut crc,
        &mut update_manager,
    )
    .await;
}
