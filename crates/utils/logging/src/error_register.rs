use core::sync::atomic::Ordering;
use enum_iterator::{Sequence, all};
use portable_atomic::AtomicU8;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorValue {
    Clean = 0,
    Ongoing = 1,
    Resolved = 2,
}

#[derive(Sequence, Clone, Copy, Debug, PartialEq)]
pub enum Error {
    ShaftPositionDetector,
    // add more later
}

pub struct ErrorRegister {
    cells: [AtomicU8; Error::CARDINALITY],
}

// you only do the match ONCE here
const fn idx(err: Error) -> usize {
    err as usize
}

impl Default for ErrorRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRegister {
    const fn new() -> Self {
        Self {
            cells: [AtomicU8::new(ErrorValue::Clean as u8); Error::CARDINALITY],
        }
    }

    pub fn shared() -> &'static Self {
        static ERROR_REGISTER: ErrorRegister = ErrorRegister::new();
        &ERROR_REGISTER
    }

    fn store(&self, e: Error, v: ErrorValue) {
        self.cells[idx(e)].store(v as u8, Ordering::SeqCst);
    }

    pub fn load(&self, e: Error) -> ErrorValue {
        match self.cells[idx(e)].load(Ordering::SeqCst) {
            0 => ErrorValue::Clean,
            1 => ErrorValue::Ongoing,
            _ => ErrorValue::Resolved,
        }
    }

    pub fn set(&self, e: Error) {
        self.store(e, ErrorValue::Ongoing);
    }
    pub fn resolve_if_set(&self, e: Error) {
        if self.load(e) == ErrorValue::Ongoing {
            self.store(e, ErrorValue::Resolved);
        }
    }
    pub fn reset(&self) {
        for e in all::<Error>() {
            self.store(e, ErrorValue::Clean);
        }
    }

    pub fn get_ongoing(&self) -> impl Iterator<Item = Error> + '_ {
        all::<Error>().filter(|e| self.load(*e) == ErrorValue::Ongoing)
    }
    
    pub fn get_resolved(&self) -> impl Iterator<Item = Error> + '_ {
        all::<Error>().filter(|e| self.load(*e) == ErrorValue::Resolved)
    }
}
