use embassy_stm32::gpio::{Level, Output};

pub struct SenseFilter<'a> {
    control: Output<'a>,
}

impl<'a> SenseFilter<'a> {
    pub fn new(control: Output<'a>) -> Self {
        Self { control }
    }

    pub fn enable(&mut self) {
        self.control.set_high();
    }

    pub fn disable(&mut self) {
        self.control.set_low();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.control.set_level(Level::from(enabled));
    }

    pub fn is_enabled(&self) -> bool {
        self.control.is_set_high()
    }
}
