# iCUE and the Commander Core XT: reverse-engineering log

What was learned about Corsair iCUE 5 and the iCUE Commander Core XT (USB `1b1c:0c2a`, firmware 1.4.62)
while building ccxt-control, 2026-10-30/31. Facts are marked with how they were established:
**[capture]** USB capture, **[code]** iCUE binaries, **[test]** tried on the hardware, **[liquidctl]**
liquidctl's `commander_core` driver. Anything else is labelled as a guess.

## 1. Sources and where they are

| Source | What it is | Location |
|---|---|---|
| `Install iCUE.exe` | iCUE web bootstrapper, 3.4 MB, sha256 `7ecefe7b…` | `~/dev/rea-test/` |
| iCUE install | 966 files, 817 MB, copied out of the Windows VM | `~/dev/rea-test/icue-files/extracted/` |
| `ccxt-icue.pcapng` | iCUE running, fan duty changes, about 4 min | `~/dev/rea-test/captures/` |
| `ccxt-icue-hwlight.pcapng` | iCUE start-up, LED setup changes, white lighting, about 7 min | same |
| `ccxt-openrgb.pcapng` | OpenRGB start, profile, stop, then liquidctl | same |
| `*.tsv` | the same captures as text: time, S/C, dir, ep, xfer, len, hex | same |
| Windows VM | `win11-icue` (libvirt), snapshot `clean-install` before iCUE | `qemu:///system` |

All captures hold only the controller's packets. The raw bus captures also held keyboard traffic and were deleted.

## 2. The installer

- **[code]** `Install iCUE.exe` is a signed (Corsair Memory, Inc., DigiCert EV) x64 web bootstrapper. It needs admin
  (`requireAdministrator`), uses WinRT networking, and downloads its modules at install time from
  `www3.corsair.com/software/CUE_V5/public/modules`. It has an expiry check, and the signing certificate ran until 2026-11-03.
- Nothing worth analysing is inside the stub itself. The real program has to be installed (VM) and copied out.
- Wine isn't viable: the installer is a downloader, and iCUE runs as Windows services plus kernel drivers.

## 3. Where the protocol lives in iCUE

- **[code]** `modules/BragiProtocols.dll` (4.1 MB) implements Corsair's **Bragi** protocol, which this controller speaks.
  It keeps full C++ RTTI names, so classes like `CoolersRotationSettingsFeature` and `OnDeviceLightingFeature` and the
  namespaces `hwp::coolers_settings` and `hwp::lighting_settings` are readable without decompiling.
- **[code]** Qt enums keep their key names in declaration order in the string table, and their values in the
  moc data. The `bragiprotocol::FileId` value table is at file offset **0x2c9a68**: 70 `(name index, value)` pairs.
  Pair the names with values shifted by one, because `None = 0` comes first.
- `devices/commander_core_xt.rcc` is a Qt resource bundle (images and device description). It wasn't unpacked yet; see §9.

## 4. Framing

- **[capture]** HID interrupt endpoint 4, **384-byte reports** both ways. Out: `08 <cmd> <args…>`; in: `00 <cmd> <status> …`.
- **[test]** 384 matters: liquidctl's 96-byte reports work for short commands, but a long lighting write split into
  96-byte `WRITE_MORE` packets lands in the wrong place (only about the first 29 LEDs light correctly).
- **[liquidctl] [capture]** Commands:

| Bytes | Meaning |
|---|---|
| `01 03 00 02` / `01 03 00 01` | set property 3 (mode): 2 = software mode, 1 = hardware mode |
| `02 <prop>` | get property; seen: 03 (mode), 11, 12, 13 (firmware), 14, 55, 59, 5a |
| `0d <h> <file lo> [<file hi>]` | open file on handle h |
| `08 <h> 01/02/03` | read initial / more / final chunk |
| `09 <h>` | read with a different header (seen from iCUE; meaning unclear) |
| `06 <h> <len u16> 00 00 <type u16> <data…>` | write first chunk; 375 data bytes fit in a 384-byte report |
| `07 <h> <data…>` | write more, 381 bytes per report |
| `05 01 <h>` | close handle h |
| `15 01` | apply lighting layout (sent by iCUE after writing 0x1d/0x1e) |
| `14 0N 01` (N = 1..6), `15 01` ×6 | seen at iCUE start-up, per fan/channel; meaning unclear |

