# h5inspect_post
When you press `i` in h5inspect on a selected dataset or group, the app will quit and launch `$H5INSPECT_POST [file] [dataset_or_group]`.
An example script is provided:

```python
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

3. Use h5inspect and press `i` on a dataset or group to launch an external script with the selected path.
