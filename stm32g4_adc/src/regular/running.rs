use crate::advanced_adc::regular::Configured;
use crate::advanced_adc::regular::pac::ModifyPac;
use crate::advanced_adc::{AdcInstance, Continuous, EndOfConversionSignal, Single};
use embassy_stm32::Peri;
use embassy_stm32::adc::{AnyAdcChannel, RxDma};
use embassy_stm32::dma::Transfer;
use embassy_stm32::interrupt::typelevel::Interrupt;
use stm32_metapac::adc::vals::{Dmacfg, SampleTime};

pub struct Running<I: AdcInstance, C, const CHANNELS: usize> {
    configured: Configured<I, C>,
    channels: [AnyAdcChannel<I>; CHANNELS],
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Single, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Single>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        Self::inner_new(configured, values)
    }

    pub async fn trigger_and_read(&self) -> [u16; CHANNELS] {
        I::start();
        // TODO do I need to stop?
        // TODO read using dma
        I::stop();
    }
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Continuous, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Continuous>,
        rx_dma: Peri<'_, impl RxDma<I>>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS], // tODO rename all values to sequence?
    ) -> Self {
        let instance = Self::inner_new(configured, values);
        // TODO do not run if AUTO = 1
        I::start();

        instance
    }

    pub async fn read_next(&self) -> [u16; CHANNELS] {
        todo!("Read using DMA")
    }
}

impl<I: AdcInstance, C, const CHANNELS: usize> Running<I, C, CHANNELS> {
    fn inner_new(
        configured: Configured<I, C>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        I::set_length(CHANNELS as u8 - 1);
        // TODO register channels

        let channels = values.map(|(ch, _)| ch);
        I::clear_end_of_conversion_signal(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal(EndOfConversionSignal::Sequence);
        unsafe { I::Interrupt::enable() }

        Self {
            configured,
            channels,
        }
    }
    pub fn release(self) -> (Configured<I, C>, [AnyAdcChannel<I>; CHANNELS]) {
        I::stop();
        I::set_end_of_conversion_signal(EndOfConversionSignal::None);
        (self.configured, self.channels)
    }
}
