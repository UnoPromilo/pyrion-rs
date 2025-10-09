use crate::{AdcInstance, injected};
use core::marker::PhantomData;
use embassy_stm32::interrupt;

pub struct InterruptHandler<T: AdcInstance> {
    _phantom: PhantomData<T>,
}

impl<T: AdcInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        injected::on_interrupt::<T>(T::state());
    }
}
