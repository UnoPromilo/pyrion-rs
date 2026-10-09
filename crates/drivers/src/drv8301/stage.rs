use super::control::{Drv8301Registers, SlowControl};
use super::spi_device::{DrvSpi, DrvSpiDevice};
#[cfg(feature = "six-pwm-tim1")]
use crate::SixPwmTim1;
use core::marker::PhantomData;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
#[cfg(feature = "six-pwm-tim1")]
use embassy_stm32::timer::AdvancedInstance4Channel;

pub struct Drv8301Stage<'a, P> {
    pwm: P,
    registers: Drv8301Registers<'a>,
    en_gate: Output<'a>,
    n_fault: ExtiInput<'a, Async>,
}

impl<'a, P> Drv8301Stage<'a, P> {
    pub fn new(
        pwm: P,
        spi: DrvSpi,
        cs: Output<'a>,
        mut en_gate: Output<'a>,
        n_fault: ExtiInput<'a, Async>,
    ) -> Self {
        en_gate.set_low();
        Self {
            pwm,
            registers: Drv8301Registers {
                device: DrvSpiDevice { bus: spi, cs },
            },
            en_gate,
            n_fault,
        }
    }

    pub fn split(self) -> (FastOutput<'a, P, Unchecked>, SlowControl<'a>) {
        (
            FastOutput {
                pwm: self.pwm,
                _en_gate: self.en_gate,
                state: PhantomData,
            },
            SlowControl {
                registers: self.registers,
                n_fault: self.n_fault,
            },
        )
    }
}

pub struct Unchecked;
pub struct Checked;

pub struct FastOutput<'a, P, State> {
    pwm: P,
    _en_gate: Output<'a>,
    state: PhantomData<State>,
}

impl<P, State> FastOutput<'_, P, State> {
    pub fn pwm(&self) -> &P {
        &self.pwm
    }

    pub fn pwm_mut(&mut self) -> &mut P {
        &mut self.pwm
    }
}

#[cfg(feature = "six-pwm-tim1")]
impl<'a, T: AdvancedInstance4Channel> FastOutput<'a, SixPwmTim1<'a, T>, Unchecked> {
    pub fn wake_for_preflight(&mut self) {
        self.pwm.disable();
        self.pwm.set_phase_duties(0, 0, 0);
        self._en_gate.set_high();
    }
}

#[cfg(feature = "six-pwm-tim1")]
impl<'a, T: AdvancedInstance4Channel, State> FastOutput<'a, SixPwmTim1<'a, T>, State> {
    pub fn finish_preflight(&mut self) {
        self._en_gate.set_low();
    }
}

pub struct PreflightPassed {
    pub(super) _private: (),
}

impl<'a, P> FastOutput<'a, P, Unchecked> {
    pub fn accept_preflight(self, _proof: PreflightPassed) -> FastOutput<'a, P, Checked> {
        FastOutput {
            pwm: self.pwm,
            _en_gate: self._en_gate,
            state: PhantomData,
        }
    }
}
