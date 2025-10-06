use embassy_stm32::peripherals;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

// TODO merge with pac_instance?
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

impl_with_state!(
    peripherals::ADC1,
    peripherals::ADC2,
    peripherals::ADC3,
    peripherals::ADC4,
    peripherals::ADC5
);

pub struct State {
    pub jeos_signal: Signal<CriticalSectionRawMutex, [u16; 4]>,
}

impl State {
    pub const fn new() -> Self {
        Self {
            jeos_signal: Signal::new(),
        }
    }
}
