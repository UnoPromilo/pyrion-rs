#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub enum Direction {
    #[default]
    Clockwise,
    CounterClockwise,
}
