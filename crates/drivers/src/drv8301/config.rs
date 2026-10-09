use drv8301_dd::{
    ControlRegister1FieldSet, ControlRegister2FieldSet, GateCurrent as ChipGateCurrent, OcpMode,
    OctwMode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GateCurrent {
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[expect(
    clippy::enum_variant_names,
    reason = "the unit is part of each threshold name"
)]
pub enum VdsThreshold {
    Vds060Mv,
    Vds068Mv,
    Vds076Mv,
    Vds086Mv,
    Vds097Mv,
    Vds109Mv,
    Vds123Mv,
    Vds138Mv,
    Vds155Mv,
    Vds175Mv,
    Vds197Mv,
    Vds222Mv,
    Vds250Mv,
    Vds282Mv,
    Vds317Mv,
    Vds358Mv,
    Vds403Mv,
    Vds454Mv,
    Vds511Mv,
    Vds576Mv,
    Vds648Mv,
    Vds730Mv,
    Vds822Mv,
    Vds926Mv,
    Vds1043Mv,
    Vds1175Mv,
    Vds1324Mv,
    Vds1491Mv,
    Vds1679Mv,
    Vds1892Mv,
    Vds2131Mv,
    Vds2400Mv,
}

impl VdsThreshold {
    pub const ALL: [Self; 32] = [
        Self::Vds060Mv,
        Self::Vds068Mv,
        Self::Vds076Mv,
        Self::Vds086Mv,
        Self::Vds097Mv,
        Self::Vds109Mv,
        Self::Vds123Mv,
        Self::Vds138Mv,
        Self::Vds155Mv,
        Self::Vds175Mv,
        Self::Vds197Mv,
        Self::Vds222Mv,
        Self::Vds250Mv,
        Self::Vds282Mv,
        Self::Vds317Mv,
        Self::Vds358Mv,
        Self::Vds403Mv,
        Self::Vds454Mv,
        Self::Vds511Mv,
        Self::Vds576Mv,
        Self::Vds648Mv,
        Self::Vds730Mv,
        Self::Vds822Mv,
        Self::Vds926Mv,
        Self::Vds1043Mv,
        Self::Vds1175Mv,
        Self::Vds1324Mv,
        Self::Vds1491Mv,
        Self::Vds1679Mv,
        Self::Vds1892Mv,
        Self::Vds2131Mv,
        Self::Vds2400Mv,
    ];

    pub const fn code(self) -> u8 {
        self as u8
    }

    pub fn validate_max_code(self, max_code: u8) -> Result<Self, ConfigurationError> {
        if max_code >= Self::ALL.len() as u8 {
            return Err(ConfigurationError::InvalidThresholdCode(max_code));
        }
        if self.code() > max_code {
            return Err(ConfigurationError::ThresholdExceedsMaximum {
                threshold: self,
                max_code,
            });
        }
        Ok(self)
    }
}

impl TryFrom<u8> for VdsThreshold {
    type Error = ConfigurationError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(code as usize)
            .copied()
            .ok_or(ConfigurationError::InvalidThresholdCode(code))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CurrentLimitOffTime {
    CycleByCycle,
    Fixed64Us,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OvercurrentPolicy {
    CurrentLimit {
        threshold: VdsThreshold,
        off_time: CurrentLimitOffTime,
    },
    LatchShutdown {
        threshold: VdsThreshold,
    },
}

impl OvercurrentPolicy {
    pub fn validate_max_threshold_code(self, max_code: u8) -> Result<Self, ConfigurationError> {
        let threshold = match self {
            Self::CurrentLimit { threshold, .. } | Self::LatchShutdown { threshold } => threshold,
        };
        threshold.validate_max_code(max_code)?;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Drv8301Configuration {
    pub gate_current: GateCurrent,
    pub overcurrent: OvercurrentPolicy,
}

impl Drv8301Configuration {
    pub fn validate_max_threshold_code(self, max_code: u8) -> Result<Self, ConfigurationError> {
        self.overcurrent.validate_max_threshold_code(max_code)?;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControlReadback {
    pub gate_current: GateCurrent,
    pub overcurrent: OvercurrentPolicy,
    pub raw_control_1: u16,
    pub raw_control_2: u16,
}

impl ControlReadback {
    pub fn validate_max_threshold_code(self, max_code: u8) -> Result<Self, ConfigurationError> {
        self.overcurrent.validate_max_threshold_code(max_code)?;
        Ok(self)
    }

    pub fn configuration(self) -> Drv8301Configuration {
        Drv8301Configuration {
            gate_current: self.gate_current,
            overcurrent: self.overcurrent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigurationError {
    InvalidThresholdCode(u8),
    ThresholdExceedsMaximum {
        threshold: VdsThreshold,
        max_code: u8,
    },
    ReservedGateCurrent,
    UnsupportedOvercurrentMode(u8),
    ThreePwmMode,
    GateResetActive,
    ReservedControl2Bits(u16),
    ReservedOctwMode,
    DcCalibrationActive,
    OffTimeWithoutCurrentLimit,
}

impl ControlReadback {
    pub fn decode(raw_control_1: u16, raw_control_2: u16) -> Result<Self, ConfigurationError> {
        let control_1 = ControlRegister1FieldSet::from(raw_control_1.to_be_bytes());
        let control_2 = ControlRegister2FieldSet::from(raw_control_2.to_be_bytes());
        let gate_current = match control_1.gate_current() {
            ChipGateCurrent::High => GateCurrent::High,
            ChipGateCurrent::Medium => GateCurrent::Medium,
            ChipGateCurrent::Low => GateCurrent::Low,
            ChipGateCurrent::Reserved => return Err(ConfigurationError::ReservedGateCurrent),
        };
        if control_1.gate_reset() {
            return Err(ConfigurationError::GateResetActive);
        }
        if control_1.pwm_mode() {
            return Err(ConfigurationError::ThreePwmMode);
        }
        if control_2.reserved() != 0 {
            return Err(ConfigurationError::ReservedControl2Bits(
                control_2.reserved().into(),
            ));
        }
        if control_2.octw_mode() == OctwMode::OcOnlyReserved {
            return Err(ConfigurationError::ReservedOctwMode);
        }
        if control_2.dc_cal_ch_1() || control_2.dc_cal_ch_2() {
            return Err(ConfigurationError::DcCalibrationActive);
        }
        let threshold = VdsThreshold::try_from(u8::from(control_1.oc_adj_set()))?;
        let overcurrent = match control_1.ocp_mode() {
            OcpMode::CurrentLimit => OvercurrentPolicy::CurrentLimit {
                threshold,
                off_time: if !control_2.oc_toff() {
                    CurrentLimitOffTime::CycleByCycle
                } else {
                    CurrentLimitOffTime::Fixed64Us
                },
            },
            OcpMode::OcLatchShutdown if !control_2.oc_toff() => {
                OvercurrentPolicy::LatchShutdown { threshold }
            }
            OcpMode::OcLatchShutdown => return Err(ConfigurationError::OffTimeWithoutCurrentLimit),
            mode => return Err(ConfigurationError::UnsupportedOvercurrentMode(mode.into())),
        };
        Ok(Self {
            gate_current,
            overcurrent,
            raw_control_1,
            raw_control_2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_physical_thresholds_round_trip_and_board_ceiling_is_explicit() {
        let board_max_code = 15;
        for (code, threshold) in VdsThreshold::ALL.into_iter().enumerate() {
            assert_eq!(threshold.code(), code as u8);
            assert_eq!(VdsThreshold::try_from(code as u8), Ok(threshold));
            let readback = ControlReadback::decode((code as u16) << 6, 0).unwrap();
            assert_eq!(
                readback.configuration(),
                Drv8301Configuration {
                    gate_current: GateCurrent::High,
                    overcurrent: readback.overcurrent,
                }
            );
            assert_eq!(
                readback.overcurrent,
                OvercurrentPolicy::CurrentLimit {
                    threshold,
                    off_time: CurrentLimitOffTime::CycleByCycle,
                }
            );
            let ceiling_result = readback.validate_max_threshold_code(board_max_code);
            if code <= board_max_code as usize {
                assert_eq!(ceiling_result, Ok(readback));
                assert_eq!(threshold.validate_max_code(board_max_code), Ok(threshold));
            } else {
                let error = ConfigurationError::ThresholdExceedsMaximum {
                    threshold,
                    max_code: board_max_code,
                };
                assert_eq!(ceiling_result, Err(error));
                assert_eq!(threshold.validate_max_code(board_max_code), Err(error));
            }
            assert_eq!(readback.validate_max_threshold_code(31), Ok(readback));
            assert_eq!(
                readback
                    .configuration()
                    .validate_max_threshold_code(27)
                    .is_ok(),
                code <= 27
            );
            let latch = OvercurrentPolicy::LatchShutdown { threshold };
            assert_eq!(
                latch.validate_max_threshold_code(board_max_code),
                readback
                    .validate_max_threshold_code(board_max_code)
                    .map(|_| latch)
            );
        }
        for code in [32, 255] {
            assert_eq!(
                VdsThreshold::try_from(code),
                Err(ConfigurationError::InvalidThresholdCode(code))
            );
        }
        assert_eq!(
            VdsThreshold::Vds060Mv.validate_max_code(32),
            Err(ConfigurationError::InvalidThresholdCode(32))
        );
    }

    #[test]
    fn decodes_each_supported_gate_current_and_policy() {
        for (code, current) in [GateCurrent::High, GateCurrent::Medium, GateCurrent::Low]
            .into_iter()
            .enumerate()
        {
            for (mode, toff) in [(0, 0), (0, 0x40), (1, 0)] {
                let control1 = (27 << 6) | (mode << 4) | code as u16;
                let result = ControlReadback::decode(control1, toff | 0x0a).unwrap();
                assert_eq!(result.gate_current, current);
                assert_eq!(result.raw_control_2, toff | 0x0a);
                match (mode, toff, result.overcurrent) {
                    (
                        0,
                        0,
                        OvercurrentPolicy::CurrentLimit {
                            threshold,
                            off_time: CurrentLimitOffTime::CycleByCycle,
                        },
                    )
                    | (
                        0,
                        0x40,
                        OvercurrentPolicy::CurrentLimit {
                            threshold,
                            off_time: CurrentLimitOffTime::Fixed64Us,
                        },
                    )
                    | (1, 0, OvercurrentPolicy::LatchShutdown { threshold }) => {
                        assert_eq!(threshold, VdsThreshold::Vds1491Mv);
                    }
                    _ => panic!("incorrect decoded policy"),
                }
            }
        }
    }

    #[test]
    fn unsupported_and_reserved_readbacks_fail_explicitly() {
        for (control1, control2, error) in [
            (3, 0, ConfigurationError::ReservedGateCurrent),
            (4, 0, ConfigurationError::GateResetActive),
            (8, 0, ConfigurationError::ThreePwmMode),
            (0x20, 0, ConfigurationError::UnsupportedOvercurrentMode(2)),
            (0x30, 0, ConfigurationError::UnsupportedOvercurrentMode(3)),
            (0x10, 0x40, ConfigurationError::OffTimeWithoutCurrentLimit),
            (0, 0x80, ConfigurationError::ReservedControl2Bits(1)),
            (0, 0x03, ConfigurationError::ReservedOctwMode),
            (0, 0x10, ConfigurationError::DcCalibrationActive),
            (0, 0x20, ConfigurationError::DcCalibrationActive),
        ] {
            assert_eq!(ControlReadback::decode(control1, control2), Err(error));
        }
    }
}
