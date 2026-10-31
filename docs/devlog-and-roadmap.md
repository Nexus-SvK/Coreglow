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
- [ ] **Effect previews** 🎨: small animated swatches on the effect buttons, and a 69-LED preview strip.
- [x] **Colour presets row** 🎨 next to the custom picker (0.3.0). Still to do: remember recently used colours.
- [x] **Keyboard support on the fan curve**: click a dot, arrow keys move it (0.3.0).
- [ ] **"Test fans" button** 🎨: full speed for 5 s, then back to the curve (a `coreglowd` command via a file or socket).
- [ ] **Edit the hardware fallback curve** 🎨 in the GUI (today it's a constant in `device.rs`). Store it in the config.

### Then
- [ ] **Automatic profile switching** 🎨: rules by time of day, CPU or GPU temperature, a running process
      (e.g. Steam games), screen locked. Evaluate in coreglowd; show the rule that's active.
- [ ] **Tray menu** 🎨: profile switch, brightness, lights off. Needs a StatusNotifierItem (ksni crate) or
      a GNOME extension.
- [ ] **Control socket** (`$XDG_RUNTIME_DIR/coreglowd.sock`) instead of file polling: instant switching, "test fans",
      status push to the GUI.
- [ ] **Night dimming** 🎨 (brightness schedule), perhaps via the 0x1f brightness file instead of scaling RGB.
- [ ] **GPU temperature without NVIDIA's driver**: nouveau doesn't report it; consider the proprietary driver,
      or NVML if installed.

### Later / research
- [ ] **Configurable hardware layout**: read 0x20 autodetection and 0x1e at start-up instead of a fixed
      `PORT_CONFIG`; let the user name devices (needs the LED-type table, open question 3).
- [ ] **Sync with motherboard RGB**: talk to OpenRGB's SDK server (port 6742, localhost) so effects span both.
- [ ] **Answer the open protocol questions** (icue-reverse-engineering.md §9), especially pump control in
      software mode and stored (hardware) lighting.
- [ ] **Packaging polish**: man pages, lintian-clean `.deb`, `dpkg-shlibdeps` for Depends, signed apt repo,
      CI build.
- [ ] **Other Commander models** (Core 0x0c1c, ST 0x0c32): same Bragi protocol; needs their geometry and
      the pump on channel 0.

## Release notes

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
