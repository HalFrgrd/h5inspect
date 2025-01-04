//! Create, write, and read a chunked dataset

use hdf5::{File, Result};
use hdf5_metno as hdf5;
use ndarray::Array2;

pub fn generate_dummy_file() -> Result<()> {
    let file = File::create("dummy.h5")?;

    let (ny, nx) = (100, 100);
    let arr = Array2::from_shape_fn((ny, nx), |(j, i)| (1000 * j + i) as f32);

    let ds = file
        .new_dataset::<f32>()
        .chunk((1, ny, nx)) // each chunk contains ny * nx elements
        .shape((1.., ny, nx)) // first axis is unlimited with initial size of 1
        .deflate(3)
        .create("variable")?;

    // writing one chunk at a time is the most efficient
    ds.write_slice(&arr, (0, .., ..))?;

    // dataset can be resized along an unlimited dimension
    ds.resize((10, ny, nx))?;
    ds.write_slice(&arr, (1, .., ..))?;

    let chunksize = ds.chunk().unwrap();
    assert_eq!(chunksize, &[1, ny, nx]);

    let shape = ds.shape();
    assert_eq!(shape, &[10, ny, nx]);

    // it's best to read from a chunked dataset in a chunk-wise fashion
    for k in 0..shape[0] {
        let _arr: Array2<f32> = ds.read_slice((k, .., ..))?;
    }

    let group1 = file.create_group("group1")?;
    let group1_d1 = group1
        .new_dataset::<i32>()
        .shape((ny, nx))
        .create("something")?;
    group1_d1.write(&arr)?;

    let group2 = group1.create_group("group2")?;
    let group2_d1 = group2
        .new_dataset::<i32>()
        .shape((ny, nx))
        .create("qweqwe")?;
    group2_d1.write(&arr)?;

    // create a group with 1000 datasets
    let group3 = file.create_group("group3")?;
    for i in 0..2000 {
        let dataset = group3
            .new_dataset::<i32>()
            .shape((ny, nx))
            .create(format!("dataset_{}", i).as_str())?;
        dataset.write(&arr)?;
    }

    Ok(())
}
