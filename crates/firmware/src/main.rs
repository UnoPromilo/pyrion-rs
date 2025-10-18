#![no_std]
#![no_main]
#![allow(clippy::bool_comparison)]

use embassy_executor::Spawner;
use portable_atomic::AtomicU16;
use static_cell::StaticCell;

mod app;
mod board;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = {
        let result = board::Board::init().await;
        match result {
            Ok(board) => board,
            Err(e) => {
                panic!("Failed to initialize board: {:?}", e);
            }
        }
    };
    static RAW_ANGLE: StaticCell<AtomicU16> = StaticCell::new();
    let raw_angle = RAW_ANGLE.init(AtomicU16::new(0));
    let (board_adc, board_inverter, board_encoder, board_uart, board_crc) = board.split();
    spawner.must_spawn(app::task_encoder(board_encoder, raw_angle));
    spawner.must_spawn(app::task_adc(board_adc, board_inverter, raw_angle));
    spawner.must_spawn(app::task_uart(board_uart, board_crc));
}
