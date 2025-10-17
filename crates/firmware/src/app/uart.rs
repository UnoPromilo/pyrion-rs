use crate::board::{BoardCrc, BoardUart};
use logging::error;
use transport::{Command, Parser, parser};
#[embassy_executor::task]
pub async fn task_uart(mut uart: BoardUart<'static>, mut crc: BoardCrc<'static>) {
    const BUF_SIZE: usize = 256; // TODO: decide on optimal size
    let mut buffer = [0u8; BUF_SIZE];
    let mut parser = Parser::new();

    loop {
        let size = match uart.read_until_idle(&mut buffer).await {
            Ok(n) => n,
            Err(err) => {
                handle_error(err.into()).await;
                continue;
            }
        };

        for &byte in &buffer[..size] {
            match parser.feed(byte, &mut crc) {
                Some(Ok(command)) => execute_command(command).await,
                Some(Err(err)) => handle_error(err.into()).await,
                None => {} // still parsing
            }
        }
    }
}

async fn execute_command(_command: Command) {
    // TODO execute command
}

async fn handle_error(error: Error) {
    error!("UART error: {:?}", error);
    // TODO handle error
}

#[derive(Debug, defmt::Format)]
pub enum Error {
    Uart(embassy_stm32::usart::Error),
    Parser(parser::Error),
}

impl From<embassy_stm32::usart::Error> for Error {
    fn from(e: embassy_stm32::usart::Error) -> Self {
        Self::Uart(e)
    }
}

impl From<parser::Error> for Error {
    fn from(e: parser::Error) -> Self {
        Self::Parser(e)
    }
}
