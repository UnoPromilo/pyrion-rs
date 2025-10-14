pub struct RawSnapshot {
    pub i_u: u16,
    pub i_v: u16,
    pub i_w: u16,
    pub v_ref: u16,

    pub max_duty: u16,

    pub angle: u16,
}

pub struct RawInverterValues {
    pub u: u16,
    pub v: u16,
    pub w: u16,
}
