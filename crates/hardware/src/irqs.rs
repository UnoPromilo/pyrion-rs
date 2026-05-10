#[cfg(feature = "full")]
use embassy_stm32::peripherals::{
    ADC1, ADC2, ADC3, ADC4, ADC5, DMA1_CH1, DMA1_CH2, DMA1_CH3, DMA1_CH4, DMA1_CH5, DMA1_CH6,
    DMA1_CH7, DMA1_CH8, DMA2_CH1, DMA2_CH2, FDCAN2, I2C3, I2C4, USART1, USB,
};
#[cfg(feature = "full")]
use embassy_stm32::{bind_interrupts, can, dma, i2c, usart, usb};

#[cfg(not(feature = "full"))]
use embassy_stm32::peripherals::USB;
#[cfg(not(feature = "full"))]
use embassy_stm32::{bind_interrupts, usb};

#[cfg(feature = "full")]
bind_interrupts!(pub struct Irqs{
    ADC1_2 => adc::MultiInterruptHandler<ADC1, ADC2>;
    ADC3 => adc::SingleInterruptHandler<ADC3>;
    ADC4 => adc::SingleInterruptHandler<ADC4>;
    ADC5 => adc::SingleInterruptHandler<ADC5>;

    I2C3_EV => i2c::EventInterruptHandler<I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<I2C3>;

    I2C4_EV => i2c::EventInterruptHandler<I2C4>;
    I2C4_ER => i2c::ErrorInterruptHandler<I2C4>;

    USART1 => usart::InterruptHandler<USART1>;

    USB_LP => usb::InterruptHandler<USB>;

    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;

    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<DMA1_CH2>;
    DMA1_CHANNEL3 => dma::InterruptHandler<DMA1_CH3>;
    DMA1_CHANNEL4 => dma::InterruptHandler<DMA1_CH4>;
    DMA1_CHANNEL5 => dma::InterruptHandler<DMA1_CH5>;
    DMA1_CHANNEL6 => dma::InterruptHandler<DMA1_CH6>;
    DMA1_CHANNEL7 => dma::InterruptHandler<DMA1_CH7>;
    DMA1_CHANNEL8 => dma::InterruptHandler<DMA1_CH8>;

    DMA2_CHANNEL1 => dma::InterruptHandler<DMA2_CH1>;
    DMA2_CHANNEL2 => dma::InterruptHandler<DMA2_CH2>;
});

#[cfg(not(feature = "full"))]
bind_interrupts!(pub struct Irqs{
    USB_LP => usb::InterruptHandler<USB>;
});
