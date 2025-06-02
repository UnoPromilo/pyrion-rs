use embassy_rp::Peripheral;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::pwm::{ChannelAPin, Config, Pwm, PwmBatch, Slice};
use embassy_time::Timer;
use embedded_hal::pwm::SetDutyCycle;
use hardware_abstraction::motor_driver;
use hardware_abstraction::motor_driver::CommutationStep;

pub struct MotorDriver<'d> {
    a: Channel<'d>,
    b: Channel<'d>,
    c: Channel<'d>,

    current_step: Option<CommutationStep>,
}

pub struct Channel<'d> {
    high: Pwm<'d>,
    low: Output<'d>,
    duty_cycle: u16,
    is_high: bool,
}

impl<'d> Channel<'d> {
    pub fn new_synced<T: Slice>(
        slice: T,
        high_pin: impl ChannelAPin<T>,
        low_pin: impl Peripheral<P = impl Pin> + 'd,
    ) -> Self {
        let mut config = Config::default();
        config.enable = false;
        let mut high = Pwm::new_output_a(slice, high_pin, config);
        // Safe because 0 is always less than max duty
        high.set_duty_cycle_fully_off().unwrap();
        let low = Output::new(low_pin, Level::Low);
        high.set_counter(0);
        Channel {
            high,
            low,
            duty_cycle: 0,
            is_high: false,
        }
    }

    pub fn register_in_batch(&self, batch: &mut PwmBatch) {
        batch.enable(&self.high);
    }
}
impl Channel<'_> {
    pub async fn commutate(&mut self) {
        self.low.set_low();

        // Dead time
        Timer::after_micros(1).await;

        self.high
            .set_duty_cycle_fraction(self.duty_cycle, u16::MAX)
            .unwrap();

        self.is_high = true;
    }

    pub async fn decommutate(&mut self) {
        self.high.set_duty_cycle_fully_off().unwrap();

        // Dead time
        Timer::after_micros(1).await;

        self.low.set_high();

        self.is_high = false;
    }

    fn disable(&mut self) {
        self.high.set_duty_cycle_fully_off().unwrap();
        self.low.set_low();
        self.is_high = false;
    }

    fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.duty_cycle = duty_cycle;
        if self.is_high {
            self.high
                .set_duty_cycle_fraction(self.duty_cycle, u16::MAX)
                .unwrap();
        }
    }
}

impl<'d> MotorDriver<'d> {
    pub fn new(a: Channel<'d>, b: Channel<'d>, c: Channel<'d>) -> Self {
        Self {
            a,
            b,
            c,
            current_step: None,
        }
    }

    async fn set_commutation(high: &mut Channel<'d>, low: &mut Channel<'d>, off: &mut Channel<'d>) {
        off.disable();
        low.decommutate().await;
        high.commutate().await;
    }
}

impl motor_driver::MotorDriver for MotorDriver<'_> {
    async fn set_step(&mut self, value: CommutationStep) {
        use CommutationStep::*;

        match value {
            AB => Self::set_commutation(&mut self.a, &mut self.b, &mut self.c),
            AC => Self::set_commutation(&mut self.a, &mut self.c, &mut self.b),
            BA => Self::set_commutation(&mut self.b, &mut self.a, &mut self.c),
            BC => Self::set_commutation(&mut self.b, &mut self.c, &mut self.a),
            CA => Self::set_commutation(&mut self.c, &mut self.a, &mut self.b),
            CB => Self::set_commutation(&mut self.c, &mut self.b, &mut self.a),
        }
        .await;

        self.current_step = Some(value);
    }

    fn get_step(&self) -> Option<CommutationStep> {
        self.current_step
    }

    fn disable(&mut self) {
        self.a.disable();
        self.b.disable();
        self.c.disable();
        self.current_step = None;
    }

    fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.a.set_duty_cycle(duty_cycle);
        self.b.set_duty_cycle(duty_cycle);
        self.c.set_duty_cycle(duty_cycle);
    }

    fn get_duty_cycle(&self) -> u16 {
        self.a.duty_cycle
    }
}
