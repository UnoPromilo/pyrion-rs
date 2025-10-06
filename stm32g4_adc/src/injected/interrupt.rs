use crate::injected::pac::{ModifyPac, ReadPac};
use crate::state::State;
use crate::{AdcInstance, EndOfConversionSignal};
use core::sync::atomic::{Ordering, compiler_fence};

pub fn on_interrupt<T: AdcInstance>(state: &State) {
    // TODO if T::regs().isr().read().jeoc()

    // TODO move to pac.rs?
    if T::regs().isr().read().jeos() {
        // In theory, it is not required to clear the signal because reading the value will clear it.
        T::clear_end_of_conversion_signal(EndOfConversionSignal::Sequence);
        let values = [
            T::read_value(0),
            T::read_value(1),
            T::read_value(2),
            T::read_value(3),
        ];

        compiler_fence(Ordering::SeqCst);
        state.jeos_signal.signal(values);
    }
}
