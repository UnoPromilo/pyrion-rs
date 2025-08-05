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
pub struct PhaseCurrentCalibrationData {
    pub offset_zero_a: i16,
    pub offset_zero_b: i16,
    pub offset_zero_c: i16,
}

impl PhaseCurrentCalibrationData {
    fn zero() -> Self {
        Self {
            offset_zero_a: 0,
            offset_zero_b: 0,
            offset_zero_c: 0,
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
pub struct MotorStateEnvelope {
    pub state: MotorState,
    pub state_set_at: Instant,
}

impl MotorStateEnvelope {
    pub fn new(state: MotorState) -> Self {
        Self {
            state,
            state_set_at: Instant::now(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MotorState {
    NotInitialized,
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
    pub current_calibration_data: Mutex<NoopRawMutex, PhaseCurrentCalibrationData>,
    pub shaft: Mutex<NoopRawMutex, Option<ShaftData>>,
    pub state: Mutex<NoopRawMutex, MotorStateEnvelope>,
}

#[derive(Debug, Format)]
pub struct MotorFrozen {
    pub current: Option<PhaseCurrent>,
    pub shaft: Option<ShaftData>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            current_calibration_data: Mutex::new(PhaseCurrentCalibrationData::zero()),
            shaft: Mutex::new(None),
            state: Mutex::new(MotorStateEnvelope::new(MotorState::NotInitialized)),
        }
    }

    /// Returns frozen copy of the current state.
    pub async fn freeze(&self) -> MotorFrozen {
        MotorFrozen {
            current: self.current.lock().await.clone(),
            shaft: self.shaft.lock().await.clone(),
        }
    }
}
