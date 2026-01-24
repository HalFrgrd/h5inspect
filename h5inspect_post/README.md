# h5inspect_post
When you press `i` in h5inspect, the app will quit and launch `$H5INSPECT_POST [file] [dataset]`.
An example script is provided:

```python
#!/usr/bin/env python3
import sys
import h5py
from IPython import embed

def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <file> <dataset_path>")
        sys.exit(1)

    filename, dataset_path = sys.argv[1], sys.argv[2]

    try:
        with h5py.File(filename, "r") as f:
            if dataset_path not in f:
                print(f"Dataset path '{dataset_path}' not found in file.")
                sys.exit(1)

            dset = f[dataset_path]
            print(f"Opened dataset '{dataset_path}' from '{filename}'")
            print(f"Shape: {dset.shape}, Dtype: {dset.dtype}")

            # Drop into IPython shell with useful locals
            embed(header="Interactive HDF5 session.\nAvailable vars: f (file), dset (dataset)")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## Usage

1. Make the script executable:
   ```bash
   chmod +x h5inspect_post_example.py
   ```

2. Set the environment variable:
   ```bash
   export H5INSPECT_POST=/path/to/h5inspect_post_example.py
   ```

3. Use h5inspect and press `i` on a dataset to launch an interactive IPython session with the dataset loaded.
