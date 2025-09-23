#![no_std]
#![no_main]

use embassy_time::Timer;
use embassy_stm32::gpio::Speed;
use embassy_stm32::gpio::Level;
use embassy_stm32::gpio::Output;
use shared::info;
use embassy_executor::Spawner;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB7, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
