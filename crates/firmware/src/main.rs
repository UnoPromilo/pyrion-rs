#![no_std]
#![no_main]

use core::arch::asm;
use embassy_executor::Spawner;

mod board;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut board = board::Board::init();
    loop {
        unsafe { asm!("nop") };
    }
}
