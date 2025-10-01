use crate::advanced_adc::config::{
    EndOfConversionSignal, GainCompensation, OversamplingRatio, OversamplingShift,
};
use crate::advanced_adc::pac_instance::PacInstance;
use embassy_time::{Duration, block_for};
use stm32_metapac::adc::vals::{Adcaldif, Difsel, Dmacfg, Exten, Ovrmod, Res, Rovsm, Trovs};

pub trait RegManipulations {
    fn power_up();
    fn set_difsel_all(val: Difsel);
    fn calibrate(val: Adcaldif);
    fn enable();
    fn configure_single_conv_soft_trigger();
    fn set_resolution(val: Res);
    fn set_end_of_conversion_signal_regular(val: EndOfConversionSignal);
    fn set_end_of_conversion_signal_injected(val: EndOfConversionSignal);
    fn set_data_align(left: bool);
    fn set_gain_compensation(val: GainCompensation);
    fn set_low_power_auto_wait_mode(enabled: bool);
    fn set_dma_config(val: Dmacfg);
    fn set_overrun(val: Ovrmod);
    fn set_common_oversampling(shift: OversamplingShift, ratio: OversamplingRatio);
    fn set_regular_oversampling_modes(regular_mode: Rovsm, triggered_mode: Trovs);
    fn set_regular_oversampling_enabled(val: bool);
    fn set_injected_oversampling_enabled(val: bool);
    fn is_vrefint_enabled() -> bool;
    fn is_temperature_enabled() -> bool;
    fn enable_vrefint();
    fn enable_temperature();
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

        if Self::regs().cr().read().aden() == false {
            Self::regs().isr().modify(|reg| reg.set_adrdy(true));
            Self::regs().cr().modify(|reg| reg.set_aden(true));
            while Self::regs().isr().read().adrdy() == false {}
        }
    }

    fn configure_single_conv_soft_trigger() {
        Self::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_exten(Exten::DISABLED);
        });
    }

    fn set_resolution(val: Res) {
        Self::regs().cfgr().modify(|reg| reg.set_res(val));
    }

    // TODO split into two, injected and regular
    fn set_end_of_conversion_signal_regular(val: EndOfConversionSignal) {
        Self::regs().ier().modify(|reg| match val {
            EndOfConversionSignal::None => {
                reg.set_eosie(false);
                reg.set_eocie(false);
            }
            EndOfConversionSignal::Single => {
                reg.set_eosie(false);
                reg.set_eocie(true);
            }
            EndOfConversionSignal::Sequence => {
                reg.set_eosie(true);
                reg.set_eocie(false);
            }
            EndOfConversionSignal::Both => {
                reg.set_eosie(true);
                reg.set_eocie(true);
            }
        });
    }

    fn set_end_of_conversion_signal_injected(val: EndOfConversionSignal) {
        Self::regs().ier().modify(|reg| match val {
            EndOfConversionSignal::None => {
                reg.set_jeosie(false);
                reg.set_jeocie(false);
            }
            EndOfConversionSignal::Single => {
                reg.set_jeosie(false);
                reg.set_jeocie(true);
            }
            EndOfConversionSignal::Sequence => {
                reg.set_jeosie(true);
                reg.set_jeocie(false);
            }
            EndOfConversionSignal::Both => {
                reg.set_jeosie(true);
                reg.set_jeocie(true);
            }
        });
    }

    fn set_data_align(left: bool) {
        Self::regs().cfgr().modify(|reg| {
            reg.set_align(left);
        });
    }

    fn set_gain_compensation(val: GainCompensation) {
        Self::regs().cfgr2().modify(|reg| reg.set_gcomp(val.0 != 0));
        Self::regs().gcomp().modify(|reg| reg.set_gcompcoeff(val.0));
    }

    fn set_low_power_auto_wait_mode(enabled: bool) {
        Self::regs().cfgr().modify(|reg| reg.set_autdly(enabled));
    }

    fn set_dma_config(val: Dmacfg) {
        Self::regs().cfgr().modify(|reg| reg.set_dmacfg(val));
    }

    fn set_overrun(val: Ovrmod) {
        Self::regs().cfgr().modify(|reg| reg.set_ovrmod(val));
    }

    fn set_common_oversampling(shift: OversamplingShift, ratio: OversamplingRatio) {
        T::regs().cfgr2().modify(|reg| {
            reg.set_ovss(shift.into());
            reg.set_ovsr(ratio.into());
        });
    }

    fn set_regular_oversampling_modes(regular_mode: Rovsm, triggered_mode: Trovs) {
        T::regs().cfgr2().modify(|reg| {
            reg.set_rovsm(regular_mode.into());
            reg.set_trovs(triggered_mode.into());
        });
    }

    fn set_regular_oversampling_enabled(val: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_rovse(val));
    }

    fn set_injected_oversampling_enabled(val: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_jovse(val));
    }

    fn is_vrefint_enabled() -> bool {
        T::common_regs().ccr().read().vrefen()
    }

    fn is_temperature_enabled() -> bool {
        T::common_regs().ccr().read().vsenseen()
    }

    fn enable_vrefint() {
        T::common_regs().ccr().modify(|reg| reg.set_vrefen(true));
    }

    fn enable_temperature() {
        T::common_regs().ccr().modify(|reg| reg.set_vsenseen(true))
    }
}
