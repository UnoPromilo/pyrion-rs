use portable_atomic::{AtomicF32, AtomicU32};

// TODO create abstraction over Atomics for keeping units
pub struct State {
    pub foc_loop_frequency: AtomicU32,
    pub encoder_loop_frequency: AtomicU32,
    pub cpu_temp: AtomicF32,
    pub i_u: AtomicF32,
    pub i_v: AtomicF32,
    pub i_w: AtomicF32,
    pub v_bus: AtomicF32,
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
            cpu_temp: AtomicF32::new(0.0),
            i_u: AtomicF32::new(0.0),
            i_v: AtomicF32::new(0.0),
            i_w: AtomicF32::new(0.0),
            v_bus: AtomicF32::new(0.0),
        }
    }
}

pub fn state() -> &'static State {
    static STATE: State = State::new();
    &STATE
}
