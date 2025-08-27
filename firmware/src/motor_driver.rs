use crate::config::MotorConfig;
use crate::current_sensor::ThreePhaseCurrentTrigger;
use crate::map::map;
use core::cell::RefCell;
use embassy_rp::pac::interrupt;
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embassy_rp::{Peri, pac, pwm};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal::pwm::SetDutyCycle;
use foc::Motor;
use hardware_abstraction::motor_driver;
use shared::debug;

const CLOCK_FREQUENCY: u32 = 125_000_000;
const DESIRED_FREQ: u32 = 30_000;
const PWM_PERIOD: i32 = (CLOCK_FREQUENCY / DESIRED_FREQ / 2) as i32;
const HALF_DEAD_TIME: i32 = (CLOCK_FREQUENCY as i32) * 5 / 10_000_000; // ~500ns

static TRIGGER: Mutex<CriticalSectionRawMutex, RefCell<Option<ThreePhaseCurrentTrigger<'static, 'static>>>> =
    Mutex::new(RefCell::new(None));

pub struct MotorDriver<'d> {
    a: Pwm<'d>,
    b: Pwm<'d>,
    c: Pwm<'d>,

    // These are used to enable the interrupt on the correct slice. If none, then the interrupt is disabled
    a_slice: Option<usize>,
    b_slice: Option<usize>,
    c_slice: Option<usize>,
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
        Self {
            a,
            b,
            c,
            a_slice: None,
            b_slice: None,
            c_slice: None,
        }
    }

    pub fn new_with_trigger(
        config: MotorConfig,
        current_sensor_trigger: ThreePhaseCurrentTrigger<'static, 'static>,
    ) -> Self {
        let a_slice = Some(config.a_slice.number());
        let b_slice = Some(config.b_slice.number());
        let c_slice = Some(config.c_slice.number());
        let a = new_pwm_synced(config.a_slice, config.a_high, config.a_low);
        let b = new_pwm_synced(config.b_slice, config.b_high, config.b_low);
        let c = new_pwm_synced(config.c_slice, config.c_high, config.c_low);
        TRIGGER.lock(|handler| {
            handler.borrow_mut().replace(current_sensor_trigger);
        });
        unsafe {
            cortex_m::peripheral::NVIC::unmask(interrupt::PWM_IRQ_WRAP);
        }
        Self {
            a,
            b,
            c,
            a_slice,
            b_slice,
            c_slice,
        }
    }
}

impl motor_driver::MotorDriver for MotorDriver<'_> {
    fn enable_synced(&mut self) {
        if let Some(a_slice_index) = self.a_slice {
            setup_interrupt(a_slice_index);
        }

        let config = default_config();
        self.a.set_config(&config);
        self.b.set_config(&config);
        self.c.set_config(&config);
        self.set_pwm_enabled(true);
        debug!("Motor driver enabled");
    }

    fn enable_phase_a(&mut self) {
        if let Some(a_slice_index) = self.a_slice {
            setup_interrupt(a_slice_index);
        }
        let mut config = default_config();
        config.enable = true;
        self.a.set_counter(0);
        self.a.set_config(&config);
        debug!("Motor driver enabled (phase A)");
    }

    fn enable_phase_b(&mut self) {
        if let Some(b_slice_index) = self.b_slice {
            setup_interrupt(b_slice_index);
        }
        let mut config = default_config();
        config.enable = true;
        self.b.set_counter(0);
        self.b.set_config(&config);
        debug!("Motor driver enabled (phase B)");
    }

    fn enable_phase_c(&mut self) {
        if let Some(c_slice_index) = self.c_slice {
            setup_interrupt(c_slice_index);
        }
        let mut config = default_config();
        config.enable = true;
        self.c.set_counter(0);
        self.c.set_config(&config);
        debug!("Motor driver enabled (phase C)");
    }

    fn set_voltages(&mut self, ua: i16, ub: i16, uc: i16) {
        Self::set_voltage(&mut self.a, ua);
        Self::set_voltage(&mut self.b, ub);
        Self::set_voltage(&mut self.c, uc);
    }

    fn set_voltage_a(&mut self, ua: i16) {
        Self::set_voltage(&mut self.a, ua);
    }

    fn set_voltage_b(&mut self, ub: i16) {
        Self::set_voltage(&mut self.b, ub);
    }

    fn set_voltage_c(&mut self, uc: i16) {
        Self::set_voltage(&mut self.c, uc);
    }

    fn disable(&mut self) {
        let mut config = default_config();
        config.invert_b = false;
        self.a.set_config(&config);
        self.b.set_config(&config);
        self.c.set_config(&config);

        self.set_pwm_enabled(false);
        self.a.phase_advance();
        self.b.phase_advance();
        self.c.phase_advance();

        if let Some(a_slice_index) = self.a_slice {
            disable_interrupt(a_slice_index);
        }

        if let Some(b_slice_index) = self.b_slice {
            disable_interrupt(b_slice_index);
        }

        if let Some(c_slice_index) = self.c_slice {
            disable_interrupt(c_slice_index);
        }

        debug!("Motor driver disabled");
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

        if let Some(mut h) = high {
            h.set_duty_cycle(high_duty_cycle).unwrap();
        }
        if let Some(mut l) = low {
            l.set_duty_cycle(low_duty_cycle).unwrap();
        }
    }
}

#[embassy_executor::task]
pub async fn drive_motor_task(
    motor: &'static Motor,
    hardware_config: MotorConfig,
    current_sensor_trigger: Option<ThreePhaseCurrentTrigger<'static, 'static>>,
) {
    let mut motor_driver = match current_sensor_trigger {
        None => MotorDriver::new(hardware_config),
        Some(trigger) => MotorDriver::new_with_trigger(hardware_config, trigger),
    };
    loop {
        foc::state_machine::on_tick(motor, &mut motor_driver).await;
        embassy_futures::yield_now().await;
    }
}

fn setup_interrupt(slice_index: usize) {
    match slice_index {
        0 => pac::PWM.inte().modify(|w| w.set_ch0(true)),
        1 => pac::PWM.inte().modify(|w| w.set_ch1(true)),
        2 => pac::PWM.inte().modify(|w| w.set_ch2(true)),
        3 => pac::PWM.inte().modify(|w| w.set_ch3(true)),
        4 => pac::PWM.inte().modify(|w| w.set_ch4(true)),
        5 => pac::PWM.inte().modify(|w| w.set_ch5(true)),
        6 => pac::PWM.inte().modify(|w| w.set_ch6(true)),
        7 => pac::PWM.inte().modify(|w| w.set_ch7(true)),
        _ => panic!("Invalid slice index"),
    }
}

fn disable_interrupt(slice_index: usize) {
    match slice_index {
        0 => pac::PWM.inte().modify(|w| w.set_ch0(false)),
        1 => pac::PWM.inte().modify(|w| w.set_ch1(false)),
        2 => pac::PWM.inte().modify(|w| w.set_ch2(false)),
        3 => pac::PWM.inte().modify(|w| w.set_ch3(false)),
        4 => pac::PWM.inte().modify(|w| w.set_ch4(false)),
        5 => pac::PWM.inte().modify(|w| w.set_ch5(false)),
        6 => pac::PWM.inte().modify(|w| w.set_ch6(false)),
        7 => pac::PWM.inte().modify(|w| w.set_ch7(false)),
        _ => panic!("Invalid slice index"),
    }
}

#[interrupt]
unsafe fn PWM_IRQ_WRAP() {
    critical_section::with(|cs| {
        let status = pac::PWM.intr().read();
        pac::PWM.intr().write_value(status);
        TRIGGER
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .update_buffer();
    });
}
