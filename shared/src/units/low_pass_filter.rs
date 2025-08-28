use fixed::types::{I32F32, U1F15};

pub trait LowPassFilter<T> {
    fn low_pass_filter(&self, value: T, alpha: U1F15) -> T;
}

impl LowPassFilter<I32F32> for I32F32 {
    fn low_pass_filter(&self, value: I32F32, alpha: U1F15) -> I32F32 {
        let alpha = I32F32::from_num(alpha.to_num::<I32F32>());
        let change = (self - value) * alpha;
        self - change
    }
}
