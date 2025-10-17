use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use transport::Command;

pub struct State {
    pub command_signal: Signal<NoopRawMutex, Command>,
    pub response_signal: Signal<NoopRawMutex, Command>,
}
