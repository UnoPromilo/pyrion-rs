use crate::board::BoardEncoder;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer};
use logging::{FreqMeter, error};
use portable_atomic::AtomicU16;

#[embassy_executor::task]
pub async fn task_encoder(mut encoder: BoardEncoder<'static>, raw_angle: &'static AtomicU16) {
    let state = controller_shared::state::state();
    let mut freq_meter = FreqMeter::named("ENC");
    freq_meter.link(&state.encoder_loop_frequency);
    loop {
        let result = encoder.read_angle().await;
        match result {
            Ok(angle) => raw_angle.store(angle, Ordering::Relaxed),
            Err(error) => {
                error!("Failed to read an angle: {}", error);
                Timer::after(Duration::from_millis(100)).await;
                let _ = encoder.write_config().await;
            }
        }
        freq_meter.tick();
    }
}
