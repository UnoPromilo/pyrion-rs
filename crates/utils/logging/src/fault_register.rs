use core::sync::atomic::Ordering;
use enum_iterator::{Sequence, all};
use portable_atomic::{AtomicU8, AtomicUsize};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaultState {
    Clean = 0,
    Active = 1,
    Latched = 2,
}

#[derive(Sequence, Clone, Copy, Debug, PartialEq)]
pub enum FaultType {
    Encoder,
    // add more later
}

pub struct FaultRegister {
    cells: [AtomicU8; FaultType::CARDINALITY],
    active_count: AtomicUsize,
    resolved_count: AtomicUsize,
}

const fn idx(err: FaultType) -> usize {
    err as usize
}

impl From<u8> for FaultState {
    fn from(v: u8) -> Self {
        match v {
            0 => FaultState::Clean,
            1 => FaultState::Active,
            2 => FaultState::Latched,
            _ => unreachable!(),
        }
    }
}

impl Default for FaultRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl FaultRegister {
    const fn new() -> Self {
        Self {
            cells: [AtomicU8::new(FaultState::Clean as u8); FaultType::CARDINALITY],
            active_count: AtomicUsize::new(0),
            resolved_count: AtomicUsize::new(0),
        }
    }

    pub fn shared() -> &'static Self {
        static ERROR_REGISTER: FaultRegister = FaultRegister::new();
        &ERROR_REGISTER
    }

    fn store(&self, e: FaultType, v: FaultState) {
        self.cells[idx(e)].store(v as u8, Ordering::SeqCst);
    }

    pub fn load(&self, e: FaultType) -> FaultState {
        self.cells[idx(e)].load(Ordering::SeqCst).into()
    }

    pub fn set(&self, e: FaultType) {
        let prev = self.cells[idx(e)]
            .swap(FaultState::Active as u8, Ordering::SeqCst);

        match prev.into() {
            FaultState::Clean => {
                self.active_count.fetch_add(1, Ordering::SeqCst);
            }
            FaultState::Latched => {
                self.resolved_count.fetch_sub(1, Ordering::SeqCst);
                self.active_count.fetch_add(1, Ordering::SeqCst);
            }
            FaultState::Active => {}
        }
    }

    pub fn resolve_if_set(&self, e: FaultType) {
        if self.cells[idx(e)]
            .compare_exchange(
                FaultState::Active as u8,
                FaultState::Latched as u8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            self.active_count.fetch_sub(1, Ordering::SeqCst);
            self.resolved_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn reset(&self) {
        for e in all::<FaultType>() {
            self.store(e, FaultState::Clean);
        }
        self.active_count.store(0, Ordering::SeqCst);
        self.resolved_count.store(0, Ordering::SeqCst);
    }

    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    pub fn latched_count(&self) -> usize {
        self.resolved_count.load(Ordering::SeqCst)
    }

    pub fn any_active(&self) -> bool {
        self.active_count() != 0
    }

    pub fn any_latched(&self) -> bool {
        self.latched_count() != 0
    }

    pub fn snapshot(&self) -> [FaultState; FaultType::CARDINALITY] {
        core::array::from_fn(|i| match self.cells[i].load(Ordering::SeqCst) {
            0 => FaultState::Clean,
            1 => FaultState::Active,
            _ => FaultState::Latched,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_register() -> FaultRegister {
        FaultRegister::new()
    }

    #[test]
    fn new_register_should_be_clean() {
        let reg = fresh_register();

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Clean);

        assert!(!reg.any_active());
        assert!(!reg.any_latched());
    }

    #[test]
    fn set_should_mark_fault_as_active() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Active);

        assert!(reg.any_active());
        assert!(!reg.any_latched());
    }

    #[test]
    fn resolve_if_set_should_mark_active_as_resolved() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Latched);

        assert!(!reg.any_active());
        assert!(reg.any_latched());
    }

    #[test]
    fn resolve_if_set_should_not_change_clean_fault() {
        let reg = fresh_register();

        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Clean);

        assert!(!reg.any_active());
        assert!(!reg.any_latched());
    }

    #[test]
    fn reset_should_clear_all_faults() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        reg.reset();

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Clean);

        assert!(!reg.any_active());
        assert!(!reg.any_latched());
    }

    #[test]
    fn active_count_should_return_number_of_active_faults() {
        let reg = fresh_register();

        assert_eq!(reg.active_count(), 0);

        reg.set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 1);
    }

    #[test]
    fn resolved_count_should_return_number_of_resolved_faults() {
        let reg = fresh_register();

        assert_eq!(reg.latched_count(), 0);

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.latched_count(), 1);
    }

    #[test]
    fn set_after_resolve_should_return_to_active() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Latched);

        reg.set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Active);

        assert!(reg.any_active());
        assert!(!reg.any_latched());
    }

    #[test]
    fn resolve_if_set_should_be_idempotent() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);

        reg.resolve_if_set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Latched);
    }

    #[test]
    fn snapshot_should_return_all_clean_for_new_register() {
        let reg = fresh_register();

        assert_eq!(
            reg.snapshot(),
            [FaultState::Clean; FaultType::CARDINALITY]
        );
    }

    #[test]
    fn snapshot_should_return_current_states() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);

        assert_eq!(
            reg.snapshot(),
            [FaultState::Active]
        );
    }

    #[test]
    fn snapshot_should_include_resolved_faults() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(
            reg.snapshot(),
            [FaultState::Latched]
        );
    }

    #[test]
    fn snapshot_should_reflect_reset() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.reset();

        assert_eq!(
            reg.snapshot(),
            [FaultState::Clean]
        );
    }

    #[test]
    fn set_should_not_increment_active_count_twice() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.set(FaultType::Encoder);
        reg.set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 1);
        assert_eq!(reg.latched_count(), 0);
    }

    #[test]
    fn resolve_should_move_count_from_active_to_resolved() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 1);
        assert_eq!(reg.latched_count(), 0);

        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 0);
        assert_eq!(reg.latched_count(), 1);
    }

    #[test]
    fn set_after_resolve_should_move_count_back_to_active() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 0);
        assert_eq!(reg.latched_count(), 1);

        reg.set(FaultType::Encoder);

        assert_eq!(reg.active_count(), 1);
        assert_eq!(reg.latched_count(), 0);
    }

    #[test]
    fn reset_should_clear_counters() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        reg.reset();

        assert_eq!(reg.active_count(), 0);
        assert_eq!(reg.latched_count(), 0);
    }
}
