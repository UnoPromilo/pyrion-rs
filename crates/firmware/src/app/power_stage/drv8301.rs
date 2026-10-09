use crate::app::safety;
use controller_shared::RawInverterValues;
use controller_shared::output::{InhibitCause, InverterOutput, OutputState, SafeOutput};
use drivers::{Checked, FastOutput, PreflightPassed, SlowControl, Unchecked};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use hardware::BoardSixPwmTim1;
use logging::fault_register::{FaultRegister, FaultType};
use logging::{error, info};

pub(crate) type UncheckedOutput = FastOutput<'static, BoardSixPwmTim1<'static>, Unchecked>;

pub(crate) enum GateRequest {
    Wake,
    Finish(Option<PreflightPassed>),
}

static GATE_REQUEST: Signal<CriticalSectionRawMutex, GateRequest> = Signal::new();
static GATE_ACK: Signal<CriticalSectionRawMutex, bool> = Signal::new();

async fn request_gate(request: GateRequest) -> bool {
    GATE_REQUEST.signal(request);
    GATE_ACK.wait().await
}

pub(crate) async fn next_gate_request() -> GateRequest {
    GATE_REQUEST.wait().await
}

fn acknowledge_gate(accepted: bool) {
    GATE_ACK.signal(accepted);
}

pub(crate) fn handle_request(request: GateRequest, inverter: &mut SafeOutput<BoardOutput>) {
    let faults = FaultRegister::shared();
    match request {
        GateRequest::Wake => {
            let accepted = !faults.any_active()
                && !faults.any_latched()
                && !safety::dfu_shutdown_requested()
                && matches!(
                    inverter.state(),
                    OutputState::Inhibited(InhibitCause::Startup)
                )
                && inverter
                    .with_startup_driver(BoardOutput::wake_for_preflight)
                    .unwrap_or(false);
            acknowledge_gate(accepted);
        }
        GateRequest::Finish(proof) => {
            let safe_to_accept = !faults.any_active()
                && !faults.any_latched()
                && !safety::dfu_shutdown_requested()
                && matches!(
                    inverter.state(),
                    OutputState::Inhibited(InhibitCause::Startup)
                );
            let cause = match inverter.state() {
                OutputState::Inhibited(cause) => cause,
                _ => InhibitCause::Fault,
            };
            inverter.inhibit(cause);
            let accepted = if let (true, Some(proof)) = (safe_to_accept, proof) {
                inverter
                    .with_startup_driver(|output| output.accept_preflight(proof))
                    .unwrap_or(false)
            } else {
                false
            };
            acknowledge_gate(accepted);
        }
    }
}

pub(crate) enum BoardOutput {
    Unchecked(Option<UncheckedOutput>),
    Checked(FastOutput<'static, BoardSixPwmTim1<'static>, Checked>),
}

impl BoardOutput {
    pub(crate) fn new(output: UncheckedOutput) -> Self {
        Self::Unchecked(Some(output))
    }

    fn pwm(&self) -> &BoardSixPwmTim1<'static> {
        match self {
            Self::Unchecked(Some(output)) => output.pwm(),
            Self::Checked(output) => output.pwm(),
            Self::Unchecked(None) => unreachable!("fast output is being promoted"),
        }
    }

    fn pwm_mut(&mut self) -> &mut BoardSixPwmTim1<'static> {
        match self {
            Self::Unchecked(Some(output)) => output.pwm_mut(),
            Self::Checked(output) => output.pwm_mut(),
            Self::Unchecked(None) => unreachable!("fast output is being promoted"),
        }
    }

    fn wake_for_preflight(&mut self) -> bool {
        if let Self::Unchecked(Some(output)) = self {
            output.wake_for_preflight();
            true
        } else {
            false
        }
    }

    fn accept_preflight(&mut self, proof: PreflightPassed) -> bool {
        if let Self::Unchecked(output) = self {
            let checked = output
                .take()
                .expect("unchecked output must exist until preflight")
                .accept_preflight(proof);
            *self = Self::Checked(checked);
            true
        } else {
            false
        }
    }
}

impl InverterOutput for BoardOutput {
    fn max_duty(&self) -> u32 {
        self.pwm().get_max_duty()
    }

    fn disable_outputs(&mut self) {
        match self {
            Self::Unchecked(Some(output)) => output.finish_preflight(),
            Self::Checked(output) => output.finish_preflight(),
            Self::Unchecked(None) => unreachable!("fast output is being promoted"),
        }
        self.pwm_mut().disable();
    }

    fn write_phase_duties(&mut self, duties: RawInverterValues) {
        self.pwm_mut()
            .set_phase_duties(duties.u, duties.v, duties.w);
    }

    fn enable_outputs(&mut self) {
        if let Self::Checked(output) = self {
            output.pwm_mut().enable();
        } else {
            panic!("cannot enable TIM1 before DRV8301 preflight");
        }
    }
}

#[embassy_executor::task]
pub async fn task_gate_driver(mut control: SlowControl<'static>) {
    if !request_gate(GateRequest::Wake).await {
        FaultRegister::shared().set(FaultType::GateDriverStartup);
        error!("DRV8301 startup failed: fast output could not wake gate safely");
        return;
    }

    embassy_time::Timer::after_millis(10).await;
    let result = control.check_startup().await;
    let (snapshot, proof) = match result {
        Ok((snapshot, proof)) => (snapshot, proof),
        Err(cause) => {
            request_gate(GateRequest::Finish(None)).await;
            FaultRegister::shared().set(FaultType::GateDriverStartup);
            error!("DRV8301 startup failed: {:?}", cause);
            return;
        }
    };
    if request_gate(GateRequest::Finish(Some(proof))).await {
        info!("DRV8301 responding: {:?}", snapshot);
    } else {
        FaultRegister::shared().set(FaultType::GateDriverStartup);
        error!("DRV8301 startup failed: output inhibited during preflight");
    }
}
