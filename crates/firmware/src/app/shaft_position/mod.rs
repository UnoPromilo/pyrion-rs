#[cfg(feature = "cap-external-i2c")]
use hardware::BoardI2c;
#[cfg(feature = "cap-external-i2c")]
use logging::info;
#[cfg(feature = "cap-external-i2c")]
use user_config::{ShaftPositionDetector, UserConfig};

#[cfg(feature = "cap-external-i2c")]
mod as5600;

#[cfg(feature = "cap-external-i2c")]
#[embassy_executor::task]
pub async fn task_shaft_position(ext_i2c: BoardI2c<'static>, user_config: &'static UserConfig) {
    info!(
        "Selected shaft position detector: {:?}",
        user_config.shaft_position_detector
    );
    match user_config.shaft_position_detector {
        ShaftPositionDetector::OpenLoop => {}
        ShaftPositionDetector::AS5600 => as5600::task_as5600(ext_i2c).await,
    }
}
