# Coreglow

[![CI](https://github.com/Nexus-SvK/Coreglow/actions/workflows/ci.yml/badge.svg)](https://github.com/Nexus-SvK/Coreglow/actions/workflows/ci.yml)

Lighting effects, profiles and fan control for the **Corsair iCUE Commander Core XT** on Linux, without iCUE.
The **Commander CORE** (the hub of the ELITE CAPELLIX and ELITE LCD coolers) is supported experimentally.

![Coreglow home screen](docs/screenshots/home.png)

| Lighting | Fans | Profiles |
|---|---|---|
| ![Lighting](docs/screenshots/lighting.png) | ![Fans](docs/screenshots/fans.png) | ![Profiles](docs/screenshots/profiles.png) |

- `coreglowd`: user service that drives the controller (lighting at ~30 fps, fan duties for fans 1 and 2).
  When it stops, it stores a backup curve in the controller (fans 1 and 2 follow probe 1, pump at 100 %);
  the curve is editable on the Fans page or as `[fallback]` in the config.
- `coreglowctl`: command line: `status`, `list`, `set NAME`, `brightness NAME N`, `auto [on|off]`, `motherboard [on|off]`, `night [off|FROM-TO [N]]`, `closed [keep|off|RRGGBB|EFFECT [slow|medium|fast]]`, `devices`, `test-fans`, `edit`, `fallback`, `gui`.
- Night dimming: all lighting at a lower brightness between two times (Lighting page, or `[night]` in the config).
- Lighting while Coreglow isn't running (at start-up, after shutdown): off, one colour, or one of 13 built-in
  effects (rainbow, waves, colour cycle, visor…) at slow, medium or fast speed, stored in the controller itself
  (Lighting page, `coreglowctl closed`, or `closed_lighting` in the config; Core XT only).
- `coreglow`: window ("Coreglow" in the app menu) for profiles, effects, per-part lighting, fan curves,
  automatic profile switching (by temperature, running program, time of day or locked screen) and what's connected
  to each port (layout editor).
- `coreglow-tray`: tray icon (starts at login) for switching profiles, automatic switching on/off and testing fans.
- Optional motherboard sync through OpenRGB's server, so effects span the whole PC.
- English and Slovak interface (from the system language; `COREGLOW_LANG=en|sk` overrides it). Man pages for all commands.

Profiles live in `~/.config/coreglow/config.toml` (created from `/usr/share/coreglow/config.toml`,
or copied from `~/.config/ccxt/` if you used ccxt-control 0.1,
on first run) and apply within a second of saving.

## Install

From the apt repository, so updates come with the system's other updates:

    sudo curl -fsSLo /usr/share/keyrings/coreglow-archive-keyring.gpg https://nexus-svk.github.io/Coreglow/coreglow-archive-keyring.gpg
    echo "deb [signed-by=/usr/share/keyrings/coreglow-archive-keyring.gpg] https://nexus-svk.github.io/Coreglow stable main" | sudo tee /etc/apt/sources.list.d/coreglow.list
    sudo apt update && sudo apt install coreglow
    systemctl --user start coreglowd

Or download the `.deb` from the [latest release](https://github.com/Nexus-SvK/Coreglow/releases/latest) and
`sudo apt install ./coreglow_*_amd64.deb`.

## Build from source

    sudo apt install rustc cargo build-essential pkg-config   # Rust 1.85 or newer (else use rustup)
    packaging/build-deb.sh
    sudo apt install ./target/debian/coreglow_*_amd64.deb
    systemctl --user start coreglowd

## Other builds (experimental)

Without a `[layout]` table Coreglow uses this build's layout. Another build can describe its RGB ports in the
window (Lighting → What's connected → Change what's connected…) or in `~/.config/coreglow/config.toml`;
`coreglowctl devices` shows what the controller detects:

```toml
[layout]
external = ["rgb-strip", "rgb-strip"]       # chain order; LED strips can't be mixed with fans on one port
port1 = ["ql-fan", "ql-fan", "ql-fan"]
port2 = ["pump-cap"]
names = { fans = "port1", pump = "port2", strips = "external" }
```

Device ids: rgb-strip, strip-15/21/27/40/49/84, triangle-9, ll-fan, ml-fan, hd-fan, 8-led-fan, ql-fan, pump-cap.
Without `names`, parts are called external, external1…, port1…, port11…. Profiles may only use part names the
layout has; the file is rejected otherwise, so rename profile parts when you change the layout.

## Commander CORE (experimental)

Written from iCUE's device manifest ([docs/corsair-devices.md](docs/corsair-devices.md)) and not yet tried
on the hardware, so please [report](https://github.com/Nexus-SvK/Coreglow/issues) how it works. Differences
from the Core XT:

- USB IDs `1b1c:0c1c` and `1b1c:0c32` (the udev rule covers both).
- Port 0 is the pump: its lighting ring (ELITE CAPELLIX, 29 LEDs; ELITE LCD, 24 LEDs) and its speed.
  The pump speed is set per profile (`pump = 50`…`100`, or the slider on the Fans page; never below 50 %).
- Without a `[layout]` the parts come from what the controller detects: `pump`, `all-fans`, `port1`…`port6`.
  A `[layout]` for it uses `pump = ["capellix-pump"]` or `["lcd-pump-ring"]` and fans on port1…port6.
- Its temperature sensor is the liquid (`liquid`); the backup curve follows it and drives all six fan ports.

## Hardware notes

- On this build's Core XT the pump on fan port 6 runs at full speed whenever the lighting is driven from the PC (software mode).
  iCUE behaves the same way.
- The lighting layout (4 LED strips on the external port, fans on ports 1 and 2, pump head on port 3)
  is fixed in `src/zones.rs` for this build.
- GPU temperature needs `nvidia-smi` (NVIDIA's driver); nouveau doesn't report it.

Scripts can talk to coreglowd through `$XDG_RUNTIME_DIR/coreglowd.sock`, one line per request: `set NAME`,
`test-fans`, `status` (JSON), `watch` (a JSON line per update). For example `echo status | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/coreglowd.sock`.

Protocol notes: Bragi file IDs from iCUE's `BragiProtocols.dll`, framing from USB captures and
liquidctl's `commander_core` driver. Lighting needs 384-byte reports.

## Docs

- [docs/icue-reverse-engineering.md](docs/icue-reverse-engineering.md): protocol facts from iCUE captures and binaries.
- [docs/devlog-and-roadmap.md](docs/devlog-and-roadmap.md): how this was built, mistakes, testing checklist, roadmap.
- [docs/corsair-devices.md](docs/corsair-devices.md): every device iCUE 5 supports, with USB IDs and protocol family.
- [docs/icue-link-hub.md](docs/icue-link-hub.md): groundwork for the iCUE LINK System Hub (sub-devices, what support would take).

## Upgrading from ccxt-control 0.1

Coreglow was called ccxt-control before 0.2.0 (renamed to avoid a clash with the CCXT trading library).
Installing the coreglow package replaces it; then switch the running service over:

    systemctl --user stop ccxtd
    systemctl --user daemon-reload
    systemctl --user start coreglowd

Profiles are copied from `~/.config/ccxt/` on first start; the old folder is left as it was.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

Not affiliated with Corsair. Corsair and iCUE are trademarks of Corsair Memory, Inc.
