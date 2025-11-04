use crate::board::{BoardAdc, BoardInverter};
use controller_shared::{RawSnapshot, control_step};
use core::sync::atomic::Ordering;
use embassy_futures::join::join5;
use embassy_time::{Duration, Instant, with_timeout};
use logging::FreqMeter;

#[embassy_executor::task]
pub async fn task_adc(adc: BoardAdc<'static>, mut inverter: BoardInverter<'static>) {
    let adc_1 = adc.adc1_running;
    let adc_2 = adc.adc2_running;
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
            Duration::from_millis(100),
            join5(
                adc_1.read_next(),
                adc_2.read_next(),
                adc_3.read_next(),
                adc_4.read_next(),
                adc_5.read_next(),
            ),
        )
        .await;
        let start_time = Instant::now();
        let raw_reading = match result {
            Ok(values) => Some(RawSnapshot {
                i_u: values.0[0],
                i_v: values.2[0],
                i_w: values.4[0],

                v_u: values.0[1],
                v_v: values.2[1],
                v_w: values.4[1],

                v_ref: values.3[0],
                v_bus: values.1[2],

                temp_cpu: values.4[2],
                temp_motor: values.1[1],
                temp_driver: values.1[0],

                analog_input: values.0[2],

                max_duty,
                angle: controller_state.raw_angle.load(Ordering::Relaxed),
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
        let elapsed_us = start_time.elapsed().as_micros() as u16;
        controller_state
            .last_foc_loop_time_us
            .store(elapsed_us, Ordering::Relaxed);
    }
}
