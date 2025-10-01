use crate::advanced_adc::injected::Trigger;

#[derive(Copy, Clone, Debug)]
pub struct Config<T: Trigger> {
    pub trigger: T,
}

impl<T: Trigger> Default for Config<T> {
    fn default() -> Self {
        Self {
            trigger: Default::default(),
        }
    }
}
