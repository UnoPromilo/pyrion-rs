use crate::advanced_adc::injected::configured::Configured;
use crate::advanced_adc::injected::pac::{ModifyPac, ReadPac};
use crate::advanced_adc::pac_instance::PacInstance;
use core::marker::PhantomData;
use defmt::info;

pub struct Running<I: PacInstance, const CHANNELS: usize> {
    _phantom: PhantomData<I>,
}

impl<I: PacInstance, const CHANNELS: usize> Running<I, CHANNELS> {
    pub(crate) fn new(_val: Configured<I>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn stop(self) -> Configured<I> {
        Self::destruct();
        Configured::downgrade(self)
    }

    /// Reads the value of the latest conversion
    pub fn read_now(&self) -> [u16; CHANNELS] {
        let mut values = [0; CHANNELS];
        for i in 0..CHANNELS {
            values[i] = I::read_value(i);
        }
        values
    }

    // TODO do I need this?
    fn destruct() {
        info!("Działa destroy");
        I::stop();
    }
}

impl<I: PacInstance, const CHANNELS: usize> Drop for Running<I, CHANNELS> {
    // TODO do I need to do it in stop?
    fn drop(&mut self) {
        I::stop();
        info!("Działa drop");
        Self::destruct()
    }
}
