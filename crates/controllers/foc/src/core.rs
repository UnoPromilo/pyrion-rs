use crate::clarke_transformation::balanced_clarke_transformation;
use crate::park_transformation::{inverse_park_transformation, park_transformation};
use crate::snapshot::{FocInput, FocOutput};
use crate::space_vector_modulation::alternate_reverse_space_vector_modulation;
use pid::pi::PiController;
use units::{ElectricCurrent, ElectricPotential};

// TODO make sure the alpha beta to V_Bus / sqrt(3)
pub fn foc_step(
    input: FocInput,
    iq_pi: &mut PiController<ElectricCurrent, ElectricPotential>,
    id_pi: &mut PiController<ElectricCurrent, ElectricPotential>,
) -> FocOutput {
    let (alpha, beta) = balanced_clarke_transformation(input.u, input.v, input.w);

    let (d, q) = park_transformation(alpha, beta, input.angle.sin, input.angle.cos);
    let d_error = input.d_requested - d;
    let q_error = input.q_requested - q;

    let d_ref = id_pi.step(d_error);
    let q_ref = iq_pi.step(q_error);

    let (alpha, beta) = inverse_park_transformation(d_ref, q_ref, input.angle.sin, input.angle.cos);
    let (u, v, w) = alternate_reverse_space_vector_modulation(alpha, beta, input.v_bus);
    FocOutput { u, v, w }
}
