use crate::pac_instance::PacInstance;
use crate::{DataAlignment, GainCompensation, OversamplingRatio, OversamplingShift};
use embassy_time::{Duration, block_for};
use stm32_metapac::adc::vals::{Adcaldif, Difsel, Dmacfg, Exten, Ovrmod, Res, Rovsm, Trovs};

pub trait RegManipulations {
    fn power_up();
    fn set_difsel_all(val: Difsel);
    fn calibrate(val: Adcaldif);
    fn enable();
    fn configure_single_conv_soft_trigger();
    fn set_resolution(val: Res);

    fn set_data_align(val: DataAlignment);
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
    fn is_vbat_enabled() -> bool;
    fn enable_vrefint();
    fn enable_temperature();
    fn enable_vbat();
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

    fn set_data_align(val: DataAlignment) {
        Self::regs().cfgr().modify(|reg| {
            reg.set_align(val.into());
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
        Self::regs().cfgr2().modify(|reg| {
            reg.set_ovss(shift.into());
            reg.set_ovsr(ratio.into());
        });
    }

    fn set_regular_oversampling_modes(regular_mode: Rovsm, triggered_mode: Trovs) {
        Self::regs().cfgr2().modify(|reg| {
            reg.set_rovsm(regular_mode);
            reg.set_trovs(triggered_mode);
        });
    }

    fn set_regular_oversampling_enabled(val: bool) {
        Self::regs().cfgr2().modify(|reg| reg.set_rovse(val));
    }

    fn set_injected_oversampling_enabled(val: bool) {
        Self::regs().cfgr2().modify(|reg| reg.set_jovse(val));
    }

    fn is_vrefint_enabled() -> bool {
        Self::common_regs().ccr().read().vrefen()
    }

    fn is_temperature_enabled() -> bool {
        Self::common_regs().ccr().read().vsenseen()
    }

    fn is_vbat_enabled() -> bool {
        Self::common_regs().ccr().read().vbaten()
    }

    fn enable_vrefint() {
        Self::common_regs().ccr().modify(|reg| reg.set_vrefen(true));
    }

    fn enable_temperature() {
        Self::common_regs()
            .ccr()
            .modify(|reg| reg.set_vsenseen(true))
    }

    fn enable_vbat() {
        Self::common_regs().ccr().modify(|reg| reg.set_vbaten(true))
    }
}
