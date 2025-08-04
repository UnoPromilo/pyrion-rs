use crate::map::map;
use defmt::info;
use embassy_rp::pwm;
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embedded_hal::pwm::SetDutyCycle;
use hardware_abstraction::motor_driver;

const CLOCK_FREQUENCY: u32 = 125_000_000;
const DESIRED_FREQ: u32 = 20_000;
const PWM_PERIOD: i32 = (CLOCK_FREQUENCY / DESIRED_FREQ / 2) as i32;
const HALF_DEAD_TIME: i32 = 31; //*2 = 496 ns, should be enough

pub struct MotorDriver<'d> {
    a: Pwm<'d>,
    b: Pwm<'d>,
    c: Pwm<'d>,
}

// TODO remove if still not used
#[allow(dead_code)]
fn new_pwm_synced<'a, T: Slice>(
    slice: T,
    high_pin: impl ChannelAPin<T> + 'a,
    low_pin: impl ChannelBPin<T> + 'a,
) -> Pwm<'a> {
    let mut config = Config::default();
    config.invert_a = false;
    config.invert_b = true;
    config.phase_correct = true;
    config.enable = false;
    config.compare_a = 0;
    config.compare_b = 0;
    config.top = PWM_PERIOD as u16;

    let mut pwm = Pwm::new_output_ab(slice, high_pin, low_pin, config);
    // Safe because 0 is always less than max duty
    pwm.set_duty_cycle_fully_off().unwrap();
    pwm.set_counter(0);
    pwm
}

impl<'d> MotorDriver<'d> {
    // TODO remove if still not used
    #[allow(dead_code)]
    pub fn new(a: Pwm<'d>, b: Pwm<'d>, c: Pwm<'d>) -> Self {
        Self { a, b, c }
    }
}

impl motor_driver::MotorDriver for MotorDriver<'_> {
    fn enable(&mut self) {
        self.set_voltages(0, 0, 0);
        self.set_pwm_enabled(true);
        info!("Motor driver enabled");
    }

    fn set_voltages(&mut self, ua: i16, ub: i16, uc: i16) {
        Self::set_voltage(&mut self.a, ua);
        Self::set_voltage(&mut self.b, ub);
        Self::set_voltage(&mut self.c, uc);
    }

    fn disable(&mut self) {
        self.set_pwm_enabled(false);
        self.set_voltages(0, 0, 0);
        info!("Motor driver disabled");
    }
}

impl MotorDriver<'_> {
    fn set_pwm_enabled(&self, enable: bool) {
        pwm::PwmBatch::set_enabled(enable, |batch| {
            batch.enable(&self.a);
            batch.enable(&self.b);
            batch.enable(&self.c);
        });
    }

    fn set_voltage(channel: &mut Pwm, voltage: i16) {
        let dc = ((voltage as i32 + i16::MAX as i32) / 2) as u16;
        Self::set_duty_cycle(channel, dc);
    }

    fn set_duty_cycle(channel: &mut Pwm, duty_cycle: u16) {
        let duty_cycle = map(duty_cycle, u16::MAX, PWM_PERIOD as u16) as i32;
        let (high, low) = channel.split_by_ref();
        let high_duty_cycle = (duty_cycle - HALF_DEAD_TIME).max(0) as u16;
        let low_duty_cycle = (duty_cycle + HALF_DEAD_TIME).min(PWM_PERIOD) as u16;

        high.expect("High channel is mandatory")
            .set_duty_cycle(high_duty_cycle)
            .unwrap();
        low.expect("Low channel is mandatory")
            .set_duty_cycle(low_duty_cycle)
            .unwrap();
    }
}
