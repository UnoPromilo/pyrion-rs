use crate::interface::common::{DeviceInfo, Interface};

#[derive(Debug)]
pub struct InterfaceManager {
    interfaces: Vec<Box<dyn Interface>>,
}

impl InterfaceManager {
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }

    pub fn add_interface(&mut self, interface: Box<dyn Interface>) {
        self.interfaces.push(interface);
    }

    pub fn discover_devices(&self) -> Vec<DeviceInfo> {
        let mut all_devices = Vec::new();

        for interface in &self.interfaces {
            match interface.discover_devices() {
                Ok(devices) => all_devices.extend(devices),
                Err(e) => {
                    log::warn!(
                        "Failed to discover devices on {} interface: {}",
                        interface.interface_type(),
                        e
                    );
                }
            }
        }

        all_devices
    }
}
