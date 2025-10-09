pub struct Config {
    pub start_angle: u16,
    pub end_angle: u16,
    pub hysteresis: Hysteresis,
    pub fast_filter_threshold: FastFilterThreshold,
    pub slow_filter: SlowFilter,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Hysteresis {
    Off = 0b00,
    LSB1 = 0b01,
    LSB2 = 0b10,
    LSB3 = 0b11,
}

impl From<Hysteresis> for u8 {
    fn from(value: Hysteresis) -> Self {
        value as u8
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum SlowFilter {
    X16 = 0b00,
    X8 = 0b01,
    X4 = 0b10,
    X2 = 0b11,
}

impl From<SlowFilter> for u8 {
    fn from(value: SlowFilter) -> Self {
        value as u8
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum FastFilterThreshold {
    SlowFilterOnly = 0b00,
    LSB6 = 0b001,
    LSB7 = 0b010,
    LSB9 = 0b011,
    LSB18 = 0b100,
    LSB21 = 0b101,
    LSB24 = 0b110,
    LSB10 = 0b111,
}

impl From<FastFilterThreshold> for u8 {
    fn from(value: FastFilterThreshold) -> Self {
        (value as u8) << 2
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_angle: 0,
            end_angle: 4095,
            hysteresis: Hysteresis::LSB2,
            fast_filter_threshold: FastFilterThreshold::LSB6,
            slow_filter: SlowFilter::X8,
        }
    }
}

impl Config {
    pub fn get_low_config_byte(&self) -> u8 {
        self.hysteresis.into()
    }

    pub fn get_high_config_byte(&self) -> u8 {
        let fast_filter: u8 = self.fast_filter_threshold.into();
        let slow_filter: u8 = self.slow_filter.into();

        fast_filter | slow_filter
    }

    pub fn get_low_z_pos(&self) -> u8 {
        (self.start_angle & 0xFF) as u8
    }

    pub fn get_high_z_pos(&self) -> u8 {
        ((self.start_angle >> 8) & 0x0F) as u8
    }

    pub fn get_low_m_pos(&self) -> u8 {
        (self.end_angle & 0xFF) as u8
    }

    pub fn get_high_m_pos(&self) -> u8 {
        ((self.end_angle >> 8) & 0x0F) as u8
    }
}
