use crate::advanced_adc::regular::Trigger;

#[derive(Debug, Copy, Clone)]
pub struct Config<T: Trigger> {
    pub enable_scan_conversion_mode: bool, // TODO disabled by default. enable should also ask for nbr of conv, continuous mode, discontinuous mode
    pub enable_continuous_mode: bool,      // TODO move to regular?
    pub enable_discontinuous_mode: bool, // TODO move to regular? TODO assert that  enable_continuous_mode && enable_discontinuous_mode == false
    pub trigger: T,                      // TODO separate Triggers for both adc?
}

impl<T: Trigger> Default for Config<T> {
    fn default() -> Self {
        Self {
            enable_scan_conversion_mode: false,
            enable_continuous_mode: false,
            enable_discontinuous_mode: false,
            trigger: Default::default(),
        }
    }
}
