use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Instant;
use shared::units::angle::{AngleAny, Electrical, Mechanical};
use shared::units::{Angle, Current, Velocity, Voltage};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct PhaseCurrent {
    pub a: Current,
    pub b: Current,
    pub c: Current,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
    pub shaft_calibration: ShaftCalibrationConstants,
}

#[derive(Debug, Clone, Copy)]
pub struct ShaftCalibrationConstants {
    pub offset: Angle<Electrical>,
    pub pole_pairs: i16,
}

impl Default for ShaftCalibrationConstants {
    fn default() -> Self {
        Self {
            offset: Angle::zero(),
            pole_pairs: 1,
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ShaftCalibrationConstants {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ShaftCalibrationConstants {{ pole_pairs: {}, offset: {} }}",
            self.pole_pairs,
            self.offset,
        );
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct MotorStateSnapshot {
    pub state: MotorState,
    pub state_set_at: Instant,
}

impl MotorStateSnapshot {
    pub fn new(state: MotorState) -> Self {
        Self {
            state,
            state_set_at: Instant::now(),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum MotorState {
    Uninitialized,
    Initializing(InitializationState),
    Idle,
    Powered(Powered),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum InitializationState {
    CalibratingCurrentSensor(CalibratingCurrentSensorState),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum CalibratingCurrentSensorState {
    PhaseAPowered,
    PhaseBPowered,
    PhaseCPowered,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum Powered {
    EncoderCalibration(EncoderCalibrationState),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum EncoderCalibrationState {
    WarmUp(Angle<Electrical>),
    Measuring(Angle<Electrical>, u8),
    Return(Angle<Electrical>, u8),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum ControlCommand {
    CalibrateShaft,
    SetTargetZero,
    SetTargetVoltage(Voltage),
    SetTargetTorque(Current),
    SetTargetVelocity(Velocity),
    SetTargetPosition(Angle<Mechanical>),
}

pub struct Motor {
    pub current: Mutex<CriticalSectionRawMutex, Option<PhaseCurrent>>,
    pub shaft: Mutex<CriticalSectionRawMutex, Option<ShaftData>>,
    pub state: Mutex<CriticalSectionRawMutex, MotorStateSnapshot>,
    pub command: Signal<CriticalSectionRawMutex, ControlCommand>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct MotorSnapshot {
    pub current: Option<PhaseCurrent>,
    pub shaft: Option<ShaftData>,
    pub state: MotorStateSnapshot,
    pub command: Option<ControlCommand>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            shaft: Mutex::new(None),
            state: Mutex::new(MotorStateSnapshot::new(MotorState::Uninitialized)),
            command: Signal::new(),
        }
    }

    pub async fn snapshot(&self) -> MotorSnapshot {
        let copied_command = {
            let value = self.command.try_take();
            if let Some(value) = value {
                self.command.signal(value);
            }
            value
        };
        MotorSnapshot {
            current: self.current.lock().await.clone(),
            shaft: self.shaft.lock().await.clone(),
            state: self.state.lock().await.clone(),
            command: copied_command,
        }
    }
}
