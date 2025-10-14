use units::{ElectricCurrent, ElectricPotential};

pub struct ControlSnapshot {
    pub phase_current: [ElectricCurrent; 3],
    pub bus_voltage: ElectricPotential,
}