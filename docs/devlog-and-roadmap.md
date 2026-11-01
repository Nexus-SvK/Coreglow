# Coreglow: development log and roadmap

Companion to [icue-reverse-engineering.md](icue-reverse-engineering.md) (protocol facts).
GUI ideas for the next version are drawn in the design canvas "PC Lighting & Fans — GUI updates"
(https://claude.ai/artifact/48j7oeqQvnBWdVsha9Cy9G, private to the owner).

## The machine this was built for

- Ubuntu 26.04, kernel 7.0, Ryzen 7 7800X3D, Gigabyte B650 AORUS ELITE AX V2, RTX 3090 on nouveau.
- Corsair iCUE Commander Core XT, firmware 1.4.62, on USB bus 1.
  - Fan ports 1 and 2: the two radiator fans (8 LEDs each, on RGB ports 1 and 2).
  - Fan port 6: the AIO pump. Its 13-LED head is on RGB port 3.
  - External RGB port: 4 LED strips, 10 LEDs each.
  - Temperature probes 1 and 2: case air (about 35 °C and 27 °C).
- The motherboard RGB (ITE 5702) is driven by OpenRGB (`openrgb.service`, Corsair detector off).

## How we got here (2026-10-30/31)

1. **iCUE under Wine: rejected.** The installer is a web bootstrapper that needs admin and WinRT. iCUE itself needs
   Windows services and kernel drivers.
2. **OpenRGB + liquidctl.** Both support the controller, but they fight. OpenRGB keeps it in software mode, where
   liquidctl's hardware fan curve is ignored, so the pump and fans stuck at full speed.
3. **Windows VM + USB passthrough + usbmon captures** of iCUE and OpenRGB, then iCUE's `BragiProtocols.dll`
   (RTTI and Qt enum tables) for names. That explained 0x18 (software fan duty) and the full file ID map.
4. **Python daemon (prototype).** It reused liquidctl's framing. It found the four things lighting needs:
   384-byte reports, the stored geometry, `15 01`, and the exact frame length. Then profiles, zones and a browser panel.
5. **Rust rewrite (this repo)**, first as ccxt-control 0.1, renamed Coreglow in 0.2. Direct hidraw, no Python or liquidctl at runtime, an egui window, and a .deb with a
   user service and udev rule. Installed 2026-10-31. The Python prototype is kept in `~/.local/share/ccxt-python-backup`.

## Mistakes worth remembering

| Symptom | Cause | Fix |
|---|---|---|
| Fan 6 stuck at ~3050 rpm | It's the pump; software mode always runs it full | Accepted. Fallback stores 100 % fixed for port 6 |
| Pump at ~760 rpm with OpenRGB stopped | Hardware fallback curve also applied to port 6 | Fixed 100 % for port 6 |
| Left fan, strips and pump partly dark | Copied OpenRGB's geometry (external port off) | Use the stored layout |
| Still dark after the layout change | Layout written but never applied | Send `15 01` after writing 0x1e |
| Still dark | Frame length didn't match the layout (233 vs 69) | Length from the layout |
| Only ~29 LEDs correct | 96-byte reports (liquidctl default) | 384-byte reports |
| Strips "red" in a test | The profile really was red (changed in the panel) | Check the config before suspecting the code |
| `pkill -f` killed its own shell | The pattern matched the command line running it | Use `pkill -x NAME` or a pid file |
| Browser panel said "connection failed" | `localhost` resolves to ::1 first; server bound 127.0.0.1 | Bind `localhost` (both) |

## Code map

| File | Role |
|---|---|
| `src/device.rs` | hidraw discovery (interface 0), Bragi framing, files, software and hardware mode, fallback |
| `src/zones.rs` | `PORT_CONFIG` geometry, LED counts per type, zone → frame indices, labels |
| `src/effects.rs` | effect rendering, temperature colours, fan duty from curves, frame building |
| `src/config.rs` | TOML schema (compatible with the Python version), validation, atomic save |
| `src/sensors.rs` | probes, hwmon (k10temp Tctl, amdgpu edge), nvidia-smi, smoothing |
| `src/status.rs` | `$XDG_RUNTIME_DIR/coreglow-status.json`, written every second by coreglowd |
| `src/bin/coreglowd.rs` | main loop: 30 fps when animated, sensors 1 Hz, reload on file change, reconnect |
| `src/bin/coreglowctl.rs` | CLI |
| `src/bin/coreglow.rs` | egui window, autosave after 400 ms, reads status every second |
| `packaging/` | service, udev rule, desktop entry, icon, maintainer scripts, `build-deb.sh` |

Interfaces between the parts are files only: `config.toml` + `active` (written by the GUI and CLI, watched by coreglowd)
and `coreglow-status.json` (written by coreglowd). No socket or D-Bus yet.

## Testing checklist (until there are hardware tests)

1. `cargo test --release` (layout, default profiles render, curve maths).
2. `systemctl --user stop coreglowd && ./target/release/coreglowd` → "controller at /dev/hidrawN", fans line, no errors.
3. `coreglowctl status` while it runs: temps, duties, rpm, pump about 3000.
4. Switch through every profile and look at the PC: strips, both fans and the pump head all lit.
5. Ctrl+C → "controller back in hardware mode". With `coreglowctl fallback` / `liquidctl status`: pump about 3000,
   fans on the probe curve.
6. Unplug or pass the device to the VM while coreglowd runs → it logs and reconnects when the device is back.

## Roadmap

Ordered by value for the user. The design canvas has screens for items marked 🎨.

### Next (small, high value)
- [x] **Sidebar layout and Home screen** 🎨 (0.3.0): live tiles with 10-minute sparklines, click-a-part picture,
      quick buttons (lights off, quiet fans). Needs coreglowd to keep a short history in the status file.
- [x] **Effect previews** 🎨: animated preview on every effect button (0.4.0); LED preview strip (0.3.0).
- [x] **Colour presets row** 🎨 next to the custom picker (0.3.0), recently used colours (0.4.0).
- [x] **Keyboard support on the fan curve**: click a dot, arrow keys move it (0.3.0).
- [x] **"Test fans" button** 🎨 and `coreglowctl test-fans`: full speed for 5 s via a request file (0.4.0).
- [x] **Edit the hardware fallback curve** 🎨: `[fallback]` in the config, editor on the Fans page, re-stored on change (0.4.0).

### Then
- [x] **Slovak translation** of the window and tray (0.6.0); review by a native speaker welcome.
- [x] **Automatic profile switching** 🎨: rules by temperature (with 3 °C hysteresis), running program, time of day,
      locked screen; first match wins; editor on the Profiles page, `coreglowctl auto` (0.5.0).
- [x] **Tray menu** 🎨: `coreglow-tray` (ksni, pure-Rust D-Bus): profiles, automatic on/off, test fans, open window;
      autostarts at login (0.5.0). Brightness isn't in the menu (tray menus have no sliders).
- [~] **Control socket** (`$XDG_RUNTIME_DIR/coreglowd.sock`) instead of file polling: instant switching, "test fans",
      status push to the GUI.
- [x] **Night dimming** 🎨 (0.8.0): `[night]` from/to/brightness, card on the Lighting page, `coreglowctl night`.
      Colours are scaled in software (the 0x1f brightness file is still untried).
- [ ] **GPU temperature without NVIDIA's driver**: nouveau doesn't report it; consider the proprietary driver,
      or NVML if installed.

### Later / research
- [x] **Configurable hardware layout** (0.7.0, experimental): device table from iCUE's manifest, `[layout]` in the
      config, detection report (`coreglowctl devices`). Still to do: a layout editor in the window; trying a custom
      layout (especially LED strips via 0x1d) on hardware.
- [x] **Sync with motherboard RGB** (0.6.0): OpenRGB SDK client (protocol v0); the motherboard continues the whole-PC
      effect or gets its own part. Live colour sending still to be tried on the hardware.
- [ ] **Answer the open protocol questions** (icue-reverse-engineering.md §9), especially pump control in
      software mode and stored (hardware) lighting.
- [~] **Packaging polish**: man pages and CI build done (0.6.0, 0.3.0); still lintian, `dpkg-shlibdeps`, a signed apt repo.
- [~] **Other Commander models**: Commander CORE (0x0c1c, and 0x0c32, which iCUE lists as a newer Commander
      CORE rather than an "ST") supported experimentally in 0.8.0 from its manifest; needs a hardware report.
      Next candidates in [corsair-devices.md](corsair-devices.md): the iCUE LINK System Hub (0x0c3f).

## Release notes

### 0.8.0 (2026-11-01)
- **Commander CORE (experimental)**, the hub of the ELITE CAPELLIX and ELITE LCD coolers (USB 0c1c, 0c32), written
  from iCUE's device manifest (`aio_pro_cooler`, `aio_pro_cooler_screen`) and not yet tried on hardware:
  - pump on cooling port 0 with a per-profile speed (`pump = 50`…`100`, slider on the Fans page), fans on ports 1–6;
  - lighting: port 0 is the pump ring (CAPELLIX: geometry byte 0x08, 29 LEDs in the frame; ELITE LCD: 0x0e, 24 LEDs),
    ports 1–6 take fans; no apply command (the manifest doesn't ask for one);
  - without a `[layout]`, the layout comes from the controller's detection and is shared with the window and CLI
    through the status file; parts `pump`, `all-fans`, `port1`…;
  - the `liquid` temperature sensor; the backup curve follows it and drives all six fan ports.
- **Night dimming**: dim all lighting (and the motherboard) between two times; Lighting page card, `[night]`,
  `coreglowctl night 22:00-07:00 30`.
- The udev rule covers the Commander CORE; `coreglowctl status`/`devices` and the window name the controller.
- **Device catalogue**: [corsair-devices.md](corsair-devices.md), 385 devices from iCUE 5's device packages.

### 0.7.0 (2026-10-31)
- **Lighting layout**: device kinds and LED counts from iCUE's device manifest (`commander_core_xt.rcc`). Without a
  `[layout]` table the layout is this build's, byte for byte (unit-tested). A `[layout]` table lists devices per port
  (`external`, `port1`…`port6`) and optional part `names`; the geometry (and 0x1d LED totals for strips) is written
  from it, and part names follow it. Experimental: custom layouts aren't tried on hardware yet.
- **Detection**: coreglowd reads what the controller detects on each port at start; `coreglowctl devices` and the
  "What's connected" section on the Lighting page show it. On this PC: 8-LED fans on ports 1–2, pump cap on port 3.

### 0.6.0 (2026-10-31)
- **Motherboard sync**: `[openrgb] enabled = true` (window: Lighting page, CLI: `coreglowctl motherboard on|off`).
  coreglowd talks to the OpenRGB server (localhost:6742, protocol v0), switches its devices to direct mode and
  sends the motherboard part: the whole-PC effect continued after the pump, or the profile's own `motherboard`
  part. Reconnects every 10 s; the status file reports what's connected. Verified: reading the device list from the
  running server (B650 AORUS ELITE AX V2, 2 LEDs). Not yet tried: sending colours.
- **Slovak translation** of the window, tray and rule descriptions (`src/i18n.rs`), chosen from LC_ALL / LC_MESSAGES /
  LANG; `COREGLOW_LANG=en|sk` overrides it.
- **Man pages**: coreglowctl(1), coreglowd(1), coreglow(1) / coreglow-tray(1).

### 0.5.0 (2026-10-31)
- **Automatic profile switching**: `[auto]` in the config (`enabled`, `rules`); rule kinds `temp-above` (sensor, temp;
  off again 3 °C below), `process` (part of a command line, e.g. `steamapps`), `time` (from/to HH:MM, may cross
  midnight), `locked` (logind LockedHint). First matching rule wins, else the profile chosen by hand. The status file
  reports `manual_profile` and `auto_rule`; the window shows a notice on every page while a rule is in charge.
  Editor on the Profiles page; `coreglowctl auto [on|off]`.
- **Tray icon**: `coreglow-tray` with profile list, automatic switching on/off, test fans, open window, summary
  tooltip. Autostarts via /etc/xdg/autostart.
- **Instant apply**: coreglowd checks the config files on every frame (two stat calls) instead of once a second.
  The control socket stays on the roadmap; with this it's no longer needed for responsiveness.

### 0.4.0 (2026-10-31)
- Backup curve (what the controller does when Coreglow isn't running) is now a `[fallback]` table in the config
  (2-7 points, case probe 1), editable on the Fans page. coreglowd stores it at start, on change and on exit.
  Verified on the controller by reading file 0x6d62 back.
- "Test fans": button on the Fans page and `coreglowctl test-fans`; coreglowd runs fans 1 and 2 at 100 % for 5 s
  (request file `$XDG_RUNTIME_DIR/coreglow-test-fans`, `testing_fans` in the status file).
- Effect buttons show an animated preview with the current colours.
- Recently used custom colours (`~/.config/coreglow/recent-colors`) next to the colour swatches.

### 0.3.0 (2026-10-31)
New window layout from the design: sidebar with Home, Lighting, Fans and Profiles pages. Home has live tiles with
10-minute graphs (coreglowd now keeps `history` in the status file), a clickable picture of the PC and quick buttons.
Lighting: pick a part on the left, edit it on the right, colour swatches, a preview strip of all LEDs.
Fans: curve with the last 10 minutes' temperature range, arrow-key fine tuning, live rpm. Profiles: cards with
colour previews. `coreglow --page lighting|fans|profiles` opens on a page.

### 0.2.0 (2026-10-31)
Renamed from ccxt-control to **Coreglow** (the name ccxt clashes with the CCXT trading library).
Binaries coreglowd / coreglowctl / coreglow, service coreglowd.service, udev rule 70-coreglow.rules,
config ~/.config/coreglow (copied from ~/.config/ccxt on first start), status file coreglow-status.json,
sensor source `controller:N` (`ccxt:N` still accepted). The package replaces ccxt-control.

### 0.1.0 (2026-10-31)
First packaged version: coreglowd, coreglowctl, coreglow; 9 starter profiles; effects static, off, breathing, flash, colour cycle,
rainbow, rainbow wave, colour wave, temperature (with alarm); zones strip1–4, strips, right-fan, left-fan, fans, pump;
fan curves on probe1, probe2, cpu, igpu, gpu; hardware fallback on exit; reconnect.
