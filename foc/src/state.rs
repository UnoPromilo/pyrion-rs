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
pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
}

#[derive(Debug)]
pub struct Motor {
    pub current: Mutex<NoopRawMutex, Option<PhaseCurrent>>,
    pub shaft: Mutex<NoopRawMutex, Option<ShaftData>>,
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
            shaft: Mutex::new(None),
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
