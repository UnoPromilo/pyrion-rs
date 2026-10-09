#[macro_export]
macro_rules! at_most_one_of {
    ($msg:literal, $($feature:literal),+ $(,)?) => {
        const _: () = {
            let count = 0usize $(+ cfg!(feature = $feature) as usize)+;
            assert!(count <= 1, $msg);
        };
    };
}

#[macro_export]
macro_rules! exactly_one_of {
    ($msg:literal, $($feature:literal),+ $(,)?) => {
        const _: () = {
            let count = 0usize $(+ cfg!(feature = $feature) as usize)+;
            assert!(count == 1, $msg);
        };
    };
}

exactly_one_of!(
    "hardware: select exactly one profile (mcu-core or a board-*)",
    "mcu-core",
    "board-pyrion-ovo",
    "board-pyrion-nullo",
);

at_most_one_of!(
    "hardware: select at most one shunt count (cap-shunt-two or cap-shunt-three)",
    "cap-shunt-two",
    "cap-shunt-three",
);

#[cfg(feature = "cap-drv8301")]
exactly_one_of!(
    "hardware: select exactly one DTC resistance (drv8301-dtc-10k or drv8301-dtc-20k)",
    "cap-drv8301-dtc-10k",
    "cap-drv8301-dtc-20k",
);

#[cfg(all(
    not(feature = "cap-drv8301"),
    any(feature = "cap-drv8301-dtc-10k", feature = "cap-drv8301-dtc-20k")
))]
compile_error!("hardware: DRV8301 DTC resistance requires cap-drv8301");

#[cfg(all(feature = "cap-drv8301", not(feature = "cap-six-pwm-tim1")))]
compile_error!("hardware: DRV8301 requires a PWM backend (currently cap-six-pwm-tim1)");

#[cfg(all(feature = "cap-six-pwm-tim1", not(feature = "board")))]
compile_error!("hardware: PWM requires a board profile");
