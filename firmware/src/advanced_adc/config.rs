use stm32_metapac::adc::vals::{Dmacfg, Ovrmod, Res, Rovsm, Trovs};

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub resolution: Res,
    pub align_left: bool,
    pub gain_compensation: GainCompensation,
    pub end_of_conversion_signal_regular: EndOfConversionSignal,
    pub end_of_conversion_signal_injected: EndOfConversionSignal,
    pub dma_config: Dmacfg,
    pub overrun_mode: Ovrmod,
    pub oversampling_config: OversamplingConfig,
    pub enable_low_power_auto_wait: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct OversamplingConfig {
    pub shift: OversamplingShift,
    pub ratio: OversamplingRatio,
    pub regular_mode: Rovsm,
    pub triggered_mode: Trovs,
    pub enable_injected_oversampling: bool,
    pub enable_regular_oversampling: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Res::BITS12,
            align_left: false,
            gain_compensation: GainCompensation::zero(),
            end_of_conversion_signal_regular: EndOfConversionSignal::None,
            end_of_conversion_signal_injected: EndOfConversionSignal::None,
            dma_config: Dmacfg::ONE_SHOT,
            overrun_mode: Ovrmod::PRESERVE,
            oversampling_config: Default::default(),
            enable_low_power_auto_wait: false,
        }
    }
}

impl Default for OversamplingConfig {
    fn default() -> Self {
        Self {
            shift: OversamplingShift::None,
            ratio: OversamplingRatio::Ratio2,
            regular_mode: Rovsm::CONTINUED,
            triggered_mode: Trovs::AUTOMATIC,
            enable_injected_oversampling: false,
            enable_regular_oversampling: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GainCompensation(pub u16);

#[derive(Debug, Clone, Copy)]
pub enum EndOfConversionSignal {
    None,
    Single,
    Sequence,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub enum OversamplingRatio {
    Ratio2 = 0,
    Ratio4 = 1,
    Ratio8 = 2,
    Ratio16 = 3,
    Ratio32 = 4,
    Ratio64 = 5,
    Ratio128 = 6,
    Ratio256 = 7,
}

#[derive(Debug, Clone, Copy)]
pub enum OversamplingShift {
    None = 0,
    Shift1 = 1,
    Shift2 = 2,
    Shift3 = 3,
    Shift4 = 4,
    Shift5 = 5,
    Shift6 = 6,
    Shift7 = 7,
    Shift8 = 8,
}

impl From<OversamplingRatio> for u8 {
    #[inline(always)]
    fn from(val: OversamplingRatio) -> Self {
        val as u8
    }
}

impl From<OversamplingShift> for u8 {
    #[inline(always)]
    fn from(val: OversamplingShift) -> Self {
        val as u8
    }
}

impl GainCompensation {
    pub const fn from_bytes(val: u16) -> Self {
        const MAX: u16 = 0x3FFF;
        assert!(
            val <= MAX,
            "The gain compensation value must be in the range from 0 to 0x3FFF"
        );
        Self(val)
    }

    #[allow(dead_code)]
    pub const fn from(val: f32) -> Self {
        const MAX: f32 = 3.999756;
        const SCALE: u16 = 0x3FFF;

        assert!(
            val >= 0.0 && val <= MAX,
            "The gain compensation value must be in the range from 0 to 3.999756"
        );
        Self::from_bytes((val / MAX * SCALE as f32) as u16)
    }

    pub const fn zero() -> Self {
        Self::from_bytes(0)
    }
}
