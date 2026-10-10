use crate::adc::pac_instance::PacInstance;
use embassy_time::{Duration, block_for};
use stm32_metapac::adc::vals::{Adcaldif, Difsel, Exten};

pub trait RegManipulations {
    fn power_up();
    fn set_difsel_all(val: Difsel);
    fn calibrate(val: Adcaldif);
    fn enable();
    fn configure_single_conv_soft_trigger();
}

impl<T: PacInstance> RegManipulations for T {
    fn power_up() {
        Self::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        block_for(Duration::from_micros(20));
    }

    fn set_difsel_all(val: Difsel) {
        Self::regs().difsel().modify(|w| {
            for n in 0..18 {
                w.set_difsel(n, val);
            }
        })
    }

    fn calibrate(val: Adcaldif) {
        Self::regs().cr().modify(|reg| {
            reg.set_adcaldif(val);
            reg.set_adcal(true);
        });

        block_for(Duration::from_micros(20));
        while Self::regs().cr().read().adcal() {}
        block_for(Duration::from_micros(20));
    }

    fn enable() {
        while Self::regs().cr().read().addis() {}

        if !Self::regs().cr().read().aden() {
            Self::regs().isr().modify(|reg| reg.set_adrdy(true));
            Self::regs().cr().modify(|reg| reg.set_aden(true));
            while !Self::regs().isr().read().adrdy() {}
        }
    }

    fn configure_single_conv_soft_trigger() {
        Self::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_exten(Exten::DISABLED);
        });
    }
}
