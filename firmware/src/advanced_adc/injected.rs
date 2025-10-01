use crate::advanced_adc::ExternalTriggerEdge;
use crate::advanced_adc::pac_instance::PacInstance;
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Adstp, Exten, SampleTime};

pub trait Trigger: Copy + Default + Into<AnyTriggerSource> {}

#[derive(Copy, Clone, Debug)]
pub struct Config<T: Trigger> {
    pub trigger: T,
}

pub struct Configured<T: PacInstance> {
    _phantom: PhantomData<T>,
}

impl<I: PacInstance> Configured<I> {
    pub(crate) fn new<T: Trigger>(config: Config<T>) -> Self {
        I::set_trigger(config.trigger);
        I::set_auto_conversion_mode(false);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start_single_channel(
        self,
        channel: AnyAdcChannel<I>,
        sample_time: SampleTime,
    ) -> Running<I> {
        I::set_channel_sample_time(channel.get_hw_channel(), sample_time);
        I::register_channels([channel.get_hw_channel()].into_iter());
        I::start();
        Running::new()
    }

    fn downgrade() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

pub struct Running<I: PacInstance> {
    _phantom: PhantomData<I>,
}

impl<I: PacInstance> Running<I> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn stop(self) -> Configured<I> {
        I::stop();
        // TODO cancel conversions
        Configured::downgrade()
    }

    /// Reads the value of the latest conversion
    pub fn read_now(&self) -> u16 {
        I::read_value(0)
    }
}

impl<T: Trigger> Default for Config<T> {
    fn default() -> Self {
        Self {
            trigger: Default::default(),
        }
    }
}

trait RegManipulations {
    fn set_trigger(trigger: impl Trigger);
    fn start();
    fn stop();
    fn set_channel_sample_time(channel: u8, sample_time: SampleTime);
    fn register_channels(sequence: impl ExactSizeIterator<Item = u8>);
    fn read_value(index: usize) -> u16;
    fn set_auto_conversion_mode(enabled: bool);
}

impl<T: PacInstance> RegManipulations for T {
    fn set_trigger(trigger: impl Trigger) {
        match trigger.into() {
            AnyTriggerSource::ADC12(TriggerADC12::Software)
            | AnyTriggerSource::ADC345(TriggerADC345::Software) => {
                Self::regs().jsqr().modify(|reg| {
                    reg.set_jexten(Exten::DISABLED);
                    reg.set_jextsel(0);
                })
            }
            AnyTriggerSource::ADC12(TriggerADC12::External(source, edge)) => {
                Self::regs().jsqr().modify(|reg| {
                    reg.set_jexten(edge.into());
                    reg.set_jextsel(source.into());
                })
            }
            AnyTriggerSource::ADC345(TriggerADC345::External(source, edge)) => {
                Self::regs().jsqr().modify(|reg| {
                    reg.set_jexten(edge.into());
                    reg.set_jextsel(source.into());
                })
            }
        }
    }

    fn start() {
        Self::regs().cr().modify(|regs| regs.set_jadstart(true));
    }

    fn stop() {
        Self::regs()
            .cr()
            .modify(|regs| regs.set_jadstp(Adstp::STOP));
    }

    fn set_channel_sample_time(channel: u8, sample_time: SampleTime) {
        if channel <= 9 {
            Self::regs()
                .smpr()
                .modify(|reg| reg.set_smp(channel as _, sample_time));
        } else {
            Self::regs()
                .smpr2()
                .modify(|reg| reg.set_smp((channel - 10) as _, sample_time));
        }
    }

    fn register_channels(sequence: impl ExactSizeIterator<Item = u8>) {
        assert!(sequence.len() > 0);
        assert!(sequence.len() <= 4);
        Self::regs().jsqr().modify(|reg| {
            reg.set_jl(sequence.len() as u8);
            for (index, channel) in sequence.enumerate() {
                reg.set_jsq(index, channel)
            }
        });
    }

