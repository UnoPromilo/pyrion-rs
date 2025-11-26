use units::{Angle, DutyCycle, ElectricCurrent, ElectricPotential};
pub struct FocInput {
    pub v_bus: ElectricPotential,
    pub angle: AngleSnapshot,
    pub u: ElectricCurrent,
    pub v: ElectricCurrent,
    pub w: ElectricCurrent,
}

pub struct AngleSnapshot {
    pub value: Angle,
    pub sin: f32,
    pub cos: f32,
}

pub struct FocOutput {
    pub u: DutyCycle,
    pub v: DutyCycle,
    pub w: DutyCycle,
}
