# ccxt-control

Lighting effects, profiles and fan control for the **Corsair iCUE Commander Core XT** on Linux, without iCUE.

- `ccxtd`: user service that drives the controller (lighting at ~30 fps, fan duties for fans 1 and 2).
  When it stops, it stores a fallback curve in the controller (fans 1 and 2 follow probe 1, pump at 100 %).
- `ccxtctl`: command line: `status`, `list`, `set NAME`, `brightness NAME N`, `edit`, `fallback`, `gui`.
- `ccxt-gui`: window ("PC Lighting & Fans" in the app menu) for profiles, effects, per-part lighting and fan curves.

Profiles live in `~/.config/ccxt/config.toml` (created from `/usr/share/ccxt-control/config.toml`
on first run) and apply within a second of saving.

## Build and install

    sudo apt install rustc cargo build-essential pkg-config
    packaging/build-deb.sh
    sudo apt install ./target/debian/ccxt-control_*_amd64.deb
    systemctl --user start ccxtd

## Hardware notes

- The pump on fan port 6 runs at full speed whenever the lighting is driven from the PC (software mode).
  iCUE behaves the same way.
- The lighting layout (4 LED strips on the external port, fans on ports 1 and 2, pump head on port 3)
  is fixed in `src/zones.rs` for this build.
- GPU temperature needs `nvidia-smi` (NVIDIA's driver); nouveau doesn't report it.

Protocol notes: Bragi file IDs from iCUE's `BragiProtocols.dll`, framing from USB captures and
liquidctl's `commander_core` driver. Lighting needs 384-byte reports.

## Docs

- [docs/icue-reverse-engineering.md](docs/icue-reverse-engineering.md): protocol facts from iCUE captures and binaries.
- [docs/devlog-and-roadmap.md](docs/devlog-and-roadmap.md): how this was built, mistakes, testing checklist, roadmap.
