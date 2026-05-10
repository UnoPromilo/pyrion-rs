#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};
use defmt::info;
use embassy_boot_stm32::*;
use embassy_stm32::flash::{BANK1_REGION, WRITE_SIZE};
use embassy_usb::Builder;
use hardware::configure_dfu_win_usb;
use hardware::usb::{UsbBuffers, WinUsbExt, get_usb_config};

use crate::dfu::{new_state, usb_dfu};
#[allow(unused_imports)]
use defmt_rtt as _;

mod dfu;

#[entry]
fn main() -> ! {
    let mut board = hardware::Board::init();

    let config = BootLoaderConfig::from_linkerfile_blocking(
        &board.flash_bank1,
        &board.flash_bank2,
        &board.flash_bank1,
    );
    let active_offset = config.active.offset();
    let bl = BootLoader::prepare::<_, _, _, 8>(config);

    if bl.state == State::DfuDetach {
        info!("Entering detached state");
        let fw_config =
            FirmwareUpdaterConfig::from_linkerfile_blocking(&board.flash_bank2, &board.flash_bank1);
        let mut aligned_buffer = AlignedBuffer([0; WRITE_SIZE]);
        let updater = BlockingFirmwareUpdater::new(fw_config, &mut aligned_buffer.0[..]);

        let usb_config = get_usb_config(&board.serial_number);
        let mut usb_buffers = UsbBuffers::new();
        board.leds.green.set_high();
        let mut dfu_state = new_state(updater, board.leds);

        let mut builder = Builder::new(
            board.usb,
            usb_config,
            &mut usb_buffers.config,
            &mut usb_buffers.bos,
            &mut usb_buffers.msos,
            &mut usb_buffers.control,
        );

        builder.apply_win_usb();
        usb_dfu::<_, _, _, 4096>(&mut builder, &mut dfu_state, |func| {
            configure_dfu_win_usb!(func);
        });

        let mut dev = builder.build();
        embassy_futures::block_on(dev.run());
    }

    info!("Booting");
    unsafe { bl.load(BANK1_REGION.base() + active_offset) }
}

#[unsafe(no_mangle)]
#[cfg_attr(target_os = "none", unsafe(link_section = ".HardFault.user"))]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = unsafe { core::ptr::read_volatile(SCB_ICSR) } as u8 as i16 - 16;

    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
