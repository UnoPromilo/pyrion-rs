#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::{entry, exception};
use embassy_boot_stm32::*;
use embassy_stm32::flash::{BANK1_REGION, Flash, WRITE_SIZE};
use embassy_stm32::rcc::mux::{Clk48sel, Fdcansel};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::Mutex;
use embassy_usb::class::dfu::consts::DfuAttributes;
use embassy_usb::{Builder, msos};
use embassy_usb_dfu::ResetImmediate;
use embassy_usb_dfu::dfu::{new_state, usb_dfu};

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{EB67CAAB-F4DD-4066-BFF0-9B87C827660C}"];

#[entry]
fn main() -> ! {
    let config = {
        use embassy_stm32::rcc::*;
        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(24),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL85,
            divp: None,
            divq: Some(PllQDiv::DIV8),
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.mux.adc345sel = mux::Adcsel::SYS;
        config.rcc.mux.clk48sel = Clk48sel::HSI48;
        config.rcc.mux.fdcansel = Fdcansel::PLL1_Q;
        config.rcc.boost = true;
        config
    };
    let p = embassy_stm32::init(config);

    let layout = Flash::new_blocking(p.FLASH).into_blocking_regions();
    let flash_bank1 = Mutex::new(RefCell::new(layout.bank1_region));
    let flash_bank2 = Mutex::new(RefCell::new(layout.bank2_region));

    let config =
        BootLoaderConfig::from_linkerfile_blocking(&flash_bank1, &flash_bank2, &flash_bank1);
    let active_offset = config.active.offset();
    let bl = BootLoader::prepare::<_, _, _, 8>(config);

    if bl.state == State::DfuDetach {
        let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
        let mut config = embassy_usb::Config::new(0x1209, 0x2aaa);
        config.manufacturer = Some("UnoProgramo");
        config.product = Some("Pyrion Bootloader");
        let serial_hex = serial_hex(embassy_stm32::uid::uid());
        config.serial_number = Some(core::str::from_utf8(&serial_hex).unwrap());
        let fw_config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash_bank2, &flash_bank1);
        let mut buffer = AlignedBuffer([0; WRITE_SIZE]);
        let updater = BlockingFirmwareUpdater::new(fw_config, &mut buffer.0[..]);

        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut msos_descriptor = [0; 512];
        let mut control_buf = [0; 4096];
        let mut state = new_state(updater, DfuAttributes::CAN_DOWNLOAD, ResetImmediate);

        let mut builder = Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );

        builder.msos_descriptor(msos::windows_version::WIN8_1, 2);
        builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
        ));

        usb_dfu::<_, _, _, _, 4096>(&mut builder, &mut state, |func| {
            func.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
            func.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
                "DeviceInterfaceGUIDs",
                msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
            ));
        });

        let mut dev = builder.build();
        embassy_futures::block_on(dev.run());
    }

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

fn serial_hex(b: [u8; 12]) -> [u8; 24] {
    fn hex(n: u8) -> u8 {
        b"0123456789abcdef"[n as usize]
    }

    let mut out = [0u8; 24];
    let mut i = 0;
    for &byte in &b {
        out[i] = hex(byte >> 4);
        out[i + 1] = hex(byte & 0x0F);
        i += 2;
    }
    out
}
