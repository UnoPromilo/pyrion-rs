#![no_std]

extern crate uom;

use core::marker::PhantomData;
use core::sync::atomic::Ordering;
use portable_atomic::AtomicF32;
pub use uom::fmt::DisplayStyle;
pub use uom::si;
use uom::si::electric_current::ampere;
use uom::si::electric_potential::volt;
pub use uom::si::f32::AngularVelocity;
pub use uom::si::f32::Ratio;
pub use uom::si::f32::*;
use uom::si::thermodynamic_temperature::kelvin;

pub type DutyCycle = Ratio;

pub trait F32UnitType {
    fn from_f32(value: f32) -> Self;
    fn into_f32(self) -> f32;
}

macro_rules! impl_atomic_unit_type {
    ($ty:ty, $unit:ty) => {
        impl F32UnitType for $ty {
            fn from_f32(value: f32) -> Self {
                Self::new::<$unit>(value)
            }

            fn into_f32(self) -> f32 {
                self.value
            }
        }
    };
}

impl_atomic_unit_type!(ElectricPotential, volt);
impl_atomic_unit_type!(ElectricCurrent, ampere);
impl_atomic_unit_type!(ThermodynamicTemperature, kelvin);

pub struct AtomicUnit<T: F32UnitType> {
    value: AtomicF32,
    _marker: PhantomData<T>,
}

impl<T: F32UnitType> AtomicUnit<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: AtomicF32::new(value.into_f32()),
            _marker: PhantomData,
        }
    }

    pub const fn zero() -> Self {
        Self {
            value: AtomicF32::new(0.0),
            _marker: PhantomData,
        }
    }

    pub fn store(&self, value: T, ordering: Ordering) {
        self.value.store(value.into_f32(), ordering);
    }

    pub fn load(&self, ordering: Ordering) -> T {
        T::from_f32(self.value.load(ordering))
    }
}

#[cfg(test)]
mod test {
    use crate::AtomicUnit;
    use core::sync::atomic::Ordering;
    use uom::si::electric_potential::volt;
    use uom::si::f32::ElectricPotential;

    #[test]
    fn test_atomic_unit() {
        let p = ElectricPotential::new::<volt>(1.0);
        let a = AtomicUnit::zero();
        a.store(p, Ordering::Relaxed);
        let p2 = a.load(Ordering::Relaxed);
        assert_eq!(p, p2);
    }
}
