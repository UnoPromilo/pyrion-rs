use crate::board::BoardI2c;
use as5600::AS5600;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer};
use logging::error_register::ErrorRegister;
use logging::{FreqMeter, error, error_register};

pub async fn task_as5600(ext_i2c: BoardI2c<'static>) {
    let config = as5600::Config::default();
    let mut as5600 = AS5600::new(ext_i2c, config);
    loop {
        let error = run_until_failure(&mut as5600).await.unwrap_err();
        error!("AS5600 error: {}", error);
        ErrorRegister::shared().set(error_register::Error::ShaftPositionDetector);
        Timer::after(Duration::from_millis(100)).await;
    }
}

async fn run_until_failure<'a>(as5600: &mut AS5600<BoardI2c<'a>>) -> Result<(), as5600::Error> {
    let state = controller_shared::state::state();
    as5600.write_config().await?;
    let mut freq_meter = FreqMeter::named("ENC");
    freq_meter.link(&state.encoder_loop_frequency);
    ErrorRegister::shared().resolve_if_set(error_register::Error::ShaftPositionDetector);
    loop {
        let angle = as5600.read_angle().await?;
        state.raw_angle.store(angle, Ordering::Relaxed);
        freq_meter.tick();
    }
}
