# h5inspect

A terminal UI for inspecting HDF5 files.
HDF5 file are great but current command line tools make it difficult view the file's layout and key information about groups and datasets.
`h5inspect` allows you to easily navigate the file on the command line to understand your data.
Additonally, 1D numeric datasets are plotted as histograms.

![Demo GIF](vhs/demo.gif)


## Features
- Keyboard navigation (arrow keys + vim bindings)
- Mouse support
- Fuzzy search
- Data visualization

## Installation
HDF5 can be built from source and linked statically with the `static` flag.
Without the `static` flag, `hdf5-metno` requires the `HDF5_DIR` environment variable to find an existing installation.
See [hdf5-metno](https://crates.io/crates/hdf5-metno) for more information.
