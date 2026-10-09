use super::probe::{self, DiagnosticSnapshot, ProbeHealthError, ResponseError};
use super::spi_device::DrvSpiDevice;
use super::stage::PreflightPassed;
use drv8301_dd::DrvError;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::mode::Async;
use embassy_stm32::spi;
use embassy_time::{Duration, with_timeout};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StartupError {
    Timeout,
    Spi(spi::Error),
    Frame,
    InvalidResponse { expected: u8, received: u8 },
    UnexpectedTransaction,
    Unsupported(&'static str),
    FaultPinLow(DiagnosticSnapshot),
    ReportedFault(DiagnosticSnapshot),
}

impl From<DrvError<ResponseError<spi::Error>>> for StartupError {
    fn from(error: DrvError<ResponseError<spi::Error>>) -> Self {
        match error {
            DrvError::Spi(ResponseError::Spi(error)) => Self::Spi(error),
            DrvError::Spi(ResponseError::EchoMismatch { expected, received }) => {
                Self::InvalidResponse { expected, received }
            }
            DrvError::Spi(ResponseError::UnexpectedTransaction) => Self::UnexpectedTransaction,
            DrvError::FrameError => Self::Frame,
            DrvError::NotSupported(feature) => Self::Unsupported(feature),
        }
    }
}

impl From<ProbeHealthError> for StartupError {
    fn from(error: ProbeHealthError) -> Self {
        match error {
            ProbeHealthError::FaultPinLow(snapshot) => Self::FaultPinLow(snapshot),
            ProbeHealthError::ReportedFault(snapshot) => Self::ReportedFault(snapshot),
        }
    }
}

pub struct Drv8301Registers<'a> {
    pub(super) device: DrvSpiDevice<'a>,
}

pub struct SlowControl<'a> {
    pub(super) registers: Drv8301Registers<'a>,
    pub(super) n_fault: ExtiInput<'a, Async>,
}

impl SlowControl<'_> {
    pub async fn check_startup(
        &mut self,
    ) -> Result<(DiagnosticSnapshot, PreflightPassed), StartupError> {
        let response = with_timeout(
            Duration::from_millis(50),
            probe::probe(&mut self.registers.device),
        )
        .await;
        let n_fault_low = self.n_fault.is_low();
        let result = response
            .map_err(|_| StartupError::Timeout)?
            .map_err(StartupError::from)?;
        let snapshot = result.classify(n_fault_low).map_err(StartupError::from)?;
        Ok((snapshot, PreflightPassed { _private: () }))
    }

    pub async fn wait_for_fault(&mut self) {
        loop {
            if self.n_fault.is_low() {
                return;
            }
            self.n_fault.wait_for_low().await;
        }
    }
}