- **[capture]** iCUE keeps **handle 0** open for the lighting stream and does everything else on **handle 1**.
  liquidctl and ccxtd use handle 0 for everything (fine, as long as one process owns the device).

## 5. File IDs (`bragiprotocol::FileId`) **[code]**, with what was seen

| ID | Name | Notes |
|---|---|---|
| 0x01 | RealTimeRgbLightings | |
| 0x06–0x08 | StoredProfile1–3 | **[test]** iCUE never wrote these for this device |
| 0x09 | ProfilesList | **[test]** status 03 (not available) on fw 1.4.62 |
| 0x10 | RealTimeSingleColorLightings | |
| 0x15 | CoolersAutodetectionSettings | status 03 |
| 0x16 | CoolersConnectionTypes | status 03 (enum ThreePin, FourPin) |
| 0x17 | CoolersActualRPMs | type `06 00`: count, u16 rpm per channel |
| 0x18 | CoolersTargetRotationSettings | type `07 00`: count, `[ch, 00, duty u16]`; **software-mode fan duty** |
| 0x1a | CoolersAutodetectionResults | `07 07 01 01 01 07`: connected fans 1, 2 and 6 |
| 0x1b | LightingChannelsAutodetectionSettings | |
| 0x1c | LightingChannelsControllerTypes | status 03 |
| 0x1d | LightingChannelsControllersQuantity | type `0c 00`: count, u16 per channel (LED count for generic strips) |
| 0x1e | LightingChannelsGeometries | type `0d 00`: see §6 |
| 0x1f | LightingChannelsBrightnesses | type `0e 00`: count, u16 per channel (iCUE wrote 0xd9) |
| 0x20 | LightingChannelsAutodetectionResults | type `0f 00`: count, `[status u16, leds u16]` per channel |
| 0x21 | TemperatureMeasurementsStatus | type `10 00`: count, `[status, temp×10 u16]` (status 0 = connected) |
| 0x22 | RealTimeLightingsBuffer | type `12 00`: RGB triplets, see §6 |
| 0x30 | CoolersTargetRotationMode | status 03 (enum BuiltIn, HostProvided) |
| 0x6d60 | free file (hardware cooling modes) | type `03 00`: count, mode per channel (0 = fixed %, 2 = curve %) |
| 0x6d61 | free file (hardware fixed duty) | type `04 00`: count, u16 per channel |
| 0x6d62 | free file (hardware curves) | type `05 00`: count, per channel `[sensor, n, (temp×10 u16, duty u16)×n]` |
| 0x6d63–0x6d66 | free files | iCUE read them; 0x6d64/0x6d65 look like sensor mapping (`7e 20 09 …`) |

Other enums **[code]**: CoolerWorkMode {FixedSpeedPWMDutyCycle, FixedSpeedRPM, CurvePWMDutyCycle, CurveRPM,
DefaultBuiltinCurveMode}; hardware lighting layouts {StaticColorLighting, ColorPulse, ColorShift, RainbowWave,
SpiralRainbow, …}; LED type enums under `hwp::DeviceLedType`.

## 6. Lighting

- **[capture]** iCUE renders every effect on the PC and streams it to 0x22 at **24.7 frames/s**. Nothing is
  stored in the controller: with everything set to white, iCUE wrote no stored profile.
- **[capture] [test]** Geometry (0x1e) = channel count, then per channel `device count, type byte × count`.
  This build's stored layout, which iCUE re-writes at start-up:
  `07 | 06 06 06 06 06 06 06 | 01 05 | 01 05 | 01 0c | 00 | 00 | 00`
  - external port: 6 devices of type 06 (34 LED slots each, 204 total)
  - port 1: one type 05 device (8 LEDs) = **right radiator fan**
  - port 2: one type 05 device (8 LEDs) = **left radiator fan**
  - port 3: one type 0c device (13 LEDs) = **pump head**
  - ports 4–6: empty
- **[test]** After writing the geometry, send `15 01` or it isn't applied.
- **[capture] [test]** The frame length must match the geometry exactly: 6×34 + 8 + 8 + 13 = **233 LEDs** (701 bytes
  with the type). With a different layout (one generic strip, 0x1d = 40) iCUE sent 69-LED frames.
