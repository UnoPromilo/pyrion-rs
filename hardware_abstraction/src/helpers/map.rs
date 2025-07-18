pub fn map(x: u16, in_max: u16, out_max: u16) -> u16 {
    (x as u32 * out_max as u32 / in_max as u32) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_basic() {
        assert_eq!(map(50, 100, 200), 100);
        assert_eq!(map(50, 200, 100), 25);
        assert_eq!(map(50, 200, 200), 50);
    }

    #[test]
    fn test_map_real() {
        const PWM_PERIOD: u16 = 3125;

        assert_eq!(map(32767, u16::MAX, PWM_PERIOD), 1562);
        assert_eq!(map(16383, u16::MAX, PWM_PERIOD), 781);
    }
}
