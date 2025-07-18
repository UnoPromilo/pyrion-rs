pub struct Config {
    pub pole_pairs: u8,
    pub angle_offset: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pole_pairs: 7,
            angle_offset: 0,
        }
    }
}
