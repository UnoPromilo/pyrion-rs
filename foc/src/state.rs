use defmt::Format;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Instant;
use shared::units::angle::{AngleAny, Electrical, Mechanical};
use shared::units::{Angle, Current, Direction, Velocity, Voltage};

#[derive(Debug, Format, Clone, Copy)]
pub struct PhaseCurrent {
    pub a: Current,
    pub b: Current,
    pub c: Current,
}

#[derive(Debug, Format, Clone, Copy)]
pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
    pub encoder_calibration: EncoderCalibrationConstants,
}

#[derive(Debug, Format, Clone, Copy, Default)]
pub struct EncoderCalibrationConstants {
    pub direction: Direction,
    pub offset: u16,
    pub pole_pairs: u16,
}

#[derive(Debug, Format, Clone, Copy)]
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

#[derive(Debug, Format, Clone, Copy)]
pub enum MotorState {
    Uninitialized,
    Initializing(InitializationState),
    Idle,
    Measuring(MeasurementState),
}

#[derive(Debug, Format, Clone, Copy)]
pub enum InitializationState {
    CalibratingCurrentSensor(CalibratingCurrentSensorState),
}

#[derive(Debug, Format, Clone, Copy)]
pub enum CalibratingCurrentSensorState {
    PhaseAPowered,
    PhaseBPowered,
    PhaseCPowered,
}

#[derive(Debug, Format, Clone, Copy)]
pub enum MeasurementState {
    Direction(Angle<Electrical>),
    MagneticPoles(Angle<Electrical>, u8),
    MagneticOffset(Angle<Electrical>),
}

#[derive(Debug, Format, Clone, Copy)]
pub enum ControlCommand {
    CalibrateEncoder,
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

#[derive(Debug, Format, Clone, Copy)]
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
