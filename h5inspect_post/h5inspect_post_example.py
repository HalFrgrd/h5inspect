#!/usr/bin/env python3
import sys
import h5py
from IPython import embed

def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <file> <dataset_or_group_path>")
        sys.exit(1)

    filename, path = sys.argv[1], sys.argv[2]

    try:
        with h5py.File(filename, "r") as f:
            if path not in f:
                print(f"Path '{path}' not found in file.")
                sys.exit(1)

            h5_obj = f[path]
            print(f"Opened '{path}' from '{filename}'")
            if isinstance(h5_obj, h5py.Dataset):
                print(f"Shape: {h5_obj.shape}, Dtype: {h5_obj.dtype}")
            elif isinstance(h5_obj, h5py.Group):
                print(f"Group contains {len(h5_obj)} items")

            # Drop into IPython shell with useful locals
            embed(header="Interactive HDF5 session.\nAvailable vars: f (file), h5_obj (dataset or group)")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

