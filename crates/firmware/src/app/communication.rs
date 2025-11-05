use crate::board::BoardCrc;

use communication::channel_types::{CommandChannel, EventChannel};
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::PubSubChannel;

#[embassy_executor::task]
pub async fn task_communication(mut crc: BoardCrc<'static>) {
    communication::run(&COMMAND_CHANNEL, &EVENT_CHANNEL, &mut crc).await;
}

pub static COMMAND_CHANNEL: CommandChannel = Channel::new();
pub static EVENT_CHANNEL: EventChannel = PubSubChannel::new();
