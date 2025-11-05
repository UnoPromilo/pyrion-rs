use crate::app::{COMMAND_CHANNEL, EVENT_CHANNEL};
use crate::board::BoardUart;
use communication::packet::{Interface, PACKET_SIZE, Packet};
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use logging::error;

#[embassy_executor::task]
pub async fn task_uart(uart: BoardUart<'static>) {
    let mut rx_subscriber = EVENT_CHANNEL.subscriber().expect("Can't subscribe to uart");
    let (mut tx, mut rx) = uart.split();

    let tx_fut = async move {
        let mut buffer = [0u8; PACKET_SIZE];
        loop {
            let read_result = rx.read_until_idle(&mut buffer).await;
            handle_uart(read_result, &mut buffer).await
        }
    };

    let rx_fut = async move {
        loop {
            let packet = rx_subscriber.next_message_pure().await;
            if matches!(packet.interface, Interface::Serial) && packet.length > 0 {
                send_buffer(&mut tx, &packet.buffer[..packet.length]).await;
            }
        }
    };

    embassy_futures::join::join(tx_fut, rx_fut).await;
}

async fn handle_uart(uart_result: Result<usize, embassy_stm32::usart::Error>, data: &mut [u8]) {
    let size = match uart_result {
        Ok(n) => n,
        Err(err) => {
            handle_error(err.into()).await;
            return;
        }
    };

    // data is <= than PACKET_SIZE, so there is no need to split
    let packet = Packet::from_slice(&data[..size], Interface::Serial);
    COMMAND_CHANNEL.send(packet).await;
}

async fn handle_error(error: Error) {
    error!("UART error: {:?}", error);
    // TODO handle error?
}

async fn send_buffer(tx: &mut UartTx<'static, Async>, buffer: &[u8]) {
    let write_tx = tx.write(buffer).await;
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
