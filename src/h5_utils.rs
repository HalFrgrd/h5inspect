use hdf5::{File, Result};
use std::path::PathBuf;

// pub fn read_hdf5(file_path: &PathBuf) -> Result<()> {
//     let file = File::open(file_path)?; // open for reading
//     let ds = file.dataset("random_data")?; // open the dataset
//     let asd = ds.attr_names()?;
//     dbg!(asd);
//     Ok(())
// }
