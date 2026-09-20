use crate::RawInverterValues;

pub trait InverterOutput {
    fn max_duty(&self) -> u32;
    fn disable_outputs(&mut self);
    fn write_phase_duties(&mut self, duties: RawInverterValues);
    fn enable_outputs(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InhibitCause {
    Startup,
    Fault,
    FirmwareUpdate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputState {
    Inhibited(InhibitCause),
    ArmedSafe,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidDuty {
    pub requested: RawInverterValues,
    pub max_duty: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DriveError {
    NotArmed,
    InvalidDuty(InvalidDuty),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ArmError {
    AlreadyEnabled,
    FirmwareUpdatePending,
}

pub struct SafeOutput<D: InverterOutput> {
    driver: D,
    state: OutputState,
}

impl<D: InverterOutput> SafeOutput<D> {
    pub fn new(mut driver: D) -> Self {
        driver.disable_outputs();
        driver.write_phase_duties(RawInverterValues::ZERO);
        Self {
            driver,
            state: OutputState::Inhibited(InhibitCause::Startup),
        }
    }

    pub fn max_duty(&self) -> u32 {
        self.driver.max_duty()
    }

    pub fn state(&self) -> OutputState {
        self.state
    }

    pub fn arm_after_preflight(&mut self) -> Result<(), ArmError> {
        match self.state {
            OutputState::Inhibited(InhibitCause::FirmwareUpdate) => {
                Err(ArmError::FirmwareUpdatePending)
            }
            OutputState::Inhibited(_) => {
                self.state = OutputState::ArmedSafe;
                Ok(())
            }
            OutputState::ArmedSafe => Ok(()),
            OutputState::Enabled => Err(ArmError::AlreadyEnabled),
        }
    }

    pub fn enter_armed_safe(&mut self) {
        if matches!(self.state, OutputState::Inhibited(_)) {
            return;
        }

        self.driver.disable_outputs();
        self.driver.write_phase_duties(RawInverterValues::ZERO);
        self.state = OutputState::ArmedSafe;
    }

    pub fn inhibit(&mut self, cause: InhibitCause) {
        self.driver.disable_outputs();
        self.driver.write_phase_duties(RawInverterValues::ZERO);
        self.state = OutputState::Inhibited(cause);
    }

    pub fn drive(&mut self, duties: RawInverterValues) -> Result<(), DriveError> {
        if matches!(self.state, OutputState::Inhibited(_)) {
            return Err(DriveError::NotArmed);
        }

        let max_duty = self.driver.max_duty();
        if duties.u > max_duty || duties.v > max_duty || duties.w > max_duty {
            let invalid_duty = InvalidDuty {
                requested: duties,
                max_duty,
            };
            self.inhibit(InhibitCause::Fault);
            return Err(DriveError::InvalidDuty(invalid_duty));
        }

        self.driver.write_phase_duties(duties);
        if self.state == OutputState::ArmedSafe {
            self.driver.enable_outputs();
            self.state = OutputState::Enabled;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Operation {
        Disable,
        Write(RawInverterValues),
        Enable,
    }

    struct FakeDriver {
        max_duty: u32,
        operations: Vec<Operation>,
    }

    impl FakeDriver {
        fn new(max_duty: u32) -> Self {
            Self {
                max_duty,
                operations: Vec::new(),
            }
        }
    }

    impl InverterOutput for FakeDriver {
        fn max_duty(&self) -> u32 {
            self.max_duty
        }

        fn disable_outputs(&mut self) {
            self.operations.push(Operation::Disable);
        }

        fn write_phase_duties(&mut self, duties: RawInverterValues) {
            self.operations.push(Operation::Write(duties));
        }

        fn enable_outputs(&mut self) {
            self.operations.push(Operation::Enable);
        }
    }

    #[test]
    fn startup_disables_outputs_and_writes_safe_duties() {
        let output = SafeOutput::new(FakeDriver::new(1000));

        assert_eq!(
            output.state(),
            OutputState::Inhibited(InhibitCause::Startup)
        );
        assert_eq!(
            output.driver.operations,
            [
                Operation::Disable,
                Operation::Write(RawInverterValues::ZERO)
            ]
        );
    }

    #[test]
    fn drive_is_rejected_until_explicitly_armed() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 100,
            v: 200,
            w: 300,
        };

        assert_eq!(output.drive(duties), Err(DriveError::NotArmed));
        assert_eq!(
            output.state(),
            OutputState::Inhibited(InhibitCause::Startup)
        );
        assert!(output.driver.operations.is_empty());
    }

    #[test]
    fn first_armed_drive_writes_fresh_duties_before_enable() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.arm_after_preflight().unwrap();
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 100,
            v: 200,
            w: 300,
        };

        output.drive(duties).unwrap();

        assert_eq!(output.state(), OutputState::Enabled);
        assert_eq!(
            output.driver.operations,
            [Operation::Write(duties), Operation::Enable]
        );
    }

    #[test]
    fn repeated_drive_updates_without_reenabling() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.arm_after_preflight().unwrap();
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 400,
            v: 500,
            w: 600,
        };

        output.drive(duties).unwrap();

        assert_eq!(output.driver.operations, [Operation::Write(duties)]);
    }

    #[test]
    fn invalid_duty_disables_before_writing_safe_duties() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.arm_after_preflight().unwrap();
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.driver.operations.clear();

        let result = output.drive(RawInverterValues {
            u: 1001,
            v: 0,
            w: 0,
        });

        assert_eq!(
            result,
            Err(DriveError::InvalidDuty(InvalidDuty {
                requested: RawInverterValues {
                    u: 1001,
                    v: 0,
                    w: 0,
                },
                max_duty: 1000,
            }))
        );
        assert_eq!(output.state(), OutputState::Inhibited(InhibitCause::Fault));
        assert_eq!(
            output.driver.operations,
            [
                Operation::Disable,
                Operation::Write(RawInverterValues::ZERO)
            ]
        );
    }

    #[test]
    fn developer_stop_keeps_the_output_armed() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.arm_after_preflight().unwrap();
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.enter_armed_safe();
        assert_eq!(output.state(), OutputState::ArmedSafe);
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 300,
            v: 200,
            w: 100,
        };

        output.drive(duties).unwrap();

        assert_eq!(
            output.driver.operations,
            [Operation::Write(duties), Operation::Enable]
        );
    }

    #[test]
    fn inhibition_requires_an_explicit_rearm() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.arm_after_preflight().unwrap();
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.inhibit(InhibitCause::Fault);
        output.driver.operations.clear();

        assert_eq!(
            output.drive(RawInverterValues {
                u: 300,
                v: 200,
                w: 100,
            }),
            Err(DriveError::NotArmed)
        );
        assert!(output.driver.operations.is_empty());

        output.arm_after_preflight().unwrap();
        output
            .drive(RawInverterValues {
                u: 300,
                v: 200,
                w: 100,
            })
            .unwrap();
        assert_eq!(
            output.driver.operations,
            [
                Operation::Write(RawInverterValues {
                    u: 300,
                    v: 200,
                    w: 100,
                }),
                Operation::Enable
            ]
        );
    }

    #[test]
    fn firmware_update_inhibition_cannot_be_rearmed() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.inhibit(InhibitCause::FirmwareUpdate);

        assert_eq!(
            output.arm_after_preflight(),
            Err(ArmError::FirmwareUpdatePending)
        );
        assert_eq!(
            output.state(),
            OutputState::Inhibited(InhibitCause::FirmwareUpdate)
        );
    }
}
