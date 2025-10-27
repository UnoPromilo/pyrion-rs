use crate::features::interface_kind::InterfaceKind;
use crate::features::session::codec::DeviceCoded;
use crate::features::session::error::{DecoderError, EncoderError};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;
use tonic::codegen::tokio_stream::StreamExt as tokio_stream_ext;
use transport::{Command, Event};

#[derive(Debug)]
pub struct DeviceHandle<T: AsyncRead + AsyncWrite> {
    pub framed: Framed<T, DeviceCoded>,
}

#[derive(Debug)]
pub struct DeviceReader<T: AsyncRead> {
    pub reader: SplitStream<Framed<T, DeviceCoded>>,
}

#[derive(Debug)]
pub struct DeviceWriter<T: AsyncWrite> {
    pub writer: SplitSink<Framed<T, DeviceCoded>, Command>,
}

impl<T: AsyncRead + AsyncWrite> DeviceHandle<T> {
    pub fn new(io: T) -> Self {
        let codec = DeviceCoded::new();
        let framed = Framed::new(io, codec);
        Self { framed }
    }

    pub fn split(self) -> (DeviceReader<T>, DeviceWriter<T>) {
        let (sink, stream) = self.framed.split();
        (DeviceReader::new(stream), DeviceWriter::new(sink))
    }
}

impl<T: AsyncRead> DeviceReader<T> {
    pub fn new(stream: SplitStream<Framed<T, DeviceCoded>>) -> Self {
        Self { reader: stream }
    }
}

impl<T: AsyncWrite> DeviceWriter<T> {
    pub fn new(sink: SplitSink<Framed<T, DeviceCoded>, Command>) -> Self {
        Self { writer: sink }
    }
}

#[derive(Debug)]
pub enum DeviceHandleWrapper {
    Serial(DeviceHandle<tokio_serial::SerialStream>),
}

#[derive(Debug)]
pub enum DeviceWriterWrapper {
    Serial(DeviceWriter<tokio_serial::SerialStream>),
}

#[derive(Debug)]
pub enum DeviceReaderWrapper {
    Serial(DeviceReader<tokio_serial::SerialStream>),
}

impl DeviceHandleWrapper {
    pub fn interface_type(&self) -> InterfaceKind {
        match self {
            DeviceHandleWrapper::Serial(_) => InterfaceKind::Serial,
        }
    }

    pub fn split(self) -> (DeviceReaderWrapper, DeviceWriterWrapper) {
        match self {
            DeviceHandleWrapper::Serial(handle) => {
                let (reader, writer) = handle.split();
                (
                    DeviceReaderWrapper::Serial(reader),
                    DeviceWriterWrapper::Serial(writer),
                )
            }
        }
    }
}

impl DeviceReaderWrapper {
    pub async fn read_next(&mut self) -> Option<Result<Event, DecoderError>> {
        match self {
            DeviceReaderWrapper::Serial(reader) => tokio_stream_ext::next(&mut reader.reader).await,
        }
    }
}

impl DeviceWriterWrapper {
    pub async fn write(&mut self, command: Command) -> Result<(), EncoderError> {
        match self {
            DeviceWriterWrapper::Serial(writer) => writer.writer.send(command).await,
        }
    }
}
