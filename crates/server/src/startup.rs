use crate::configuration::Configuration;
use crate::features::interface::SerialInterface;
use crate::proto_services::{
    DeviceDiscoveryServer, DeviceDiscoveryService, DeviceSessionServer, DeviceSessionService,
};
use tonic::transport::Server;
use tonic::transport::server::Router;

pub struct Application {
    router: Router,
    address: std::net::SocketAddr,
}

impl Application {
    pub async fn build(config: Configuration) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port).parse()?;

        let mut interfaces = crate::features::interface::InterfaceManager::new();
        if config.interfaces.serial.enabled {
            interfaces
                .add_serial_interface(SerialInterface::new_from_config(config.interfaces.serial));
        }
        let interfaces = std::sync::Arc::new(interfaces);

        let discovery = DeviceDiscoveryService::new(interfaces.clone());
        let discovery = DeviceDiscoveryServer::new(discovery);

        let session = DeviceSessionService::new(interfaces.clone());
        let session = DeviceSessionServer::new(session);

        let router = Server::builder()
            .add_service(discovery)
            .add_service(session);

        Ok(Self { router, address })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!("Starting server on {}", self.address);
        self.router.serve(self.address).await?;
        Ok(())
    }
}
