use crate::state::CalibratingCurrentSensorState;
use hardware_abstraction::current_sensor::RawOutput;

#[derive(Default)]
struct ChannelAccumulator {
    sum: u64,
    count: u64,
}

#[derive(Default)]
pub struct CalibrationAccumulator {
    a: ChannelAccumulator,
    b: ChannelAccumulator,
    c: ChannelAccumulator,
}

impl CalibrationAccumulator {
    pub fn update(&mut self, state: CalibratingCurrentSensorState, raw_output: RawOutput) {
        match state {
            CalibratingCurrentSensorState::PhaseAPowered => {
                let a = match raw_output {
                    RawOutput::TwoPhases(a, _) => a,
                    RawOutput::ThreePhases(a, _, _) => a,
                };
                self.a.add(a);
            }
            CalibratingCurrentSensorState::PhaseBPowered => {
                let b = match raw_output {
                    RawOutput::TwoPhases(_, b) => b,
                    RawOutput::ThreePhases(_, b, _) => b,
                };
                self.b.add(b);
            }
            CalibratingCurrentSensorState::PhaseCPowered => {
                let c = match raw_output {
                    RawOutput::ThreePhases(_, _, c) => c,
                    _ => 0,
                };
                self.c.add(c);
            }
        }
    }

    pub fn finalize(self) -> (u16, u16, u16) {
        (self.a.average(), self.b.average(), self.c.average())
    }
}

impl ChannelAccumulator {
    fn add(&mut self, value: u16) {
        self.sum += value as u64;
        self.count += 1;
    }

    fn average(&self) -> u16 {
        if self.count > 0 {
            (self.sum / self.count) as u16
        } else {
            u16::MAX / 2
        }
    }
}
