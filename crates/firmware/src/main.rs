#![no_std]
#![no_main]
#![allow(clippy::bool_comparison)]
use crate::version::populate_version;
use embassy_executor::Spawner;

mod app;
mod board;
mod version;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    populate_version();
    let board = {
        let result = board::Board::init().await;
        match result {
            Ok(board) => board,
            Err(e) => {
                panic!("Failed to initialize board: {:?}", e);
            }
        }
    };
    let (board_adc, board_inverter, board_uart, board_crc, board_usb) = board.split();
    //spawner.must_spawn(app::task_encoder(board_encoder));
    spawner.must_spawn(app::task_adc(board_adc, board_inverter));

    spawner.must_spawn(app::task_communication(board_crc));
    spawner.must_spawn(app::task_uart(board_uart));
    spawner.must_spawn(app::task_usb(board_usb));
}
