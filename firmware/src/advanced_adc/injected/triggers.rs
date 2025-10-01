use crate::advanced_adc::trigger_edge::ExternalTriggerEdge;

pub trait Trigger: Copy + Default + Into<AnyTriggerSource> {}

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
