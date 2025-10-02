use crate::advanced_adc::injected::triggers::AnyExtTrigger;
use crate::advanced_adc::pac_instance::PacInstance;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Adstp, Dmacfg, Dmaen, Exten, SampleTime};

pub trait ModifyPac {
    fn set_software_trigger();
    fn set_ext_trigger(trigger: AnyExtTrigger);
    fn set_auto_conversion_mode(enabled: bool);
    fn set_length(length: u8);
    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime);
    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize);
    fn set_discontinuous_mode(enabled: bool);
    fn set_dma_config(value: Dmacfg);
    fn set_dma_enable(enabled: bool);
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

    fn set_discontinuous_mode(enabled: bool) {
        Self::regs().cfgr().modify(|reg| reg.set_jdiscen(enabled));
    }

    fn set_dma_config(value: Dmacfg) {
        Self::regs().cfgr().modify(|reg| reg.set_dmacfg(value));
    }

    fn set_dma_enable(enabled: bool) {
        Self::regs().cfgr().modify(|reg| {
            reg.set_dmaen(match enabled {
                true => Dmaen::ENABLE,
                false => Dmaen::DISABLE,
            })
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
