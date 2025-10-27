use portable_atomic::{AtomicU8, AtomicU16, AtomicU32};
use units::AtomicUnit;

pub struct State {
    pub version: Version,
    pub raw_angle: AtomicU16,
    pub foc_loop_frequency: AtomicU32,
    pub encoder_loop_frequency: AtomicU32,
    pub last_foc_loop_time_us: AtomicU16,
    pub cpu_temp: AtomicUnit<units::ThermodynamicTemperature>,
    pub i_u: AtomicUnit<units::ElectricCurrent>,
    pub i_v: AtomicUnit<units::ElectricCurrent>,
    pub i_w: AtomicUnit<units::ElectricCurrent>,
    pub v_bus: AtomicUnit<units::ElectricPotential>,
}

pub struct Version {
    pub major: AtomicU8,
    pub minor: AtomicU8,
    pub patch: AtomicU8,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub const fn new() -> Self {
        Self {
            version: Version::new(),
            raw_angle: AtomicU16::new(0),
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

impl Version {
    const fn new() -> Self {
        Self {
            major: AtomicU8::new(0),
            minor: AtomicU8::new(0),
            patch: AtomicU8::new(0),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

pub fn state() -> &'static State {
    static STATE: State = State::new();
    &STATE
}
