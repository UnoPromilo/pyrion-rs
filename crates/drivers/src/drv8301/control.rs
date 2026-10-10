use super::probe::{self, DiagnosticSnapshot, ProbeHealthError, ResponseError};
use super::spi_device::DrvSpiDevice;
use super::stage::PreflightPassed;
use core::future::Future;
use drv8301_dd::DrvError;
use embassy_futures::select::{Either, select};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::mode::Async;
use embassy_stm32::spi;
use embassy_time::{Duration, with_timeout};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StartupError {
    Timeout,
    FaultPinLowDuringPreflight,
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

async fn guard_preflight<T>(
    work: impl Future<Output = T>,
    fault: impl Future<Output = ()>,
) -> Result<T, StartupError> {
    match select(work, fault).await {
        Either::First(result) => Ok(result),
        Either::Second(()) => Err(StartupError::FaultPinLowDuringPreflight),
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
        let response = guard_preflight(
            with_timeout(
                Duration::from_millis(50),
                probe::probe(&mut self.registers.device),
            ),
            Self::wait_for_fault(&mut self.n_fault),
        )
        .await?;
        let n_fault_low = self.n_fault.is_low();
        let result = response
            .map_err(|_| StartupError::Timeout)?
            .map_err(StartupError::from)?;
        let snapshot = result.classify(n_fault_low).map_err(StartupError::from)?;
        Ok((snapshot, PreflightPassed { _private: () }))
    }

    async fn wait_for_fault(n_fault: &mut ExtiInput<'_, Async>) {
        if !n_fault.is_low() {
            n_fault.wait_for_low().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::future::{pending, ready};
    use core::task::{Context, Poll, Waker};

    fn run<F: Future>(future: F) -> F::Output {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = core::pin::pin!(future);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(result) => result,
            Poll::Pending => panic!("test future should be ready"),
        }
    }

    #[test]
    fn fault_during_preflight_rejects_pending_work() {
        assert!(matches!(
            run(guard_preflight(pending::<()>(), ready(()))),
            Err(StartupError::FaultPinLowDuringPreflight)
        ));
    }

    #[test]
    fn completed_preflight_does_not_wait_for_a_fault() {
        assert!(matches!(
            run(guard_preflight(ready(42), pending::<()>())),
            Ok(42)
        ));
    }
}
