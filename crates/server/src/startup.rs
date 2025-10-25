use crate::configuration::Configuration;
use crate::interface::serial::SerialInterface;
use crate::proto_services::{DeviceDiscoveryServer, DeviceDiscoveryService};
use tonic::transport::Server;
use tonic::transport::server::Router;

pub struct Application {
    router: Router,
    address: std::net::SocketAddr,
}

impl Application {
    pub async fn build(config: Configuration) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port).parse()?;
        let mut interfaces = crate::interface::InterfaceManager::new();
        if config.interfaces.serial.enabled {
            interfaces.add_interface(Box::new(SerialInterface::new_from_config(
                config.interfaces.serial,
            )));
        }
        let interfaces = std::sync::Arc::new(tokio::sync::RwLock::new(interfaces));
        let discovery = DeviceDiscoveryService::new(interfaces.clone());
        let svc = DeviceDiscoveryServer::new(discovery);
        let router = Server::builder().add_service(svc);
        Ok(Self { router, address })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!("Starting server on {}", self.address);
        self.router.serve(self.address).await?;
        Ok(())
    }
}