- **[test]** The 4 LED strips (10 LEDs each) are the first 40 slots of the external chain. Slots 40–203 are unused.
  Mapped with a colour test.
- **[test]** OpenRGB's layout (`07 00 01 06 ×6`) disables the external port, so the strips stay dark under OpenRGB.
- Keepalive: OpenRGB re-sends a full frame every 10 s. ccxtd re-sends unchanged frames every 2 s. There's no
  separate keepalive command; software mode persists until `01 03 00 01`.

## 7. Fans and pump

- **[capture]** iCUE runs its fan curves on the PC and sends duties to 0x18 only when they change.
- **[capture] [test]** In software mode, **port 6 (the AIO pump) ignores 0x18** and runs at full speed (~3050 rpm),
  under iCUE as well. Fans 1 and 2 follow 0x18.
- **[test]** In hardware mode, the stored curves (0x6d60–0x6d62) do slow port 6. The old fallback curve took the pump
  down to ~760 rpm, which is too low for a pump, so ccxtd's fallback stores 100 % fixed for port 6.
- **[test]** Hardware curves always use sensor 0 (probe 1). Other sensors (CPU) are only possible in software mode.

## 8. Behaviour notes

- **[test]** Running OpenRGB's detection (even `--list-devices`) leaves the controller in software mode, so the
  hardware fan curve stops applying until the device is put back in hardware mode.
- **[test]** `liquidctl status` wakes and sleeps the device around each read. Running it alongside a software-mode
  owner (OpenRGB, ccxtd) fights over the mode.
- **[test]** USB passthrough to the VM: libvirt `hostdev` vendor/product, `--live`. A Windows restart or VM shutdown
  drops it; the device address (bus 1, dev 8) stayed stable.
- **[test]** Stopping iCUE's Windows services leaves iCUE showing "No supported CORSAIR devices found".

## 9. Open questions (worth a look next time)

1. What do `09 <h>`, `14 0N 01` and the extra `15 01` at start-up do? Possibly autodetection subcommands
   (`CoolerAutodetectionSubcommand` and `LightingChannelAutodetectionSubcommand` exist in the enum list).
2. What are properties 0x11, 0x12, 0x14, 0x55, 0x59 and 0x5a? (Firmware is 0x13.) Find the `PropertyId` enum's value
   table the same way as FileId (§3).
3. LED counts per device type: 06 = 34, 05 = 8, 0c = 13 come from frame sizes. A table of all types probably lives in
   `commander_core_xt.rcc` or a JSON in `predefined_data/`. Unpack the `.rcc` (Qt `rcc` format) to confirm, and to
   learn which real Corsair devices those types are.
4. Can the pump be slowed in software mode at all (another file, or a 0x18 mode byte other than 00)? Not found yet.
5. Hardware (stored) lighting: the enum names exist (`OnDeviceLighting`, `StaticColorLighting`…), but iCUE didn't use
   them for this device. Maybe the firmware doesn't support it, or iCUE exposes it elsewhere.
6. Brightness file 0x1f: is 0xd9 a 0–255 scale or 0–1000? It might allow hardware dimming instead of scaling RGB.

## 10. How to repeat the work

1. Start the VM (`virsh -c qemu:///system start win11-icue`) and stop `ccxtd` on Linux.
2. Pass the controller through:
   `virsh -c qemu:///system attach-device win11-icue ccxt.xml --live`, with `ccxt.xml` =
   `<hostdev mode='subsystem' type='usb' managed='yes'><source><vendor id='0x1b1c'/><product id='0x0c2a'/></source></hostdev>`.
3. Capture on the host: `dumpcap -i usbmon1 -w raw.pcapng` (the wireshark group can read `/dev/usbmon*`
   through `/etc/udev/rules.d/60-usbmon.rules`). Don't type passwords while it runs.
4. Change one thing at a time in iCUE and note the time.
5. Keep only the controller's packets (filter by bus 1 and the device address; the address with the most 384-byte
   transfers on endpoint 4 is the controller) and delete the raw file.
6. Detach (`detach-device … --live`) and start `ccxtd` again.
