use core::sync::atomic::Ordering;
use core::sync::atomic::{AtomicU8, AtomicUsize};
use enum_iterator::{Sequence, all};

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
    AdcTimeout,
    InvalidMeasurement,
    InvalidControllerOutput,
    GateDriverStartup,
    GateDriverRuntime,
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
            cells: [const { AtomicU8::new(FaultState::Clean as u8) }; FaultType::CARDINALITY],
            active_count: AtomicUsize::new(0),
            resolved_count: AtomicUsize::new(0),
        }
    }

    pub fn shared() -> &'static Self {
        static FAULT_REGISTER: FaultRegister = FaultRegister::new();
        &FAULT_REGISTER
    }

    pub fn load(&self, e: FaultType) -> FaultState {
        self.cells[idx(e)].load(Ordering::SeqCst).into()
    }

    pub fn set(&self, e: FaultType) {
        let prev = self.cells[idx(e)].swap(FaultState::Active as u8, Ordering::SeqCst);

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

    pub fn latch(&self, e: FaultType) {
        let previous = self.cells[idx(e)].swap(FaultState::Latched as u8, Ordering::SeqCst);

        match previous.into() {
            FaultState::Clean => {
                self.resolved_count.fetch_add(1, Ordering::SeqCst);
            }
            FaultState::Active => {
                self.active_count.fetch_sub(1, Ordering::SeqCst);
                self.resolved_count.fetch_add(1, Ordering::SeqCst);
            }
            FaultState::Latched => {}
        }
    }

    pub fn clear_latched(&self) {
        for e in all::<FaultType>() {
            if self.cells[idx(e)]
                .compare_exchange(
                    FaultState::Latched as u8,
                    FaultState::Clean as u8,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                self.resolved_count.fetch_sub(1, Ordering::SeqCst);
            }
        }
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
    fn clear_latched_should_clear_resolved_faults() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        reg.clear_latched();

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

        assert_eq!(reg.snapshot(), [FaultState::Clean; FaultType::CARDINALITY]);
    }

    #[test]
    fn snapshot_should_return_current_states() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);

        assert_eq!(
            reg.snapshot(),
            [
                FaultState::Active,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
            ]
        );
    }

    #[test]
    fn snapshot_should_include_resolved_faults() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        assert_eq!(
            reg.snapshot(),
            [
                FaultState::Latched,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
                FaultState::Clean,
            ]
        );
    }

    #[test]
    fn snapshot_should_reflect_cleared_latched_faults() {
        let reg = fresh_register();

        reg.latch(FaultType::Encoder);
        reg.clear_latched();

        assert_eq!(reg.snapshot(), [FaultState::Clean; FaultType::CARDINALITY]);
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
    fn clear_latched_should_clear_latched_counter() {
        let reg = fresh_register();

        reg.set(FaultType::Encoder);
        reg.resolve_if_set(FaultType::Encoder);

        reg.clear_latched();

        assert_eq!(reg.active_count(), 0);
        assert_eq!(reg.latched_count(), 0);
    }

    #[test]
    fn latch_should_mark_clean_fault_as_latched() {
        let reg = fresh_register();

        reg.latch(FaultType::AdcTimeout);

        assert_eq!(reg.load(FaultType::AdcTimeout), FaultState::Latched);
        assert_eq!(reg.active_count(), 0);
        assert_eq!(reg.latched_count(), 1);
    }

    #[test]
    fn clear_latched_should_not_clear_active_faults() {
        let reg = fresh_register();
        reg.set(FaultType::Encoder);

        reg.clear_latched();

        assert_eq!(reg.load(FaultType::Encoder), FaultState::Active);
        assert_eq!(reg.active_count(), 1);
    }

    #[test]
    fn gate_driver_startup_remains_active_when_latched_faults_are_cleared() {
        let reg = fresh_register();
        reg.set(FaultType::GateDriverStartup);
        reg.latch(FaultType::Encoder);

        reg.clear_latched();

        assert_eq!(FaultType::GateDriverStartup as usize, 4);
        assert_eq!(FaultType::CARDINALITY, 6);
        assert_eq!(reg.load(FaultType::GateDriverStartup), FaultState::Active);
        assert_eq!(reg.snapshot()[4], FaultState::Active);
        assert_eq!(reg.active_count(), 1);
        assert_eq!(reg.latched_count(), 0);
    }

    #[test]
    fn gate_driver_runtime_is_distinct_and_remains_active_for_the_boot() {
        let reg = fresh_register();
        reg.set(FaultType::GateDriverRuntime);
        reg.set(FaultType::GateDriverRuntime);
        reg.latch(FaultType::Encoder);

        reg.clear_latched();

        assert_eq!(FaultType::GateDriverRuntime as usize, 5);
        assert_eq!(reg.load(FaultType::GateDriverStartup), FaultState::Clean);
        assert_eq!(reg.load(FaultType::GateDriverRuntime), FaultState::Active);
        assert_eq!(reg.snapshot()[5], FaultState::Active);
        assert_eq!(reg.active_count(), 1);
        assert_eq!(reg.latched_count(), 0);
    }
}
