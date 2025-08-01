use core::f32::consts::PI;
pub use hardware_abstraction::models::Angle;
use micromath::F32Ext;

pub struct ElectricalAngle(pub u16);

impl ElectricalAngle {
    pub fn from_angle(angle: &Angle, offset: u16, pole_pairs: u8) -> Self {
        Self(
            ((angle.get_raw().wrapping_sub(offset) as u32 * pole_pairs as u32) % u16::MAX as u32)
                as u16,
        )
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
