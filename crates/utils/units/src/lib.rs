#![no_std]

extern crate uom;

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
pub use uom::fmt::DisplayStyle;
use uom::num_traits::float::FloatCore;
pub use uom::si;
use uom::si::electric_current::ampere;
use uom::si::electric_potential::volt;
pub use uom::si::f32::AngularVelocity;
pub use uom::si::f32::Ratio;
pub use uom::si::f32::*;
use uom::si::ratio::ratio;
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
impl_atomic_unit_type!(Ratio, ratio);
impl_atomic_unit_type!(ThermodynamicTemperature, kelvin);

pub struct AtomicUnit<T: F32UnitType> {
    value: AtomicU32,
    _marker: PhantomData<T>,
}

impl<T: F32UnitType> AtomicUnit<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: AtomicU32::new(value.into_f32().to_bits()),
            _marker: PhantomData,
        }
    }

    pub const fn zero() -> Self {
        Self {
            value: AtomicU32::new(0.0f32.to_bits()),
            _marker: PhantomData,
        }
    }

    pub fn store(&self, value: T, ordering: Ordering) {
        self.value.store(value.into_f32().to_bits(), ordering);
    }

    pub fn load(&self, ordering: Ordering) -> T {
        T::from_f32(f32::from_bits(self.value.load(ordering)))
    }
}

pub trait IntoRawDutyCycle {
    fn into_raw_duty_cycle(self, max: u32) -> u32;
}

impl IntoRawDutyCycle for DutyCycle {
    #[inline(always)]
    fn into_raw_duty_cycle(self, max: u32) -> u32 {
        let max_f = max as f32;
        FloatCore::round((self.value * max_f).clamp(0f32, max_f)) as u32
    }
}

#[cfg(test)]
mod test {
    use crate::{AtomicUnit, F32UnitType};
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

    #[test]
    fn atomic_unit_preserves_float_bits() {
        let p = ElectricPotential::new::<volt>(-1.25);
        let a = AtomicUnit::new(p);

        assert_eq!(
            a.load(Ordering::Relaxed).into_f32().to_bits(),
            (-1.25f32).to_bits()
        );
    }
}
