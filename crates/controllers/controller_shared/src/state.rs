use portable_atomic::{AtomicU16, AtomicU32};
use units::AtomicUnit;

pub struct State {
    pub foc_loop_frequency: AtomicU32,
    pub encoder_loop_frequency: AtomicU32,
    pub last_foc_loop_time_us: AtomicU16,
    pub cpu_temp: AtomicUnit<units::ThermodynamicTemperature>,
    pub i_u: AtomicUnit<units::ElectricCurrent>,
    pub i_v: AtomicUnit<units::ElectricCurrent>,
    pub i_w: AtomicUnit<units::ElectricCurrent>,
    pub v_bus: AtomicUnit<units::ElectricPotential>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub const fn new() -> Self {
        Self {
            foc_loop_frequency: AtomicU32::new(0),
            encoder_loop_frequency: AtomicU32::new(0),
            last_foc_loop_time_us: AtomicU16::new(0),
            cpu_temp: AtomicUnit::zero(),
            i_u: AtomicUnit::zero(),
            i_v: AtomicUnit::zero(),
            i_w: AtomicUnit::zero(),
            v_bus: AtomicUnit::zero(),
        }
    }
}

pub fn state() -> &'static State {
    static STATE: State = State::new();
    &STATE
}
