#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub enable_continuous_mode: bool,
    //pub enable_scan_conversion_mode: bool, // TODO disabled by default. enable should also ask for nbr of conv, continuous mode, discontinuous mode
    //pub enable_discontinuous_mode: bool, // TODO move to regular? TODO assert that  enable_continuous_mode && enable_discontinuous_mode == false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_continuous_mode: false,
        }
    }
}
