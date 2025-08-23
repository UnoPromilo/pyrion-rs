use defmt::Format;

#[derive(Debug, Format, Eq, PartialEq, Copy, Clone, Default)]
pub enum Direction {
    #[default]
    Clockwise,
    CounterClockwise,
}
