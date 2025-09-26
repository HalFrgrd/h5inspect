#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use ndarray::{self, Array1};

use crate::num_utils::{IsNan, Summable};
use core::f64;
use num_traits::{self, ToPrimitive, Zero};
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

pub type HistogramData = Vec<(f32, f32)>;

#[derive(Debug)]
pub enum AnalysisResult {
    Stats(Vec<(String, String)>, HistogramData),
    NotAvailable,
    Failed(String),
}

fn compute_histogram(d: &Array1<f64>) -> Result<HistogramData, Box<dyn Error>> {
    let n_bins = 30; // Number of bins
    let min = d.iter().fold(f64::INFINITY, |acc, &x| f64::min(acc, x));
    let max = d.iter().fold(f64::NEG_INFINITY, |acc, &x| f64::max(acc, x));
    let bin_width = (max - min) / n_bins as f64;

    if !min.is_finite() || !max.is_finite() || bin_width <= 0.0 {
        return Err(format!(
            "Problem with histogram gen: min = {}, max = {}, bin_width = {}",
            min, max, bin_width
        )
        .into());
    }

    let mut counts = vec![0; n_bins];
    for &value in d.iter() {
        if !value.is_nan() {
            let bin_index = ((value - min) / bin_width).floor() as usize;
            if bin_index < n_bins {
                counts[bin_index] += 1;
            }
        }
    }

    // Convert to Vec<(bin_center, count)> as f32
    let mut result = Vec::new();
    for i in 0..n_bins {
        let bin_center = min + (bin_width * (i as f64)) + (bin_width / 2.0);
        let count = counts[i] as f32;
        result.push((bin_center as f32, count));
    }

    Ok(result)
}

fn analysis_1d<T>(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error>>
where
    T: H5Type + Summable + IsNan + num_traits::FromPrimitive + Clone + Display + ToPrimitive,
{
    let mut info: Vec<(String, String)> = Vec::new();

    let v: Array1<T> = d.read_1d()?;

    info.push(("Data".to_owned(), format!("{}", v)));

    let sum: T::AccumulatorType = v.iter().fold(T::AccumulatorType::zero(), |acc, x| {
        acc + x.to_owned().into()
    });

    info.push((
        "Mean".to_owned(),
        format!(
            "{:.5}",
            (sum.to_f64().unwrap_or(f64::NAN)) / (v.len() as f64)
        ),
    ));

    info.push((
        "NaN count".to_owned(),
        v.mapv(|x| x.my_is_nan() as u32).sum().to_string(),
    ));

    let arr_f64: Array1<f64> = v.mapv(|x| x.to_f64().unwrap_or(f64::NAN));
    info.push(("Std".to_owned(), format!("{:.5}", arr_f64.std(1.))));

    let hist = compute_histogram(&arr_f64)?;

    Ok(AnalysisResult::Stats(info, hist))
}

pub fn hdf5_dataset_analysis(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error>> {
    if d.ndim() != 1 || d.size() == 0 {
        return Ok(AnalysisResult::NotAvailable);
    }

    let dtype = d.dtype()?;
    if dtype.is::<f32>() {
        analysis_1d::<f32>(d)
    } else if dtype.is::<f64>() {
        analysis_1d::<f64>(d)
    } else if dtype.is::<i8>() {
        analysis_1d::<i8>(d)
    } else if dtype.is::<u8>() {
        analysis_1d::<u8>(d)
    } else if dtype.is::<i16>() {
        analysis_1d::<i16>(d)
    } else if dtype.is::<u16>() {
        analysis_1d::<u16>(d)
    } else if dtype.is::<i32>() {
        analysis_1d::<i32>(d)
    } else if dtype.is::<u32>() {
        analysis_1d::<u32>(d)
    } else if dtype.is::<i64>() {
        analysis_1d::<i64>(d)
    } else if dtype.is::<u64>() {
        analysis_1d::<u64>(d)
    } else {
        Ok(AnalysisResult::NotAvailable)
    }
}
