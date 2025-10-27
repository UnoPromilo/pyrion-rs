use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::features::connection_string::encode_connection_string;
use crate::features::interface::InterfaceManager;
use crate::features::interface_kind::InterfaceKind;
use crate::proto::pyrion::v1 as pyrion_v1;
pub use pyrion_v1::discovery::device_discovery_server::DeviceDiscoveryServer;

#[derive(Debug)]
pub struct DeviceDiscoveryService {
    interfaces: Arc<InterfaceManager>,
}

impl DeviceDiscoveryService {
    pub fn new(interfaces: Arc<InterfaceManager>) -> Self {
        Self { interfaces }
    }
}

#[tonic::async_trait]
impl pyrion_v1::discovery::device_discovery_server::DeviceDiscovery for DeviceDiscoveryService {
    #[tracing::instrument(skip(self))]
    async fn list_devices(
        &self,
        _request: Request<pyrion_v1::discovery::DiscoveryParams>,
    ) -> Result<Response<pyrion_v1::discovery::ListDiscoveredDevice>, Status> {
        let devices = self.interfaces.discover_devices();
        Ok(Response::new(pyrion_v1::discovery::ListDiscoveredDevice {
            devices: devices
                .iter()
                .map(|device| pyrion_v1::discovery::DiscoveredDevice {
                    address: device.address.clone(),
                    interface: map_interface(device.interface).into(),
                    firmware: device.firmware.clone(),
                    name: device.name.clone(),
                    connection_string: encode_connection_string(&device.interface, &device.address),
                })
                .collect(),
        }))
    }
}

fn map_interface(value: InterfaceKind) -> pyrion_v1::interface::Interface {
    match value {
        InterfaceKind::Serial => pyrion_v1::interface::Interface::Serial,
    }
}
