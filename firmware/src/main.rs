#![no_std]
#![no_main]

use embassy_rp::watchdog::Watchdog;
use crate::as5600_angle_sensor::AS5600Sensor;
use crate::rp_motor_driver::{Channel, MotorDriver};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c;
use embassy_rp::pwm::PwmBatch;
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

mod as5600_angle_sensor;
mod rp_motor_driver;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let p = embassy_rp::init(Default::default());
    // TODO(safety): Implement commutation watchdog timeout
    // If angle read or update loop fails, shut off motor to avoid damage
    // let mut watchdog = Watchdog::new(p.WATCHDOG);
    // watchdog.start(Duration::from_millis(100));
    
    let channel_a = Channel::new_synced(p.PWM_SLICE2, p.PIN_4, p.PIN_5);
    let channel_b = Channel::new_synced(p.PWM_SLICE3, p.PIN_6, p.PIN_7);
    let channel_c = Channel::new_synced(p.PWM_SLICE4, p.PIN_8, p.PIN_9);

    PwmBatch::set_enabled(true, |batch| {
        channel_a.register_in_batch(batch);
        channel_b.register_in_batch(batch);
        channel_c.register_in_batch(batch);
    });

    let motor_driver = MotorDriver::new(channel_a, channel_b, channel_c);

    let i2c_config = i2c::Config::default();
    let i2c = i2c::I2c::new_async(p.I2C1, p.PIN_15, p.PIN_14, Irqs, i2c_config);
    let as5600_config = as5600::Config::default();
    let as5600 = as5600::AS5600::new(i2c, as5600_config).await.unwrap();
    let as5600_sensor = AS5600Sensor::from(as5600);

    loop {}
}
