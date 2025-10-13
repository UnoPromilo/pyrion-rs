use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::timer::{
    AdvancedInstance4Channel, Ch1, Ch2, Ch4, Channel, TimerComplementaryPin, TimerPin,
};
use embassy_stm32::{Peri, pac};
use logging::trace;
use stm32_metapac::timer::vals::Mms;

const TRGO_OFFSET: u16 = 2;

pub struct Inverter<'a, T: AdvancedInstance4Channel> {
    pwm: ComplementaryPwm<'a, T>,
}

impl<'a, T: AdvancedInstance4Channel> Inverter<'a, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tim: Peri<'a, T>,
        ch_u: Peri<'a, impl TimerPin<T, Ch1>>,
        ch_un: Peri<'a, impl TimerComplementaryPin<T, Ch1>>,
        ch_v: Peri<'a, impl TimerPin<T, Ch2>>,
        ch_vn: Peri<'a, impl TimerComplementaryPin<T, Ch2>>,
        ch_w: Peri<'a, impl TimerPin<T, Ch4>>,
        ch_wn: Peri<'a, impl TimerComplementaryPin<T, Ch4>>,
        freq: Hertz,
    ) -> Self {
        let ch_u = PwmPin::new(ch_u, OutputType::PushPull);
        let ch_un = ComplementaryPwmPin::new(ch_un, OutputType::PushPull);
        let ch_v = PwmPin::new(ch_v, OutputType::PushPull);
        let ch_vn = ComplementaryPwmPin::new(ch_vn, OutputType::PushPull);
        let ch_w = PwmPin::new(ch_w, OutputType::PushPull);
        let ch_wn = ComplementaryPwmPin::new(ch_wn, OutputType::PushPull);

        let mut pwm = ComplementaryPwm::new(
            tim,
            Some(ch_u),
            Some(ch_un),
            Some(ch_v),
            Some(ch_vn),
            None,
            None,
            Some(ch_w),
            Some(ch_wn),
            freq,
            CountingMode::CenterAlignedDownInterrupts,
        );

        pwm.set_master_output_enable(false);
        pwm.enable(Channel::Ch1);
        pwm.enable(Channel::Ch2);
        pwm.enable(Channel::Ch3);
        pwm.enable(Channel::Ch4);

        trace!("Max PWM duty: {}", pwm.get_max_duty());
        assert!(
            pwm.get_max_duty() > TRGO_OFFSET,
            "Max PWM duty ({}) is lower than TRGO offset ({})",
            pwm.get_max_duty(),
            TRGO_OFFSET,
        );

        pwm.set_duty(Channel::Ch3, pwm.get_max_duty() - TRGO_OFFSET);
        Self::configure_trgo();

        Self { pwm }
    }

    pub fn enable(&mut self) {
        self.pwm.set_master_output_enable(true);
    }

    pub fn disable(&mut self) {
        self.pwm.set_master_output_enable(false);
    }

    pub fn set_dead_time(&mut self, dead_time: u16) {
        self.pwm.set_dead_time(dead_time);
    }

    pub fn set_phase_duties(&mut self, u: u16, v: u16, w: u16) {
        self.pwm.set_duty(Channel::Ch1, u);
        self.pwm.set_duty(Channel::Ch2, v);
        self.pwm.set_duty(Channel::Ch4, w);
    }

    pub fn get_max_duty(&self) -> u16 {
        self.pwm.get_max_duty()
    }

    fn configure_trgo() {
        // Safe because T is AdvancedInstance4Channel so it is TimAdv
        unsafe {
            pac::timer::TimAdv::from_ptr(T::regs())
                .cr2()
                .modify(|reg| reg.set_mms(Mms::COMPARE_OC3))
        }
    }
}
