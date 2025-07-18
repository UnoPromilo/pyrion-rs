use bldc_logic::helpers;
use embassy_rp::pwm;
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embedded_hal::pwm::SetDutyCycle;
use hardware_abstraction::motor_driver;

const CLOCK_FREQUENCY: u32 = 125_000_000;
const DESIRED_FREQ: u32 = 20_000;
const PWM_PERIOD: u16 = (CLOCK_FREQUENCY / DESIRED_FREQ / 2) as u16;
const HALF_DEAD_TIME: u16 = 31; //*2 =  496ns, should be enough

pub struct MotorDriver<'d> {
    a: Pwm<'d>,
    b: Pwm<'d>,
    c: Pwm<'d>,
}
pub fn new_pwm_synced<'a, T: Slice>(
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
    config.top = PWM_PERIOD;

    let mut pwm = Pwm::new_output_ab(slice, high_pin, low_pin, config);
    // Safe because 0 is always less than max duty
    pwm.set_duty_cycle_fully_off().unwrap();
    pwm.set_counter(0);
    pwm
}

impl<'d> MotorDriver<'d> {
    pub fn new(a: Pwm<'d>, b: Pwm<'d>, c: Pwm<'d>) -> Self {
        Self { a, b, c }
    }
}

impl motor_driver::MotorDriver for MotorDriver<'_> {
    fn init(&mut self) {
        self.disable()
    }

    fn enable(&mut self) {
        self.set_pwm_values(0, 0, 0);
        self.set_pwm_enabled(true);
    }

    fn disable(&mut self) {
        self.set_pwm_enabled(false);
        self.set_pwm_values(0, 0, 0);
    }

    fn set_pwm_values(&mut self, a: u16, b: u16, c: u16) {
        Self::set_duty_cycle(&mut self.a, a);
        Self::set_duty_cycle(&mut self.b, b);
        Self::set_duty_cycle(&mut self.c, c);
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

    fn set_duty_cycle(channel: &mut Pwm, duty_cycle: u16) {
        let duty_cycle = helpers::map(duty_cycle, u16::MAX, PWM_PERIOD);
        let (high, low) = channel.split_by_ref();
        let high_duty_cycle = (duty_cycle - HALF_DEAD_TIME).max(0);
        let low_duty_cycle = (duty_cycle + HALF_DEAD_TIME).min(PWM_PERIOD);

        high.expect("High channel is mandatory")
            .set_duty_cycle(high_duty_cycle)
            .unwrap();
        low.expect("Low channel is mandatory")
            .set_duty_cycle(low_duty_cycle)
            .unwrap();
    }
}
