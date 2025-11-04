use crate::{AdcInstance, injected};
use core::marker::PhantomData;
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::typelevel::Interrupt;

pub struct MultiInterruptHandler<T, U>
where
    T: AdcInstance,
    U: AdcInstance<Interrupt = T::Interrupt>,
{
    _phantom: PhantomData<(T, U)>,
}

pub struct SingleInterruptHandler<T: AdcInstance> {
    _phantom: PhantomData<T>,
}

pub trait InterruptHandler<I: Interrupt>: interrupt::typelevel::Handler<I> {}

impl<T: AdcInstance> InterruptHandler<T::Interrupt> for SingleInterruptHandler<T> {}

impl<T, U> InterruptHandler<T::Interrupt> for MultiInterruptHandler<T, U>
where
    T: AdcInstance,
    U: AdcInstance<Interrupt = T::Interrupt>,
{
}

impl<T: AdcInstance> interrupt::typelevel::Handler<T::Interrupt> for SingleInterruptHandler<T> {
    unsafe fn on_interrupt() {
        injected::on_interrupt::<T>(T::state());
    }
}

impl<T, U> interrupt::typelevel::Handler<T::Interrupt> for MultiInterruptHandler<T, U>
where
    T: AdcInstance,
    U: AdcInstance<Interrupt = T::Interrupt>,
{
    unsafe fn on_interrupt() {
        injected::on_interrupt::<T>(T::state());
        injected::on_interrupt::<U>(U::state());
    }
}
