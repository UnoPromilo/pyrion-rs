use crate::board::Board;
use controller_shared::{RawSnapshot, control_step};
use embassy_futures::join::join3;
use embassy_time::{Duration, with_timeout};

#[embassy_executor::task]
pub async fn task(board: Board<'static>) {
    let adc_3 = board.adc3_running;
    let adc_4 = board.adc4_running;
    let adc_5 = board.adc5_running;
    let mut inverter = board.inverter;
    let mut inverter_enabled = false;
    let max_duty = inverter.get_max_duty();
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
                i_w: values.2[0],
                v_ref: values.2[1],
                max_duty,
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
    }
}
