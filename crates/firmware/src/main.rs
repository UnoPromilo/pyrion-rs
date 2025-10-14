#![no_std]
#![no_main]
#![allow(clippy::bool_comparison)]

use embassy_executor::Spawner;
use embassy_futures::yield_now;

mod app;
mod board;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = board::Board::init();
    spawner.must_spawn(app::task(board));
    loop {
        yield_now().await;
    }
}
