use crate::advanced_adc::regular::{Config, Trigger};
use core::marker::PhantomData;
use embassy_stm32::adc::Instance;

pub struct Configured<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<'d, I: Instance> Configured<I> {
    pub(crate) fn new<T: Trigger>(config: Config<T>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
