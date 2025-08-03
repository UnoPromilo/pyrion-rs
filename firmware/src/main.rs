#![no_std]
#![no_main]

use crate::angle_sensor::update_angle_task;
use crate::config::PhysicalConfig;
use crate::current_sensor::update_current_dma_task;
use crate::i2c::init_i2c;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, peripherals, uart};
use foc::Motor;
use static_cell::StaticCell;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

mod angle_sensor;
mod command_task;
mod config;
mod current_sensor;
mod i2c;
mod map;
mod rp_motor_driver;

bind_interrupts!(struct Irqs {

    UART0_IRQ => uart::BufferedInterruptHandler<peripherals::UART0>;
});

static MOTOR: StaticCell<Motor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let physical_config = PhysicalConfig::rp2040();
    let motor = init_motor();
    let i2c = init_i2c(physical_config.i2c);
    spawner.must_spawn(update_current_dma_task(&motor, physical_config.current));
    spawner.must_spawn(update_angle_task(&motor, i2c, physical_config.angle_sensor));
}

fn init_motor() -> &'static Motor {
    MOTOR.init(Motor::new())
}
