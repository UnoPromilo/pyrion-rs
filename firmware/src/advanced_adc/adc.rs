use crate::advanced_adc::config::Config;
use crate::advanced_adc::pac::RegManipulations;
use crate::advanced_adc::pac_instance::PacInstance;
use crate::advanced_adc::prescaler::Prescaler;
use crate::advanced_adc::{injected, regular};
use core::marker::PhantomData;
use embassy_stm32::adc::{Instance, Temperature, TemperatureChannel, VrefChannel, VrefInt};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Peri, peripherals, rcc};
use shared::trace;
use stm32_metapac::adc::vals::{Adcaldif, Difsel};
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
    #[allow(dead_code)]
    pub fn configure_injected_adc12(
        self,
        config: injected::Config<injected::TriggerADC12>,
    ) -> (AdvancedAdc<'d, T, Taken, R>, injected::Configured<T>) {
        (self.take_injected(), injected::Configured::new(config))
    }
}

impl<'d, T: Adc345Instance, R> AdvancedAdc<'d, T, Free, R> {
    #[allow(dead_code)]
    pub fn configure_injected_adc345(
        self,
        config: injected::Config<injected::TriggerADC345>,
    ) -> (AdvancedAdc<'d, T, Taken, R>, injected::Configured<T>) {
        (self.take_injected(), injected::Configured::new(config))
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
    #[allow(dead_code)]
    pub fn configure_regular_adc12(
        self,
        config: regular::Config<regular::TriggerADC12>,
    ) -> (AdvancedAdc<'d, T, I, Taken>, regular::Configured<T>) {
        (Self::take_regular(self), regular::Configured::new(config))
    }
}

impl<'d, T: Adc345Instance, I> AdvancedAdc<'d, T, I, Free> {
    #[allow(dead_code)]
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
impl<'d, T: PacInstance, R, I> AdvancedAdc<'d, T, I, R> {
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
