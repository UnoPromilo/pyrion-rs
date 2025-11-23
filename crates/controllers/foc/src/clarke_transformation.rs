use units::ElectricCurrent;

pub const ONE_OVER_SQRT3: f32 = 0.577_350_26_f32;
pub const TWO_OVER_SQRT_3: f32 = 2.0 * ONE_OVER_SQRT3;
pub const TWO_OVER_THREE: f32 = 2.0 / 3.0;
pub const ONE_OVER_THREE: f32 = 1.0 / 3.0;
pub fn balanced_clarke_transformation(
    u: ElectricCurrent,
    v: ElectricCurrent,
    _w: ElectricCurrent,
) -> (ElectricCurrent, ElectricCurrent) {
    (u, ONE_OVER_SQRT3 * u + TWO_OVER_SQRT_3 * v)
}

#[allow(unused)]
pub fn full_clarke_transformation(
    u: ElectricCurrent,
    v: ElectricCurrent,
    w: ElectricCurrent,
) -> (ElectricCurrent, ElectricCurrent) {
    (
        TWO_OVER_THREE * u - ONE_OVER_THREE * v - ONE_OVER_THREE * w,
        ONE_OVER_SQRT3 * v - ONE_OVER_SQRT3 * w,
    )
}
