use crate::BoardSerialNumber;
use embassy_usb::msos;

pub const DEVICE_INTERFACE_GUIDS: &[&str] = &["{EB67CAAB-F4DD-4066-BFF0-9B87C827660C}"];

pub struct UsbBuffers {
    pub config: [u8; 256],
    pub bos: [u8; 256],
    pub msos: [u8; 512],
    pub control: [u8; 4096],
}

impl Default for UsbBuffers {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbBuffers {
    pub const fn new() -> Self {
        Self {
            config: [0; 256],
            bos: [0; 256],
            msos: [0; 512],
            control: [0; 4096],
        }
    }
}

pub trait WinUsbExt {
    fn apply_win_usb(&mut self);
}

impl<'d, D: embassy_usb::driver::Driver<'d>> WinUsbExt for embassy_usb::Builder<'d, D> {
    fn apply_win_usb(&mut self) {
        self.msos_descriptor(msos::windows_version::WIN8_1, 2);
        self.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        self.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
        ));
    }
}

#[macro_export]
macro_rules! configure_dfu_win_usb {
    ($func:expr) => {
        $func.msos_feature(embassy_usb::msos::CompatibleIdFeatureDescriptor::new(
            "WINUSB", "",
        ));
        $func.msos_feature(embassy_usb::msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            embassy_usb::msos::PropertyData::RegMultiSz($crate::usb::DEVICE_INTERFACE_GUIDS),
        ));
    };
}

pub fn get_usb_config(serial_number: &'_ BoardSerialNumber) -> embassy_usb::Config<'_> {
    let mut config = embassy_usb::Config::new(0x1209, 0x2aaa);
    config.manufacturer = Some("UnoProgramo");
    config.product = Some("Pyrion Ovo");
    config.serial_number = Some(core::str::from_utf8(serial_number).unwrap());
    config.max_power = 500;
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
}
