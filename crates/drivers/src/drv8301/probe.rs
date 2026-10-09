use core::fmt::Debug;
use drv8301_dd::{Drv8301Async, DrvError, FaultStatus};
use embedded_hal_async::spi::{Error, ErrorKind, ErrorType, Operation, SpiDevice};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DiagnosticSnapshot {
    pub device_id: u8,
    pub fault: bool,
    pub gvdd_uv: bool,
    pub gvdd_ov: bool,
    pub pvdd_uv: bool,
    pub otsd: bool,
    pub otw: bool,
    pub fet_oc: [bool; 6],
}

impl DiagnosticSnapshot {
    fn from_status(device_id: u8, status: FaultStatus) -> Self {
        Self {
            device_id,
            fault: status.fault,
            gvdd_uv: status.gvdd_uv,
            gvdd_ov: status.gvdd_ov,
            pvdd_uv: status.pvdd_uv,
            otsd: status.otsd,
            otw: status.otw,
            fet_oc: [
                status.fetha_oc,
                status.fetla_oc,
                status.fethb_oc,
                status.fetlb_oc,
                status.fethc_oc,
                status.fetlc_oc,
            ],
        }
    }

    fn has_problem(&self) -> bool {
        self.fault
            || self.gvdd_uv
            || self.gvdd_ov
            || self.pvdd_uv
            || self.otsd
            || self.otw
            || self.fet_oc.iter().any(|fault| *fault)
    }
}

#[derive(Debug)]
pub(crate) enum ProbeHealthError {
    FaultPinLow(DiagnosticSnapshot),
    ReportedFault(DiagnosticSnapshot),
}

#[derive(Debug)]
pub(crate) enum ResponseError<E> {
    Spi(E),
    UnexpectedTransaction,
    EchoMismatch { expected: u8, received: u8 },
}

impl<E: Error> Error for ResponseError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Spi(error) => error.kind(),
            Self::UnexpectedTransaction | Self::EchoMismatch { .. } => ErrorKind::Other,
        }
    }
}

pub(crate) struct EchoCheckedDevice<S> {
    inner: S,
    previous_read: Option<u8>,
}

impl<S> EchoCheckedDevice<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            previous_read: None,
        }
    }
}

impl<S: SpiDevice<u8>> ErrorType for EchoCheckedDevice<S> {
    type Error = ResponseError<S::Error>;
}

impl<S: SpiDevice<u8>> SpiDevice<u8> for EchoCheckedDevice<S> {
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        let next_read = match operations {
            [Operation::Transfer(read, write)] if read.len() == 2 && write.len() == 2 => {
                (write[0] & 0x80 != 0).then_some((write[0] >> 3) & 0x0f)
            }
            _ => return Err(ResponseError::UnexpectedTransaction),
        };

        let previous_read = self.previous_read.take();
        self.inner
            .transaction(operations)
            .await
            .map_err(ResponseError::Spi)?;

        if let (Some(expected), [Operation::Transfer(read, _)]) = (previous_read, &*operations) {
            let received = (read[0] >> 3) & 0x0f;
            if received != expected {
                return Err(ResponseError::EchoMismatch { expected, received });
            }
        }
        self.previous_read = next_read;
        Ok(())
    }
}

pub(crate) struct ProbeResult {
    pub device_id: u8,
    pub faults: FaultStatus,
}

impl ProbeResult {
    pub(crate) fn classify(
        self,
        n_fault_low: bool,
    ) -> Result<DiagnosticSnapshot, ProbeHealthError> {
        let snapshot = DiagnosticSnapshot::from_status(self.device_id, self.faults);
        if n_fault_low {
            return Err(ProbeHealthError::FaultPinLow(snapshot));
        }
        if snapshot.has_problem() {
            return Err(ProbeHealthError::ReportedFault(snapshot));
        }
        Ok(snapshot)
    }
}

