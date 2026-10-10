#[cfg(not(feature = "board"))]
mod core_board;

#[cfg(feature = "board")]
mod adc_builder;
#[cfg(feature = "board")]
mod adc_frame;
#[cfg(feature = "board")]
pub(crate) use adc_builder::build_adc;
#[cfg(feature = "board")]
pub use adc_frame::{AuxiliaryAdcCounts, BoardAdcFrame, FastAdcCounts, Phase};

#[cfg(feature = "cap-drv8301")]
fn drv8301_spi_config() -> embassy_stm32::spi::Config {
    use embassy_stm32::spi::{self, BitOrder};
    use embassy_stm32::time::mhz;

    let mut config = spi::Config::default();
    config.mode = spi::MODE_1;
    config.bit_order = BitOrder::MsbFirst;
    config.frequency = mhz(1);
    config
}

#[cfg(feature = "board-pyrion-ovo")]
mod pyrion_ovo;
#[cfg(feature = "board-pyrion-ovo")]
pub use pyrion_ovo::{BOARD_ID, limits, validate_drv8301_configuration};

#[cfg(feature = "board-pyrion-nullo")]
mod pyrion_nullo;
#[cfg(feature = "board-pyrion-nullo")]
pub use pyrion_nullo::{BOARD_ID, limits};
