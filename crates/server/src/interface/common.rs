use std::fmt::{Debug, Display, Formatter};
use tonic::async_trait;
use crate::interface::error::DiscoveryError;

#[async_trait]
pub trait Interface: Send + Sync + Debug {
    fn interface_type(&self) -> InterfaceType;

    fn discover_devices(&self) -> Result<Vec<DeviceInfo>, DiscoveryError>;
}

pub struct DeviceInfo {
    pub address: String,
    pub interface: InterfaceType,
    pub name: Option<String>,
    pub firmware: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum InterfaceType {
    Serial,
}

impl Display for InterfaceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceType::Serial => write!(f, "Serial"),
        }
    }
}

