// pub fn read_hdf5(file_path: &PathBuf) -> Result<()> {
//     let file = File::open(file_path)?; // open for reading
//     let ds = file.dataset("random_data")?; // open the dataset
//     let asd = ds.attr_names()?;
//     dbg!(asd);
//     Ok(())
// }
use hdf5_metno as hdf5;

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

pub fn get_text_for_dataset(dataset: &hdf5::Dataset) -> String {
    let shape = dataset.shape();
    let datatype = dataset.dtype();
    let space = dataset.space();
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

    format!(
        "Dataset info:\nPath: {}\nShape: {:?}\nDatatype: {:?}\nSpace: {:?}\nStorage: {}\nCompression: {}\nStorage size: {} bytes\nData size: {} bytes\nCompression ratio: {:.2}",
        dataset.name(), shape, datatype, space, chunk_info, compression_info, storage_size, data_size, compression_ratio
    )
}

pub fn get_text_for_group(group: &hdf5::Group) -> String {
    let num_groups = group.groups().unwrap_or(vec![]).len();
    let num_datasets = group.datasets().unwrap_or(vec![]).len();
    let attrs = group.attr_names().unwrap_or(vec![]);
    let num_attrs = attrs.len();

    format!(
        "Group info:\nPath: {}\nNumber of groups: {}\nNumber of datasets: {}\nNumber of attributes: {}\nAttribute names: {:?}",
        group.name(),
        num_groups,
        num_datasets,
        num_attrs,
        attrs
    )
}
