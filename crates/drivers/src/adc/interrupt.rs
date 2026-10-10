use crate::adc::{AdcInstance, injected};
use core::marker::PhantomData;
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::typelevel::Interrupt;

pub struct SingleInterruptHandler<T: AdcInstance> {
    _phantom: PhantomData<T>,
}

pub trait InterruptHandler<I: Interrupt>: interrupt::typelevel::Handler<I> {}

impl<T: AdcInstance> InterruptHandler<T::Interrupt> for SingleInterruptHandler<T> {}

impl<T: AdcInstance> interrupt::typelevel::Handler<T::Interrupt> for SingleInterruptHandler<T> {
    unsafe fn on_interrupt() {
        injected::on_interrupt::<T>(T::state());
    }
}
