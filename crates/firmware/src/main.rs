#![no_std]
#![no_main]
#![allow(clippy::bool_comparison)]

use crate::version::populate_version;
use cortex_m_rt::entry;
use embassy_executor::{Executor, InterruptExecutor};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use static_cell::StaticCell;
use user_config::UserConfig;

mod app;
mod board;
mod version;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();
static USER_CONFIG: StaticCell<UserConfig> = StaticCell::new();

#[interrupt]
unsafe fn UART4() {
    unsafe { EXECUTOR_HIGH.on_interrupt() }
}

#[interrupt]
unsafe fn UART5() {
    unsafe { EXECUTOR_MED.on_interrupt() }
}

#[entry]
fn main() -> ! {
    populate_version();
    let user_config = USER_CONFIG.init(UserConfig::default());
    let board = {
        let result = board::Board::init(user_config);
        match result {
            Ok(board) => board,
            Err(e) => {
                panic!("Failed to initialize board: {:?}", e);
            }
        }
    };

    interrupt::UART4.set_priority(Priority::P6);
    let high_priority_spawner = EXECUTOR_HIGH.start(interrupt::UART4);
    high_priority_spawner.must_spawn(app::task_adc(board.adc, board.inverter));

    interrupt::UART5.set_priority(Priority::P7);
    let medium_priority_spawner = EXECUTOR_MED.start(interrupt::UART5);
    medium_priority_spawner.must_spawn(app::task_shaft_position(board.ext_i2c, user_config));

    let low_priority_executor = EXECUTOR_LOW.init(Executor::new());
    low_priority_executor.run(|low_priority_spawner| {
        low_priority_spawner.must_spawn(app::task_communication(board.crc, board.flash));
        low_priority_spawner.must_spawn(app::task_uart(board.uart));
        low_priority_spawner.must_spawn(app::task_usb(board.usb, user_config));
    });
}
