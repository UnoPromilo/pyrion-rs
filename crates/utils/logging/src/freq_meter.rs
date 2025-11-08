use core::sync::atomic::Ordering;
use embassy_time::{Duration, Instant};
use portable_atomic::AtomicU32;
use crate::debug;

pub struct FreqMeter {
    print_last: Instant,
    print_count: u32,
    linked_last: Instant,
    linked_count: u32,
    name: &'static str,
    linked: Option<&'static AtomicU32>,
}

impl FreqMeter {
    pub fn named(name: &'static str) -> Self {
        Self::new(name)
    }

    fn new(name: &'static str) -> Self {
        Self {
            print_last: Instant::now(),
            print_count: 0,
            linked_last: Instant::now(),
            linked_count: 0,
            linked: None,
            name,
        }
    }
    #[cfg(feature = "freq-meter")]
    pub fn tick(&mut self) {
        const PRINT_INTERVAL: Duration = Duration::from_secs(5);
        const LINKED_INTERVAL: Duration = Duration::from_hz(60);

        self.print_count += 1;
        self.linked_count += 1;
        let print_elapsed = self.print_last.elapsed();
        if print_elapsed > PRINT_INTERVAL {
            debug!(
                "{} frequency: {} Hz",
                self.name,
                self.print_count / print_elapsed.as_secs() as u32
            );
            self.print_count = 0;
            self.print_last = Instant::now();
        }

        let linked_elapsed = self.linked_last.elapsed();
        if let Some(linked) = self.linked
            && linked_elapsed > LINKED_INTERVAL
        {
            let millis_from_last = linked_elapsed.as_millis() as u32;
            let freq = self.linked_count * 1000 / millis_from_last;
            linked.store(freq, Ordering::Relaxed);
            self.linked_count = 0;
            self.linked_last = Instant::now();
        }
    }

    pub fn link(&mut self, destination: &'static AtomicU32) {
        self.linked = Some(destination);
    }

    #[cfg(not(feature = "freq-meter"))]
    #[inline(always)]
    pub fn tick(&mut self) {}
}

impl Drop for FreqMeter {
    fn drop(&mut self) {
        if let Some(linked) = self.linked {
            linked.store(0, Ordering::Relaxed);
        }
    }
}
