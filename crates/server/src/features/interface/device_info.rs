use crate::features::interface_kind::InterfaceKind;

pub struct DeviceInfo {
    pub address: String,
    pub interface: InterfaceKind,
    pub name: Option<String>,
    pub firmware: Option<String>,
}
