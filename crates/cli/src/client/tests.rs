use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::{Notify, mpsc, oneshot};
use tokio::time::timeout;
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use super::discovery::select_only_device;
use super::{
    ClearResolvedFaultsResult, ClientConfig, ClientError, ConnectionResult, DeviceSummary,
    FaultSummary, PyrionClient,
};
use crate::proto::pyrion::v1::controller_message::ControllerMessage;
use crate::proto::pyrion::v1::controller_message::controller_message::Payload as ControllerPayload;
use crate::proto::pyrion::v1::device_message::device_message::Payload as DevicePayload;
use crate::proto::pyrion::v1::device_message::{
    DeviceIntroduction, DeviceMessage, FaultEntry, FaultRegister, FaultState, FaultType, Success,
    Telemetry,
};
use crate::proto::pyrion::v1::discovery::device_discovery_server::{
    DeviceDiscovery, DeviceDiscoveryServer,
};
use crate::proto::pyrion::v1::discovery::{
    DiscoveredDevice, DiscoveryParams, ListDiscoveredDevice,
};
use crate::proto::pyrion::v1::session::device_session_server::{
    DeviceSession, DeviceSessionServer,
};

const CONNECTION_STRING: &str = "serial::/dev/test-bldc";
const TELEMETRY_ONLY_CONNECTION_STRING: &str = "serial::/dev/telemetry-only";

#[derive(Debug, Default)]
struct FakePyrion {
    disconnected: Option<Arc<Notify>>,
}

#[tonic::async_trait]
impl DeviceDiscovery for FakePyrion {
    async fn list_devices(
        &self,
        _request: Request<DiscoveryParams>,
    ) -> Result<Response<ListDiscoveredDevice>, Status> {
        Ok(Response::new(ListDiscoveredDevice {
            devices: vec![DiscoveredDevice {
                address: "/dev/test-bldc".to_owned(),
                interface: crate::proto::pyrion::v1::interface::Interface::Serial.into(),
                firmware: None,
                name: Some("BLDC".to_owned()),
                connection_string: CONNECTION_STRING.to_owned(),
            }],
        }))
    }
}

#[tonic::async_trait]
impl DeviceSession for FakePyrion {
    type OpenStream = Pin<Box<dyn Stream<Item = Result<DeviceMessage, Status>> + Send + 'static>>;

    async fn open(
        &self,
        request: Request<Streaming<ControllerMessage>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let connection_string = request
            .metadata()
            .get("connection-string")
            .and_then(|value| value.to_str().ok());
        if !matches!(
            connection_string,
            Some(CONNECTION_STRING | TELEMETRY_ONLY_CONNECTION_STRING)
        ) {
            return Err(Status::not_found("device not found"));
        }
        let telemetry_only = connection_string == Some(TELEMETRY_ONLY_CONNECTION_STRING);

        let mut requests = request.into_inner();
        let (response_tx, response_rx) = mpsc::channel(1);
        let disconnected = self.disconnected.clone();
        if telemetry_only {
            tokio::spawn(async move {
                let _ = requests.next().await;
                for _ in 0..20 {
                    if response_tx
                        .send(Ok(DeviceMessage {
                            payload: Some(DevicePayload::Telemetry(Telemetry::default())),
                        }))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
            return Ok(Response::new(Box::pin(ReceiverStream::new(response_rx))));
        }

        tokio::spawn(async move {
            let response = match requests.next().await {
                Some(Ok(ControllerMessage {
                    payload: Some(ControllerPayload::IntroduceYourself(_)),
                })) => Ok(DeviceMessage {
                    payload: Some(DevicePayload::DeviceIntroduction(DeviceIntroduction {
                        firmware: "0.1.0".to_owned(),
                        uid: "TEST-UID".to_owned(),
                    })),
                }),
                Some(Ok(ControllerMessage {
                    payload: Some(ControllerPayload::ReportFaults(_)),
                })) => Ok(DeviceMessage {
                    payload: Some(DevicePayload::FaultRegister(FaultRegister {
                        faults: vec![
                            FaultEntry {
                                r#type: FaultType::Encoder.into(),
                                state: FaultState::Active.into(),
                            },
                            FaultEntry {
                                r#type: FaultType::AdcTimeout.into(),
                                state: FaultState::Latched.into(),
                            },
                            FaultEntry {
                                r#type: FaultType::GateDriverStartup.into(),
                                state: FaultState::Active.into(),
                            },
                            FaultEntry {
                                r#type: FaultType::GateDriverRuntime.into(),
                                state: FaultState::Active.into(),
                            },
                        ],
                    })),
                }),
                Some(Ok(ControllerMessage {
                    payload: Some(ControllerPayload::ResetFaults(_)),
                })) => Ok(DeviceMessage {
                    payload: Some(DevicePayload::Success(Success {})),
                }),
                Some(Ok(_)) => Err(Status::invalid_argument("unsupported request")),
                Some(Err(error)) => Err(error),
                None => Err(Status::invalid_argument("missing request")),
            };
            let _ = response_tx.send(response).await;
            if requests.next().await.is_none()
                && let Some(disconnected) = disconnected
            {
                disconnected.notify_one();
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(response_rx))))
    }
}

#[tokio::test]
async fn discovers_connects_identifies_and_disconnects() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let disconnected = Arc::new(Notify::new());
    let server_disconnected = disconnected.clone();

    let server = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(DeviceDiscoveryServer::new(FakePyrion::default()))
            .add_service(DeviceSessionServer::new(FakePyrion {
                disconnected: Some(server_disconnected),
            }))
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    let client = PyrionClient::new(ClientConfig {
        endpoint: format!("http://{address}"),
        timeout: Duration::from_secs(2),
    });

    let devices = client.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].connection_string, CONNECTION_STRING);

