use crate::app::communication::CONTROL_COMMAND_CHANNEL;
use crate::app::power_stage::{self, BoardOutput, GateRequest, UncheckedOutput};
use crate::app::safety;
use controller_shared::command::ControlCommand;
use controller_shared::output::{InhibitCause, SafeOutput};
use controller_shared::state::State;
use controller_shared::strategy::ControlStrategy;
use controller_shared::{
    BoardSensorScales, ControlFault, ControlOutput, CurrentZeroOffsets, RawInverterValues,
    RawSnapshot, control_step,
};
use core::num::NonZeroU16;
use core::sync::atomic::{AtomicU16, Ordering};
use drivers::adc::epoch::{FrameAlignment, TriggerEpoch, align_fast_frame, is_invalid};
use drivers::adc::injected::Running;
use drivers::adc::slow_vref::{SlowVref, SlowVrefError};
use embassy_futures::join::join3;
use embassy_futures::select::{Either3, select3};
use embassy_stm32::adc::RingBufferedAdc;
use embassy_stm32::peripherals::{ADC1, ADC2, ADC3, ADC5};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use hardware::{AuxiliaryAdcCounts, BoardAdcFast, BoardAdcFrame};
use logging::FreqMeter;
use logging::error;
use logging::fault_register::{FaultRegister, FaultState, FaultType};
use user_config::UserConfig;

static VREFINT_SNAPSHOT: AtomicU16 = AtomicU16::new(0);
const FAST_STARTUP_TIMEOUT: Duration = Duration::from_millis(250);

struct TriggeredFrame {
    epoch: TriggerEpoch,
    readings: BoardAdcFrame,
}

enum FastEvent {
    Gate(GateRequest),
    Command(ControlCommand),
    AdcTimeout,
    Frame(TriggeredFrame),
}

async fn read_triggered_frame(
    adc1: &Running<'_, ADC1>,
    adc3: &Running<'_, ADC3>,
    adc5: &Running<'_, ADC5>,
    last_epoch: TriggerEpoch,
) -> TriggeredFrame {
    let (mut a, mut c, mut d) = join3(
        adc1.read_next_tagged(),
        adc3.read_next_tagged(),
        adc5.read_next_tagged(),
    )
    .await;
    loop {
        let alignment = align_fast_frame(a.epoch, c.epoch, d.epoch, TriggerEpoch::current());
        let FrameAlignment::Refresh {
            adc1: refresh1,
            adc3: refresh3,
            adc5: refresh5,
        } = alignment
        else {
            if is_invalid(a.epoch) || !a.epoch.is_newer_than(last_epoch) {
                #[cfg(feature = "adc-timing")]
                if is_invalid(a.epoch) {
                    drivers::adc::timing::record_reject_invalid();
                } else {
                    drivers::adc::timing::record_reject_stale();
                }
                (a, c, d) = join3(
                    adc1.read_next_tagged(),
                    adc3.read_next_tagged(),
                    adc5.read_next_tagged(),
                )
                .await;
                continue;
            }
            return TriggeredFrame {
                epoch: a.epoch,
                readings: BoardAdcFrame::from_injected(a.values, c.values, d.values),
            };
        };
        #[cfg(feature = "adc-timing")]
        drivers::adc::timing::record_refresh();
        (a, c, d) = join3(
            async {
                if refresh1 {
                    adc1.read_next_tagged().await
                } else {
                    a
                }
            },
            async {
                if refresh3 {
                    adc3.read_next_tagged().await
                } else {
                    c
                }
            },
            async {
                if refresh5 {
                    adc5.read_next_tagged().await
                } else {
                    d
                }
            },
        )
        .await;
    }
}

async fn next_fast_event(
    adc1: &Running<'_, ADC1>,
    adc3: &Running<'_, ADC3>,
    adc5: &Running<'_, ADC5>,
    last_epoch: TriggerEpoch,
) -> FastEvent {
    let adc_read = with_timeout(
        Duration::from_millis(1),
        read_triggered_frame(adc1, adc3, adc5, last_epoch),
    );
    match select3(
        power_stage::next_gate_request(),
        CONTROL_COMMAND_CHANNEL.receive(),
        adc_read,
    )
    .await
    {
        Either3::First(request) => FastEvent::Gate(request),
        Either3::Second(command) => FastEvent::Command(command),
        Either3::Third(Err(_)) => FastEvent::AdcTimeout,
        Either3::Third(Ok(frame)) => FastEvent::Frame(frame),
    }
}

