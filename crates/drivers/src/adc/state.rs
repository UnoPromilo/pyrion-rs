use crate::adc::epoch::TriggerEpoch;
use embassy_stm32::peripherals;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub trait WithState {
    fn state() -> &'static State;
}

macro_rules! impl_with_state {
    ($($peripheral:ty),+ $(,)?) => {
        $(impl WithState for $peripheral {
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        })*
    }
}

impl_with_state!(peripherals::ADC1, peripherals::ADC3, peripherals::ADC5);

pub struct State {
    pub jeos_signal: Signal<CriticalSectionRawMutex, TaggedResult>,
}

#[derive(Clone, Copy)]
pub struct TaggedResult {
    pub epoch: TriggerEpoch,
    pub values: [u16; 2],
    #[cfg(feature = "adc-timing")]
    pub sequence: u32,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub const fn new() -> Self {
        Self {
            jeos_signal: Signal::new(),
        }
    }
}
