# icue-catalogue

Builds [docs/corsair-devices.md](../../docs/corsair-devices.md) from iCUE 5's device packages.
Corsair's files stay on your machine; only facts (names, USB IDs, protocol and LED geometry) end up in the repo.

Run the scripts with `python3 -I` from outside the download directory: the packages are untrusted input.

```bash
# 1. list and download every dev-* package named in an installed iCUE's registry (~570 MB)
python3 -I fetch_devices.py "/path/to/Corsair iCUE5 Software/cuepkg.registry" ~/icue-packages
# 2. read auxiliary.json + the device manifest of each package into JSON
python3 -I catalogue.py ~/icue-packages ../../docs/corsair-devices.json
# 3. render the Markdown page
python3 -I render.py ../../docs/corsair-devices.json ../../docs/corsair-devices.md
```

`rcc_extract.py BUNDLE OUTDIR` unpacks a single Qt `.rcc` resource bundle, for a closer look at one manifest.
