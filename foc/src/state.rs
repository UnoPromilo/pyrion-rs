use defmt::Format;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use shared::units::angle::{AngleAny, Electrical, Mechanical};
use shared::units::{Angle, Current, Velocity, Voltage};

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
pub enum ControlCommand {
    None,
    Voltage(Voltage),
    Torque(Current),
    Velocity(Velocity),
    Position(Angle<Mechanical>),
}

#[derive(Debug)]
pub struct Motor {
    pub current: Mutex<CriticalSectionRawMutex, Option<PhaseCurrent>>,
    pub shaft: Mutex<CriticalSectionRawMutex, Option<ShaftData>>,
    pub state: Mutex<CriticalSectionRawMutex, MotorStateSnapshot>,
    pub command: Mutex<CriticalSectionRawMutex, ControlCommand>,
}

#[derive(Debug, Format, Clone, Copy)]
pub struct MotorSnapshot {
    pub current: Option<PhaseCurrent>,
    pub shaft: Option<ShaftData>,
    pub state: MotorStateSnapshot,
    pub command: ControlCommand,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            shaft: Mutex::new(None),
            state: Mutex::new(MotorStateSnapshot::new(MotorState::Uninitialized)),
            command: Mutex::new(ControlCommand::None),
        }
    }

    pub async fn snapshot(&self) -> MotorSnapshot {
        MotorSnapshot {
            current: self.current.lock().await.clone(),
            shaft: self.shaft.lock().await.clone(),
            state: self.state.lock().await.clone(),
            command: self.command.lock().await.clone(),
        }
    }
}
