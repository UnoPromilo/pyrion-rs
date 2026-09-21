use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{self, Spi};

pub type Drv8301Spi<'a> = Spi<'a, Async, spi::mode::Master>;

pub struct Drv8301<'a> {
    _spi: Drv8301Spi<'a>,
    _cs: Output<'a>,
    en_gate: Output<'a>,
    _nfault: Input<'a>,
}

impl<'a> Drv8301<'a> {
    pub fn new(spi: Drv8301Spi<'a>, cs: Output<'a>, en_gate: Output<'a>, nfault: Input<'a>) -> Self {
        let mut driver = Self {
            _spi: spi,
            _cs: cs,
            en_gate,
            _nfault: nfault,
        };
        driver.disable();
        driver
    }

    pub fn enable(&mut self) {
        self.en_gate.set_high();
    }

    pub fn disable(&mut self) {
        self.en_gate.set_low();
    }
}
