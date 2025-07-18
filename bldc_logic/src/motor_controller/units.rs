use std::f32::consts::PI;

pub struct Angle(pub u16);

pub struct ElectricalAngle(pub Angle);

impl Angle {
    pub fn from_degrees(degrees: u16) -> Self {
        debug_assert!(degrees < 360);
        Angle(((degrees as u32 * u16::MAX as u32) / 360) as u16)
    }

    // TODO remove
    pub fn as_rad(&self) -> f32 {
        self.0 as f32 * 2.0 * PI / u16::MAX as f32
    }
    
    pub fn sin_q15(&self) -> i16 {
        // TODO Replace by SIN_TABLE
        let rad = self.as_rad();
        (rad.sin() * i16::MAX as f32) as i16
    }

    pub fn cos_q15(&self) -> i16 {
        // TODO Replace by COS_TABLE
        let rad = self.as_rad();
        (rad.cos() * i16::MAX as f32) as i16
    }
}
