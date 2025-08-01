use defmt::{Format, warn};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::CurrentReader;
use shared::units::Current;

#[derive(Debug, Format)]
pub struct PhaseCurrent {
    a: Current,
    b: Current,
    c: Current,
}

pub struct Motor {
    power: Mutex<NoopRawMutex, Option<PhaseCurrent>>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            power: Mutex::new(None),
        }
    }

    pub async fn update_power(&self, current_sensor: &mut impl CurrentReader) {
        let result = current_sensor.read().await;

        match result {
            Err(_) => {
                // TODO add error flag to Motor to keep the errors in state?
                warn!("Error during reading current");
            }
            Ok(value) => {
                let phase_current = match value {
                    current_sensor::Output::TwoPhases(a, b) => {
                        let c = -a - b;
                        PhaseCurrent { a, b, c }
                    }
                    current_sensor::Output::ThreePhases(a, b, c) => {
                        //TODO add logic about calculating third current if low duty cycle
                        PhaseCurrent { a, b, c }
                    }
                };
                let mut power = self.power.lock().await;
                *power = Some(phase_current);
            }
        }
    }
}
