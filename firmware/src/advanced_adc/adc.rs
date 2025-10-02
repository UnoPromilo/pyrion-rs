use crate::advanced_adc::config::Config;
use crate::advanced_adc::injected::configured::{Continuous, Single};
use crate::advanced_adc::pac::RegManipulations;
use crate::advanced_adc::pac_instance::PacInstance;
use crate::advanced_adc::prescaler::Prescaler;
use crate::advanced_adc::state::WithState;
use crate::advanced_adc::trigger_edge::ExtTriggerEdge;
use crate::advanced_adc::{EndOfConversionSignal, injected, regular};
use core::marker::PhantomData;
use embassy_stm32::adc::{Temperature, TemperatureChannel, VrefChannel, VrefInt};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Peri, peripherals, rcc};
use shared::trace;
use stm32_metapac::adc::vals::{Adcaldif, Difsel};
use stm32_metapac::adccommon::vals::Presc;
// TODO add analog watchdog

pub trait AdcFamily {
    type InjectedExtTrigger: injected::IntoAnyExtTrigger;
}
pub struct Family12;
pub struct Family345;

impl AdcFamily for Family12 {
    type InjectedExtTrigger = injected::ExtTriggerSourceADC12;
}
impl AdcFamily for Family345 {
    type InjectedExtTrigger = injected::ExtTriggerSourceADC345;
}

pub trait AdcInstance: PacInstance + WithState {
    type Family: AdcFamily;
}
macro_rules! adc_instance {
    ($($adc:ident => $family:ty),+) => {
        $(impl AdcInstance for peripherals::$adc {
            type Family = $family;
        })+
    }
}

adc_instance!(
    ADC1 => Family12,
    ADC2 => Family12,
    ADC3 => Family345,
    ADC4 => Family345,
    ADC5 => Family345
);

pub struct Free;
pub struct Taken;

pub struct AdvancedAdc<'d, T: AdcInstance, I = Free, R = Free> {
    #[allow(unused)]
    adc: Peri<'d, T>,

    _phantom_data: PhantomData<(I, R)>,
}

impl<'d, T> AdvancedAdc<'d, T>
where
    T: AdcInstance,
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
        T::set_end_of_conversion_signal_regular(EndOfConversionSignal::None);
        T::set_end_of_conversion_signal_injected(EndOfConversionSignal::None);
        T::set_data_align(config.dataline_alignment);
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

impl<'d, T: AdcInstance, R> AdvancedAdc<'d, T, Free, R> {
    pub fn configure_injected_ext_trigger(
        self,
        trigger: <T::Family as AdcFamily>::InjectedExtTrigger,
        edge: ExtTriggerEdge,
        config: injected::Config,
    ) -> (
        AdvancedAdc<'d, T, Taken, R>,
        injected::Configured<T, Continuous>,
    ) {
        (
            self.take_injected(),
            injected::Configured::new_triggered(
                injected::IntoAnyExtTrigger::into(trigger, edge),
                config,
            ),
        )
    }

    pub fn configure_injected_auto(
        self,
        config: injected::Config,
    ) -> (
        AdvancedAdc<'d, T, Taken, R>,
        injected::Configured<T, Continuous>,
    ) {
        (self.take_injected(), injected::Configured::new_auto(config))
    }

    pub fn configure_injected_single_conversion(
        self,
        config: injected::Config,
    ) -> (
        AdvancedAdc<'d, T, Taken, R>,
        injected::Configured<T, Single>,
    ) {
        (
            self.take_injected(),
            injected::Configured::new_single(config),
        )
    }
}

impl<'d, T: AdcInstance, R> AdvancedAdc<'d, T, Free, R> {
    fn take_injected(self) -> AdvancedAdc<'d, T, Taken, R> {
        AdvancedAdc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}

// TODO merge into single function
impl<'d, T: AdcInstance<Family = Family12>, I> AdvancedAdc<'d, T, I, Free> {
    #[allow(dead_code)]
    pub fn configure_regular_adc12(
        self,
        config: regular::Config<regular::TriggerADC12>,
    ) -> (AdvancedAdc<'d, T, I, Taken>, regular::Configured<T>) {
        (Self::take_regular(self), regular::Configured::new(config))
    }
}

impl<'d, T: AdcInstance<Family = Family345>, I> AdvancedAdc<'d, T, I, Free> {
    #[allow(dead_code)]
    pub fn configure_regular_adc345(
        self,
        config: regular::Config<regular::TriggerADC345>,
    ) -> (AdvancedAdc<'d, T, I, Taken>, regular::Configured<T>) {
        (Self::take_regular(self), regular::Configured::new(config))
    }
}

impl<'d, T: AdcInstance, I> AdvancedAdc<'d, T, I, Free> {
    fn take_regular(self) -> AdvancedAdc<'d, T, I, Taken> {
        AdvancedAdc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}

// TODO make this compile time check instead of runtime panic
impl<'d, T: AdcInstance, R, I> AdvancedAdc<'d, T, I, R> {
    pub fn enable_vrefint(&self) -> VrefInt
    where
        T: VrefChannel,
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
        T: TemperatureChannel,
    {
        if T::is_temperature_enabled() {
            panic!("Temperature is already enabled");
        }
        T::enable_temperature();

        Temperature {}
    }
}
