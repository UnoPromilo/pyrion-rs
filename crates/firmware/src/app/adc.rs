use crate::board::{BoardAdc, BoardInverter};
use controller_shared::{RawSnapshot, control_step};
use core::sync::atomic::Ordering;
use embassy_futures::join::join3;
use embassy_time::{Duration, with_timeout};
use logging::FreqMeter;
use portable_atomic::AtomicU16;

#[embassy_executor::task]
pub async fn task_adc(
    adc: BoardAdc<'static>,
    mut inverter: BoardInverter<'static>,
    raw_angle: &'static AtomicU16,
) {
    let adc_3 = adc.adc3_running;
    let adc_4 = adc.adc4_running;
    let adc_5 = adc.adc5_running;
    let mut inverter_enabled = false;
    let max_duty = inverter.get_max_duty();
    let controller_state = controller_shared::state::state();
    let mut freq_meter = FreqMeter::named("ADC");
    freq_meter.link(&controller_state.foc_loop_frequency);

    loop {
        let result = with_timeout(
            Duration::from_millis(10),
            join3(adc_3.read_next(), adc_4.read_next(), adc_5.read_next()),
        )
        .await;

        let raw_reading = match result {
            Ok(values) => Some(RawSnapshot {
                i_u: values.0[0],

                i_v: values.1[0],
                v_ref: values.1[1],
                v_bus: values.1[2],

                i_w: values.2[0],
                temp_cpu: values.2[1],

                max_duty,
                angle: raw_angle.load(Ordering::Relaxed),
            }),
            Err(_) => None,
        };

        let pwm = control_step(&raw_reading);

        match pwm {
            Some(values) => {
                if inverter_enabled == false {
                    inverter_enabled = true;
                    inverter.enable();
                }
                inverter.set_phase_duties(values.u, values.v, values.w);
            }
            None => {
                if inverter_enabled == true {
                    inverter_enabled = false;
                    inverter.disable();
                }
            }
        }
        freq_meter.tick();
    }
}
