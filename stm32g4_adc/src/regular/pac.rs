use crate::advanced_adc::EndOfConversionSignal;
use crate::advanced_adc::injected::AnyExtTrigger;
use crate::advanced_adc::pac_instance::PacInstance;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::{Dmacfg, Dmaen, Exten, SampleTime};

pub trait ModifyPac {
    fn set_software_trigger();
    fn set_ext_trigger(trigger: AnyExtTrigger);
    fn set_discontinuous_mode(enabled: bool);
    fn set_continuous_mode(enabled: bool);
    fn set_length(length: u8);
    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime);
    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize);
    fn set_end_of_conversion_signal(val: EndOfConversionSignal);
    fn clear_end_of_conversion_signal(val: EndOfConversionSignal);
    fn set_dma_config(value: Dmacfg);
    fn set_dma_enable(enabled: bool);

    fn start();
    fn stop();
}

impl<T: PacInstance> ModifyPac for T {
    fn set_software_trigger() {
        Self::regs().cfgr().modify(|reg| {
            reg.set_exten(Exten::DISABLED);
            reg.set_extsel(0);
        })
    }

    fn set_ext_trigger(trigger: AnyExtTrigger) {
        match trigger {
            AnyExtTrigger::ADC12(source, edge) => Self::regs().cfgr().modify(|reg| {
                reg.set_exten(edge.into());
                reg.set_extsel(source.into());
            }),
            AnyExtTrigger::ADC345(source, edge) => Self::regs().cfgr().modify(|reg| {
                reg.set_exten(edge.into());
                reg.set_extsel(source.into());
            }),
        }
    }

    fn set_discontinuous_mode(enabled: bool) {
        Self::regs().cfgr().modify(|regs| regs.set_discen(enabled));
    }

    fn set_continuous_mode(enabled: bool) {
        Self::regs().cfgr().modify(|regs| regs.set_cont(enabled));
    }

    fn set_length(length: u8) {
        todo!()
    }

    fn set_channel_sample_time<C>(channel: &AnyAdcChannel<C>, sample_time: SampleTime) {
        todo!()
    }

    fn register_channel<C>(channel: &AnyAdcChannel<C>, index: usize) {
        todo!()
    }

    fn set_end_of_conversion_signal(val: EndOfConversionSignal) {
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

    fn clear_end_of_conversion_signal(val: EndOfConversionSignal) {
        Self::regs().isr().modify(|reg| match val {
            EndOfConversionSignal::None => {}
            EndOfConversionSignal::Single => {
                reg.set_eoc(true);
            }
            EndOfConversionSignal::Sequence => {
                reg.set_eos(true);
            }
            EndOfConversionSignal::Both => {
                reg.set_eoc(true);
                reg.set_eos(true);
            }
        });
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

    fn stop() {
        todo!()
    }
}
