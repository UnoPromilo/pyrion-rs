#![no_std]

#[cfg(feature = "hardware-support")]
use embassy_time::Timer;
use embassy_time::{Duration, Instant};
#[cfg(feature = "hardware-support")]
use hardware::BoardLeds;
use logging::error_register::ErrorRegister;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisualState {
    pub armed: bool,
    pub warning: bool,
    pub fault: bool,
    pub calibrating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualEvent {
    Initialized,
    ConfigSaved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedState {
    pub red: bool,
    pub green: bool,
}

pub struct LedAnimator {
    overlay: Option<Overlay>,
}

struct Overlay {
    kind: VisualEvent,
    started_at: Option<Instant>,
}

impl Default for LedAnimator {
    fn default() -> Self {
        Self::new()
    }
}

impl LedAnimator {
    pub const fn new() -> Self {
        Self { overlay: None }
    }

    pub fn trigger(&mut self, event: VisualEvent) {
        self.overlay = Some(Overlay {
            kind: event,
            started_at: None,
        });
    }

    #[cfg(feature = "hardware-support")]
    pub async fn run<'a>(&mut self, board_leds: &mut BoardLeds<'a>) -> ! {
        loop {
            let now = Instant::now();

            let state = calculate_visual_state();

            let leds = self.render(state, now);

            apply_leds(board_leds, leds);

            Timer::after(Duration::from_millis(25)).await;
        }
    }

    fn render(&mut self, state: VisualState, now: Instant) -> LedState {
        if let Some(overlay) = &mut self.overlay {
            let started_at = match overlay.started_at {
                None => {
                    overlay.started_at = Some(now);
                    now
                }
                Some(started_at) => started_at,
            };

            let elapsed = now - started_at;

            let animation = match overlay.kind {
                VisualEvent::Initialized => initialized_animation(elapsed),
                VisualEvent::ConfigSaved => config_saved_animation(elapsed),
            };

            if let Some(animation) = animation {
                return animation;
            } else {
                self.overlay = None;
            }
        }

        render_base_state(state, now)
    }
}

#[cfg(feature = "hardware-support")]
fn calculate_visual_state() -> VisualState {
    VisualState {
        armed: false,
        calibrating: false,
        warning: ErrorRegister::shared().any_resolved(),
        fault: ErrorRegister::shared().any_ongoing(),
    }
}

fn render_base_state(state: VisualState, now: Instant) -> LedState {
    let mut leds = LedState {
        red: false,
        green: false,
    };

    // GREEN
    if state.calibrating {
        leds.green = blink(now, 200);
    } else if state.armed {
        leds.green = true;
    } else {
        leds.green = pulse(now, 2050, 50);
    }

    // RED
    if state.fault {
        leds.red = true;
    } else if state.warning {
        leds.red = pulse(now, 1050, 50);
    }

    leds
}

fn config_saved_animation(elapsed: Duration) -> Option<LedState> {
    let ms = elapsed.as_millis();

    if ms > 400 {
        return None;
    }

    Some(LedState {
        red: false,
        green: matches!(
            ms,
            0..=99 |
            200..=299
        ),
    })
}

fn initialized_animation(elapsed: Duration) -> Option<LedState> {
    const INITIAL_FLASH_DURATION: u64 = 300;
    let ms = elapsed.as_millis();

    if ms > INITIAL_FLASH_DURATION {
        return None;
    }

    Some(LedState {
        red: true,
        green: true,
    })
}

fn pulse(now: Instant, period_ms: u64, on_ms: u64) -> bool {
    now.as_millis() % period_ms < on_ms
}

fn blink(now: Instant, period_ms: u64) -> bool {
    now.as_millis() % period_ms < (period_ms / 2)
}

#[cfg(feature = "hardware-support")]
fn apply_leds(leds: &mut BoardLeds<'_>, state: LedState) {
    leds.red.set_level(state.red.into());
    leds.green.set_level(state.green.into());
}

#[cfg(test)]
mod tests {
    use crate::{
        LedAnimator, LedState, VisualEvent, VisualState, blink, config_saved_animation,
        initialized_animation, pulse, render_base_state,
    };
    use embassy_time::{Duration, Instant};

    fn instant(ms: u64) -> Instant {
        Instant::from_millis(ms)
    }

    fn duration(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }

    #[test]
    fn pulse_should_be_on_inside_window() {
        assert!(pulse(instant(25), 1000, 50));
    }

    #[test]
    fn pulse_should_be_off_outside_window() {
        assert!(!pulse(instant(100), 1000, 50));
    }

    #[test]
    fn blink_should_toggle_half_period() {
        assert!(blink(instant(50), 200));
        assert!(!blink(instant(150), 200));
    }

    #[test]
    fn config_saved_animation_should_double_flash() {
        assert_eq!(
            config_saved_animation(duration(50)),
            Some(LedState {
                red: false,
                green: true,
            })
        );

        assert_eq!(
            config_saved_animation(duration(150)),
            Some(LedState {
                red: false,
                green: false,
            })
        );

        assert_eq!(
            config_saved_animation(duration(250)),
            Some(LedState {
                red: false,
                green: true,
            })
        );

        assert_eq!(
            config_saved_animation(duration(350)),
            Some(LedState {
                red: false,
                green: false,
            })
        );
    }

    #[test]
    fn config_saved_animation_should_end_after_timeout() {
        assert_eq!(config_saved_animation(duration(401)), None);
    }

    #[test]
    fn initialized_animation_should_turn_on_both_leds() {
        assert_eq!(
            initialized_animation(duration(0)),
            Some(LedState {
                red: true,
                green: true,
            })
        );

        assert_eq!(
            initialized_animation(duration(150)),
            Some(LedState {
                red: true,
                green: true,
            })
        );

        assert_eq!(
            initialized_animation(duration(300)),
            Some(LedState {
                red: true,
                green: true,
            })
        );
    }

    #[test]
    fn initialized_animation_should_end_after_timeout() {
        assert_eq!(initialized_animation(duration(301)), None);
    }

    #[test]
    fn armed_should_have_solid_green() {
        let leds = render_base_state(
            VisualState {
                armed: true,
                warning: false,
                fault: false,
                calibrating: false,
            },
            instant(0),
        );

        assert_eq!(
            leds,
            LedState {
                red: false,
                green: true,
            }
        );
    }

    #[test]
    fn fault_should_have_solid_red() {
        let leds = render_base_state(
            VisualState {
                armed: false,
                warning: false,
                fault: true,
                calibrating: false,
            },
            instant(60),
        );

        assert_eq!(
            leds,
            LedState {
                red: true,
                green: false,
            }
        );
    }

    #[test]
    fn calibrating_should_blink_green() {
        let on = render_base_state(
            VisualState {
                armed: false,
                warning: false,
                fault: false,
                calibrating: true,
            },
            instant(50),
        );

        let off = render_base_state(
            VisualState {
                armed: false,
                warning: false,
                fault: false,
                calibrating: false,
            },
            instant(150),
        );

        assert_eq!(
            on,
            LedState {
                red: false,
                green: true,
            }
        );

        assert_eq!(
            off,
            LedState {
                red: false,
                green: false,
            }
        );
    }

    #[test]
    fn warning_should_pulse_red_and_green() {
        let active = render_base_state(
            VisualState {
                armed: false,
                warning: true,
                fault: false,
                calibrating: false,
            },
            instant(25),
        );

        let inactive = render_base_state(
            VisualState {
                armed: false,
                warning: false,
                fault: false,
                calibrating: false,
            },
            instant(500),
        );

        assert_eq!(
            active,
            LedState {
                red: true,
                green: true,
            }
        );

        assert_eq!(
            inactive,
            LedState {
                red: false,
                green: false,
            }
        );
    }

    #[test]
    fn overlay_should_override_base_state() {
        let mut animator = LedAnimator::new();

        animator.trigger(VisualEvent::Initialized);

        animator.render(VisualState::default(), instant(0));

        let leds = animator.render(
            VisualState {
                armed: true,
                warning: false,
                fault: false,
                calibrating: false,
            },
            instant(10),
        );

        assert_eq!(
            leds,
            LedState {
                red: true,
                green: true,
            }
        );
    }

    #[test]
    fn overlay_should_expire_and_return_to_base_state() {
        let mut animator = LedAnimator::new();

        animator.trigger(VisualEvent::Initialized);

        animator.render(VisualState::default(), instant(0));

        let leds = animator.render(
            VisualState {
                armed: true,
                warning: false,
                fault: false,
                calibrating: false,
            },
            instant(500),
        );

        assert_eq!(
            leds,
            LedState {
                red: false,
                green: true,
            }
        );

        assert!(animator.overlay.is_none());
    }
}
