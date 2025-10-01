use crate::advanced_adc::pac_instance::PacInstance;
use crate::advanced_adc::prescaler::Prescaler;
use crate::advanced_adc::{injected, regular};
use core::marker::PhantomData;
use embassy_stm32::adc::{Instance, Temperature, TemperatureChannel, VrefChannel, VrefInt};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Peri, peripherals, rcc};
use embassy_time::{Duration, block_for};
use shared::trace;
use stm32_metapac::adc::vals::Res;
use stm32_metapac::adc::vals::{Adcaldif, Difsel, Dmacfg, Exten, Ovrmod, Rovsm, Trovs};
use stm32_metapac::adccommon::vals::Presc;

pub trait Adc12Instance: PacInstance {}
pub trait Adc345Instance: PacInstance {}

// blanket impls for the specific types
impl Adc12Instance for peripherals::ADC1 {}
impl Adc12Instance for peripherals::ADC2 {}

impl Adc345Instance for peripherals::ADC3 {}
impl Adc345Instance for peripherals::ADC4 {}
impl Adc345Instance for peripherals::ADC5 {}

pub struct Free;
pub struct Taken;

pub struct AdvancedAdc<'d, T: Instance, I = Free, R = Free> {
    #[allow(unused)]
    adc: Peri<'d, T>,

    _phantom_data: PhantomData<(I, R)>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub resolution: Res,
    pub align_left: bool,
    pub gain_compensation: GainCompensation,
    pub end_of_conversion_signal_regular: EndOfConversionSignal,
    pub end_of_conversion_signal_injected: EndOfConversionSignal,
    pub dma_config: Dmacfg,
    pub overrun_mode: Ovrmod,
    pub oversampling_config: OversamplingConfig,
    pub enable_low_power_auto_wait: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct OversamplingConfig {
    pub shift: OversamplingShift,
    pub ratio: OversamplingRatio,
    pub regular_mode: Rovsm,
    pub triggered_mode: Trovs,
    pub enable_injected_oversampling: bool,
    pub enable_regular_oversampling: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GainCompensation(u16);

#[derive(Debug, Clone, Copy)]
pub enum EndOfConversionSignal {
    None,
    Single,
    Sequence,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub enum OversamplingRatio {
    Ratio2 = 0,
    Ratio4 = 1,
    Ratio8 = 2,
    Ratio16 = 3,
    Ratio32 = 4,
    Ratio64 = 5,
    Ratio128 = 6,
    Ratio256 = 7,
}

impl From<OversamplingRatio> for u8 {
    #[inline(always)]
    fn from(val: OversamplingRatio) -> Self {
        val as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OversamplingShift {
    None = 0,
    Shift1 = 1,
    Shift2 = 2,
    Shift3 = 3,
    Shift4 = 4,
    Shift5 = 5,
    Shift6 = 6,
    Shift7 = 7,
    Shift8 = 8,
}

impl From<OversamplingShift> for u8 {
    #[inline(always)]
    fn from(val: OversamplingShift) -> Self {
        val as u8
    }
}

impl GainCompensation {
    pub const fn from_bytes(val: u16) -> Self {
        const MAX: u16 = 0x3FFF;
        assert!(
            val <= MAX,
            "The gain compensation value must be in the range from 0 to 0x3FFF"
        );
        Self(val)
    }

    #[allow(dead_code)]
    pub const fn from(val: f32) -> Self {
        const MAX: f32 = 3.999756;
        const SCALE: u16 = 0x3FFF;

        assert!(
            val >= 0.0 && val <= MAX,
            "The gain compensation value must be in the range from 0 to 3.999756"
        );
        Self::from_bytes((val / MAX * SCALE as f32) as u16)
    }

    pub const fn zero() -> Self {
        Self::from_bytes(0)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Res::BITS12,
            align_left: false,
            gain_compensation: GainCompensation::zero(),
            end_of_conversion_signal_regular: EndOfConversionSignal::None,
            end_of_conversion_signal_injected: EndOfConversionSignal::None,
            dma_config: Dmacfg::ONE_SHOT,
            overrun_mode: Ovrmod::PRESERVE,
            oversampling_config: Default::default(),
            enable_low_power_auto_wait: false,
        }
    }
}

impl Default for OversamplingConfig {
    fn default() -> Self {
        Self {
            shift: OversamplingShift::None,
            ratio: OversamplingRatio::Ratio2,
            regular_mode: Rovsm::CONTINUED,
            triggered_mode: Trovs::AUTOMATIC,
            enable_injected_oversampling: false,
            enable_regular_oversampling: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ExternalTriggerEdge {
    Rising,
    Falling,
    Both,
}

impl Into<Exten> for ExternalTriggerEdge {
    fn into(self) -> Exten {
        match self {
            ExternalTriggerEdge::Rising => Exten::RISING_EDGE,
            ExternalTriggerEdge::Falling => Exten::FALLING_EDGE,
            ExternalTriggerEdge::Both => Exten::BOTH_EDGES,
        }
    }
}

impl<'d, T> AdvancedAdc<'d, T>
where
    T: PacInstance,
{
    pub fn new(adc: Peri<'d, T>, config: Config) -> Self {
        rcc::enable_and_reset::<T>();
        let freq = rcc::frequency::<T>();
        let presc = Presc::from_kernel_clock(freq);
        T::common_regs().ccr().modify(|w| w.set_presc(presc));
        let freq = Hertz::hz(freq.0 / presc.divisor());
        trace!("ADC frequency set to {}", freq);

        T::power_up();
        // TODO move to config
        T::set_difsel_all(Difsel::SINGLE_ENDED);
        T::calibrate(Adcaldif::SINGLE_ENDED);
        T::calibrate(Adcaldif::DIFFERENTIAL);
        T::enable();
        T::configure_single_conv_soft_trigger();
        T::set_resolution(config.resolution);
        T::set_end_of_conversion_signal_regular(config.end_of_conversion_signal_regular);
        T::set_end_of_conversion_signal_injected(config.end_of_conversion_signal_injected);
        T::set_data_align(config.align_left);
        T::set_gain_compensation(config.gain_compensation);
        T::set_low_power_auto_wait_mode(config.enable_low_power_auto_wait);
        T::set_dma_config(config.dma_config);
        T::set_overrun(config.overrun_mode);
        T::set_common_oversampling(
            config.oversampling_config.shift,
            config.oversampling_config.ratio,
        );
        T::set_regular_oversampling_modes(
            config.oversampling_config.regular_mode,
            config.oversampling_config.triggered_mode,
        );
        T::set_regular_oversampling_enabled(config.oversampling_config.enable_regular_oversampling);
        T::set_injected_oversampling_enabled(
            config.oversampling_config.enable_injected_oversampling,
        );

        Self {
            adc,
            _phantom_data: PhantomData,
        }
    }
}
/*
TODO
impl Drop for VrefInt {
    fn drop(&mut self) {
        T::disable_vrefint();
    }
}

impl Drop for Temperature {
    fn drop(&mut self) {
        T::disable_temperature();
    }
}
*/

impl<'d, T: Adc12Instance, R> AdvancedAdc<'d, T, Free, R> {
    pub fn configure_injected_adc12(
        self,
        config: injected::Config<injected::TriggerADC12>,
    ) -> (AdvancedAdc<'d, T, Taken, R>, injected::Configured<T>) {
        (Self::take_injected(self), injected::Configured::new(config))
    }
}

impl<'d, T: Adc345Instance, R> AdvancedAdc<'d, T, Free, R> {
    pub fn configure_injected_adc345(
        self,
        config: injected::Config<injected::TriggerADC345>,
    ) -> (AdvancedAdc<'d, T, Taken, R>, injected::Configured<T>) {
        (Self::take_injected(self), injected::Configured::new(config))
    }
}

impl<'d, T: Instance, R> AdvancedAdc<'d, T, Free, R> {
    fn take_injected(self) -> AdvancedAdc<'d, T, Taken, R> {
        AdvancedAdc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}

impl<'d, T: Adc12Instance, I> AdvancedAdc<'d, T, I, Free> {
    pub fn configure_regular_adc12(
        self,
        config: regular::Config<regular::TriggerADC12>,
    ) -> (AdvancedAdc<'d, T, I, Taken>, regular::Configured<T>) {
        (Self::take_regular(self), regular::Configured::new(config))
    }
}

impl<'d, T: Adc345Instance, I> AdvancedAdc<'d, T, I, Free> {
    pub fn configure_regular_adc345(
        self,
        config: regular::Config<regular::TriggerADC345>,
    ) -> (AdvancedAdc<'d, T, I, Taken>, regular::Configured<T>) {
        (Self::take_regular(self), regular::Configured::new(config))
    }
}

impl<'d, T: Instance, I> AdvancedAdc<'d, T, I, Free> {
    fn take_regular(self) -> AdvancedAdc<'d, T, I, Taken> {
        AdvancedAdc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}

// TODO make this compile time check instead of runtime panic
impl<'d, T: Instance, R, I> AdvancedAdc<'d, T, I, R> {
    pub fn enable_vrefint(&self) -> VrefInt
    where
        T: VrefChannel + PacInstance,
    {
        if T::is_vrefint_enabled() {
            panic!("Vrefint is already enabled");
        }
        T::enable_vrefint();

        VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> Temperature
    where
        T: TemperatureChannel + PacInstance,
    {
        if T::is_temperature_enabled() {
            panic!("Temperature is already enabled");
        }
        T::enable_temperature();

        Temperature {}
    }
}

trait RegManipulations {
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
