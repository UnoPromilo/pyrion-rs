use core::sync::atomic::{AtomicU32, Ordering};

static TRIGGER_EPOCH: AtomicU32 = AtomicU32::new(0);
static INVALID_EPOCH: AtomicU32 = AtomicU32::new(0);
static LAST_JEOS_EPOCH: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FastAdc {
    Adc1,
    Adc3,
    Adc5,
}

impl FastAdc {
    pub const fn index(self) -> usize {
        self as usize
    }

    const fn mask(self) -> u8 {
        1 << (self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerEpoch(u32);

impl TriggerEpoch {
    pub const UNSTARTED: Self = Self(0);

    #[cfg(feature = "adc-timing")]
    pub const fn count(self) -> u32 {
        self.0
    }

    pub fn current() -> Self {
        Self(TRIGGER_EPOCH.load(Ordering::Acquire))
    }

    pub const fn is_started(self) -> bool {
        self.0 != 0
    }

    pub fn is_newer_than(self, other: Self) -> bool {
        if !other.is_started() {
            return self.is_started();
        }
        self.is_started() && (self.0.wrapping_sub(other.0) as i32) > 0
    }

    pub fn latest(self, other: Self) -> Self {
        if self.is_newer_than(other) {
            self
        } else {
            other
        }
    }

    pub fn skipped_since(self, previous: Self) -> u32 {
        if !previous.is_started() || !self.is_newer_than(previous) {
            return 0;
        }
        let distance = self.0.wrapping_sub(previous.0);
        if self.0 < previous.0 {
            distance.saturating_sub(2)
        } else {
            distance - 1
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrameAlignment {
    Ready(TriggerEpoch),
    Refresh { adc1: bool, adc3: bool, adc5: bool },
}

pub fn align_fast_frame(
    a: TriggerEpoch,
    c: TriggerEpoch,
    d: TriggerEpoch,
    current: TriggerEpoch,
) -> FrameAlignment {
    let newest = a.latest(c).latest(d).latest(current);
    if newest.is_started() && [a, c, d].iter().all(|&epoch| epoch == newest) {
        FrameAlignment::Ready(newest)
    } else {
        FrameAlignment::Refresh {
            adc1: !newest.is_started() || a != newest,
            adc3: !newest.is_started() || c != newest,
            adc5: !newest.is_started() || d != newest,
        }
    }
}

fn pending_is_ambiguous(
    pending_mask: u8,
    preceding: TriggerEpoch,
    serviced: [TriggerEpoch; 3],
    current_edge_confirmed: bool,
) -> bool {
    pending_mask != 0
        && (!preceding.is_started()
            || !current_edge_confirmed
            || [FastAdc::Adc1, FastAdc::Adc3, FastAdc::Adc5]
                .iter()
                .any(|&adc| pending_mask & adc.mask() != 0 && serviced[adc.index()] != preceding))
}

pub fn on_adc_jeos(adc: FastAdc, epoch: TriggerEpoch) {
    LAST_JEOS_EPOCH[adc.index()].store(epoch.0, Ordering::Release);
}

pub fn on_tim1_trigger(pending_mask: u8, current_edge_confirmed: bool) -> bool {
    let preceding = TriggerEpoch(TRIGGER_EPOCH.load(Ordering::Relaxed));
    let invalid = if pending_mask == 0 {
        false
    } else {
        let serviced = core::array::from_fn(|index| {
            TriggerEpoch(LAST_JEOS_EPOCH[index].load(Ordering::Acquire))
        });
        pending_is_ambiguous(pending_mask, preceding, serviced, current_edge_confirmed)
    };
    let next = preceding.0.wrapping_add(1);
    let next = next.max(1);
    INVALID_EPOCH.store(if invalid { next } else { 0 }, Ordering::Relaxed);
    TRIGGER_EPOCH.store(next, Ordering::Release);
    invalid
}

pub fn is_invalid(epoch: TriggerEpoch) -> bool {
    epoch.is_started() && epoch.0 == INVALID_EPOCH.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unstarted_is_older_than_a_real_epoch() {
        assert!(TriggerEpoch(1).is_newer_than(TriggerEpoch::UNSTARTED));
        assert!(!TriggerEpoch::UNSTARTED.is_newer_than(TriggerEpoch(1)));
    }

    #[test]
    fn compares_wrapping_epochs_without_requiring_signed_absolute_order() {
        assert!(TriggerEpoch(1).is_newer_than(TriggerEpoch(u32::MAX)));
        assert_eq!(
            TriggerEpoch(u32::MAX).latest(TriggerEpoch(1)),
            TriggerEpoch(1)
        );
        assert!(TriggerEpoch(100).is_newer_than(TriggerEpoch(99)));
        assert!(!TriggerEpoch(99).is_newer_than(TriggerEpoch(100)));
    }

    #[test]
    fn refreshes_only_missing_sources_and_never_accepts_a_stale_frame() {
        let e = TriggerEpoch(10);
        assert_eq!(
            align_fast_frame(e, TriggerEpoch(9), e, e),
            FrameAlignment::Refresh {
                adc1: false,
                adc3: true,
                adc5: false,
            }
        );
        assert_eq!(
            align_fast_frame(e, e, e, TriggerEpoch(11)),
            FrameAlignment::Refresh {
                adc1: true,
                adc3: true,
                adc5: true,
            }
        );
        assert_eq!(align_fast_frame(e, e, e, e), FrameAlignment::Ready(e));
        assert_eq!(
            align_fast_frame(
                TriggerEpoch::UNSTARTED,
                TriggerEpoch::UNSTARTED,
                TriggerEpoch::UNSTARTED,
                TriggerEpoch::UNSTARTED
            ),
            FrameAlignment::Refresh {
                adc1: true,
                adc3: true,
                adc5: true,
            }
        );
    }

    #[test]
    fn handles_wrap_and_skipped_epochs() {
        assert_eq!(TriggerEpoch(1).skipped_since(TriggerEpoch(u32::MAX)), 0);
        assert_eq!(TriggerEpoch(3).skipped_since(TriggerEpoch(u32::MAX)), 2);
        assert_eq!(TriggerEpoch(15).skipped_since(TriggerEpoch(12)), 2);
        assert_eq!(TriggerEpoch(12).skipped_since(TriggerEpoch(15)), 0);
    }

    #[test]
    fn pending_jeos_poisons_only_the_next_trigger_epoch() {
        on_tim1_trigger(0b111, false);
        let poisoned = TriggerEpoch::current();
        assert!(is_invalid(poisoned));
        on_tim1_trigger(0, false);
        assert!(!is_invalid(TriggerEpoch::current()));
        assert!(!is_invalid(poisoned));
    }

    #[test]
    fn pending_results_need_previous_completion_and_current_edge() {
        let previous = TriggerEpoch(7);
        let complete = [previous; 3];
        assert!(!pending_is_ambiguous(0b111, previous, complete, true));
        assert!(!pending_is_ambiguous(0b110, previous, complete, true));
        assert!(!pending_is_ambiguous(
            0b001,
            previous,
            [previous, TriggerEpoch(6), TriggerEpoch(6)],
            true
        ));
        assert!(pending_is_ambiguous(
            0b111,
            previous,
            [previous, TriggerEpoch(6), previous],
            true
        ));
        assert!(pending_is_ambiguous(0b111, previous, complete, false));
        assert!(pending_is_ambiguous(
            0b111,
            TriggerEpoch::UNSTARTED,
            [TriggerEpoch::UNSTARTED; 3],
            true
        ));
    }
}
