use units::{ElectricCurrent, ElectricPotential};

pub fn park_transformation(
    alpha: ElectricCurrent,
    beta: ElectricCurrent,
    angle_sin: f32,
    angle_cos: f32,
) -> (ElectricCurrent, ElectricCurrent) {
    (
        alpha * angle_cos + beta * angle_sin,
        -alpha * angle_sin + beta * angle_cos,
    )
}

pub fn inverse_park_transformation(
    d: ElectricPotential,
    q: ElectricPotential,
    angle_sin: f32,
    angle_cos: f32,
) -> (ElectricPotential, ElectricPotential) {
    (d * angle_cos - q * angle_sin, d * angle_sin + q * angle_cos)
}
