use fixed::traits::FixedSigned;

#[derive(Debug)]
pub struct Controller<T> {
    target: T,

    kp: T,
    ki: T,
    kd: T,

    p_limit: Option<T>,
    i_limit: Option<T>,
    d_limit: Option<T>,
    output_limit: Option<T>,

    integral_state: T,
    derive_state: Option<T>,

    zero: T,
}

impl<T: FixedSigned> Default for Controller<T> {
    fn default() -> Self {
        let zero = T::from_num(0);
        let one = T::from_num(1);
        Self {
            target: zero,
            kp: one,
            ki: zero,
            kd: zero,
            p_limit: None,
            i_limit: None,
            d_limit: None,
            output_limit: None,
            integral_state: zero,
            derive_state: None,
            zero,
        }
    }
}

impl<T: FixedSigned> Controller<T> {
    pub fn set_p(mut self, p: T, limit: Option<T>) -> Self {
        self.kp = p;
        self.p_limit = limit;
        self
    }

    pub fn set_i(mut self, i: T, limit: Option<T>) -> Self {
        self.ki = i;
        self.i_limit = limit;
        self
    }

    pub fn set_d(mut self, d: T, limit: Option<T>) -> Self {
        self.kd = d;
        self.d_limit = limit;
        self
    }

    pub fn set_output_limit(mut self, limit: Option<T>) -> Self {
        self.output_limit = limit;
        self
    }

    pub fn update(&mut self, position: T) -> T {
        let error = self.target - position;

        let p_term = Self::apply_limit(error * self.kp, self.p_limit);

        self.integral_state =
            Self::apply_limit(self.integral_state + error * self.ki, self.i_limit);
        let i_term = self.integral_state;

        let d_term = Self::apply_limit(
            match self.derive_state {
                Some(previous_position) => previous_position - position,
                None => self.zero,
            },
            self.d_limit,
        ) * self.kd;
        self.derive_state = Some(position);

        Self::apply_limit(p_term + i_term + d_term, self.output_limit)
    }

    pub fn set_target(&mut self, target: T) {
        self.target = target;
    }

    pub fn reset(&mut self) {
        self.integral_state = self.zero;
        self.derive_state = None;
    }

    #[inline]
    fn apply_limit(value: T, limit: Option<T>) -> T {
        if let Some(limit) = limit {
            debug_assert!(limit >= 0);
            if value > limit {
                limit
            } else if value < -limit {
                -limit
            } else {
                value
            }
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixed::types::I32F32;

    #[test]
    fn proportional_only() {
        let mut c = Controller::<I32F32>::default().set_p(I32F32::from_num(2), None);

        c.set_target(I32F32::from_num(10));
        c.update(I32F32::from_num(8));
        assert_eq!(
            c.zero + I32F32::from_num(4),
            c.zero + c.output_limit.unwrap_or(I32F32::from_num(4))
        );
    }

    #[test]
    fn integral_accumulates() {
        let mut c = Controller::<I32F32>::default()
            .set_p(I32F32::from_num(0), None)
            .set_i(I32F32::from_num(2), None);

        c.set_target(I32F32::from_num(10));
        c.update(I32F32::from_num(8));
        c.update(I32F32::from_num(8));
        let last_result = c.update(I32F32::from_num(8));

        assert_eq!(last_result, I32F32::from_num(12));
    }

    #[test]
    fn derivative_response() {
        let mut c = Controller::<I32F32>::default()
            .set_p(I32F32::from_num(0), None)
            .set_d(I32F32::from_num(2), None);

        c.set_target(I32F32::from_num(10));
        let first_response = c.update(I32F32::from_num(9));

        let second_response = c.update(I32F32::from_num(8.0));

        assert_eq!(first_response, I32F32::from_num(0));
        assert_eq!(second_response, I32F32::from_num(2));
    }

    #[test]
    fn p_limit_applied() {
        let mut c =
            Controller::<I32F32>::default().set_p(I32F32::from_num(10), Some(I32F32::from_num(5)));

        c.set_target(I32F32::from_num(10));
        let output = c.update(I32F32::from_num(0));
        assert_eq!(I32F32::from_num(5), output);
    }

    #[test]
    fn i_limit_applied() {
        let mut c = Controller::<I32F32>::default()
            .set_p(I32F32::from_num(0), None)
            .set_i(I32F32::from_num(10), Some(I32F32::from_num(5)));
        c.set_target(I32F32::from_num(10));
        let output = c.update(I32F32::from_num(0));
        assert_eq!(I32F32::from_num(5.0), output);
    }

    #[test]
    fn output_limit_applied() {
        let mut c = Controller::<I32F32>::default().set_output_limit(Some(I32F32::from_num(3.0)));

        c.set_target(I32F32::from_num(10.0));
        let output = c.update(I32F32::from_num(0.0));
        assert_eq!(I32F32::from_num(3.0), output);
    }

    #[test]
    fn reset_clears_state() {
        let mut c = Controller::<I32F32>::default();

        c.set_target(I32F32::from_num(10.0));
        c.update(I32F32::from_num(5.0));
        c.reset();

        assert_eq!(c.integral_state, I32F32::from_num(0.0));
        assert!(c.derive_state.is_none());
    }

    #[test]
    fn negative_error_handling() {
        let mut c = Controller::<I32F32>::default();

        c.set_target(I32F32::from_num(-5.0));
        c.update(I32F32::from_num(5.0));
        assert!(c.kp * (c.target - I32F32::from_num(5.0)) < I32F32::from_num(0.0));
    }
}
