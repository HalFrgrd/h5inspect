use hdf5::{File, Result};


pub fn read_hdf5() -> Result<()> {
    let file = File::open("dummy_data.h5")?; // open for reading
    let ds = file.dataset("random_data")?; // open the dataset
    let asd = ds.attr_names()?;
    dbg!(asd);
    Ok(())
}