#[embassy_executor::task]
pub async fn task_slow_aux(
    mut adc: RingBufferedAdc<'static, ADC2>,
    vrefint_cal: u16,
    sensor_scales: BoardSensorScales,
) {
    loop {
        let mut samples = [0; 3];
        if adc.read_latest(&mut samples) == samples.len() {
            let auxiliary = AuxiliaryAdcCounts::from_regular(samples);
            if let (Some(vref), Some(cal)) = (
                NonZeroU16::new(VREFINT_SNAPSHOT.load(Ordering::Relaxed)),
                NonZeroU16::new(vrefint_cal).filter(|cal| cal.get() <= 4095),
            ) {
                controller_shared::store_bus_voltage(
                    auxiliary.bus_voltage,
                    vref,
                    cal,
                    &sensor_scales,
                );
            }
        }
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
pub async fn task_slow_vref(mut adc: SlowVref<'static>) {
    if !(1..=4095).contains(&adc.calibrated_value()) {
        error!(
            "Invalid factory VREFINT calibration value: {}",
            adc.calibrated_value()
        );
    }
    let mut previous_error = None;
    loop {
        match adc.poll_latest() {
            Ok(value) => {
                VREFINT_SNAPSHOT.store(value, Ordering::Relaxed);
                previous_error = None;
            }
            Err(SlowVrefError::NoSample) => {}
            Err(fault) => {
                if previous_error != Some(fault) {
                    error!("ADC4 VREFINT conversion returned zero");
                    previous_error = Some(fault);
                }
            }
        }
        Timer::after_millis(1).await;
    }
}

struct FastControl {
    inverter: SafeOutput<BoardOutput>,
    max_duty: u32,
    current_zero: CurrentZeroOffsets,
    vrefint_cal: u16,
    sensor_scales: BoardSensorScales,
    strategy: ControlStrategy,
    last_epoch: TriggerEpoch,
    startup_started: Instant,
    first_valid_frame: bool,
    startup_failed: bool,
    ready_signaled: bool,
}

impl FastControl {
    fn new(
        inverter: SafeOutput<BoardOutput>,
        current_zero: CurrentZeroOffsets,
        vrefint_cal: u16,
        sensor_scales: BoardSensorScales,
    ) -> Self {
        Self {
            max_duty: inverter.max_duty(),
            inverter,
            current_zero,
            vrefint_cal,
            sensor_scales,
            strategy: ControlStrategy::Disabled,
            last_epoch: TriggerEpoch::UNSTARTED,
            startup_started: Instant::now(),
            first_valid_frame: false,
            startup_failed: false,
            ready_signaled: false,
        }
    }

    fn waiting_for_first_frame(&self) -> bool {
        !self.first_valid_frame && self.startup_started.elapsed() < FAST_STARTUP_TIMEOUT
    }

    fn decide_frame(
        &mut self,
        TriggeredFrame {
            epoch,
            readings: frame,
        }: TriggeredFrame,
        state: &State,
        faults: &FaultRegister,
    ) -> Option<Decision> {
        let v_ref = VREFINT_SNAPSHOT.load(Ordering::Relaxed);
        let missed = epoch.skipped_since(self.last_epoch);
        if missed != 0 {
            state.adc_missed_frames.fetch_add(missed, Ordering::Relaxed);
        }
        self.last_epoch = epoch;
        faults.resolve_if_set(FaultType::AdcTimeout);

        if v_ref == 0 && self.waiting_for_first_frame() {
            return None;
        }
        if !self.first_valid_frame && !self.waiting_for_first_frame() && !self.startup_failed {
            self.startup_failed = true;
            if v_ref == 0 {
                error!("Fast control startup failed: VREFINT not ready");
            } else {
                faults.set(FaultType::AdcTimeout);
                error!("Fast control startup failed: first ADC frame arrived after deadline");
            }
        }

        #[cfg(feature = "adc-timing")]
        let observed_frame = Some((epoch, drivers::adc::timing::record_frame(epoch)));

        if faults.any_active() {
            self.strategy = ControlStrategy::Disabled;
        }
        if faults.any_active() && faults.load(FaultType::InvalidMeasurement) != FaultState::Active {
            return Some(Decision {
                requested: RequestedOutput::Inhibited(InhibitCause::Fault),
                #[cfg(feature = "adc-timing")]
                observed_frame,
            });
        }

        let raw = RawSnapshot {
            i_u: frame.fast.current.u,
            i_v: frame.fast.current.v,
            i_w: frame.fast.current.w,
            v_u: frame.fast.voltage.u,
            v_v: frame.fast.voltage.v,
            v_w: frame.fast.voltage.w,
            v_ref,
            max_duty: self.max_duty,
            angle: state.raw_angle.load(Ordering::Relaxed),
        };
        let requested = match control_step(
            &raw,
            &mut self.strategy,
            &self.current_zero,
            self.vrefint_cal,
            &self.sensor_scales,
        ) {
            ControlOutput::Safe => {
                self.first_valid_frame = true;
                faults.resolve_if_set(FaultType::InvalidMeasurement);
                if !self.startup_failed
                    && !self.ready_signaled
                    && !faults.any_active()
                    && !safety::dfu_shutdown_requested()
                {
                    self.ready_signaled = true;
                    safety::acknowledge_fast_control_ready();
                }
                if faults.any_active() {
                    RequestedOutput::Inhibited(InhibitCause::Fault)
                } else {
                    RequestedOutput::ArmedSafe
                }
            }
            ControlOutput::Drive(values) => {
                self.first_valid_frame = true;
                RequestedOutput::Drive(values)
            }
            ControlOutput::Fault(fault) => RequestedOutput::Fault(fault),
        };
        Some(Decision {
            requested,
            #[cfg(feature = "adc-timing")]
            observed_frame,
        })
    }

    fn apply(&mut self, decision: &Decision, faults: &FaultRegister) {
        match decision.requested {
            RequestedOutput::ArmedSafe => self.inverter.enter_armed_safe(),
            RequestedOutput::Inhibited(cause) => self.inverter.inhibit(cause),
            RequestedOutput::DfuInhibited { request_id } => {
                self.inverter.inhibit(InhibitCause::FirmwareUpdate);
                safety::acknowledge_dfu_output_inhibited(request_id);
            }
            RequestedOutput::Drive(values) => {
                let result = self.inverter.drive(values);
                #[cfg(feature = "adc-timing")]
                if result.is_ok() {
                    drivers::adc::timing::record_duty_write(
                        decision.observed_frame.map(|(epoch, _)| epoch),
                    );
                }
                if result.is_err() {
                    faults.set(FaultType::InvalidControllerOutput);
                    self.strategy = ControlStrategy::Disabled;
                    self.inverter.inhibit(InhibitCause::Fault);
                }
            }
            RequestedOutput::Fault(fault) => {
                faults.set(match fault {
                    ControlFault::InvalidMeasurement => FaultType::InvalidMeasurement,
                    ControlFault::InvalidControllerOutput => FaultType::InvalidControllerOutput,
                });
                self.strategy = ControlStrategy::Disabled;
                self.inverter.inhibit(InhibitCause::Fault);
            }
        }
    }
}

struct Decision {
    requested: RequestedOutput,
    #[cfg(feature = "adc-timing")]
    observed_frame: Option<(TriggerEpoch, u32)>,
}

impl Decision {
    fn command(requested: RequestedOutput) -> Self {
        Self {
            requested,
            #[cfg(feature = "adc-timing")]
            observed_frame: None,
        }
    }
}

#[embassy_executor::task]
pub async fn task_adc(
    adc: BoardAdcFast<'static>,
    pwm: UncheckedOutput,
    user_config: &'static UserConfig,
    vrefint_cal: u16,
    sensor_scales: BoardSensorScales,
) {
    let adc_1 = adc.adc1_running;
    let adc_3 = adc.adc3_running;
    let adc_5 = adc.adc5_running;
    let inverter = SafeOutput::new(BoardOutput::new(pwm));
    let mut control = FastControl::new(
        inverter,
        CurrentZeroOffsets {
            u: user_config.current_zero_u,
            v: user_config.current_zero_v,
            w: user_config.current_zero_w,
        },
        vrefint_cal,
        sensor_scales,
    );
    let controller_state = controller_shared::state::state();
    let faults = FaultRegister::shared();

    let mut freq_meter = FreqMeter::named("ADC");
    freq_meter.link(&controller_state.foc_loop_frequency);

    loop {
        let start_time = Instant::now();
        let decision = match next_fast_event(&adc_1, &adc_3, &adc_5, control.last_epoch).await {
            FastEvent::Gate(request) => {
                power_stage::handle_request(request, &mut control.inverter);
                continue;
            }
            FastEvent::Command(ControlCommand::Stop) => {
                control.strategy = ControlStrategy::Disabled;
                Decision::command(RequestedOutput::ArmedSafe)
            }
            FastEvent::Command(ControlCommand::InhibitForDfu { request_id }) => {
                control.strategy = ControlStrategy::Disabled;
                Decision::command(RequestedOutput::DfuInhibited { request_id })
            }
            FastEvent::AdcTimeout if control.waiting_for_first_frame() => {
                continue;
            }
            FastEvent::AdcTimeout => {
                if !control.first_valid_frame && !control.startup_failed {
                    control.startup_failed = true;
                    error!("Fast control startup failed: ADC frame not ready");
                }
                faults.set(FaultType::AdcTimeout);
                control.strategy = ControlStrategy::Disabled;
                Decision::command(RequestedOutput::Inhibited(InhibitCause::Fault))
            }
            FastEvent::Frame(frame) => {
                let Some(decision) = control.decide_frame(frame, controller_state, faults) else {
                    continue;
                };
                decision
            }
        };

        if safety::dfu_shutdown_requested()
            && !matches!(decision.requested, RequestedOutput::DfuInhibited { .. })
        {
            control.strategy = ControlStrategy::Disabled;
            control.inverter.inhibit(InhibitCause::FirmwareUpdate);
            continue;
        }

        if faults.any_active()
            && matches!(
                decision.requested,
                RequestedOutput::ArmedSafe | RequestedOutput::Drive(_)
            )
        {
            control.strategy = ControlStrategy::Disabled;
            control.inverter.inhibit(InhibitCause::Fault);
            continue;
        }
        control.apply(&decision, faults);

        #[cfg(feature = "adc-timing")]
        if let Some((epoch, frame_cycle)) = decision.observed_frame {
            drivers::adc::timing::record_decision(epoch, frame_cycle);
        }

        freq_meter.tick();
        let elapsed_us = start_time.elapsed().as_micros() as u16;
        controller_state
            .last_foc_loop_time_us
            .store(elapsed_us, Ordering::Relaxed);
    }
}

#[cfg(feature = "adc-timing")]
#[embassy_executor::task]
pub async fn task_adc_timing_report() {
    let mut previous_missed = 0u32;
    let mut previous_report = Instant::now();
    loop {
        Timer::after_secs(1).await;
        let now = Instant::now();
        let window_us = (now - previous_report).as_micros();
        previous_report = now;
        let window = drivers::adc::timing::take_window();
        let total_missed = controller_shared::state::state()
            .adc_missed_frames
            .load(Ordering::Relaxed);
        let missed = total_missed.wrapping_sub(previous_missed);
        previous_missed = total_missed;

        if window.triggers > 1 && window.period_max == 0 {
            logging::error!("ADC timing DWT cycle counter is not advancing");
            continue;
        }

        logging::info!(
            "ADC diag: window_us={} triggers={} period_min={} period_max={} missed_epochs={}",
            window_us,
            window.triggers,
            window.period_min,
            window.period_max,
            missed
        );
        logging::info!(
            "ADC diag: pending_all={} partial={} recovered={} phase=[{},{},{}] clear_late={} uncertain_phase={} invalid={} stale={} refresh={}",
            window.pending_all,
            window.pending_partial,
            window.pending_recovered,
            window.pending_phase[0],
            window.pending_phase[1],
            window.pending_phase[2],
            window.clear_late,
            window.uncertain_phase,
            window.reject_invalid,
            window.reject_stale,
            window.refreshes
        );
        logging::info!(
            "ADC diag: JEOS counts=[{},{},{}] max=[{},{},{}] unmatched={} undelivered=[{},{},{}]",
            window.jeos_count[0],
            window.jeos_count[1],
            window.jeos_count[2],
            window.jeos_max[0],
            window.jeos_max[1],
            window.jeos_max[2],
            window.unmatched_jeos,
            window.undelivered[0],
            window.undelivered[1],
            window.undelivered[2]
        );
        logging::info!(
            "ADC diag: frames={} frame_max={} unmatched_frames={} decisions={} decision_max={} work_max={} unmatched_decisions={} duty_writes={} duty_max={} unmatched_duty_writes={}",
            window.frames,
            window.frame_max,
            window.unmatched_frames,
            window.decisions,
            window.decision_max,
            window.decision_work_max,
            window.unmatched_decisions,
            window.duty_writes,
            window.duty_write_max,
            window.unmatched_duty_writes
        );
    }
}

#[derive(Clone, Copy)]
enum RequestedOutput {
    ArmedSafe,
    Inhibited(InhibitCause),
    DfuInhibited { request_id: u32 },
    Drive(RawInverterValues),
    Fault(ControlFault),
}
