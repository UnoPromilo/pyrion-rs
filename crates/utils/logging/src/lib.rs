#![no_std]
#![macro_use]

use embassy_time::{Duration, Instant};

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

pub struct FreqMeter {
    last: Instant,
    count: u64,
    interval: Duration,
    name: &'static str,
}

impl FreqMeter {
    pub fn named(name: &'static str) -> Self {
        Self::new(Duration::from_secs(10), name)
    }

    fn new(interval: Duration, name: &'static str) -> Self {
        Self {
            last: Instant::now(),
            count: 0,
            interval,
            name,
        }
    }
    #[cfg(feature = "freq-meter")]
    pub fn tick(&mut self) {
        self.count += 1;
        if self.last.elapsed() > self.interval {
            debug!(
                "{} frequency: {} Hz",
                self.name,
                self.count / self.interval.as_secs()
            );
            self.count = 0;
            self.last = Instant::now();
        }
    }

    #[cfg(not(feature = "freq-meter"))]
    #[inline(always)]
    pub fn tick(&mut self) {}
}
