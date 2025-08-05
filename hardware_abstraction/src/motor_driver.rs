pub trait MotorDriver {
    fn enable_synced(&mut self);
    
    fn enable_phase_a(&mut self);
    
    fn enable_phase_b(&mut self);
    
    fn enable_phase_c(&mut self);

    /// Set pwm value, a value should be between 0 and u16::MAX
    fn set_voltages(&mut self, ua: i16, ub: i16, uc: i16);

    fn disable(&mut self);
}
