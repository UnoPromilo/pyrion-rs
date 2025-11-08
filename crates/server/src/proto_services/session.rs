use crate::features::connection_string::decode_connection_string;
use crate::features::interface;
use crate::features::interface::InterfaceManager;
use crate::features::session::DeviceHandleWrapper;
use crate::proto::pyrion::v1 as pyrion_v1;
use crate::proto::pyrion::v1::controller_message::controller_message::Payload as ControllerMessagePayload;
use crate::proto::pyrion::v1::device_message::device_message::Payload as DeviceMessagePayload;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use transport::event::Event;
use uuid::Uuid;

use crate::proto::pyrion::v1::controller_message::ControllerMessage;
use crate::proto::pyrion::v1::device_message::{DeviceIntroduction, DeviceMessage, Telemetry};
pub use pyrion_v1::session::device_session_server::DeviceSessionServer;
use transport::Command;

#[derive(Debug)]
pub struct DeviceSessionService {
    interfaces: Arc<InterfaceManager>,
}

impl DeviceSessionService {
    pub fn new(interfaces: Arc<InterfaceManager>) -> Self {
        Self { interfaces }
    }

    async fn get_device(
        &self,
        connection_string: &str,
    ) -> Result<DeviceHandleWrapper, ConnectionError> {
        let (interface, address) = decode_connection_string(connection_string)
            .ok_or(ConnectionError::InvalidConnectionString)?;

        let handler = self.interfaces.get_device_handler(interface, &address)?;

        Ok(handler)
    }
}

#[tonic::async_trait]
impl pyrion_v1::session::device_session_server::DeviceSession for DeviceSessionService {
    type OpenStream = Pin<Box<dyn Stream<Item = Result<DeviceMessage, Status>> + Send + 'static>>;

    async fn open(
        &self,
        request: Request<Streaming<ControllerMessage>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let connection_string = request
            .metadata()
            .get("connection-string")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("connection-string"))?;

        let device_handler = self.get_device(connection_string).await?;
        let (mut reader, mut writer) = device_handler.split();

        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        let shutdown = Arc::new(tokio::sync::Notify::new());
        let shutdown_reader = shutdown.clone();
        let shutdown_writer = shutdown.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    next = reader.read_next() => {
                        match next {
                            Some(Ok(event)) => {
                                if !matches!(event, Event::Telemetry(_)){
                                    tracing::info!("Received event: {:?}", event);
                                }
                                let device_message = map_event_to_proto(event);
                                if let Err(error) = tx.send(Ok(device_message)).await {
                                    tracing::error!("Error sending event: {:?}", error);
                                    break;
                                }
                            }
                            Some(Err(error)) => {
                                tracing::error!("Error reading event: {:?}", error);
                                // TODO ask for retransmission?
                            }
                            None => {
                                tracing::info!("Device stream closed");
                                break;
                            }
                        }
                    }
                    _ = shutdown_reader.notified() => {
                        tracing::info!("Reader task shutdown requested");
                        break;
                    }
                }
            }
            shutdown_reader.notify_one();
        });

        // Writer task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    next = in_stream.next() => {
                        match next {
                            Some(Ok(controller_message)) => {
                                tracing::info!("Received controller message: {:?}", controller_message);
                                if let Some(command) = map_proto_to_command(controller_message) {
                                    if let Err(error) = writer.write(command).await {
                                        tracing::error!("Error writing command: {:?}", error);
                                        break;
                                    }
                                } else {
                                    tracing::warn!("Received invalid command, payload was empty");
                                }
                            }
                            Some(Err(error)) => {
                                tracing::error!("Error while receiving a controller message: {:?}", error);
                                break;
                            }
                            None => {
                                tracing::info!("Controller stream closed");
                                break;
                            }
                        }
                    }
                    _ = shutdown_writer.notified() => {
                        tracing::info!("Writer task shutdown requested");
                        break;
                    }
                }
            }
            shutdown_writer.notify_one(); // Notify reader to stop
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream)))
    }
}

enum ConnectionError {
    InvalidConnectionString,
    InterfaceError(interface::error::ConnectionError),
}

impl From<ConnectionError> for Status {
    fn from(err: ConnectionError) -> Self {
        match err {
            ConnectionError::InvalidConnectionString => {
                Status::invalid_argument("connection-string")
            }
            ConnectionError::InterfaceError(interface::error::ConnectionError::DeviceNotFound) => {
                Status::not_found("Device not found")
            }
            ConnectionError::InterfaceError(
                interface::error::ConnectionError::InterfaceNotAvailable,
            ) => Status::unavailable("Interface not available"),
        }
    }
}

impl From<interface::error::ConnectionError> for ConnectionError {
    fn from(value: interface::error::ConnectionError) -> Self {
        Self::InterfaceError(value)
    }
}

fn map_event_to_proto(event: Event) -> DeviceMessage {
    match event {
        Event::DeviceIntroduction(device_introduction) => DeviceMessage {
            payload: Some(DeviceMessagePayload::DeviceIntroduction(
                DeviceIntroduction {
                    firmware: format!(
                        "{}.{}.{}",
                        device_introduction.firmware_version[0],
                        device_introduction.firmware_version[1],
                        device_introduction.firmware_version[2]
                    ),
                    uid: map_uid_to_uuid(&device_introduction.uid)
                        .to_string()
                        .to_uppercase(),
                },
            )),
        },
        Event::Telemetry(telemetry) => DeviceMessage {
            payload: Some(DeviceMessagePayload::Telemetry(Telemetry {
                cpu_temp: telemetry.cpu_temperature,
                driver_temp: telemetry.driver_temperature,
                motor_temp: telemetry.motor_temperature,
                v_bus: telemetry.v_bus,
                power: telemetry.power_consumption,
                current: telemetry.current_consumption,
                duty_cycle: telemetry.duty_cycle,
                uptime: telemetry.uptime,
                ongoing_errors: telemetry.ongoing_errors,
                resolved_errors: telemetry.resolved_errors,
            })),
        },
    }
}

fn map_proto_to_command(message: ControllerMessage) -> Option<Command> {
    message.payload.map(|payload| match payload {
        ControllerMessagePayload::IntroduceYourself(_) => Command::IntroduceYourself,
        ControllerMessagePayload::Stop(_) => Command::Stop,
    })
}

fn map_uid_to_uuid(uid: &[u8]) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[..12].copy_from_slice(uid);
    Uuid::from_bytes(bytes)
}
