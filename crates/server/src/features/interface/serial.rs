use crate::features::interface::device_info::DeviceInfo;
use crate::features::interface::error::{ConnectionError, DiscoveryError};
use crate::features::interface_kind::InterfaceKind;
use crate::features::session::DeviceHandle;
use tokio_serial::{
    SerialPortBuilderExt, SerialPortInfo, SerialPortType, SerialStream, available_ports,
};

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

    pub fn discover_devices(&self) -> Result<Vec<DeviceInfo>, DiscoveryError> {
        let available = available_ports()?;

        let devices = available
            .iter()
            .filter(|port| {
                !self.show_only_usb || matches!(port.port_type, SerialPortType::UsbPort(_))
            })
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
