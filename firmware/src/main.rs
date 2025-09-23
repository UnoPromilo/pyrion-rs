#![no_std]
#![no_main]

use crate::angle_sensor::update_angle_task;
use crate::config::{
    AngleSensorConfig, CurrentConfig, HardwareConfig, I2cConfig, MotorConfig, UartConfig,
};
use crate::current_sensor::{
    ThreePhaseCurrentReader, ThreePhaseCurrentTrigger, setup_current_sensor, update_current_task,
};
use crate::i2c::init_i2c;
use crate::motor_driver::drive_motor_task;
use crate::serial::read_from_serial_task;
use embassy_executor::{Executor, Spawner};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker};
use foc::Motor;
use shared::units::Velocity;
use shared::units::angle::Electrical;
use static_cell::StaticCell;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

mod angle_sensor;
mod config;
mod current_sensor;
mod i2c;
mod map;
mod motor_driver;
mod serial;

static MOTOR: StaticCell<Motor> = StaticCell::new();
static ADC_SIGNAL: Channel<CriticalSectionRawMutex, [u16; 3], 4> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let hardware_config = HardwareConfig::rp2040();
    let motor = init_motor();

    spawner.must_spawn(core0_task(
        spawner,
        motor,
        hardware_config.i2c,
        hardware_config.angle_sensor,
        hardware_config.uart,
        hardware_config.motor,
        hardware_config.current,
    ));
}

#[embassy_executor::task]
async fn core0_task(
    spawner: Spawner,
    motor: &'static Motor,
    i2c_config: Option<I2cConfig>,
    angle_sensor_config: AngleSensorConfig,
    uart_config: Option<UartConfig>,
    motor_config: MotorConfig,
    current_config: Option<CurrentConfig>,
) {
    let i2c = init_i2c(i2c_config);
    spawner.must_spawn(update_angle_task(&motor, i2c, angle_sensor_config));
    spawner.must_spawn(read_from_serial_task(&motor, uart_config));
    let current_sensor = setup_current_sensor(current_config, &ADC_SIGNAL);

    let (trigger, reader) = match current_sensor {
        None => (None, None),
        Some(current_sensor) => (Some(current_sensor.trigger), Some(current_sensor.reader)),
    };
    if let Some(reader) = reader {
        spawner.must_spawn(update_current_task(&motor, reader));
    }

    // Wait for other tasks to stabilize before running a motor driver
    embassy_time::Timer::after_millis(500).await;
    spawner.must_spawn(drive_motor_task(&motor, motor_config, trigger));
}

fn init_motor() -> &'static Motor {
    MOTOR.init(Motor::new())
}

#[allow(dead_code)]
#[embassy_executor::task]
async fn cyclic_logger(motor: &'static Motor) {
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        ticker.next().await;
        defmt::info!(
            "{:?}",
            motor
                .snapshot()
                .await
                .shaft
                .map(|s| s.filtered_velocity)
                .unwrap_or(Velocity::<Electrical>::ZERO)
        );
    }
}
