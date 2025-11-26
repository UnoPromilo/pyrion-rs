use foc::state::FocState;

pub enum ControlStrategy {
    Disabled,
    Foc(FocState),
}
