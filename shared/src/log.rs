#![macro_use]
#[cfg(all(feature = "defmt", feature = "log"))]
compile_error!("You may not enable both `defmt` and `log` features.");

#[cfg(feature = "log")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::log::trace!($s $(, $x)*);
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::defmt::trace!($s $(, $x)*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        let _ = ($(& $x),*);
    }};
}

#[cfg(feature = "log")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::log::debug!($s $(, $x)*);
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::defmt::debug!($s $(, $x)*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        let _ = ($(& $x),*);
    }};
}

#[cfg(feature = "log")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::log::info!($s $(, $x)*);
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::defmt::info!($s $(, $x)*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        let _ = ($(& $x),*);
    }};
}

#[cfg(feature = "log")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::log::warn!($s $(, $x)*);
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::defmt::warn!($s $(, $x)*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        let _ = ($(& $x),*);
    }};
}

#[cfg(feature = "log")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::log::error!($s $(, $x)*);
        }
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            ::defmt::error!($s $(, $x)*);
        }
    };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        let _ = ($(& $x),*);
    }};
}
