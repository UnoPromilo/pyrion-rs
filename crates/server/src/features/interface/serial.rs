use crate::features::interface::device_info::DeviceInfo;
use crate::features::interface::error::{ConnectionError, DiscoveryError};
use crate::features::interface_kind::InterfaceKind;
use crate::features::session::DeviceHandle;
use tokio_serial::{
    SerialPortBuilderExt, SerialPortInfo, SerialPortType, SerialStream, available_ports,
};

#[derive(Debug)]
pub struct SerialInterface {
    show_only_usb_devices: bool,
    hide_call_up_devices: bool,
}

impl SerialInterface {
    pub fn new_from_config(config: crate::configuration::SerialConfiguration) -> Self {
        Self {
            show_only_usb_devices: config.show_only_usb_devices,
            hide_call_up_devices: config.hide_call_up_devices,
        }
    }

    pub fn discover_devices(&self) -> Result<Vec<DeviceInfo>, DiscoveryError> {
        let available = available_ports()?;

        let devices = available
            .iter()
            .filter(|port| should_show_device(port, self.show_only_usb_devices, self.hide_call_up_devices))
            .map(|port| DeviceInfo {
                address: port.port_name.to_string(),
                interface: InterfaceKind::Serial,
                name: try_get_name(port),
                firmware: None,
            })
            .collect();
        Ok(devices)
    }

    pub fn get_device_handler(
        &self,
        address: &str,
    ) -> Result<DeviceHandle<SerialStream>, ConnectionError> {
        let device = tokio_serial::new(address, 115200).open_native_async()?;
        Ok(DeviceHandle::new(device))
    }
}

fn try_get_name(serial_port_info: &SerialPortInfo) -> Option<String> {
    match &serial_port_info.port_type {
        SerialPortType::UsbPort(port_info) => port_info.product.clone(),
        _ => None,
    }
}

fn should_show_device(port: &SerialPortInfo, show_only_usb_devices: bool, hide_call_up_devices: bool) -> bool {
    if show_only_usb_devices && !matches!(port.port_type, SerialPortType::UsbPort(_)) {
        return false;
    }

    if hide_call_up_devices && port.port_name.contains("cu") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_serial::UsbPortInfo;

    #[derive(Debug)]
    struct TestCase {
        name: &'static str,
        port_name: &'static str,
        port_type: SerialPortType,
        show_only_usb_devices: bool,
        hide_call_up_devices: bool,
        expected: bool,
    }

    #[test]
    fn test_should_show_device() {
        let cases = [
            TestCase {
                name: "usb port, show all, don't hide cu",
                port_name: "ttyUSB0",
                port_type: SerialPortType::UsbPort(get_dummy_usb_port_info()),
                show_only_usb_devices: false,
                hide_call_up_devices: false,
                expected: true,
            },
            TestCase {
                name: "usb port, only usb, don't hide cu",
                port_name: "ttyUSB1",
                port_type: SerialPortType::UsbPort(get_dummy_usb_port_info()),
                show_only_usb_devices: true,
                hide_call_up_devices: false,
                expected: true,
            },
            TestCase {
                name: "usb port, only usb, hide cu but no cu in name",
                port_name: "ttyUSB2",
                port_type: SerialPortType::UsbPort(get_dummy_usb_port_info()),
                show_only_usb_devices: true,
                hide_call_up_devices: true,
                expected: true,
            },
            TestCase {
                name: "usb port, only usb, hide cu and cu in name",
                port_name: "cu.usbserial",
                port_type: SerialPortType::UsbPort(get_dummy_usb_port_info()),
                show_only_usb_devices: true,
                hide_call_up_devices: true,
                expected: false,
            },
            TestCase {
                name: "pci port, show all, don't hide cu",
                port_name: "ttyS0",
                port_type: SerialPortType::PciPort,
                show_only_usb_devices: false,
                hide_call_up_devices: false,
                expected: true,
            },
            TestCase {
                name: "pci port, only usb, don't hide cu",
                port_name: "ttyS1",
                port_type: SerialPortType::PciPort,
                show_only_usb_devices: true,
                hide_call_up_devices: false,
                expected: false,
            },
            TestCase {
                name: "bluetooth port, only usb, don't hide cu",
                port_name: "rfcomm0",
                port_type: SerialPortType::BluetoothPort,
                show_only_usb_devices: true,
                hide_call_up_devices: false,
                expected: false,
            },
            TestCase {
                name: "unknown port, show all, hide cu",
                port_name: "cu.unknown",
                port_type: SerialPortType::Unknown,
                show_only_usb_devices: false,
                hide_call_up_devices: true,
                expected: false,
            },
            TestCase {
                name: "unknown port, show all, don't hide cu",
                port_name: "cu.unknown",
                port_type: SerialPortType::Unknown,
                show_only_usb_devices: false,
                hide_call_up_devices: false,
                expected: true,
            },
        ];

        for case in cases {
            let port = SerialPortInfo {
                port_name: case.port_name.to_string(),
                port_type: case.port_type.clone(),
            };

            let result = should_show_device(&port, case.show_only_usb_devices, case.hide_call_up_devices);

            assert_eq!(
                result, case.expected,
                "failed case: {} -> got {}, expected {}",
                case.name, result, case.expected
            );
        }

        fn get_dummy_usb_port_info() -> UsbPortInfo {
            UsbPortInfo {
                vid: 0,
                pid: 0,
                serial_number: None,
                manufacturer: None,
                product: None,
            }
        }
    }
}
