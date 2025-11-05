use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber};
use crate::packet::Packet;

pub type CommandChannel = Channel<CriticalSectionRawMutex, Packet, 10>;
pub type EventChannel = PubSubChannel<CriticalSectionRawMutex, Packet, 10, 2, 1>;
pub type EventSubscriber<'a> = Subscriber<'a, CriticalSectionRawMutex, Packet, 10, 2, 1>;
