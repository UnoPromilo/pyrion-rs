use crate::adc::pac_instance::PacInstance;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Exten, SampleTime};

pub trait ModifyPac {
    fn set_tim1_trigger();
    fn set_auto_conversion_mode(enabled: bool);
    fn set_length(length: u8);
    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime);
    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize);
    fn set_discontinuous_mode(enabled: bool);
    fn enable_jeos_interrupt();
    fn clear_jeos();
    fn start();
}

pub trait ReadPac {
    fn read_value(index: usize) -> u16;
}

impl<T: PacInstance> ModifyPac for T {
    fn set_tim1_trigger() {
        Self::regs().jsqr().modify(|reg| {
            reg.set_jexten(Exten::RISING_EDGE);
            reg.set_jextsel(0);
        });
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

    fn enable_jeos_interrupt() {
        Self::regs().ier().modify(|reg| {
            reg.set_jeosie(true);
            reg.set_jeocie(false);
        });
    }

    fn clear_jeos() {
        Self::regs().isr().modify(|reg| reg.set_jeos(true));
    }

    fn start() {
        Self::regs().cr().modify(|regs| regs.set_jadstart(true));
    }
}

impl<T: PacInstance> ReadPac for T {
    fn read_value(index: usize) -> u16 {
        Self::regs().jdr(index).read().jdata()
    }
}
