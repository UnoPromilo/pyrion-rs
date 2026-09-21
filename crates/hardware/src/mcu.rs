use embassy_stm32::Peripherals;
use embassy_stm32::pac::rcc::vals::Pllq;
use embassy_stm32::rcc::mux::{Clk48sel, Fdcansel};
use embassy_stm32::time::Hertz;

pub(crate) fn configure_mcu() -> Peripherals {
    let config = {
        use embassy_stm32::rcc::*;
        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(24),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL85,
            divp: None,
            divq: Some(Pllq::DIV8),
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.mux.adc345sel = mux::Adcsel::SYS;
        config.rcc.mux.clk48sel = Clk48sel::HSI48;
        config.rcc.mux.fdcansel = Fdcansel::PLL1_Q;
        config.rcc.boost = true;
        config
    };
    embassy_stm32::init(config)
}
