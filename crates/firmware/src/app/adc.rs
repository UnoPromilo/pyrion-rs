use crate::app::communication::CONTROL_COMMAND_CHANNEL;
use crate::app::power_stage::{self, BoardOutput, UncheckedOutput};
use crate::app::safety;
use controller_shared::command::ControlCommand;
use controller_shared::output::{InhibitCause, SafeOutput};
use controller_shared::strategy::ControlStrategy;
use controller_shared::{
    ControlFault, ControlOutput, RawInverterValues, RawSnapshot, control_step,
};
use core::num::NonZeroU16;
use core::sync::atomic::{AtomicU16, Ordering};
use drivers::adc::epoch::{FrameAlignment, TriggerEpoch, align_fast_frame, is_invalid};
use drivers::adc::injected::Running;
use drivers::adc::slow_vref::{SlowVref, SlowVrefError};
use embassy_futures::join::join3;
use embassy_futures::select::{Either, select};
use embassy_stm32::adc::RingBufferedAdc;
use embassy_stm32::peripherals::{ADC1, ADC2, ADC3, ADC5};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use hardware::{AuxiliaryAdcCounts, BoardAdcFast, BoardAdcFrame};
use logging::FreqMeter;
use logging::error;
use logging::fault_register::{FaultRegister, FaultState, FaultType};

static VREFINT_SNAPSHOT: AtomicU16 = AtomicU16::new(0);

