use pid::pi::PiController;
use units::{ElectricCurrent, ElectricPotential, F32UnitType, Ratio};

pub struct FocState {
    pub d_requested: ElectricCurrent,
    pub q_requested: ElectricCurrent,
    pub iq_pi: PiController<ElectricCurrent, ElectricPotential>,
    pub id_pi: PiController<ElectricCurrent, ElectricPotential>,
}

impl FocState {
    pub fn new(
        kp: Ratio,
        ki: Ratio,
        integrator_max: f32,
        integrator_min: f32,
        output_max: ElectricPotential,
        output_min: ElectricPotential,
    ) -> Self {
        Self {
            d_requested: ElectricCurrent::from_f32(0.0),
            q_requested: ElectricCurrent::from_f32(0.0),
            id_pi: PiController::new(
                kp,
                ki,
                integrator_max,
                integrator_min,
                output_max,
                output_min,
            ),
            iq_pi: PiController::new(
                kp,
                ki,
                integrator_max,
                integrator_min,
                output_max,
                output_min,
            ),
        }
    }
}
