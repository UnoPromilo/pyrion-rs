use core::marker::PhantomData;
use units::{F32UnitType, Ratio};

pub struct UnitlessPiController {
    kp: f32,
    ki: f32,

    integrator: f32,
    integrator_max: f32,
    integrator_min: f32,

    output_max: f32,
    output_min: f32,
}

impl UnitlessPiController {
    pub fn new(
        kp: f32,
        ki: f32,
        integrator_max: f32,
        integrator_min: f32,
        output_max: f32,
        output_min: f32,
    ) -> Self {
        Self {
            kp,
            ki,
            integrator: 0.0,
            integrator_max,
            integrator_min,
            output_max,
            output_min,
        }
    }

    pub fn step(&mut self, error: f32) -> f32 {
        let p = self.kp * error;

        self.integrator += self.ki * error;
        self.integrator = self
            .integrator
            .clamp(self.integrator_min, self.integrator_max);

        let output = p + self.integrator;
        output.clamp(self.output_min, self.output_max)
    }
}

pub struct PiController<TIn: F32UnitType, TOut: F32UnitType> {
    internal: UnitlessPiController,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn: F32UnitType, TOut: F32UnitType> PiController<TIn, TOut> {
    pub fn new(
        kp: Ratio,
        ki: Ratio,
        integrator_max: f32,
        integrator_min: f32,
        output_max: TOut,
        output_min: TOut,
    ) -> Self {
        let internal = UnitlessPiController::new(
            kp.value,
            ki.value,
            integrator_max,
            integrator_min,
            output_max.into_f32(),
            output_min.into_f32(),
        );
        Self {
            internal,
            _phantom: PhantomData,
        }
    }

    pub fn step(&mut self, error: TIn) -> TOut {
        TOut::from_f32(self.internal.step(error.into_f32()))
    }
}
