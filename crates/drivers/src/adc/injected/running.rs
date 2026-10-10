use crate::adc::AdcInstance;
use crate::adc::epoch::TriggerEpoch;
use crate::adc::injected::Configured;
use crate::adc::injected::pac::ModifyPac;
use crate::adc::state::TaggedResult;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Interrupt;
use logging::{debug, trace};
use stm32_metapac::adc::vals::SampleTime;

pub struct Running<'a, I: AdcInstance> {
    _configured: Configured<I>,
    _channels: [AnyAdcChannel<'a, I>; 2],
}

impl<'a, I: AdcInstance> Running<'a, I> {
    pub(crate) fn new(
        configured: Configured<I>,
        sequence: [(AnyAdcChannel<'a, I>, SampleTime); 2],
    ) -> Self {
        I::set_length(1);
        for (index, (channel, sample_time)) in sequence.iter().enumerate() {
            trace!(
                "Registering injected channel {} (index: {}) with sample time {:?}",
                channel.get_hw_channel(),
                index,
                sample_time
            );
            I::set_channel_sample_time(channel, *sample_time);
            I::register_channel(channel, index);
        }
        let channels = sequence.map(|(ch, _)| ch);

        I::clear_jeos();
        I::enable_jeos_interrupt();
        unsafe { I::Interrupt::enable() }
        debug!("Injected {} started", I::get_name());
        I::start();

        Self {
            _configured: configured,
            _channels: channels,
        }
    }

    pub async fn read_next_tagged(&self) -> TaggedRead {
        let result = I::state().jeos_signal.wait().await;
        #[cfg(feature = "adc-timing")]
        crate::adc::timing::record_delivery(I::EPOCH_ADC.index(), result.sequence);
        let TaggedResult { epoch, values, .. } = result;
        TaggedRead { epoch, values }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaggedRead {
    pub epoch: TriggerEpoch,
    pub values: [u16; 2],
}
