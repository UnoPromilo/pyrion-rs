use crate::app::communication::CONTROL_COMMAND_CHANNEL;
use crate::app::safety;
use controller_shared::command::ControlCommand;
use controller_shared::output::{InhibitCause, InverterOutput, SafeOutput};
use controller_shared::strategy::ControlStrategy;
use controller_shared::{
    ControlFault, ControlOutput, RawInverterValues, RawSnapshot, control_step,
};
use core::sync::atomic::Ordering;
use embassy_futures::join::join5;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, with_timeout};
use hardware::{BoardAdc, BoardInverter};
use logging::FreqMeter;
use logging::fault_register::{FaultRegister, FaultType};

#[embassy_executor::task]
pub async fn task_adc(adc: BoardAdc<'static>, inverter: BoardInverter<'static>) {
    let adc_1 = adc.adc1_running;
    let adc_2 = adc.adc2_running;
    let adc_3 = adc.adc3_running;
    let adc_4 = adc.adc4_running;
    let adc_5 = adc.adc5_running;
    let mut inverter = SafeOutput::new(BoardOutput(inverter));
    let max_duty = inverter.max_duty();
    let controller_state = controller_shared::state::state();
    let faults = FaultRegister::shared();

    let mut freq_meter = FreqMeter::named("ADC");
    freq_meter.link(&controller_state.foc_loop_frequency);

    let mut strategy = ControlStrategy::Disabled;

    loop {
        let start_time = Instant::now();
        let adc_read = with_timeout(
            Duration::from_millis(1),
            join5(
                adc_1.read_next(),
                adc_2.read_next(),
                adc_3.read_next(),
                adc_4.read_next(),
                adc_5.read_next(),
            ),
        );

        let requested_output = match select(CONTROL_COMMAND_CHANNEL.receive(), adc_read).await {
            Either::First(ControlCommand::Stop) => {
                strategy = ControlStrategy::Disabled;
                RequestedOutput::ArmedSafe
            }
            Either::First(ControlCommand::InhibitForDfu { request_id }) => {
                strategy = ControlStrategy::Disabled;
                RequestedOutput::DfuInhibited { request_id }
            }
            Either::Second(Err(_)) => {
                faults.latch(FaultType::AdcTimeout);
                strategy = ControlStrategy::Disabled;
                RequestedOutput::Inhibited(InhibitCause::Fault)
            }
            Either::Second(Ok(values)) => {
                if faults.any_active() || faults.any_latched() {
                    strategy = ControlStrategy::Disabled;
                    RequestedOutput::Inhibited(InhibitCause::Fault)
                } else {
                    let raw_reading = RawSnapshot {
                        i_u: values.0[0],
                        i_v: values.2[0],
                        i_w: values.4[0],

                        v_u: values.0[1],
                        v_v: values.2[1],
                        v_w: values.4[1],

                        v_ref: values.3[0],
                        v_bus: values.1[2],

                        // TODO fix temperature sensor or remove it totally
                        temp_cpu: 0,
                        temp_motor: values.1[1],
                        temp_driver: values.1[0],

                        analog_input: values.0[2],

                        max_duty,
                        angle: controller_state.raw_angle.load(Ordering::Relaxed),
                    };
                    match control_step(&raw_reading, &mut strategy) {
                        ControlOutput::Safe => RequestedOutput::ArmedSafe,
                        ControlOutput::Drive(values) => RequestedOutput::Drive(values),
                        ControlOutput::Fault(fault) => RequestedOutput::Fault(fault),
                    }
                }
            }
        };

        match requested_output {
            RequestedOutput::ArmedSafe => inverter.enter_armed_safe(),
            RequestedOutput::Inhibited(cause) => inverter.inhibit(cause),
            RequestedOutput::DfuInhibited { request_id } => {
                inverter.inhibit(InhibitCause::FirmwareUpdate);
                safety::acknowledge_dfu_output_inhibited(request_id);
            }
            RequestedOutput::Drive(values) => {
                if inverter.drive(values).is_err() {
                    faults.latch(FaultType::InvalidControllerOutput);
                    strategy = ControlStrategy::Disabled;
                    inverter.inhibit(InhibitCause::Fault);
                }
            }
            RequestedOutput::Fault(fault) => {
                faults.latch(match fault {
                    ControlFault::InvalidMeasurement => FaultType::InvalidMeasurement,
                    ControlFault::InvalidControllerOutput => FaultType::InvalidControllerOutput,
                });
                strategy = ControlStrategy::Disabled;
                inverter.inhibit(InhibitCause::Fault);
            }
        }

        freq_meter.tick();
        let elapsed_us = start_time.elapsed().as_micros() as u16;
        controller_state
            .last_foc_loop_time_us
            .store(elapsed_us, Ordering::Relaxed);
    }
}

enum RequestedOutput {
    ArmedSafe,
    Inhibited(InhibitCause),
    DfuInhibited { request_id: u32 },
    Drive(RawInverterValues),
    Fault(ControlFault),
}

struct BoardOutput(BoardInverter<'static>);

impl InverterOutput for BoardOutput {
    fn max_duty(&self) -> u32 {
        self.0.get_max_duty()
    }

    fn disable_outputs(&mut self) {
        self.0.disable();
    }

    fn write_phase_duties(&mut self, duties: RawInverterValues) {
        self.0.set_phase_duties(duties.u, duties.v, duties.w);
    }

    fn enable_outputs(&mut self) {
        self.0.enable();
    }
}
