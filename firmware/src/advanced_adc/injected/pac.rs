use crate::advanced_adc::injected::triggers::Trigger;
use crate::advanced_adc::injected::triggers::{AnyTriggerSource, TriggerADC12, TriggerADC345};
use crate::advanced_adc::pac_instance::PacInstance;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Adstp, Exten, SampleTime};

pub trait ModifyPac {
    fn set_trigger(trigger: impl Trigger);
    fn set_auto_conversion_mode(enabled: bool);
    fn set_length(length: u8);
    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime);
    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize);
    fn start();
    fn stop();
}

pub trait ReadPac {
    fn read_value(index: usize) -> u16;
}

impl<T: PacInstance> ModifyPac for T {
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

    fn set_auto_conversion_mode(enabled: bool) {
        Self::regs().cfgr().modify(|regs| regs.set_jauto(enabled));
    }

    fn set_length(length: u8) {
        Self::regs().jsqr().modify(|reg| reg.set_jl(length));
    }

    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime) {
        let channel = channel.get_hw_channel() as usize;
        if channel <= 9 {
            Self::regs()
                .smpr()
                .modify(|reg| reg.set_smp(channel, sample_time));
        } else {
            Self::regs()
                .smpr2()
                .modify(|reg| reg.set_smp(channel - 10, sample_time));
        }
    }

    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize) {
        Self::regs()
            .jsqr()
            .modify(|reg| reg.set_jsq(index, channel.get_hw_channel()));
    }

    fn start() {
        Self::regs().cr().modify(|regs| regs.set_jadstart(true));
    }

    fn stop() {
        Self::regs()
            .cr()
            .modify(|regs| regs.set_jadstp(Adstp::STOP));
    }
}

impl<T: PacInstance> ReadPac for T {
    fn read_value(index: usize) -> u16 {
        Self::regs().jdr(index).read().jdata()
    }
}
