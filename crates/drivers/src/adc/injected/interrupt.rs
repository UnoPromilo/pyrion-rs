use crate::adc::AdcInstance;
use crate::adc::epoch::TriggerEpoch;
use crate::adc::injected::pac::{ModifyPac, ReadPac};
use crate::adc::state::{State, TaggedResult};
use core::sync::atomic::{Ordering, compiler_fence};

pub fn on_interrupt<T: AdcInstance>(state: &State) {
    #[cfg(feature = "adc-timing")]
    let cycle = crate::adc::timing::cycle_count();
    if T::regs().isr().read().jeos() {
        let epoch = TriggerEpoch::current();
        T::clear_jeos();
        let values = [T::read_value(0), T::read_value(1)];

        compiler_fence(Ordering::SeqCst);
        crate::adc::epoch::on_adc_jeos(T::EPOCH_ADC, epoch);
        #[cfg(feature = "adc-timing")]
        let sequence = crate::adc::timing::record_jeos(T::EPOCH_ADC.index(), epoch, cycle);
        state.jeos_signal.signal(TaggedResult {
            epoch,
            values,
            #[cfg(feature = "adc-timing")]
            sequence,
        });
    }
}
