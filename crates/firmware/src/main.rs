#![no_std]
#![no_main]
#![allow(clippy::bool_comparison)]

use crate::version::populate_version;
use core::panic::PanicInfo;
use cortex_m_rt::{entry, exception};
use embassy_executor::{Executor, InterruptExecutor};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use static_cell::StaticCell;
use user_config::UserConfig;

mod app;
mod version;

#[allow(unused_imports)]
use defmt_rtt as _;
use hardware::usb::get_usb_config;
use hardware::{BoardFlashBank1, BoardFlashBank2, BoardSerialNumber};

hardware::exactly_one_of!(
    "firmware image must select exactly one board-* feature",
    "board-pyrion-ovo",
    "board-pyrion-nullo",
);

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();
static USER_CONFIG: StaticCell<UserConfig> = StaticCell::new();
static FLASH_BANK1: StaticCell<BoardFlashBank1> = StaticCell::new();
static FLASH_BANK2: StaticCell<BoardFlashBank2> = StaticCell::new();
static SERIAL_NUMBER: StaticCell<BoardSerialNumber> = StaticCell::new();

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
    let board = hardware::Board::init(user_config);
    let serial_number = SERIAL_NUMBER.init(board.serial_number);
    let flash_bank1 = FLASH_BANK1.init(board.flash_bank1);
    let flash_bank2 = FLASH_BANK2.init(board.flash_bank2);

    let usb_config = get_usb_config(serial_number);

    interrupt::UART4.set_priority(Priority::P6);
    let high_priority_spawner = EXECUTOR_HIGH.start(interrupt::UART4);
    high_priority_spawner.spawn(app::task_adc(board.adc, board.inverter).unwrap());

    interrupt::UART5.set_priority(Priority::P7);

    #[cfg(feature = "cap-external-i2c")]
    {
        let medium_priority_spawner = EXECUTOR_MED.start(interrupt::UART5);
        medium_priority_spawner
            .spawn(app::task_shaft_position(board.ext_i2c, user_config).unwrap());
    }

    let low_priority_executor = EXECUTOR_LOW.init(Executor::new());
    low_priority_executor.run(|low_priority_spawner| {
        low_priority_spawner.spawn(app::task_communication(board.crc).unwrap());
        #[cfg(feature = "cap-uart")]
        low_priority_spawner.spawn(app::task_uart(board.uart).unwrap());
        low_priority_spawner.spawn(app::task_leds(board.leds).unwrap());
        low_priority_spawner
            .spawn(app::task_usb(board.usb, usb_config, flash_bank1, flash_bank2).unwrap());
    });
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}

#[unsafe(no_mangle)]
#[cfg_attr(target_os = "none", unsafe(link_section = ".HardFault.user"))]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}
