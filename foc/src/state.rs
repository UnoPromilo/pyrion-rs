use defmt::Format;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use shared::units::angle::{AngleAny, Electrical};
use shared::units::{Angle, Current};

#[derive(Debug, Format, Clone, Copy)]
pub struct PhaseCurrent {
    pub a: Current,
    pub b: Current,
    pub c: Current,
}

#[derive(Debug, Format, Clone, Copy)]
pub struct PhaseCurrentOffset {
    pub offset_a: i16,
    pub offset_b: i16,
    pub offset_c: i16,
}

impl PhaseCurrentOffset {
    fn zero() -> Self {
        Self {
            offset_a: 0,
            offset_b: 0,
            offset_c: 0,
        }
    }
}

#[derive(Debug, Format, Clone, Copy)]
pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
}

#[derive(Debug)]
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

#[derive(Debug, Copy, Clone)]
pub enum MotorState {
    Uninitialized,
    Initializing(InitializationState),
    Idle,
}

#[derive(Debug, Copy, Clone)]
pub enum InitializationState {
    CalibratingCurrentSensor(CalibratingCurrentSensorState),
}

#[derive(Debug, Copy, Clone)]
pub enum CalibratingCurrentSensorState {
    PhaseAPowered,
    PhaseBPowered,
    PhaseCPowered,
}

#[derive(Debug)]
pub struct Motor {
    pub current: Mutex<NoopRawMutex, Option<PhaseCurrent>>,
    pub current_calibration_data: Mutex<NoopRawMutex, PhaseCurrentOffset>,
    pub shaft: Mutex<NoopRawMutex, Option<ShaftData>>,
    pub state: Mutex<NoopRawMutex, MotorStateSnapshot>,
}

#[derive(Debug, Format)]
pub struct MotorSnapshot {
    pub current: Option<PhaseCurrent>,
    pub shaft: Option<ShaftData>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            current_calibration_data: Mutex::new(PhaseCurrentOffset::zero()),
            shaft: Mutex::new(None),
            state: Mutex::new(MotorStateSnapshot::new(MotorState::Uninitialized)),
        }
    }

    pub async fn snapshot(&self) -> MotorSnapshot {
        MotorSnapshot {
            current: self.current.lock().await.clone(),
            shaft: self.shaft.lock().await.clone(),
        }
    }
}
