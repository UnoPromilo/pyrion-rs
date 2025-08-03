use defmt::Format;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use shared::units::angle::{AngleAny, Electrical};
use shared::units::{Angle, Current};

#[derive(Debug, Format)]
pub struct PhaseCurrent {
    pub a: Current,
    pub b: Current,
    pub c: Current,
}

pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
}

pub struct Motor {
    pub power: Mutex<NoopRawMutex, Option<PhaseCurrent>>,
    pub angle: Mutex<NoopRawMutex, Option<ShaftData>>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            power: Mutex::new(None),
            angle: Mutex::new(None),
        }
    }
}
