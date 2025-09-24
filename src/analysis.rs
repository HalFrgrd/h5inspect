#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use hdf5_metno::types::FixedUnicode;
use humansize::{format_size, DECIMAL};
use ndarray::{self, Array1};
use num_format::{Locale, ToFormattedString};
use std::{path::PathBuf, sync::Arc};
use tokio;
use std::error::Error;

#[derive(Debug)]
pub enum AnalysisResult {
    Stats(Vec<(String,String)>),
    NotAvailable,
    Failed,
}

pub fn hdf5_dataset_analysis(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error> > {
    if d.ndim() == 1 && d.dtype()?.is::<f32>() {
        let mut info: Vec<(String,String)> = Vec::new();

        let v: Array1<f32> = d.read_1d()?;
        info.push(("mean".to_owned(), v.mean().unwrap_or(f32::NAN).to_string() ));

        Ok(AnalysisResult::Stats(info))
    } else {
        Ok(AnalysisResult::NotAvailable)
    }

}