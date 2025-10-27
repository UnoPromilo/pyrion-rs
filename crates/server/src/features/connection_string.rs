use crate::features::interface_kind::InterfaceKind;

pub fn encode_connection_string(interface: &InterfaceKind, address: &str) -> String {
    let interface = encode_interface(interface);
    format!("{}::{}", interface, address)
}

pub fn decode_connection_string(connection_string: &str) -> Option<(InterfaceKind, String)> {
    let parts: Vec<&str> = connection_string.split("::").collect();
    let interface = parts.first()?;
    let address = parts.get(1)?;
    let interface = decode_interface(interface)?;
    Some((interface, address.to_string()))
}

fn encode_interface(interface: &InterfaceKind) -> String {
    match interface {
        InterfaceKind::Serial => "serial".to_string(),
    }
}

fn decode_interface(interface: &str) -> Option<InterfaceKind> {
    match interface {
        "serial" => Some(InterfaceKind::Serial),
        _ => None,
    }
}
