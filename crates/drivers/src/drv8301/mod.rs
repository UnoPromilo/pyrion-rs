mod config;
#[cfg(feature = "drv8301")]
mod control;
mod probe;
#[cfg(feature = "drv8301")]
mod spi_device;
#[cfg(feature = "drv8301")]
mod stage;

#[cfg(feature = "drv8301")]
pub use config::{
    ConfigurationError, ControlReadback, CurrentLimitOffTime, Drv8301Configuration, GateCurrent,
    OvercurrentPolicy, VdsThreshold,
};
#[cfg(feature = "drv8301")]
pub use control::{SlowControl, StartupError};
#[cfg(feature = "drv8301")]
pub use probe::DiagnosticSnapshot;
#[cfg(feature = "drv8301")]
pub use stage::{Checked, Drv8301Stage, FastOutput, PreflightPassed, Unchecked};

#[cfg(feature = "drv8301")]
use units::ElectricalResistance;
#[cfg(feature = "drv8301")]
use units::si::electrical_resistance::kiloohm;

#[cfg(all(
    feature = "drv8301",
    feature = "drv8301-dtc-10k",
    feature = "drv8301-dtc-20k"
))]
compile_error!("select exactly one DRV8301 DTC resistance");
#[cfg(all(
    feature = "drv8301",
    not(any(feature = "drv8301-dtc-10k", feature = "drv8301-dtc-20k"))
))]
compile_error!("select a DRV8301 DTC resistance");

#[cfg(all(
    feature = "drv8301",
    feature = "drv8301-dtc-10k",
    not(feature = "drv8301-dtc-20k")
))]
pub fn dtc_resistance() -> ElectricalResistance {
    ElectricalResistance::new::<kiloohm>(10.0)
}

#[cfg(all(feature = "drv8301", feature = "drv8301-dtc-20k"))]
pub fn dtc_resistance() -> ElectricalResistance {
    ElectricalResistance::new::<kiloohm>(20.0)
}
