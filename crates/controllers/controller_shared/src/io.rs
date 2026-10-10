#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawSnapshot {
    pub i_u: u16,
    pub i_v: u16,
    pub i_w: u16,

    pub v_u: u16,
    pub v_v: u16,
    pub v_w: u16,

    pub v_ref: u16,

    pub max_duty: u32,

    pub angle: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawInverterValues {
    pub u: u32,
    pub v: u32,
    pub w: u32,
}

impl RawInverterValues {
    pub const ZERO: Self = Self { u: 0, v: 0, w: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControlFault {
    InvalidMeasurement,
    InvalidControllerOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControlOutput {
    Safe,
    Drive(RawInverterValues),
    Fault(ControlFault),
}
