use crate::config::UartConfig;
use embassy_rp::uart::Uart;
use embassy_rp::{bind_interrupts, peripherals, uart};
use foc::Motor;
use interface::CommandResult;
use interface::serial::RawCommand;
use interface::serial::errors::CommandChainError;
use shared::{info, warn};

bind_interrupts!(struct Irqs {
    UART0_IRQ => uart::InterruptHandler<peripherals::UART0>;
    UART1_IRQ => uart::InterruptHandler<peripherals::UART1>;
});

#[embassy_executor::task]
pub async fn read_from_serial_task(motor: &'static Motor, hardware_config: Option<UartConfig>) {
    let hardware_config = match hardware_config {
        Some(config) => config,
        None => return,
    };

    let uart_config = uart::Config::default();
    let mut uart = Uart::new(
        hardware_config.uart,
        hardware_config.tx,
        hardware_config.rx,
        Irqs,
        hardware_config.tx_dma,
        hardware_config.rx_dma,
        uart_config,
    );

    let mut buffer = [0; 256];
    loop {
        let _ = uart.write("\n\r> ".as_bytes()).await;
        let result = read_with_echo_to_break(&mut uart, &mut buffer).await;
        match result {
            Ok(length) => {
                let parsing_result = str::from_utf8(&buffer[..length]);
                match parsing_result {
                    Ok(parsed_string) => {
                        let result = execute_command(parsed_string, motor).await;
                        if let Err(error) = result {
                            warn!("Failed to execute command: {:?}", error);
                        }
                    }
                    Err(_) => warn!("Failed to parse input from UART"),
                }
            }
            Err(error) => warn!("Error reading from UART: {}", error),
        }
    }
}

async fn read_with_echo_to_break<'u, 'b>(
    uart: &mut Uart<'u, uart::Async>,
    buffer: &'b mut [u8],
) -> Result<usize, uart::ReadToBreakError> {
    let mut len = 0;
    while len < buffer.len() {
        uart.read(&mut buffer[len..len + 1])
            .await
            .map_err(uart::ReadToBreakError::Other)?;

        if buffer[len] == b'\n' || buffer[len] == b'\r' {
            break;
        }

        uart.write(&buffer[len..len + 1])
            .await
            .map_err(uart::ReadToBreakError::Other)?;

        len += 1;
    }

    if len == buffer.len() {
        return Err(uart::ReadToBreakError::MissingBreak(len));
    }

    Ok(len)
}

async fn execute_command(
    raw_command: &str,
    motor: &Motor,
) -> Result<CommandResult, CommandChainError> {
    let raw_command = RawCommand::serial(raw_command);
    let parsed = interface::serial::parse(&raw_command)?;
    let command = interface::serial::decode_command(&parsed)?;
    let result = interface::execute_command(command, motor).await;
    info!("Received command result: {:?}", result);
    // TODO write to uart instead of defmt
    Ok(result)
}
