use crate::features::interface::device_info::DeviceInfo;
use crate::features::interface::error::ConnectionError;
use crate::features::interface::serial::SerialInterface;
use crate::features::interface_kind::InterfaceKind;
use crate::features::session::DeviceHandleWrapper;

#[derive(Debug)]
pub struct InterfaceManager {
    serial: Option<SerialInterface>,
}

impl Default for InterfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceManager {
    pub fn new() -> Self {
        Self { serial: None }
    }

    pub fn add_serial_interface(&mut self, interface: SerialInterface) {
        self.serial = Some(interface);
    }

    pub fn discover_devices(&self) -> Vec<DeviceInfo> {
        let mut all_devices = Vec::new();

        if let Some(interface) = &self.serial {
            match interface.discover_devices() {
                Ok(devices) => all_devices.extend(devices),
                Err(e) => {
                    tracing::warn!("Failed to discover devices on serial interface: {} ", e);
                }
            }
        }

        all_devices
    }

    pub fn get_device_handler(
        &self,
        interface_type: InterfaceKind,
        address: &str,
    ) -> Result<DeviceHandleWrapper, ConnectionError> {
        match interface_type {
            InterfaceKind::Serial => {
                let interface = self
                    .serial
                    .as_ref()
                    .ok_or(ConnectionError::InterfaceNotAvailable)?;
                let handle = interface.get_device_handler(address)?;
                Ok(DeviceHandleWrapper::Serial(handle))
            }
        }
    }
}
