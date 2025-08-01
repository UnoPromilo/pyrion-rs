#![no_std]
#![no_main]

use crate::config::PhysicalConfig;
use crate::current_sensor::update_current_dma;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, i2c, peripherals, uart};
use foc::Motor;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};


mod as5600_angle_sensor;
mod command_task;
mod config;
mod current_sensor;
mod map;
mod rp_motor_driver;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
    UART0_IRQ => uart::BufferedInterruptHandler<peripherals::UART0>;
});

static MOTOR: StaticCell<Motor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let physical_config = PhysicalConfig::rp2040();
    let motor = init_motor();
    spawner.must_spawn(update_current_dma(&motor, physical_config.current_config));
}

fn init_motor() -> &'static Motor {
    MOTOR.init(Motor::new())
}
