#[allow(unused_imports)]
use hdf5::{File, H5Type};
use hdf5_metno::{self as hdf5, Dataset};
use ndarray::{self, Array1, NdFloat};
use std::{sync::Arc};
use std::error::Error;
use num_traits;
use std::fmt;

#[derive(Debug)]
pub enum AnalysisResult {
    Stats(Vec<(String,String)>),
    NotAvailable,
    Failed(String),
}

#[derive(Debug)]
struct DataAnalysisError {
    pub msg: String
}

impl fmt::Display for DataAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataAnalysisError: {}", self.msg)
    }
}
impl Error for DataAnalysisError {}



fn analysis_1d<T>(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error> >
where 
    T : H5Type + num_traits::FromPrimitive +  num_traits::Zero + Clone + std::ops::Div<T, Output = T> + ToString,
{
    let mut info: Vec<(String,String)> = Vec::new();
        
    let v: Array1<T> = d.read_1d()?;
    info.push(("mean".to_owned(), v.mean().ok_or(DataAnalysisError{msg: "problem with mean".into()})?.to_string() ));

    Ok(AnalysisResult::Stats(info))
}

pub fn hdf5_dataset_analysis(d: Arc<Dataset>) -> Result<AnalysisResult, Box<dyn Error> > {
    if d.ndim() == 1 {
        if d.size() == 0 {
            Ok(AnalysisResult::NotAvailable)
        }else if d.dtype()?.is::<f32>() {
            analysis_1d::<f32>(d)
        }else if d.dtype()?.is::<i32>() {
            analysis_1d::<i32>(d)
        } else {   
            Ok(AnalysisResult::NotAvailable)
        }

    } else {
        Ok(AnalysisResult::NotAvailable)
    }

}