use core::sync::atomic::Ordering;
use enum_iterator::{Sequence, all};
use portable_atomic::{AtomicU8, AtomicUsize};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorValue {
    Clean = 0,
    Ongoing = 1,
    Resolved = 2,
}

#[derive(Sequence, Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Encoder,
    // add more later
}

pub struct ErrorRegister {
    cells: [AtomicU8; Error::CARDINALITY],
    ongoing_count: AtomicUsize,
    resolved_count: AtomicUsize,
}

const fn idx(err: Error) -> usize {
    err as usize
}

impl From<u8> for ErrorValue {
    fn from(v: u8) -> Self {
        match v {
            0 => ErrorValue::Clean,
            1 => ErrorValue::Ongoing,
            2 => ErrorValue::Resolved,
            _ => unreachable!(),
        }
    }
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
            ongoing_count: AtomicUsize::new(0),
            resolved_count: AtomicUsize::new(0),
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
        self.cells[idx(e)].load(Ordering::SeqCst).into()
    }

    pub fn set(&self, e: Error) {
        let prev = self.cells[idx(e)]
            .swap(ErrorValue::Ongoing as u8, Ordering::SeqCst);

        match prev.into() {
            ErrorValue::Clean => {
                self.ongoing_count.fetch_add(1, Ordering::SeqCst);
            }
            ErrorValue::Resolved => {
                self.resolved_count.fetch_sub(1, Ordering::SeqCst);
                self.ongoing_count.fetch_add(1, Ordering::SeqCst);
            }
            ErrorValue::Ongoing => {}
        }
    }

    pub fn resolve_if_set(&self, e: Error) {
        if self.cells[idx(e)]
            .compare_exchange(
                ErrorValue::Ongoing as u8,
                ErrorValue::Resolved as u8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            self.ongoing_count.fetch_sub(1, Ordering::SeqCst);
            self.resolved_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn reset(&self) {
        for e in all::<Error>() {
            self.store(e, ErrorValue::Clean);
        }
        self.ongoing_count.store(0, Ordering::SeqCst);
        self.resolved_count.store(0, Ordering::SeqCst);
    }

    pub fn ongoing_count(&self) -> usize {
        self.ongoing_count.load(Ordering::SeqCst)
    }

    pub fn resolved_count(&self) -> usize {
        self.resolved_count.load(Ordering::SeqCst)
    }

    pub fn any_ongoing(&self) -> bool {
        self.ongoing_count() != 0
    }

    pub fn any_resolved(&self) -> bool {
        self.resolved_count() != 0
    }

    pub fn snapshot(&self) -> [ErrorValue; Error::CARDINALITY] {
        core::array::from_fn(|i| match self.cells[i].load(Ordering::SeqCst) {
            0 => ErrorValue::Clean,
            1 => ErrorValue::Ongoing,
            _ => ErrorValue::Resolved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_register() -> ErrorRegister {
        ErrorRegister::new()
    }

    #[test]
    fn new_register_should_be_clean() {
        let reg = fresh_register();

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Clean);

        assert!(!reg.any_ongoing());
        assert!(!reg.any_resolved());
    }

    #[test]
    fn set_should_mark_error_as_ongoing() {
        let reg = fresh_register();

        reg.set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Ongoing);

        assert!(reg.any_ongoing());
        assert!(!reg.any_resolved());
    }

    #[test]
    fn resolve_if_set_should_mark_ongoing_as_resolved() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Resolved);

        assert!(!reg.any_ongoing());
        assert!(reg.any_resolved());
    }

    #[test]
    fn resolve_if_set_should_not_change_clean_error() {
        let reg = fresh_register();

        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Clean);

        assert!(!reg.any_ongoing());
        assert!(!reg.any_resolved());
    }

    #[test]
    fn reset_should_clear_all_errors() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        reg.reset();

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Clean);

        assert!(!reg.any_ongoing());
        assert!(!reg.any_resolved());
    }

    #[test]
    fn ongoing_count_should_return_number_of_ongoing_errors() {
        let reg = fresh_register();

        assert_eq!(reg.ongoing_count(), 0);

        reg.set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 1);
    }

    #[test]
    fn resolved_count_should_return_number_of_resolved_errors() {
        let reg = fresh_register();

        assert_eq!(reg.resolved_count(), 0);

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.resolved_count(), 1);
    }

    #[test]
    fn set_after_resolve_should_return_to_ongoing() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Resolved);

        reg.set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Ongoing);

        assert!(reg.any_ongoing());
        assert!(!reg.any_resolved());
    }

    #[test]
    fn resolve_if_set_should_be_idempotent() {
        let reg = fresh_register();

        reg.set(Error::Encoder);

        reg.resolve_if_set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.load(Error::Encoder), ErrorValue::Resolved);
    }

    #[test]
    fn snapshot_should_return_all_clean_for_new_register() {
        let reg = fresh_register();

        assert_eq!(
            reg.snapshot(),
            [ErrorValue::Clean; Error::CARDINALITY]
        );
    }

    #[test]
    fn snapshot_should_return_current_states() {
        let reg = fresh_register();

        reg.set(Error::Encoder);

        assert_eq!(
            reg.snapshot(),
            [ErrorValue::Ongoing]
        );
    }

    #[test]
    fn snapshot_should_include_resolved_errors() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(
            reg.snapshot(),
            [ErrorValue::Resolved]
        );
    }

    #[test]
    fn snapshot_should_reflect_reset() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.reset();

        assert_eq!(
            reg.snapshot(),
            [ErrorValue::Clean]
        );
    }

    #[test]
    fn set_should_not_increment_ongoing_count_twice() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.set(Error::Encoder);
        reg.set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 1);
        assert_eq!(reg.resolved_count(), 0);
    }

    #[test]
    fn resolve_should_move_count_from_ongoing_to_resolved() {
        let reg = fresh_register();

        reg.set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 1);
        assert_eq!(reg.resolved_count(), 0);

        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 0);
        assert_eq!(reg.resolved_count(), 1);
    }

    #[test]
    fn set_after_resolve_should_move_count_back_to_ongoing() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 0);
        assert_eq!(reg.resolved_count(), 1);

        reg.set(Error::Encoder);

        assert_eq!(reg.ongoing_count(), 1);
        assert_eq!(reg.resolved_count(), 0);
    }

    #[test]
    fn reset_should_clear_counters() {
        let reg = fresh_register();

        reg.set(Error::Encoder);
        reg.resolve_if_set(Error::Encoder);

        reg.reset();

        assert_eq!(reg.ongoing_count(), 0);
        assert_eq!(reg.resolved_count(), 0);
    }
}
