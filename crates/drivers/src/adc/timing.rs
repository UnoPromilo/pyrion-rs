use super::epoch::TriggerEpoch;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use cortex_m::peripheral::DWT;

static LAST_EPOCH: AtomicU32 = AtomicU32::new(0);
static LAST_TRIGGER_CYCLE: AtomicU32 = AtomicU32::new(0);
static HAS_TRIGGER: AtomicBool = AtomicBool::new(false);
static TRIGGERS: AtomicU32 = AtomicU32::new(0);
static PERIOD_MIN: AtomicU32 = AtomicU32::new(u32::MAX);
static PERIOD_MAX: AtomicU32 = AtomicU32::new(0);
static PENDING_ALL: AtomicU32 = AtomicU32::new(0);
static PENDING_PARTIAL: AtomicU32 = AtomicU32::new(0);
static PENDING_RECOVERED: AtomicU32 = AtomicU32::new(0);
static PENDING_PHASE: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static CLEAR_LATE: AtomicU32 = AtomicU32::new(0);
static UNCERTAIN_PHASE: AtomicU32 = AtomicU32::new(0);
static JEOS_COUNT: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static JEOS_MAX: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static UNMATCHED_JEOS: AtomicU32 = AtomicU32::new(0);
static PUBLICATION_SEQ: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static LAST_DELIVERED_SEQ: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static UNDELIVERED: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static REJECT_INVALID: AtomicU32 = AtomicU32::new(0);
static REJECT_STALE: AtomicU32 = AtomicU32::new(0);
static REFRESHES: AtomicU32 = AtomicU32::new(0);
static FRAMES: AtomicU32 = AtomicU32::new(0);
static FRAME_MAX: AtomicU32 = AtomicU32::new(0);
static UNMATCHED_FRAMES: AtomicU32 = AtomicU32::new(0);
static DECISIONS: AtomicU32 = AtomicU32::new(0);
static DECISION_MAX: AtomicU32 = AtomicU32::new(0);
static DECISION_WORK_MAX: AtomicU32 = AtomicU32::new(0);
static UNMATCHED_DECISIONS: AtomicU32 = AtomicU32::new(0);
static DUTY_WRITES: AtomicU32 = AtomicU32::new(0);
static DUTY_WRITE_MAX: AtomicU32 = AtomicU32::new(0);
static UNMATCHED_DUTY_WRITES: AtomicU32 = AtomicU32::new(0);

pub struct TimingWindow {
    pub triggers: u32,
    pub period_min: u32,
    pub period_max: u32,
    pub pending_all: u32,
    pub pending_partial: u32,
    pub pending_recovered: u32,
    pub pending_phase: [u32; 3],
    pub clear_late: u32,
    pub uncertain_phase: u32,
    pub jeos_count: [u32; 3],
    pub jeos_max: [u32; 3],
    pub unmatched_jeos: u32,
    pub undelivered: [u32; 3],
    pub reject_invalid: u32,
    pub reject_stale: u32,
    pub refreshes: u32,
    pub frames: u32,
    pub frame_max: u32,
    pub unmatched_frames: u32,
    pub decisions: u32,
    pub decision_max: u32,
    pub decision_work_max: u32,
    pub unmatched_decisions: u32,
    pub duty_writes: u32,
    pub duty_write_max: u32,
    pub unmatched_duty_writes: u32,
}

pub struct TimerSnapshot {
    pub pending_mask: u8,
    pub count: u16,
    pub down_before: bool,
    pub down_after: bool,
    pub compare: u16,
}

pub fn cycle_count() -> u32 {
    DWT::cycle_count()
}

