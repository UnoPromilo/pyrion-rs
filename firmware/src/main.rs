#![no_std]
#![no_main]

use crate::as5600_angle_sensor::AS5600Sensor;
use crate::rp_motor_driver::MotorDriver;
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, i2c, peripherals, uart, watchdog};
use embassy_time::{Duration};
use embedded_io_async::Read;
use hardware_abstraction::motor_driver::MotorDriver as _;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod as5600_angle_sensor;
mod command_task;
mod rp_motor_driver;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<peripherals::I2C1>;
    UART0_IRQ => uart::BufferedInterruptHandler<peripherals::UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    //let mut watchdog = init_watchdog(p.WATCHDOG);
    let mut motor_driver = init_motor_driver(
        p.PWM_SLICE2,
        p.PIN_4,
        p.PIN_5,
        p.PWM_SLICE3,
        p.PIN_6,
        p.PIN_7,
        p.PWM_SLICE4,
        p.PIN_8,
        p.PIN_9,
    );
    let _as5600_sensor = init_as5600(p.I2C1, p.PIN_15, p.PIN_14).await;
    let uart = init_uart(p.UART0, p.PIN_0, p.PIN_1);
    let (_, rx) = uart.split();
    spawner.must_spawn(reader(rx));

    motor_driver.init();
    motor_driver.enable();
    loop {
        // set correct values
        //watchdog.feed();
    }
}

#[allow(dead_code)]
fn init_watchdog(w: peripherals::WATCHDOG) -> watchdog::Watchdog {
    let mut watchdog = watchdog::Watchdog::new(w);
    watchdog.start(Duration::from_millis(100));
    watchdog.pause_on_debug(true);
    watchdog
}

fn init_motor_driver(
    slice_a: peripherals::PWM_SLICE2,
    pin_a1: peripherals::PIN_4,
    pin_a2: peripherals::PIN_5,
    slice_b: peripherals::PWM_SLICE3,
    pin_b1: peripherals::PIN_6,
    pin_b2: peripherals::PIN_7,
    slice_c: peripherals::PWM_SLICE4,
    pin_c1: peripherals::PIN_8,
    pin_c2: peripherals::PIN_9,
) -> MotorDriver<'static> {
    let channel_a = rp_motor_driver::new_pwm_synced(slice_a, pin_a1, pin_a2);
    let channel_b = rp_motor_driver::new_pwm_synced(slice_b, pin_b1, pin_b2);
    let channel_c = rp_motor_driver::new_pwm_synced(slice_c, pin_c1, pin_c2);

    MotorDriver::new(channel_a, channel_b, channel_c)
}

async fn init_as5600(
    i2c: peripherals::I2C1,
    sda: peripherals::PIN_15,
    scl: peripherals::PIN_14,
) -> AS5600Sensor<i2c::I2c<'static, peripherals::I2C1, i2c::Async>> {
    let i2c_config = i2c::Config::default();
    let i2c = i2c::I2c::new_async(i2c, sda, scl, Irqs, i2c_config);
    let as5600_config = as5600::Config::default();
    let as5600 = as5600::AS5600::new(i2c, as5600_config).await.unwrap();
    AS5600Sensor::from(as5600)
}

fn init_uart(
    uart: peripherals::UART0,
    tx: peripherals::PIN_0,
    rx: peripherals::PIN_1,
) -> uart::BufferedUart<'static, peripherals::UART0> {
    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    uart::BufferedUart::new(uart, Irqs, tx, rx, tx_buf, rx_buf, uart::Config::default())
}

#[embassy_executor::task]
async fn reader(mut rx: uart::BufferedUartRx<'static, peripherals::UART0>) {
    info!("Reading...");
    loop {
        let mut buf = [0; 31];
        let result = rx.read_exact(&mut buf).await;
        match result {
            Ok(()) => info!("read {} bytes", buf.len()),
            Err(err) => warn!("read {} bytes, err = {:?}", buf.len(), err),
        }
        info!("RX {:?}", buf);
    }
}
