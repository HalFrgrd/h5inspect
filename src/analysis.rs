#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use ndarray::{self, Array1};
use ndarray_stats::histogram::strategies::{BinsBuildingStrategy, FreedmanDiaconis};
use ndarray_stats::histogram::{Grid, Histogram};
use noisy_float::prelude::*;
use num_traits::{self, ToPrimitive};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

// use std::{thread, time};

pub type HistogramData = Vec<(f32, f32)>;

#[derive(Debug)]
pub enum AnalysisResult {
    Stats(Vec<(String, String)>, HistogramData),
    NotAvailable,
    Failed(String),
}

#[derive(Debug)]
struct DataAnalysisError {
    pub msg: String,
}

impl fmt::Display for DataAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataAnalysisError: {}", self.msg)
    }
}
impl Error for DataAnalysisError {}

fn compute_histogram(d: &Array1<f64>) -> Result<HistogramData, Box<dyn Error>> {
    // Convert Array1<f64> to Array1<N64>
    let data: Array1<N64> = d.mapv(|x| n64(x));
    // let observations: Array2<N64> = data.to_shape((data.len(), 1))?.to_owned();

    let bins = FreedmanDiaconis::<N64>::from_array(&data)?.build();
    let grid = Grid::from(vec![bins.clone()]);

    // let hist = data.histogram(grid);
    let mut hist = Histogram::new(grid);
    for x in data {
        hist.add_observation(&Array1::from_vec(vec![x]))?;
    }

    let counts = hist.counts();

    // Convert to Vec<(bin_center, count)> as f32
    let mut result = Vec::new();
    for i in 0..counts.len() {
        // log::debug!("{:?}", grid.index(&[i]).get(0));

        let bin = bins.index(i).start;
        let count = counts[i] as f32;
        result.push((bin.raw() as f32, count)); //todo
    }

    Ok(result)
}

fn analysis_1d<T>(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error>>
where
    T: H5Type
        + num_traits::FromPrimitive
        + num_traits::Zero
        + Clone
        + std::ops::Div<T, Output = T>
        + ToString
        + std::fmt::Debug
        + std::fmt::Display
        + ToPrimitive,
{
    let mut info: Vec<(String, String)> = Vec::new();

    let v: Array1<T> = d.read_1d()?;

    info.push(("Data".to_owned(), format!("{}", v)));

    info.push((
        "mean".to_owned(),
        v.mean()
            .ok_or(DataAnalysisError {
                msg: "problem with mean".into(),
            })?
            .to_string(),
    ));

    let arr_f64: Array1<f64> = v.mapv(|x| x.to_f64().unwrap_or(0.0));
    info.push(("std".to_owned(), arr_f64.std(1.).to_string()));

    let hist = compute_histogram(&arr_f64)?;

    Ok(AnalysisResult::Stats(info, hist))
}

pub fn hdf5_dataset_analysis(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error>> {
    if d.ndim() != 1 || d.size() == 0 {
        return Ok(AnalysisResult::NotAvailable);
    }

    // thread::sleep(time::Duration::from_secs(5));

    let dtype = d.dtype()?;
    if dtype.is::<f32>() {
        analysis_1d::<f32>(d)
    } else if dtype.is::<i32>() {
        analysis_1d::<i32>(d)
    } else {
        Ok(AnalysisResult::NotAvailable)
    }
}
