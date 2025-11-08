use crate::board::BoardI2c;
use logging::info;
use user_config::{ShaftPositionDetector, UserConfig};

mod as5600;

#[embassy_executor::task]
pub async fn task_shaft_position(ext_i2c: BoardI2c<'static>, user_config: &'static UserConfig) {
    info!(
        "Selected shaft position detector: {:?}",
        user_config.shaft_position_detector
    );
    match user_config.shaft_position_detector {
        ShaftPositionDetector::None => {
            // TODO how to handle that
            todo!()
        }
        ShaftPositionDetector::AS5600 => as5600::task_as5600(ext_i2c).await,
    }
}
