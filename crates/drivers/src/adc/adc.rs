use crate::adc::injected;
use crate::adc::pac::RegManipulations;
use crate::adc::pac_instance::PacInstance;
use crate::adc::state::WithState;
use core::marker::PhantomData;
use embassy_stm32::{Peri, peripherals, rcc};
use logging::debug;
use stm32_metapac::adc::vals::{Adcaldif, Difsel, Dmacfg, Ovrmod, Res};

pub trait AdcInstance: PacInstance + WithState {
    fn get_name() -> &'static str;
    const EPOCH_ADC: crate::adc::epoch::FastAdc;
}

macro_rules! adc_instance {
    ($($adc:ident => $index:ident),+) => {
        $(impl AdcInstance for peripherals::$adc {
            fn get_name() -> &'static str {
                stringify!($adc)
            }
            const EPOCH_ADC: crate::adc::epoch::FastAdc = crate::adc::epoch::FastAdc::$index;
        })+
    }
}

adc_instance!(ADC1 => Adc1, ADC3 => Adc3, ADC5 => Adc5);

pub struct Free;
pub struct Taken;

pub struct Adc<'d, T: AdcInstance, I = Free> {
    adc: Peri<'d, T>,
    _phantom_data: PhantomData<I>,
}

impl<'d, T: AdcInstance> Adc<'d, T> {
    /// Joining an enabled RCC group must not reset another ADC in that group.
    pub fn new_in_enabled_group(adc: Peri<'d, T>) -> Self {
        critical_section::with(|cs| rcc::enable_with_cs::<T>(cs));
        debug!("Configuring {}", T::get_name());

        T::power_up();
        T::set_difsel_all(Difsel::SINGLE_ENDED);
        T::calibrate(Adcaldif::SINGLE_ENDED);
        T::calibrate(Adcaldif::DIFFERENTIAL);
        T::enable();
        T::configure_single_conv_soft_trigger();
        T::regs().cfgr().modify(|reg| {
            reg.set_res(Res::BITS12);
            reg.set_align(false);
            reg.set_autdly(false);
            reg.set_dmacfg(Dmacfg::ONE_SHOT);
            reg.set_ovrmod(Ovrmod::PRESERVE);
        });
        T::regs().cfgr2().modify(|reg| {
            reg.set_gcomp(false);
            reg.set_rovse(false);
            reg.set_jovse(false);
        });

        Self {
            adc,
            _phantom_data: PhantomData,
        }
    }

    pub fn configure_tim1_triggered(self) -> (Adc<'d, T, Taken>, injected::Configured<T>) {
        (
            Adc {
                adc: self.adc,
                _phantom_data: PhantomData,
            },
            injected::Configured::new_tim1_triggered(),
        )
    }
}
