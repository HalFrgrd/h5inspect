#[allow(unused_imports)]
use hdf5::{File, H5Type, Result};
use hdf5_metno as hdf5;
use hdf5_metno::types::FixedUnicode;
use humansize::{format_size, DECIMAL};
use ndarray;
use num_format::{Locale, ToFormattedString};
use std::path::PathBuf;
use tokio;

pub fn basic(_d: ndarray::Array1<f32>) -> String {
    // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    return "asdf".into();
}

pub fn basic_int32(_d: ndarray::Array1<i32>) -> String {
    // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    return "asdf".into();
}
