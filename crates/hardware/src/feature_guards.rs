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
