#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use ndarray::{self, Array1, Array2, ArrayD, ArrayView1};
use ndarray_stats::histogram::{strategies::FreedmanDiaconis, Edges, GridBuilder, Histogram};
use ndarray_stats::{histogram, interpolate, HistogramExt, QuantileExt};
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

// fn freedman_diaconis_bin_edges(data: &Array1<f64>) -> Edges<N64> {
//     let n = data.len() as f64;

//     // TODO: this needs mut?
//     let q25 = data.clone().quantile_axis_skipnan_mut(ndarray::Axis(0), n64(0.25), &interpolate::Linear).unwrap().into_scalar();
//     let q75 = data.clone().quantile_axis_skipnan_mut(ndarray::Axis(0), n64(0.75), &interpolate::Linear).unwrap().into_scalar();
//     let iqr = q75 - q25;

//     // Freedman-Diaconis bin width
//     let bin_width = if iqr == 0.0 {1.0} else { 2.0 * iqr / (n  as f64).cbrt() }

//     let min = data.min().unwrap().to_owned();
//     let max = data.max().unwrap().to_owned();
//     let num_bins = ((max - min) / bin_width).ceil() as usize;

//     // Generate bin edges
//     let edges = Array1::linspace(min, max, num_bins + 1);
//     Edges::from(edges.mapv(|x| n64(x)))
// }

fn compute_histogram(d: &Array1<f64>) -> Result<HistogramData, Box<dyn Error>> {
    // Convert Array1<f64> to Array1<N64>
    let data: Array1<N64> = d.mapv(|x| n64(x));
    let observations: Array2<N64> = data.to_shape((data.len(), 1))?.to_owned();

    // log::debug!("{}", asdf.to_string());
    // // Build Freedman-Diaconis grid

    // let observations = Array2::from_shape_vec(
    //     (12, 1),
    //     vec![1.0, 4., 5., 2., 100., 20., 50., 65., 27., 40., 45., 23.],
    // ).unwrap();
    // let observations = observations.mapv(|x| n64(x));
    let grid = GridBuilder::<FreedmanDiaconis<N64>>::from_array(&observations)?.build();
    // log::debug!("{:?}", grid);

    // // Create histogram
    // let mut hist = Histogram::new(grid);
    // log::debug!("{:?}", hist.ndim());

    // // Add observations
    // for x in observations {

    //     hist.add_observation(x).unwrap();
    // }
    let hist = observations.histogram(grid.clone());

    // Get counts and bin edges
    let counts = hist.counts(); // Array1<usize> if 1D
                                // let edges = grid.();

    // log::debug!("{:?}", hist.counts());
    // log::debug!("{:?}", grid);

    // Convert to Vec<(bin_center, count)> as f32
    let mut result = Vec::new();
    for i in 0..counts.len() {
        // log::debug!("{:?}", grid.index(&[i]).get(0));

        let bin = grid.index(&[i]).get(0).unwrap().start.clone();
        let count = counts[i] as f32;
        result.push((bin.raw() as f32, count)); //todo
    }

    // log::debug!("{:?}", result);

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

    // info.push(("histogram".to_owned(), hist.to_string()));

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
