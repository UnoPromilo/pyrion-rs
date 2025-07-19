pub trait MotorDriver {
    fn enable(&mut self);

    /// Set pwm value, a value should be between 0 and u16::MAX
    fn set_pwm_values(&mut self, a: u16, b: u16, c: u16);

    fn disable(&mut self);
}
