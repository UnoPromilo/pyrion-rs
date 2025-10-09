use crate::EndOfConversionSignal;
use crate::injected::AnyExtTrigger;
use crate::pac_instance::PacInstance;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Adstp, Exten, SampleTime};

pub trait ModifyPac {
    fn set_software_trigger();
    fn set_ext_trigger(trigger: AnyExtTrigger);
    fn set_auto_conversion_mode(enabled: bool);
    fn set_length(length: u8);
    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime);
    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize);
    fn set_discontinuous_mode(enabled: bool);
    fn set_end_of_conversion_signal(val: EndOfConversionSignal);
    fn clear_end_of_conversion_signal(val: EndOfConversionSignal);

    fn start();
    fn stop();
}

pub trait ReadPac {
    fn read_value(index: usize) -> u16;
}

impl<T: PacInstance> ModifyPac for T {
    fn set_software_trigger() {
        Self::regs().jsqr().modify(|reg| {
            reg.set_jexten(Exten::DISABLED);
            reg.set_jextsel(0);
        });
    }

    fn set_ext_trigger(trigger: AnyExtTrigger) {
        match trigger {
            AnyExtTrigger::ADC12(source, edge) => Self::regs().jsqr().modify(|reg| {
                reg.set_jexten(edge.into());
                reg.set_jextsel(source.into());
            }),
            AnyExtTrigger::ADC345(source, edge) => Self::regs().jsqr().modify(|reg| {
                reg.set_jexten(edge.into());
                reg.set_jextsel(source.into());
            }),
        }
    }

    fn set_auto_conversion_mode(enabled: bool) {
        Self::regs().cfgr().modify(|reg| reg.set_jauto(enabled));
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

    fn set_discontinuous_mode(enabled: bool) {
        Self::regs().cfgr().modify(|reg| reg.set_jdiscen(enabled));
    }

    fn set_end_of_conversion_signal(val: EndOfConversionSignal) {
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

    fn clear_end_of_conversion_signal(val: EndOfConversionSignal) {
        Self::regs().isr().modify(|reg| match val {
            EndOfConversionSignal::None => {}
            EndOfConversionSignal::Single => {
                reg.set_jeoc(true);
            }
            EndOfConversionSignal::Sequence => {
                reg.set_jeos(true);
            }
            EndOfConversionSignal::Both => {
                reg.set_jeoc(true);
                reg.set_jeos(true);
            }
        });
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
