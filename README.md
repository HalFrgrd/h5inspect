# h5inspect

A terminal UI for inspecting HDF5 files.

## Overview
HDF5 files are great but current command line tools make it difficult to view the file's layout and key information about groups and datasets.

h5inspect is a modern, interactive terminal application designed to make exploring HDF5 files intuitive and efficient. Navigate through complex hierarchical data structures with ease, inspect metadata, and visualize datasets all from your command line.

![Demo GIF](vhs/demo.gif)

## Features
- Keyboard navigation (arrow keys + vim bindings)
- Mouse support
- Fuzzy search
- Data visualization
- External analysis scripts can be launched on the selected dataset by setting `H5INSPECT_POST`

## Installation
HDF5 can be built from source and linked statically with the `static` flag.
Without the `static` flag, `hdf5-metno` requires the `HDF5_DIR` environment variable to find an existing installation.
See [hdf5-metno](https://crates.io/crates/hdf5-metno) for more information.