    let result = client.connect_and_identify(None).await.unwrap();
    assert_eq!(
        result,
        ConnectionResult {
            connection_string: CONNECTION_STRING.to_owned(),
            firmware: "0.1.0".to_owned(),
            uid: "TEST-UID".to_owned(),
        }
    );

    let report = client.report_faults(None).await.unwrap();
    assert_eq!(
        report.faults,
        [
            FaultSummary {
                fault_type: "ENCODER".to_owned(),
                state: "ACTIVE".to_owned(),
            },
            FaultSummary {
                fault_type: "ADC_TIMEOUT".to_owned(),
                state: "LATCHED".to_owned(),
            },
            FaultSummary {
                fault_type: "GATE_DRIVER_STARTUP".to_owned(),
                state: "ACTIVE".to_owned(),
            },
            FaultSummary {
                fault_type: "GATE_DRIVER_RUNTIME".to_owned(),
                state: "ACTIVE".to_owned(),
            },
        ]
    );

    assert_eq!(
        client.clear_resolved_faults(None).await.unwrap(),
        ClearResolvedFaultsResult { cleared: true }
    );

    let timeout_client = PyrionClient::new(ClientConfig {
        endpoint: format!("http://{address}"),
        timeout: Duration::from_millis(50),
    });
    assert!(matches!(
        timeout_client
            .report_faults(Some(TELEMETRY_ONLY_CONNECTION_STRING))
            .await,
        Err(ClientError::Timeout)
    ));
    timeout(Duration::from_secs(1), disconnected.notified())
        .await
        .unwrap();

    shutdown_tx.send(()).unwrap();
    server.await.unwrap();
}

#[test]
fn rejects_ambiguous_automatic_selection() {
    let device = DeviceSummary {
        address: "/dev/test".to_owned(),
        interface: "SERIAL".to_owned(),
        firmware: None,
        name: None,
        connection_string: CONNECTION_STRING.to_owned(),
    };

    assert!(matches!(
        select_only_device(vec![device.clone(), device]),
        Err(ClientError::DeviceSelection(_))
    ));
}

#[test]
fn serializes_protobuf_style_json_names() {
    let result = ConnectionResult {
        connection_string: CONNECTION_STRING.to_owned(),
        firmware: "0.1.0".to_owned(),
        uid: "TEST-UID".to_owned(),
    };

    assert_eq!(
        serde_json::to_value(result).unwrap()["connectionString"],
        CONNECTION_STRING
    );
}

#[test]
fn maps_deadline_status_to_timeout() {
    assert!(matches!(
        super::map_rpc_status(Status::deadline_exceeded("late")),
        ClientError::Timeout
    ));
}

#[test]
fn maps_stream_unavailable_to_endpoint_error() {
    assert!(matches!(
        super::map_rpc_status(Status::unavailable("server stopped")),
        ClientError::Endpoint(_)
    ));
}
