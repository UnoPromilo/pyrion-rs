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

#[interrupt]
unsafe fn TIM1_CC() {
    #[cfg(feature = "adc-timing")]
    let cycle = drivers::adc::timing::cycle_count();
    let sr = embassy_stm32::pac::TIM1.sr();
    #[cfg(feature = "adc-timing")]
    let down_before =
        embassy_stm32::pac::TIM1.cr1().read().dir() == embassy_stm32::pac::timer::vals::Dir::DOWN;
    #[cfg(feature = "adc-timing")]
    let timer_count = embassy_stm32::pac::TIM1.cnt().read().cnt();
    #[cfg(feature = "adc-timing")]
    let down_after =
        embassy_stm32::pac::TIM1.cr1().read().dir() == embassy_stm32::pac::timer::vals::Dir::DOWN;
    let status = sr.read();
    if status.ccif(3) {
        sr.modify(|reg| {
            reg.set_ccif(3, false);
            reg.set_ccof(3, false);
        });
        #[cfg(feature = "adc-timing")]
        let pending_mask = u8::from(embassy_stm32::pac::ADC1.isr().read().jeos())
            | (u8::from(embassy_stm32::pac::ADC3.isr().read().jeos()) << 1)
            | (u8::from(embassy_stm32::pac::ADC5.isr().read().jeos()) << 2);
        #[cfg(not(feature = "adc-timing"))]
        let pending_mask = u8::from(embassy_stm32::pac::ADC1.isr().read().jeos())
            | (u8::from(embassy_stm32::pac::ADC3.isr().read().jeos()) << 1)
            | (u8::from(embassy_stm32::pac::ADC5.isr().read().jeos()) << 2);
        #[cfg(feature = "adc-timing")]
        let compare = embassy_stm32::pac::TIM1.ccr(3).read().ccr();
        #[cfg(feature = "adc-timing")]
        let current_edge_confirmed = pending_mask == 0
            || (!status.ccof(3) && down_before && down_after && timer_count <= compare);
        #[cfg(not(feature = "adc-timing"))]
        let current_edge_confirmed = pending_mask == 0
            || (!status.ccof(3)
                && embassy_stm32::pac::TIM1.cr1().read().dir()
                    == embassy_stm32::pac::timer::vals::Dir::DOWN
                && embassy_stm32::pac::TIM1.cnt().read().cnt()
                    <= embassy_stm32::pac::TIM1.ccr(3).read().ccr());
        let invalid = drivers::adc::epoch::on_tim1_trigger(pending_mask, current_edge_confirmed);
        #[cfg(not(feature = "adc-timing"))]
        let _ = invalid;
        #[cfg(feature = "adc-timing")]
        drivers::adc::timing::record_trigger(
            drivers::adc::epoch::TriggerEpoch::current(),
            cycle,
            drivers::adc::timing::TimerSnapshot {
                pending_mask,
                count: timer_count,
                down_before,
                down_after,
                compare,
            },
            invalid,
        );
    }
}

#[entry]
fn main() -> ! {
    populate_version();
    #[cfg(feature = "adc-timing")]
    {
        // SAFETY: The timing feature is the only user of DCB and DWT in this firmware.
        let mut core = unsafe { cortex_m::Peripherals::steal() };
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
    }
    let user_config = USER_CONFIG.init(UserConfig::default());
    #[cfg(feature = "adc-timing")]
    logging::info!(
        "ADC timing configured PWM Hz={}",
        user_config.pwm_frequency.0
    );
    let board = hardware::Board::init(user_config);
    let serial_number = SERIAL_NUMBER.init(board.serial_number);
    let flash_bank1 = FLASH_BANK1.init(board.flash_bank1);
    let flash_bank2 = FLASH_BANK2.init(board.flash_bank2);

    let usb_config = get_usb_config(serial_number);
    let (mut fast_output, slow_control) = board.power_stage.split();

    interrupt::TIM1_CC.set_priority(Priority::P0);
    interrupt::ADC1_2.set_priority(Priority::P1);
    interrupt::ADC3.set_priority(Priority::P1);
    interrupt::ADC5.set_priority(Priority::P1);
    fast_output.pwm_mut().enable_adc_trigger_interrupt();
    unsafe { interrupt::TIM1_CC.enable() };

    interrupt::UART4.set_priority(Priority::P6);
    let high_priority_spawner = EXECUTOR_HIGH.start(interrupt::UART4);
    high_priority_spawner.spawn(app::task_adc(board.adc.fast, fast_output).unwrap());

    interrupt::UART5.set_priority(Priority::P7);

    #[cfg(feature = "cap-external-i2c")]
    {
        let medium_priority_spawner = EXECUTOR_MED.start(interrupt::UART5);
        medium_priority_spawner
            .spawn(app::task_shaft_position(board.ext_i2c, user_config).unwrap());
    }

    let low_priority_executor = EXECUTOR_LOW.init(Executor::new());
    low_priority_executor.run(|low_priority_spawner| {
        low_priority_spawner.spawn(app::task_slow_vref(board.adc.slow_vref).unwrap());
        low_priority_spawner.spawn(app::task_slow_aux(board.adc.slow_aux).unwrap());
        #[cfg(feature = "adc-timing")]
        low_priority_spawner.spawn(app::task_adc_timing_report().unwrap());
        low_priority_spawner.spawn(app::task_gate_driver(slow_control).unwrap());
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
