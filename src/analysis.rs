#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use ndarray::{self, Array1};
// use ndarray_stats::histogram::strategies::{BinsBuildingStrategy, FreedmanDiaconis};
// use ndarray_stats::histogram::{Grid, Histogram};
use crate::num_utils::Summable;
use noisy_float::prelude::*;
use num_traits::{self, ToPrimitive, Zero};
use std::error::Error;
use std::sync::Arc;

pub type HistogramData = Vec<(f32, f32)>;

#[derive(Debug)]
pub enum AnalysisResult {
    Stats(Vec<(String, String)>, HistogramData),
    NotAvailable,
    Failed(String),
}

fn compute_histogram(d: &Array1<f64>) -> Result<HistogramData, Box<dyn Error>> {
    let data: Array1<N64> = d.mapv(|x| n64(x));

    let n_bins = 30; // Number of bins
    let min = data.iter().min().unwrap().clone();
    let max = data.iter().max().unwrap().clone();
    let bin_width = (max - min) / n64(n_bins as f64);

    let mut counts = vec![0; n_bins];
    for &value in data.iter() {
        let bin_index = ((value - min) / bin_width).floor().to_usize().unwrap();
        if bin_index < n_bins {
            counts[bin_index] += 1;
        }
    }

    // Convert to Vec<(bin_center, count)> as f32
    let mut result = Vec::new();
    for i in 0..n_bins {
        let bin_center = min + (bin_width * n64(i as f64)) + (bin_width / n64(2.0));
        let count = counts[i] as f32;
        result.push((bin_center.raw() as f32, count));
    }

    Ok(result)
}

fn analysis_1d<T>(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error>>
where
    T: H5Type + Summable + num_traits::FromPrimitive + Clone + std::fmt::Display + ToPrimitive,
{
    let mut info: Vec<(String, String)> = Vec::new();

    let v: Array1<T> = d.read_1d()?;

    info.push(("Data".to_owned(), format!("{}", v)));

    let sum: T::AccumulatorType = v.iter().fold(T::AccumulatorType::zero(), |acc, x| {
        acc + x.to_owned().into()
    });

    info.push((
        "mean".to_owned(),
        format!(
            "{:.5}",
            (sum.to_f64().unwrap_or(f64::NAN)) / (v.len() as f64)
        ),
    ));

    let arr_f64: Array1<f64> = v.mapv(|x| x.to_f64().unwrap_or(0.0)); // TODO:
    info.push(("std".to_owned(), format!("{:.5}", arr_f64.std(1.))));

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
    } else if dtype.is::<i32>() {
        analysis_1d::<i32>(d)
    } else {
        Ok(AnalysisResult::NotAvailable)
    }
}
