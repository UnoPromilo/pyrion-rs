#![no_std]
#![no_main]

use crate::angle_sensor::update_angle_task;
use crate::config::{
    AngleSensorConfig, CurrentConfig, HardwareConfig, I2cConfig, MotorConfig, UartConfig,
};
use crate::current_sensor::update_current_dma_task;
use crate::i2c::init_i2c;
use crate::motor_driver::drive_motor_task;
use crate::serial::read_from_serial_task;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Instant;
use foc::Motor;
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
static PWM_WRAP_SIGNAL: Signal<CriticalSectionRawMutex, Instant> = Signal::new();
static CORE1_STACK: StaticCell<Stack<4096>> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let hardware_config = HardwareConfig::rp2040();
    let motor = init_motor();

    embassy_rp::multicore::spawn_core1(
        hardware_config.core1,
        CORE1_STACK.init(Stack::new()),
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.must_spawn(core1_task(spawner, motor, hardware_config.current))
            });
        },
    );

    spawner.must_spawn(core0_task(
        spawner,
        motor,
        hardware_config.i2c,
        hardware_config.angle_sensor,
        hardware_config.uart,
        hardware_config.motor,
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
) {
    let i2c = init_i2c(i2c_config);
    spawner.must_spawn(update_angle_task(&motor, i2c, angle_sensor_config));
    spawner.must_spawn(read_from_serial_task(&motor, uart_config));

    // Wait for other tasks to stabilize before running a motor driver
    embassy_time::Timer::after_millis(500).await;
    spawner.must_spawn(drive_motor_task(&motor, motor_config));
}

#[embassy_executor::task]
async fn core1_task(
    spawner: Spawner,
    motor: &'static Motor,
    current_config: Option<CurrentConfig>,
) {
    spawner.must_spawn(update_current_dma_task(&motor, current_config))
}

fn init_motor() -> &'static Motor {
    MOTOR.init(Motor::new())
}
