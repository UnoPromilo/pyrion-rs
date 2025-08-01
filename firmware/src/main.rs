#![no_std]
#![no_main]

use crate::as5600_angle_sensor::AS5600Sensor;
use crate::rp_motor_driver::MotorDriver;
use as5600::Error;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, i2c, peripherals, uart, watchdog};
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod as5600_angle_sensor;
mod command_task;
mod map;
mod rp_motor_driver;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
    UART0_IRQ => uart::BufferedInterruptHandler<peripherals::UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    //let mut watchdog = init_watchdog(p.WATCHDOG);
}

#[allow(dead_code)]
fn init_watchdog(w: peripherals::WATCHDOG) -> watchdog::Watchdog {
    let mut watchdog = watchdog::Watchdog::new(w);
    watchdog.start(Duration::from_millis(100));
    watchdog.pause_on_debug(true);
    watchdog
}



