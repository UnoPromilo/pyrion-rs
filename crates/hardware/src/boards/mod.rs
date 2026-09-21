#[cfg(not(feature = "board"))]
mod core_board;

#[cfg(feature = "board")]
mod adc_builder;
#[cfg(feature = "board")]
pub(crate) use adc_builder::build_adc;

#[cfg(feature = "board-pyrion-ovo")]
mod pyrion_ovo;
#[cfg(feature = "board-pyrion-ovo")]
pub use pyrion_ovo::{BOARD_ID, limits};

#[cfg(feature = "board-pyrion-nullo")]
mod pyrion_nullo;
#[cfg(feature = "board-pyrion-nullo")]
pub use pyrion_nullo::{BOARD_ID, limits};
