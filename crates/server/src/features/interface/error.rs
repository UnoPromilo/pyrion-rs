use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DiscoveryError {
    Serial(tokio_serial::Error),
}

#[derive(Debug)]
pub enum ConnectionError {
    InterfaceNotAvailable,
    DeviceNotFound,
}

impl Display for DiscoveryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::Serial(e) => write!(f, "Serial error: {}", e),
        }
    }
}

impl From<tokio_serial::Error> for DiscoveryError {
    fn from(e: tokio_serial::Error) -> Self {
        Self::Serial(e)
    }
}

impl From<tokio_serial::Error> for ConnectionError {
    fn from(_: tokio_serial::Error) -> Self {
        ConnectionError::DeviceNotFound
    }
}
