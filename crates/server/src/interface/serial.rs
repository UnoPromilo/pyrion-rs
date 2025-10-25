use crate::interface::common::{DeviceInfo, Interface, InterfaceType};
use crate::interface::error::DiscoveryError;
use tokio_serial::{SerialPortInfo, SerialPortType, available_ports};

#[derive(Debug)]
pub struct SerialInterface {
    show_only_usb: bool,
}

impl SerialInterface {
    pub fn new_from_config(config: crate::configuration::SerialConfiguration) -> Self {
        Self {
            show_only_usb: config.show_only_usb,
        }
    }
}

impl Interface for SerialInterface {
    fn interface_type(&self) -> InterfaceType {
        InterfaceType::Serial
    }

    fn discover_devices(&self) -> Result<Vec<DeviceInfo>, DiscoveryError> {
        let available = available_ports()?;

        let devices = available
            .iter()
            .filter(|port| {
                !self.show_only_usb || matches!(port.port_type, SerialPortType::UsbPort(_))
            })
            .map(|port| DeviceInfo {
                address: port.port_name.to_string(),
                interface: InterfaceType::Serial,
                name: try_get_name(port),
                firmware: None,
            })
            .collect();
        Ok(devices)
    }
}

fn try_get_name(serial_port_info: &SerialPortInfo) -> Option<String> {
    match &serial_port_info.port_type {
        SerialPortType::UsbPort(port_info) => port_info.product.clone(),
        _ => None,
    }
}
