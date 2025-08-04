use crate::state::PhaseCurrent;
use crate::Motor;
use hardware_abstraction::current_sensor;
use hardware_abstraction::current_sensor::CurrentReader;

pub trait CurrentMeasurement {
    async fn update_current<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error>;
}

impl CurrentMeasurement for Motor {
    async fn update_current<R: CurrentReader>(
        &self,
        current_reader: &mut R,
    ) -> Result<(), R::Error> {
        let result = current_reader.read().await?;
        let phase_current = match result {
            current_sensor::Output::TwoPhases(a, b) => {
                let c = -a - b;
                PhaseCurrent { a, b, c }
            }
            current_sensor::Output::ThreePhases(a, b, c) => {
                //TODO add logic about calculating third current if low duty cycle
                PhaseCurrent { a, b, c }
            }
        };
        let mut power = self.current.lock().await;
        *power = Some(phase_current);
        Ok(())
    }
}
