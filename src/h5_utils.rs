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
