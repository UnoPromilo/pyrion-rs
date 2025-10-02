use crate::advanced_adc::trigger_edge::ExtTriggerEdge;

pub trait Trigger: Copy + Default {}

#[derive(Debug, Copy, Clone, Default)]
#[allow(dead_code)]
pub enum TriggerADC12 {
    #[default]
    Software,
    External(ExternalTriggerConversionSourceADC12, ExtTriggerEdge),
}

#[derive(Debug, Copy, Clone, Default)]
#[allow(dead_code)]
pub enum TriggerADC345 {
    #[default]
    Software,
    External(ExternalTriggerConversionSourceADC345, ExtTriggerEdge),
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExternalTriggerConversionSourceADC12 {
    T1_CC1 = 0,
    T1_CC2 = 1,
    T1_CC3 = 2,
    T2_CC2 = 3,
    T3_TRGO = 4,
    T4_CC4 = 5,
    EXT_IT11 = 6,
    T8_TRGO = 7,
    T8_TRGO2 = 8,
    T1_TRGO = 9,
    T1_TRGO2 = 10,
    T2_TRGO = 11,
    T4_TRGO = 12,
    T6_TRGO = 13,
    T15_TRGO = 14,
    T3_CC4 = 15,
    T20_TRGO = 16,
    T20_TRGO2 = 17,
    T20_CC1 = 18,
    T20_CC2 = 19,
    T20_CC3 = 20,
    HRTIM_ADC_TRG1 = 21,
    HRTIM_ADC_TRG3 = 22,
    HRTIM_ADC_TRG5 = 23,
    HRTIM_ADC_TRG6 = 24,
    HRTIM_ADC_TRG7 = 25,
    HRTIM_ADC_TRG8 = 26,
    HRTIM_ADC_TRG9 = 27,
    HRTIM_ADC_TRG10 = 28,
    LPTIM_OUT = 29,
    T7_TRGO = 30,
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExternalTriggerConversionSourceADC345 {
    T3_CC1 = 0,
    T2_CC3 = 1,
    T1_CC3 = 2,
    T8_CC1 = 3,
    T3_TRGO = 4,
    EXT_IT2 = 5,
    T4_CC1 = 6,
    T8_TRGO = 7,
    T8_TRGO2 = 8,
    T1_TRGO = 9,
    T1_TRGO2 = 10,
    T2_TRGO = 11,
    T4_TRGO = 12,
    T6_TRGO = 13,
    T15_TRGO = 14,
    T2_CC1 = 15,
    T20_TRGO = 16,
    T20_TRGO2 = 17,
    T20_CC1 = 18,
    HRTIM_ADC_TRG2 = 19,
    HRTIM_ADC_TRG4 = 20,
    HRTIM_ADC_TRG1 = 21,
    HRTIM_ADC_TRG3 = 22,
    HRTIM_ADC_TRG5 = 23,
    HRTIM_ADC_TRG6 = 24,
    HRTIM_ADC_TRG7 = 25,
    HRTIM_ADC_TRG8 = 26,
    HRTIM_ADC_TRG9 = 27,
    HRTIM_ADC_TRG10 = 28,
    LPTIM_OUT = 29,
    T7_TRGO = 30,
}

impl Trigger for TriggerADC12 {}
impl Trigger for TriggerADC345 {}
