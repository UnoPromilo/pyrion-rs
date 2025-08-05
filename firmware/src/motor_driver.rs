use crate::config::MotorConfig;
use crate::map::map;
use defmt::info;
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embassy_rp::{Peri, pwm};
use embedded_hal::pwm::SetDutyCycle;
use foc::Motor;
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

fn new_pwm_synced<'a, T: Slice>(
    slice: Peri<'a, T>,
    high_pin: Peri<'a, impl ChannelAPin<T>>,
    low_pin: Peri<'a, impl ChannelBPin<T>>,
) -> Pwm<'a> {
    let config = default_config();
    let mut pwm = Pwm::new_output_ab(slice, high_pin, low_pin, config);
    // Safe because 0 is always less than max duty
    pwm.set_duty_cycle_fully_off().unwrap();
    pwm.set_counter(0);
    pwm
}

fn default_config() -> Config {
    let mut config = Config::default();
    config.invert_a = false;
    config.invert_b = true;
    config.phase_correct = true;
    config.enable = false;
    config.compare_a = 0;
    config.compare_b = 0;
    config.top = PWM_PERIOD as u16;
    config
}

impl<'d> MotorDriver<'d> {
    pub fn new(config: MotorConfig) -> Self {
        let a = new_pwm_synced(config.a_slice, config.a_high, config.a_low);
        let b = new_pwm_synced(config.b_slice, config.b_high, config.b_low);
        let c = new_pwm_synced(config.c_slice, config.c_high, config.c_low);
        Self { a, b, c }
    }
}

impl motor_driver::MotorDriver for MotorDriver<'_> {
    fn enable_synced(&mut self) {
        self.set_pwm_enabled(true);
        info!("Motor driver enabled");
    }

    fn enable_phase_a(&mut self) {
        let mut config = default_config();
        config.enable = true;
        self.a.set_counter(0);
        self.a.set_config(&config);
    }

    fn enable_phase_b(&mut self) {
        let mut config = default_config();
        config.enable = true;
        self.b.set_counter(0);
        self.b.set_config(&config);
    }

    fn enable_phase_c(&mut self) {
        let mut config = default_config();
        config.enable = true;
        self.c.set_counter(0);
        self.c.set_config(&config);
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
        self.a.set_counter(0);
        self.b.set_counter(0);
        self.c.set_counter(0);

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

#[embassy_executor::task]
pub async fn drive_motor_task(motor: &'static Motor, hardware_config: MotorConfig) {
    let motor_driver = MotorDriver::new(hardware_config);
    loop {
        // Run as often as possible but allow other tasks to execute too
        embassy_futures::yield_now().await;
    }
}
