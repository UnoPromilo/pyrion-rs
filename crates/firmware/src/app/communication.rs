use hardware::BoardCrc;

use communication::channel_types::{CommandChannel, EventChannel};
use controller_shared::command::ControlCommandChannel;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;

pub static COMMAND_CHANNEL: CommandChannel = Channel::new();
pub static EVENT_CHANNEL: EventChannel = PubSubChannel::new();
pub static CONTROL_COMMAND_CHANNEL: ControlCommandChannel = ControlCommandChannel::new();

#[embassy_executor::task]
pub async fn task_communication(mut crc: BoardCrc<'static>) {
    communication::run(
        &COMMAND_CHANNEL,
        &EVENT_CHANNEL,
        &CONTROL_COMMAND_CHANNEL,
        &mut crc
    )
    .await;
}