struct TriggeredFrame {
    epoch: TriggerEpoch,
    readings: BoardAdcFrame,
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

#[embassy_executor::task]
pub async fn task_slow_aux(mut adc: RingBufferedAdc<'static, ADC2>) {
    loop {
        let mut samples = [0; 3];
        if adc.read_latest(&mut samples) == samples.len() {
            let auxiliary = AuxiliaryAdcCounts::from_regular(samples);
            if let Some(vref) = NonZeroU16::new(VREFINT_SNAPSHOT.load(Ordering::Relaxed)) {
                controller_shared::store_bus_voltage(auxiliary.bus_voltage, vref);
            }
        }
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
pub async fn task_slow_vref(mut adc: SlowVref<'static>) {
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

#[embassy_executor::task]
pub async fn task_adc(adc: BoardAdcFast<'static>, pwm: UncheckedOutput) {
    let adc_1 = adc.adc1_running;
    let adc_3 = adc.adc3_running;
    let adc_5 = adc.adc5_running;
    let mut inverter = SafeOutput::new(BoardOutput::new(pwm));
    let max_duty = inverter.max_duty();
    let controller_state = controller_shared::state::state();
    let faults = FaultRegister::shared();

    let mut freq_meter = FreqMeter::named("ADC");
    freq_meter.link(&controller_state.foc_loop_frequency);

    let mut strategy = ControlStrategy::Disabled;
    let mut last_epoch = TriggerEpoch::UNSTARTED;
    let startup_started = Instant::now();
    let mut first_valid_frame = false;

    loop {
        let start_time = Instant::now();
        #[cfg(feature = "adc-timing")]
        let mut observed_frame = None;
        let adc_read = with_timeout(
            Duration::from_millis(1),
            read_triggered_frame(&adc_1, &adc_3, &adc_5, last_epoch),
        );

        let requested_output = match select(
            select(
                power_stage::next_gate_request(),
                CONTROL_COMMAND_CHANNEL.receive(),
            ),
            adc_read,
        )
        .await
        {
            Either::First(Either::First(request)) => {
                power_stage::handle_request(request, &mut inverter);
                continue;
            }
            Either::First(Either::Second(ControlCommand::Stop)) => {
                strategy = ControlStrategy::Disabled;
                RequestedOutput::ArmedSafe
            }
            Either::First(Either::Second(ControlCommand::InhibitForDfu { request_id })) => {
                strategy = ControlStrategy::Disabled;
                RequestedOutput::DfuInhibited { request_id }
            }
            Either::Second(Err(_))
                if !first_valid_frame && startup_started.elapsed() < Duration::from_millis(10) =>
            {
                continue;
            }
            Either::Second(Err(_)) => {
                faults.set(FaultType::AdcTimeout);
                strategy = ControlStrategy::Disabled;
                RequestedOutput::Inhibited(InhibitCause::Fault)
            }
            Either::Second(Ok(TriggeredFrame {
                epoch,
                readings: frame,
            })) => {
                let v_ref = VREFINT_SNAPSHOT.load(Ordering::Relaxed);
                let missed = epoch.skipped_since(last_epoch);
                if missed != 0 {
                    controller_state
                        .adc_missed_frames
                        .fetch_add(missed, Ordering::Relaxed);
                }
                last_epoch = epoch;
                faults.resolve_if_set(FaultType::AdcTimeout);

                if v_ref == 0
                    && !first_valid_frame
                    && startup_started.elapsed() < Duration::from_millis(10)
                {
                    continue;
                }

                #[cfg(feature = "adc-timing")]
                {
                    observed_frame = Some((epoch, drivers::adc::timing::record_frame(epoch)));
                }

                if faults.any_active() {
                    strategy = ControlStrategy::Disabled;
                }

                if faults.any_active()
                    && faults.load(FaultType::InvalidMeasurement) != FaultState::Active
                {
                    RequestedOutput::Inhibited(InhibitCause::Fault)
                } else {
                    let raw_reading = RawSnapshot {
                        i_u: frame.fast.current.u,
                        i_v: frame.fast.current.v,
                        i_w: frame.fast.current.w,

                        v_u: frame.fast.voltage.u,
                        v_v: frame.fast.voltage.v,
                        v_w: frame.fast.voltage.w,

                        v_ref,

                        max_duty,
                        angle: controller_state.raw_angle.load(Ordering::Relaxed),
                    };
                    match control_step(&raw_reading, &mut strategy) {
                        ControlOutput::Safe => {
                            first_valid_frame = true;
                            faults.resolve_if_set(FaultType::InvalidMeasurement);
                            if faults.any_active() {
                                RequestedOutput::Inhibited(InhibitCause::Fault)
                            } else {
                                RequestedOutput::ArmedSafe
                            }
                        }
                        ControlOutput::Drive(values) => {
                            first_valid_frame = true;
                            RequestedOutput::Drive(values)
                        }
                        ControlOutput::Fault(fault) => RequestedOutput::Fault(fault),
                    }
                }
            }
        };

        if safety::dfu_shutdown_requested()
            && !matches!(requested_output, RequestedOutput::DfuInhibited { .. })
        {
            strategy = ControlStrategy::Disabled;
            inverter.inhibit(InhibitCause::FirmwareUpdate);
            continue;
        }

        if faults.any_active()
            && matches!(
                requested_output,
                RequestedOutput::ArmedSafe | RequestedOutput::Drive(_)
            )
        {
            strategy = ControlStrategy::Disabled;
            inverter.inhibit(InhibitCause::Fault);
            continue;
        }
        match requested_output {
            RequestedOutput::ArmedSafe => inverter.enter_armed_safe(),
            RequestedOutput::Inhibited(cause) => inverter.inhibit(cause),
            RequestedOutput::DfuInhibited { request_id } => {
                inverter.inhibit(InhibitCause::FirmwareUpdate);
                safety::acknowledge_dfu_output_inhibited(request_id);
            }
            RequestedOutput::Drive(values) => {
                let result = inverter.drive(values);
                #[cfg(feature = "adc-timing")]
                if result.is_ok() {
                    drivers::adc::timing::record_duty_write(observed_frame.map(|(epoch, _)| epoch));
                }
                if result.is_err() {
                    faults.set(FaultType::InvalidControllerOutput);
                    strategy = ControlStrategy::Disabled;
                    inverter.inhibit(InhibitCause::Fault);
                }
            }
            RequestedOutput::Fault(fault) => {
                faults.set(match fault {
                    ControlFault::InvalidMeasurement => FaultType::InvalidMeasurement,
                    ControlFault::InvalidControllerOutput => FaultType::InvalidControllerOutput,
                });
                strategy = ControlStrategy::Disabled;
                inverter.inhibit(InhibitCause::Fault);
            }
        }

        #[cfg(feature = "adc-timing")]
        if let Some((epoch, frame_cycle)) = observed_frame {
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

enum RequestedOutput {
    ArmedSafe,
    Inhibited(InhibitCause),
    DfuInhibited { request_id: u32 },
    Drive(RawInverterValues),
    Fault(ControlFault),
}
