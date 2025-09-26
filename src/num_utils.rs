use duplicate::duplicate_item;
use num_traits::ToPrimitive;
use std::fmt::Display;

pub trait Summable: Sized {
    type AccumulatorType: std::ops::Add<Output = Self::AccumulatorType>
        + num_traits::Zero
        + Copy
        + Display
        + From<Self>
        + ToPrimitive;
}

#[duplicate_item(
    num_type acc_type;
    [ u8 ]  [u64];
    [ u16 ] [u64];
    [ u32 ] [u64];
    [ u64 ] [u128];
    [ i8 ]  [i64];
    [ i16 ] [i64];
    [ i32 ] [i64];
    [ i64 ] [i128];
    [ f32 ] [f64];
    [ f64 ] [f64];
)]
impl Summable for num_type {
    type AccumulatorType = acc_type;
}

pub trait IsNan {
    fn my_is_nan(&self) -> bool;
}

#[duplicate_item(
    num_type is_nan_impl;
    [u8] [false];
    [u16] [false];
    [u32] [false];
    [u64] [false];
    [i8] [false];
    [i16] [false];
    [i32] [false];
    [i64] [false];
    [f32] [self.is_nan()];
    [f64] [self.is_nan()];
)]
impl IsNan for num_type {
    fn my_is_nan(&self) -> bool {
        is_nan_impl
    }
}
