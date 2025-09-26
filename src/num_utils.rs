use std::fmt::Display;

use num_traits::ToPrimitive;

pub trait Summable: Sized {
    type AccumulatorType: std::ops::Add<Output = Self::AccumulatorType>
        + num_traits::Zero
        + Copy
        + Display
        + From<Self>
        + ToPrimitive;
}

macro_rules! impl_summable_for_numbers {
    ($($t:ty),*; $accum_type:ty) => {
        $(
            impl Summable for $t {
                type AccumulatorType = $accum_type; // Now uses the provided AccumulatorType
            }
        )*
    };
}

// Implementing Summable for all signed integers, unsigned integers, and floating points
impl_summable_for_numbers!(i8, i16, i32, i64; i64);
impl_summable_for_numbers!(u8, u16, u32, u64; u64);
impl_summable_for_numbers!(f32, f64; f64);

pub trait IsNan {
    fn my_is_nan(&self) -> bool;
}

macro_rules! impl_isnan_for_floats {
    ($($t:ty),*) => {
        $(
            impl IsNan for $t {
                fn my_is_nan(&self) -> bool {
                    self.is_nan()
                }
            }
        )*
    };
}

impl_isnan_for_floats!(f32, f64);

macro_rules! impl_isnan_for_ints {

    ($($t:ty),*) => {
        $(
            impl IsNan for $t {
                fn my_is_nan(&self) -> bool {
                    false
                }
            }
        )*
    };
}

impl_isnan_for_ints!(i8, i16, i32, i64, u8, u16, u32, u64);
