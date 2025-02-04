use hdf5::{File, Result};
use hdf5_metno as hdf5;
use hdf5_metno::types::FixedUnicode;
use ndarray::Array2;
use std::path::PathBuf;

// Calling group.name() or dataset.name() was very slow for some reason.
// But group.member_names() was fast.
// So as we iterate through the group we collect the names alongside the objects.
// Not sure exactly why this is way faster than calling name() on each object.
pub fn get_all_of_type(
    group: &hdf5::Group,
    loc_type: hdf5::LocationType,
) -> hdf5::Result<Vec<(String, hdf5::Location)>> {
    group.iter_visit_default(vec![], |group, name, _info, objects| {
        if let Ok(info) = group.loc_info_by_name(name) {
            if info.loc_type == loc_type {
                if let Ok(loc) = group.open_by_token(info.token) {
                    objects.push((name.to_string(), loc));
                    return true; // ok, object extracted and pushed
                }
            } else {
                return true; // ok, object is of another type, skipped
            }
        }
        false // an error occurred somewhere along the way
    })
}

pub fn groups(group: &hdf5::Group) -> hdf5::Result<Vec<(String, hdf5::Group)>> {
    get_all_of_type(group, hdf5::LocationType::Group).map(|vec| {
        vec.into_iter()
            .map(|(name, obj)| (name, obj.as_group().unwrap()))
            .collect()
    })
}

pub fn datasets(group: &hdf5::Group) -> hdf5::Result<Vec<(String, hdf5::Dataset)>> {
    get_all_of_type(group, hdf5::LocationType::Dataset).map(|vec| {
        vec.into_iter()
            .map(|(name, obj)| (name, obj.as_dataset().unwrap()))
            .collect()
    })
}

pub fn get_text_for_dataset(dataset: &hdf5::Dataset) -> Vec<(String, String)> {
    let shape = dataset.shape();
    let datatype = dataset
        .dtype()
        .and_then(|s| s.to_descriptor())
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|_| "unknown".to_string());
    let space = dataset
        .space()
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|_| "unknown".to_string());
    let chunks = dataset.chunk();
    let chunk_info = match chunks {
        Some(chunks) => format!("Chunked ({:?})", chunks),
        None => "Contiguous".to_string(),
    };

    // Get compression info
    let compression = dataset.filters();
    let compression_info = format!("Filter pipeline: {:?}", compression);

    // Get storage size vs data size
    let storage_size = dataset.storage_size();
    let data_size = dataset.size() * dataset.dtype().map_or(0, |dt| dt.size());
    let compression_ratio = if storage_size > 0 {
        data_size as f64 / storage_size as f64
    } else {
        f64::NAN
    };

    let mut res = vec![];

    res.push(("Path".to_string(), dataset.name().to_string()));
    res.push(("Shape".to_string(), format!("{:?}", shape)));
    res.push(("Datatype".to_string(), datatype));
    res.push(("Space".to_string(), space));
    res.push(("Storage".to_string(), chunk_info));
    res.push(("Compression".to_string(), compression_info));
    res.push((
        "Storage size".to_string(),
        format!("{} bytes", storage_size),
    ));
    res.push(("Data size".to_string(), format!("{} bytes", data_size)));
    res.push((
        "Compression ratio".to_string(),
        format!("{:.2}", compression_ratio),
    ));
    res
}

pub fn get_text_for_group(group: &hdf5::Group) -> Vec<(String, String)> {
    let num_groups = group.groups().unwrap_or(vec![]).len();
    let num_datasets = group.datasets().unwrap_or(vec![]).len();
    let attrs = group.attr_names().unwrap_or(vec![]);
    let num_attrs = attrs.len();

    let mut res = vec![];
    res.push(("Path".to_string(), group.name().to_string()));
    res.push(("Number of groups".to_string(), format!("{}", num_groups)));
    res.push((
        "Number of datasets".to_string(),
        format!("{}", num_datasets),
    ));
    res.push(("Number of attributes".to_string(), format!("{}", num_attrs)));
    res.push(("Attribute names".to_string(), format!("{:?}", attrs)));
    res
}

#[allow(dead_code)]
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

    // Create a dataset with variable-length strings
    let dataset: hdf5::Dataset = group1
        .new_dataset::<FixedUnicode<5>>()
        .create("string_dataset")?;

    // Write data to the dataset
    dataset.write_scalar(&unsafe { FixedUnicode::<5>::from_str_unchecked("asdfg") })?;

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

#[allow(dead_code)]
pub fn generate_dummy_split_file() -> Result<()> {
    use hdf5::File;
    use ndarray::Array2;

    let nx = 10;
    let ny = 8;
    let arr: Array2<i32> = Array2::from_shape_fn((ny, nx), |(i, j)| (i * nx + j) as i32);

    // Create a split file - data and metadata stored separately
    let file = File::with_options()
        .with_fapl(|p| p.split_options("-m.h5", "-r.h5"))
        .create("dummy_split.h5")?;

    // Create a dataset
    let ds = file.new_dataset::<i32>().shape((ny, nx)).create("data")?;
    ds.write(&arr)?;

    // Create some groups and datasets
    let group = file.create_group("group1")?;
    let ds2 = group
        .new_dataset::<i32>()
        .shape((ny, nx))
        .create("nested_data")?;
    ds2.write(&arr)?;

    Ok(())
}

pub fn open_file(file_path: &PathBuf) -> Result<hdf5::File> {
    let file = hdf5::File::with_options()
        .with_fapl(|p| p.sec2())
        .open(file_path.clone());

    if file.is_ok() {
        return file;
    }

    // Try with split driver if sec2 fails
    let split_file = hdf5::File::with_options()
        .with_fapl(|p| p.split_options("-m.h5", "-r.h5"))
        .open(file_path.clone());

    if split_file.is_ok() {
        return split_file;
    }

    let mut clean_path = file_path.clone();
    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
        if file_name.ends_with("-m.h5") {
            clean_path.set_file_name(&file_name[..file_name.len() - 5]);
        } else if file_name.ends_with("-r.h5") {
            clean_path.set_file_name(&file_name[..file_name.len() - 5]);
        }
    }

    let split_file = hdf5::File::with_options()
        .with_fapl(|p| p.split_options("-m.h5", "-r.h5"))
        .open(clean_path);

    if split_file.is_ok() {
        return split_file;
    }

    if !file_path.exists() {
        return Err(format!("File path doesn't exist: {file_path:?}").into());
    }
    Err("Couldn't open file".into())
}
