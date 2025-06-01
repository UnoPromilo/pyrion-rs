pub trait MotorDriver {
    async fn set_step(&mut self, phase: CommutationStep);
    fn get_step(&self) -> Option<CommutationStep>;

    fn disable(&mut self);

    fn set_duty_cycle(&mut self, duty_cycle: u16);
    fn get_duty_cycle(&self) -> u16;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommutationStep {
    AB,
    AC,
    BA,
    BC,
    CA,
    CB,
}