pub(crate) async fn probe<S: SpiDevice<u8>>(
    device: &mut S,
) -> Result<ProbeResult, DrvError<ResponseError<S::Error>>>
where
    S::Error: Debug,
{
    let mut driver = Drv8301Async::new(EchoCheckedDevice::new(device));
    let faults = driver.get_fault_status().await?;
    let device_id = driver.get_device_id().await?;
    Ok(ProbeResult { device_id, faults })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::future::Future;
    use core::task::{Context, Poll, Waker};
    use std::collections::VecDeque;
    use std::vec::Vec;

    use super::*;

    #[derive(Debug)]
    struct FakeError;

    impl Error for FakeError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    struct FakeDevice {
        responses: VecDeque<Result<[u8; 2], FakeError>>,
        sent: Vec<[u8; 2]>,
    }

    impl FakeDevice {
        fn new(responses: impl IntoIterator<Item = Result<[u8; 2], FakeError>>) -> Self {
            Self {
                responses: responses.into_iter().collect(),
                sent: Vec::new(),
            }
        }
    }

    impl ErrorType for FakeDevice {
        type Error = FakeError;
    }

    impl SpiDevice<u8> for FakeDevice {
        async fn transaction(
            &mut self,
            operations: &mut [Operation<'_, u8>],
        ) -> Result<(), Self::Error> {
            match operations {
                [Operation::Transfer(read, write)] if read.len() == 2 && write.len() == 2 => {
                    self.sent.push([write[0], write[1]]);
                    let response = self.responses.pop_front().expect("unexpected frame")?;
                    read.copy_from_slice(&response);
                    Ok(())
                }
                _ => panic!("expected exactly one 16-bit transfer per CS assertion"),
            }
        }
    }

    fn run<F: Future>(future: F) -> F::Output {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = core::pin::pin!(future);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(result) => result,
            Poll::Pending => panic!("fake SPI should not suspend"),
        }
    }

    #[test]
    fn reads_faults_and_id_with_six_separate_frames() {
        let mut spi = FakeDevice::new([
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
        ]);
        let result = run(probe(&mut spi)).expect("healthy response");
        assert_eq!(result.device_id, 0);
        assert!(!result.faults.fault);
        assert_eq!(
            spi.sent,
            [
                [0x80, 0],
                [0x80, 0],
                [0x88, 0],
                [0x88, 0],
                [0x88, 0],
                [0x88, 0]
            ]
        );
    }

    #[test]
    fn all_zero_responses_are_not_mistaken_for_a_healthy_device() {
        let mut spi = FakeDevice::new(core::iter::repeat_with(|| Ok([0, 0])).take(6));
        let result = run(probe(&mut spi));
        assert!(matches!(
            result,
            Err(DrvError::Spi(ResponseError::EchoMismatch {
                expected: 1,
                received: 0
            }))
        ));
    }

    #[test]
    fn spi_failure_is_reported() {
        let mut spi = FakeDevice::new([Err(FakeError)]);
        assert!(matches!(
            run(probe(&mut spi)),
            Err(DrvError::Spi(ResponseError::Spi(FakeError)))
        ));
    }

    #[test]
    fn status_fault_bits_are_preserved_from_the_first_read() {
        let mut spi = FakeDevice::new([
            Ok([0, 0]),
            Ok([0x04, 0]),
            Ok([0, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
        ]);
        let result = run(probe(&mut spi)).expect("fault bits must be readable");
        assert!(result.faults.fault);
    }

    #[test]
    fn low_fault_pin_rejects_healthy_spi_response() {
        let mut spi = FakeDevice::new([
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
        ]);
        let result = run(probe(&mut spi)).expect("healthy SPI response");
        match result.classify(true) {
            Err(ProbeHealthError::FaultPinLow(snapshot)) => {
                assert_eq!(snapshot.device_id, 0);
                assert!(!snapshot.has_problem());
            }
            _ => panic!("low fault pin must reject preflight"),
        }
    }

    #[test]
    fn reported_fault_rejects_high_pin_and_retains_first_snapshot() {
        let mut spi = FakeDevice::new([
            Ok([0, 0]),
            Ok([0x04, 0x20]),
            Ok([0, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
        ]);
        let result = run(probe(&mut spi)).expect("fault response is readable");
        match result.classify(false) {
            Err(ProbeHealthError::ReportedFault(snapshot)) => {
                assert!(snapshot.fault);
                assert!(snapshot.fet_oc[0]);
            }
            _ => panic!("reported fault must reject preflight"),
        }
    }

    #[test]
    fn healthy_snapshot_passes_with_high_fault_pin() {
        let mut spi = FakeDevice::new([
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
            Ok([0x08, 0]),
        ]);
        let result = run(probe(&mut spi)).expect("healthy SPI response");
        assert!(!result.classify(false).unwrap().has_problem());
    }
}