    fn read_value(index: usize) -> u16 {
        Self::regs().jdr(index).read().jdata()
    }

    fn set_auto_conversion_mode(enabled: bool) {
        Self::regs().cfgr().modify(|regs| regs.set_jauto(enabled));
    }
}

pub enum AnyTriggerSource {
    ADC12(TriggerADC12),
    ADC345(TriggerADC345),
}

#[derive(Debug, Copy, Clone, Default)]
#[allow(dead_code)]
pub enum TriggerADC12 {
    #[default]
    Software,
    External(ExternalTriggerConversionSourceADC12, ExternalTriggerEdge),
}

#[derive(Debug, Copy, Clone, Default)]
#[allow(dead_code)]
pub enum TriggerADC345 {
    #[default]
    Software,
    External(ExternalTriggerConversionSourceADC345, ExternalTriggerEdge),
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExternalTriggerConversionSourceADC12 {
    T1_TRGO = 0,
    T1_CC4 = 1,
    T2_TRGO = 2,
    T2_CC1 = 3,
    T3_CC4 = 4,
    T4_TRGO = 5,
    EXT_IT15 = 6,
    T8_CC4 = 7,
    T1_TRGO2 = 8,
    T8_TRGO = 9,
    T8_TRGO2 = 10,
    T3_CC3 = 11,
    T3_TRGO = 12,
    T3_CC1 = 13,
    T6_TRGO = 14,
    T15_TRGO = 15,
    T20_TRGO = 16,
    T20_TRGO2 = 17,
    T20_CC4 = 18,
    HRTIM_ADC_TRG2 = 19,
    HRTIM_ADC_TRG4 = 20,
    HRTIM_ADC_TRG5 = 21,
    HRTIM_ADC_TRG6 = 22,
    HRTIM_ADC_TRG7 = 23,
    HRTIM_ADC_TRG8 = 24,
    HRTIM_ADC_TRG9 = 25,
    T16_CC1 = 27,
    T7_TRGO = 30,
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExternalTriggerConversionSourceADC345 {
    T1_TRGO = 0,
    T1_CC4 = 1,
    T2_TRGO = 2,
    T8_CC2 = 3,
    T4_CC3 = 4,
    T4_TRGO = 5,
    T4_CC4 = 6,
    T8_CC4 = 7,
    T1_TRGO2 = 8,
    T8_TRGO = 9,
    T8_TRGO2 = 10,
    T1_CC3 = 11,
    T3_TRGO = 12,
    EXT_IT3 = 13,
    T6_TRGO = 14,
    T15_TRGO = 15,
    T20_TRGO = 16,
    T20_TRGO2 = 17,
    T20_CC2 = 18,
    HRTIM_ADC_TRG2 = 19,
    HRTIM_ADC_TRG4 = 20,
    HRTIM_ADC_TRG5 = 21,
    HRTIM_ADC_TRG6 = 22,
    HRTIM_ADC_TRG7 = 23,
    HRTIM_ADC_TRG8 = 24,
    HRTIM_ADC_TRG9 = 25,
    HRTIM_ADC_TRG1 = 27,
    HRTIM_ADC_TRG3 = 28,
    LPTIM_OUT = 29,
    T7_TRGO = 30,
}

impl Into<AnyTriggerSource> for TriggerADC12 {
    fn into(self) -> AnyTriggerSource {
        AnyTriggerSource::ADC12(self)
    }
}

impl Trigger for TriggerADC12 {}

impl Into<AnyTriggerSource> for TriggerADC345 {
    fn into(self) -> AnyTriggerSource {
        AnyTriggerSource::ADC345(self)
    }
}

impl Trigger for TriggerADC345 {}

impl Into<u8> for ExternalTriggerConversionSourceADC12 {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Into<u8> for ExternalTriggerConversionSourceADC345 {
    fn into(self) -> u8 {
        self as u8
    }
}
