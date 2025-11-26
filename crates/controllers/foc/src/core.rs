use crate::clarke_transformation::balanced_clarke_transformation;
use crate::park_transformation::{inverse_park_transformation, park_transformation};
use crate::snapshot::{FocInput, FocOutput};
use crate::space_vector_modulation::alternate_reverse_space_vector_modulation;
use crate::state::FocState;

// TODO make sure the alpha beta to V_Bus / sqrt(3)
pub fn foc_step(input: FocInput, state: &mut FocState) -> FocOutput {
    let (alpha, beta) = balanced_clarke_transformation(input.u, input.v, input.w);

    let (d, q) = park_transformation(alpha, beta, input.angle.sin, input.angle.cos);
    let d_error = state.d_requested - d;
    let q_error = state.q_requested - q;

    let d_ref = state.id_pi.step(d_error);
    let q_ref = state.iq_pi.step(q_error);

    let (alpha, beta) = inverse_park_transformation(d_ref, q_ref, input.angle.sin, input.angle.cos);
    let (u, v, w) = alternate_reverse_space_vector_modulation(alpha, beta, input.v_bus);
    FocOutput { u, v, w }
}
