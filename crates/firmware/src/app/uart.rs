use crate::board::{BoardCrc, BoardUart};
use command_handler::{execute_command, get_telemetry};
use embassy_futures::select::Either;
use embassy_futures::select::select;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_time::{Duration, Ticker};
use logging::error;
use transport::Event;
use transport::command::decoder::Decoder;
use transport::event::encoder::Encoder;

#[embassy_executor::task]
pub async fn task_uart(uart: BoardUart<'static>, mut crc: BoardCrc<'static>) {
    const BUF_SIZE: usize = transport::MAX_PACKET_SIZE;
    let mut decoding_buffer = [0u8; BUF_SIZE];
    let mut encoding_buffer = [0u8; BUF_SIZE];
    let mut decoder = Decoder::new();
    let encoder = Encoder::new();
    let mut telemetry_ticker = Ticker::every(Duration::from_hz(10));

    let (mut tx, mut rx) = uart.split();

    loop {
        let read_future = rx.read_until_idle(&mut decoding_buffer);
        let telemetry_ticker_future = telemetry_ticker.next();

        match select(read_future, telemetry_ticker_future).await {
            Either::First(read_result) => {
                handle_uart(
                    read_result,
                    &mut decoding_buffer,
                    &mut decoder,
                    &mut encoding_buffer,
                    &encoder,
                    &mut tx,
                    &mut crc,
                )
                .await
            }
            Either::Second(_) => {
                send_telemetry(&mut tx, &mut encoding_buffer, &encoder, &mut crc).await;
            }
        }
    }
}

async fn handle_uart(
    uart_result: Result<usize, embassy_stm32::usart::Error>,
    decoder_buffer: &mut [u8],
    decoder: &mut Decoder,
    encoder_buffer: &mut [u8],
    encoder: &Encoder,
    tx: &mut UartTx<'static, Async>,
    crc: &mut BoardCrc<'static>,
) {
    let size = match uart_result {
        Ok(n) => n,
        Err(err) => {
            handle_error(err.into()).await;
            return;
        }
    };

    for &byte in &decoder_buffer[..size] {
        match decoder.feed(byte, crc) {
            Some(Ok(command)) => {
                let event = execute_command(command).await;
                if let Some(event) = event {
                    send_event(&event, tx, encoder_buffer, encoder, crc).await;
                }
            }
            Some(Err(err)) => handle_error(err.into()).await,
            None => {} // still parsing
        }
    }
}

async fn send_telemetry(
    tx: &mut UartTx<'static, Async>,
    buffer: &mut [u8],
    encoder: &Encoder,
    crc: &mut BoardCrc<'static>,
) {
    let event = Event::Telemetry(get_telemetry());
    send_event(&event, tx, buffer, encoder, crc).await;
}

async fn handle_error(error: Error) {
    error!("UART error: {:?}", error);
    // TODO handle error?
}

async fn send_event(
    event: &Event,
    tx: &mut UartTx<'static, Async>,
    buffer: &mut [u8],
    encoder: &Encoder,
    crc: &mut BoardCrc<'static>,
) {
    let length = encoder.encode(event, buffer, crc);
    let write_tx = tx.write(&buffer[..length]).await;
    match write_tx {
        Ok(_) => {}
        Err(err) => {
            handle_error(err.into()).await;
        }
    }
}

#[derive(Debug, defmt::Format)]
pub enum Error {
    Uart(embassy_stm32::usart::Error),
    Parser(transport::decoder::DecoderError<transport::command::Error>),
}

impl From<embassy_stm32::usart::Error> for Error {
    fn from(e: embassy_stm32::usart::Error) -> Self {
        Self::Uart(e)
    }
}

impl From<transport::decoder::DecoderError<transport::command::Error>> for Error {
    fn from(e: transport::decoder::DecoderError<transport::command::Error>) -> Self {
        Self::Parser(e)
    }
}
