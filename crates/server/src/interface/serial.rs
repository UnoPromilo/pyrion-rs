use crate::interface::common::{DeviceInfo, Interface, InterfaceType};
use crate::interface::error::DiscoveryError;
use tokio_serial::available_ports;

#[derive(Debug)]
pub struct SerialInterface;

impl Interface for SerialInterface {
    fn interface_type(&self) -> InterfaceType {
        InterfaceType::Serial
    }

    fn discover_devices(&self) -> Result<Vec<DeviceInfo>, DiscoveryError> {
        let available = available_ports()?;

        let devices = available
            .iter()
            .map(|port| DeviceInfo {
                address: port.port_name.to_string(),
                interface: InterfaceType::Serial,
                name: None,
                firmware: None,
            })
            .collect();
        Ok(devices)
    }
}