pub fn record_trigger(epoch: TriggerEpoch, cycle: u32, snapshot: TimerSnapshot, invalid: bool) {
    let previous = LAST_TRIGGER_CYCLE.swap(cycle, Ordering::Relaxed);
    TRIGGERS.fetch_add(1, Ordering::Relaxed);
    if HAS_TRIGGER.swap(true, Ordering::Relaxed) {
        let period = cycle.wrapping_sub(previous);
        PERIOD_MIN.fetch_min(period, Ordering::Relaxed);
        PERIOD_MAX.fetch_max(period, Ordering::Relaxed);
    }
    LAST_EPOCH.store(epoch.count(), Ordering::Release);

    if snapshot.pending_mask == 0b111 {
        PENDING_ALL.fetch_add(1, Ordering::Relaxed);
    } else if snapshot.pending_mask != 0 {
        PENDING_PARTIAL.fetch_add(1, Ordering::Relaxed);
    }
    if snapshot.pending_mask != 0 && !invalid {
        PENDING_RECOVERED.fetch_add(1, Ordering::Relaxed);
    }

    let phase = if snapshot.down_before != snapshot.down_after {
        None
    } else if snapshot.down_before {
        snapshot.compare.checked_sub(snapshot.count)
    } else {
        snapshot.compare.checked_add(snapshot.count)
    };
    match phase {
        Some(ticks) if snapshot.pending_mask != 0 => {
            let bucket = match ticks {
                0..=255 => 0,
                256..=511 => 1,
                _ => 2,
            };
            PENDING_PHASE[bucket].fetch_add(1, Ordering::Relaxed);
        }
        Some(ticks) if ticks >= 256 => {
            CLEAR_LATE.fetch_add(1, Ordering::Relaxed);
        }
        Some(_) => {}
        None => {
            UNCERTAIN_PHASE.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn delay_since_trigger(epoch: TriggerEpoch, cycle: u32) -> Option<u32> {
    let observed = LAST_EPOCH.load(Ordering::Acquire);
    let start = LAST_TRIGGER_CYCLE.load(Ordering::Relaxed);
    let delay = cycle.wrapping_sub(start);
    (observed == epoch.count()
        && observed != 0
        && LAST_EPOCH.load(Ordering::Acquire) == observed
        && (delay as i32) >= 0)
        .then_some(delay)
}

pub fn record_jeos(index: usize, epoch: TriggerEpoch, cycle: u32) -> u32 {
    JEOS_COUNT[index].fetch_add(1, Ordering::Relaxed);
    if let Some(delay) = delay_since_trigger(epoch, cycle) {
        JEOS_MAX[index].fetch_max(delay, Ordering::Relaxed);
    } else {
        UNMATCHED_JEOS.fetch_add(1, Ordering::Relaxed);
    }
    PUBLICATION_SEQ[index]
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1)
}

pub fn record_delivery(index: usize, sequence: u32) {
    let previous = LAST_DELIVERED_SEQ[index].swap(sequence, Ordering::Relaxed);
    if previous != 0 {
        UNDELIVERED[index].fetch_add(
            sequence.wrapping_sub(previous).wrapping_sub(1),
            Ordering::Relaxed,
        );
    }
}

pub fn record_reject_invalid() {
    REJECT_INVALID.fetch_add(1, Ordering::Relaxed);
}

pub fn record_reject_stale() {
    REJECT_STALE.fetch_add(1, Ordering::Relaxed);
}

pub fn record_refresh() {
    REFRESHES.fetch_add(1, Ordering::Relaxed);
}

pub fn record_frame(epoch: TriggerEpoch) -> u32 {
    let cycle = cycle_count();
    if let Some(delay) = delay_since_trigger(epoch, cycle) {
        FRAMES.fetch_add(1, Ordering::Relaxed);
        FRAME_MAX.fetch_max(delay, Ordering::Relaxed);
    } else {
        UNMATCHED_FRAMES.fetch_add(1, Ordering::Relaxed);
    }
    cycle
}

pub fn record_decision(epoch: TriggerEpoch, frame_cycle: u32) {
    let cycle = cycle_count();
    DECISION_WORK_MAX.fetch_max(cycle.wrapping_sub(frame_cycle), Ordering::Relaxed);
    if let Some(delay) = delay_since_trigger(epoch, cycle) {
        DECISIONS.fetch_add(1, Ordering::Relaxed);
        DECISION_MAX.fetch_max(delay, Ordering::Relaxed);
    } else {
        UNMATCHED_DECISIONS.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_duty_write(epoch: Option<TriggerEpoch>) {
    let cycle = cycle_count();
    DUTY_WRITES.fetch_add(1, Ordering::Relaxed);
    if let Some(delay) = epoch.and_then(|epoch| delay_since_trigger(epoch, cycle)) {
        DUTY_WRITE_MAX.fetch_max(delay, Ordering::Relaxed);
    } else {
        UNMATCHED_DUTY_WRITES.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn take_window() -> TimingWindow {
    let take = |counters: &[AtomicU32; 3]| {
        core::array::from_fn(|index| counters[index].swap(0, Ordering::Relaxed))
    };
    let period_min = PERIOD_MIN.swap(u32::MAX, Ordering::Relaxed);
    TimingWindow {
        triggers: TRIGGERS.swap(0, Ordering::Relaxed),
        period_min: if period_min == u32::MAX {
            0
        } else {
            period_min
        },
        period_max: PERIOD_MAX.swap(0, Ordering::Relaxed),
        pending_all: PENDING_ALL.swap(0, Ordering::Relaxed),
        pending_partial: PENDING_PARTIAL.swap(0, Ordering::Relaxed),
        pending_recovered: PENDING_RECOVERED.swap(0, Ordering::Relaxed),
        pending_phase: take(&PENDING_PHASE),
        clear_late: CLEAR_LATE.swap(0, Ordering::Relaxed),
        uncertain_phase: UNCERTAIN_PHASE.swap(0, Ordering::Relaxed),
        jeos_count: take(&JEOS_COUNT),
        jeos_max: take(&JEOS_MAX),
        unmatched_jeos: UNMATCHED_JEOS.swap(0, Ordering::Relaxed),
        undelivered: take(&UNDELIVERED),
        reject_invalid: REJECT_INVALID.swap(0, Ordering::Relaxed),
        reject_stale: REJECT_STALE.swap(0, Ordering::Relaxed),
        refreshes: REFRESHES.swap(0, Ordering::Relaxed),
        frames: FRAMES.swap(0, Ordering::Relaxed),
        frame_max: FRAME_MAX.swap(0, Ordering::Relaxed),
        unmatched_frames: UNMATCHED_FRAMES.swap(0, Ordering::Relaxed),
        decisions: DECISIONS.swap(0, Ordering::Relaxed),
        decision_max: DECISION_MAX.swap(0, Ordering::Relaxed),
        decision_work_max: DECISION_WORK_MAX.swap(0, Ordering::Relaxed),
        unmatched_decisions: UNMATCHED_DECISIONS.swap(0, Ordering::Relaxed),
        duty_writes: DUTY_WRITES.swap(0, Ordering::Relaxed),
        duty_write_max: DUTY_WRITE_MAX.swap(0, Ordering::Relaxed),
        unmatched_duty_writes: UNMATCHED_DUTY_WRITES.swap(0, Ordering::Relaxed),
    }
}
