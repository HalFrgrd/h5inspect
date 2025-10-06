use duplicate::duplicate_item;
use num_traits::ToPrimitive;
use numfmt::{Formatter, Precision, Scales};
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
    [ bool ]  [u64];
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
    [bool] [false];
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

pub trait MyToPrimitive {
    fn my_to_f64(&self) -> Option<f64>;
}

#[duplicate_item(
    num_type to_f64_impl;
    [bool] [if *self { Some(1.0) } else { Some(0.0) }];
    [u8] [Some((*self).to_f64().unwrap())];
    [u16] [Some((*self).to_f64().unwrap())];
    [u32] [Some((*self).to_f64().unwrap())];
    [u64] [Some((*self).to_f64().unwrap())];
    [i8] [Some((*self).to_f64().unwrap())];
    [i16] [Some((*self).to_f64().unwrap())];
    [i32] [Some((*self).to_f64().unwrap())];
    [i64] [Some((*self).to_f64().unwrap())];
    [f32] [Some((*self).to_f64().unwrap())];
    [f64] [Some((*self).to_f64().unwrap())];
)]
impl MyToPrimitive for num_type {
    fn my_to_f64(&self) -> Option<f64> {
        to_f64_impl
    }
}

pub fn format_integer_with_underscore(num: u64) -> String {
    let num_str = num.to_string();
    let mut formatted = String::new();
    let len = num_str.len();

    for (i, c) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted.push('_');
        }
        formatted.push(c);
    }

    formatted
}

pub fn file_size_fmt_no_scale(size: u64) -> String {
    format_integer_with_underscore(size) + " B"
}

pub fn file_size_fmt(size: u64) -> String {
    if size < 1024 {
        return format!("{} B", size);
    }

    let mut f: Formatter = Formatter::new()
        .scales(Scales::metric())
        .precision(Precision::Decimals(3))
        .suffix("B")
        .unwrap();

    f.fmt2(size).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_fmt() {
        assert_eq!(file_size_fmt(1u64), "1 B");
        assert_eq!(file_size_fmt(999u64), "999 B");
        assert_eq!(file_size_fmt(1024u64), "1.024 kB");
        assert_eq!(file_size_fmt(123123123123), "123.123 GB");
        assert_eq!(file_size_fmt(5123123123123), "5.123 TB");
        assert_eq!(file_size_fmt_no_scale(123123123123), "123_123_123_123 B");
    }
}
