#![no_std]

#[cfg(feature = "adc")]
pub mod adc;
#[cfg(feature = "as5600")]
pub mod as5600;
#[cfg(feature = "six-pwm-tim1")]
mod six_pwm_tim1;

#[cfg(feature = "six-pwm-tim1")]
pub use six_pwm_tim1::SixPwmTim1;

#[cfg(any(feature = "drv8301", test))]
mod drv8301;

#[cfg(feature = "drv8301")]
pub use drv8301::{
    Checked, ConfigurationError, ControlReadback, CurrentLimitOffTime, DiagnosticSnapshot,
    Drv8301Configuration, Drv8301Stage, FastOutput, GateCurrent, OvercurrentPolicy,
    PreflightPassed, SlowControl, StartupError, Unchecked, VdsThreshold, dtc_resistance,
};
