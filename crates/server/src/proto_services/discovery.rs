use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

mod proto {
    tonic::include_proto!("discovery");
}

use crate::interface::{InterfaceManager, InterfaceType};
pub use proto::device_discovery_server::DeviceDiscoveryServer;

#[derive(Debug)]
pub struct DeviceDiscoveryService {
    interfaces: Arc<RwLock<InterfaceManager>>,
}

impl DeviceDiscoveryService {
    pub fn new(interfaces: Arc<RwLock<InterfaceManager>>) -> Self {
        Self { interfaces }
    }
}

#[tonic::async_trait]
impl proto::device_discovery_server::DeviceDiscovery for DeviceDiscoveryService {
    async fn list_devices(
        &self,
        _request: Request<proto::DiscoveryParams>,
    ) -> Result<Response<proto::ListDiscoveredDevice>, Status> {
        let devices = self.interfaces.read().await.discover_devices();
        Ok(Response::new(proto::ListDiscoveredDevice {
            devices: devices
                .iter()
                .map(|device| proto::DiscoveredDevice {
                    address: device.address.clone(),
                    interface: map_interface(device.interface).into(),
                    firmware: device.firmware.clone(),
                    name: device.name.clone(),
                })
                .collect(),
        }))
    }
}

fn map_interface(value: InterfaceType) -> proto::Interface {
    match value {
        InterfaceType::Serial => proto::Interface::Serial,
    }
}
