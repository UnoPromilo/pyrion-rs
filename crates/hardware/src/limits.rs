use units::{ElectricCurrent, ElectricPotential};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BoardId {
    PyrionOvo,
    PyrionNullo,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoardLimits {
    pub max_bus_voltage: ElectricPotential,
    pub max_phase_current: ElectricCurrent,
}
