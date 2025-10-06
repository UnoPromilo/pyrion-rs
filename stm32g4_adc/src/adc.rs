use crate::pac::RegManipulations;
use crate::pac_instance::PacInstance;
use crate::prescaler::Prescaler;
use crate::state::WithState;
use crate::trigger_edge::ExtTriggerEdge;
use crate::{Config, injected};
use core::marker::PhantomData;
use embassy_stm32::adc::{
    Temperature, TemperatureChannel, VBatChannel, Vbat, VrefChannel, VrefInt,
};
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

// TODO do I need traits for this?
pub struct Single;
pub struct Continuous;

pub struct Adc<'d, T: AdcInstance, I = Free, R = Free> {
    #[allow(unused)]
    adc: Peri<'d, T>,

    _phantom_data: PhantomData<(I, R)>,
}

impl<'d, T> Adc<'d, T>
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

impl Drop for Vbat {
    fn drop(&mut self) {
        T::disable_vbat();
    }
}

*/

impl<'d, T: AdcInstance, R> Adc<'d, T, Free, R> {
    pub fn configure_injected_ext_trigger(
        self,
        trigger: <T::Family as AdcFamily>::InjectedExtTrigger,
        edge: ExtTriggerEdge,
    ) -> (Adc<'d, T, Taken, R>, injected::Configured<T, Continuous>) {
        (
            self.take_injected(),
            injected::Configured::new_triggered(injected::IntoAnyExtTrigger::into(trigger, edge)),
        )
    }

    pub fn configure_injected_auto(
        self,
    ) -> (Adc<'d, T, Taken, R>, injected::Configured<T, Continuous>) {
        (self.take_injected(), injected::Configured::new_auto())
    }

    pub fn configure_injected_single_conversion(
        self,
    ) -> (Adc<'d, T, Taken, R>, injected::Configured<T, Single>) {
        (self.take_injected(), injected::Configured::new_single())
    }
}

impl<'d, T: AdcInstance, R> Adc<'d, T, Free, R> {
    fn take_injected(self) -> Adc<'d, T, Taken, R> {
        Adc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}

// TODO
/*
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


impl<'d, T: AdcInstance, I> Adc<'d, T, I, Free> {
    fn take_regular(self) -> Adc<'d, T, I, Taken> {
        Adc {
            adc: self.adc,
            _phantom_data: PhantomData,
        }
    }
}*/

// TODO make this compile time check instead of runtime panic
impl<'d, T: AdcInstance, R, I> Adc<'d, T, I, R> {
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

    pub fn enable_vbat(&self) -> Vbat
    where
        T: VBatChannel,
    {
        if T::is_vbat_enabled() {
            panic!("Vbat is already enabled");
        }

        T::enable_vbat();

        Vbat {}
    }
}
