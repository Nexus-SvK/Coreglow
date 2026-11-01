# iCUE LINK System Hub: groundwork

Research for supporting Corsair's **iCUE LINK System Hub** (USB `1b1c:0c3f`), the controller of the iCUE LINK
fans, coolers and cases. Everything here comes from iCUE 5's device manifests (`dev-link-system-hub` and the
`Bragi_link_ecosystem_subdevice` packages, see [corsair-devices.md](corsair-devices.md)) and the names in iCUE's
`BragiProtocols.dll`. Nothing has been tried on hardware: nobody here owns a hub. Marks as in
[icue-reverse-engineering.md](icue-reverse-engineering.md): **[code]** = read from iCUE's files, **[guess]** =
inferred, to be checked.

## 1. What's the same as the Commander CORE XT

- **[code]** Bragi over USB HID (`DeviceEnumerator: Bragi`, `HidProto: Bragi_lighting_node`), so coreglow's framing,
  file reads and writes, wake/sleep and the software/hardware mode switch should carry over.
- **[code]** `RequiresApplyGeometryCommand: true` (the `15 01` command coreglow already sends to the Core XT).
- **[code]** Same `BragiFwWorkarounds.useCommonBufferIndices`, plus `clearUnusedFiles` and
  `resetToDefaultOnOperatingModeChangeFail`.
- **[code]** Sensors: `TemperatureSensor`, `FanSensor` (temperatures and rpm, presumably the same 0x21 / 0x17 files).

## 2. What's different

| | Commander CORE XT | iCUE LINK System Hub |
|---|---|---|
| Ports | 6 fan ports, 6 RGB ports + external | one LINK chain: up to 24 devices, 12 per side (`MaxSubdevicesOnSide`) |
| What's connected | fixed kinds per port, LED autodetect (0x20) | each device reports its type number (**sub-devices**) |
| Cooling | channel per fan port | per sub-device; cooling type `LinkMasterController`, 26 sensor channels |
| LEDs | at most ~280 | up to 476 (`MaxLedsCountAcrossAllChannels`) |
| Lighting writes | 384-byte reports | `LightingsBufferHunkSize: 4096` **[guess: bigger chunks of the 0x22 buffer]** |
| Brightness | full | **power limit**: `BrightnessReductionMap` caps brightness by LED count (below) |

- **[code]** Sub-device commands in `BragiProtocols.dll`: `GetConnectedSubdevicesBitmap`, `GetConnectedSubdevicesInfo`
  (a list of `ConnectedSubdeviceInfo`), `GetSubdeviceId`, `StartSubdeviceDetection`, and the notification
  `ConnectedSubdevicesBitmapChanged`; names `ConnectedSubdevicesBitmap` and `ConnectedSubdevicesIds` look like
  Bragi files or properties **[guess]**. Their numbers still have to be found (in the `FileId`/property tables, as in
  §5 of the reverse-engineering log, or from a USB capture of iCUE with a hub).
- **[code]** `SupportedSeparatedCorsairSubdevices` (types 5, 11, 21, 29, 261, 267, 277, 517, 523: the LINK adapter,
  the SHIFT power supplies and the cases) are shown by iCUE as devices of their own.
- **[code]** Brightness reduction, LED count → maximum brightness %: up to 238 LEDs 100, 273: 90, 308: 80, 343: 70,
  378: 60, 413: 50, 448: 40, 483: 30, 519: 22. Software has to respect this or the hub's power budget is exceeded
  **[guess at the reason]**.
- **[code]** The `LinkCoolerScreen40Leds` ring has its own LED order (`DeviceSpecificDynamicGeometryIndexToLedMap`,
  40 lights over 44 slots with 4 unused).

## 3. Sub-device types

**[code]** Each LINK device's manifest names its type (`DeviceKeys: CorsairDeviceId`). The low byte is the product
family, the high byte the variant (size or colour): 7 = H100i, 0x107 (263) = H115i, 0x207 (519) = H150i,
0x307 (775) = H170i, 0x407/0x507 (1031/1287) = later H100i/H150i variants; 17/273/529/785 = TITAN 240/280/360/420.

| Type | Device | Kind | LEDs |
|---|---|---|---|
| 1 | iCUE LINK QX RGB fan | QxRgbFanHub | 34 |
| 2 | iCUE LINK LX RGB fan | LxRgbFan | 18 |
| 3 | iCUE LINK RX RGB MAX fan | RxRgbMax | 8 |
| 4 | iCUE LINK RX MAX fan | (no LEDs) | 0 |
| 5 | iCUE LINK ADAPTER | LSAuroraSeries | varies |
| 6 | iCUE LINK COOLER PUMP LCD | LinkCoolerScreen40Leds | 40 |
| 7, 263, 519, 775, 1031, 1287 | iCUE LINK H100i/H115i/H150i/H170i RGB pump | CoolerPump20Leds | 20 |
| 9 | iCUE LINK XC7 ELITE CPU block | CpuBlockXC7Elite | 24 |
| 10 | iCUE LINK XG3 HYBRID | XG3Hybrid | 18 |
| 11, 267, 277, 523 | HX1000i/HX1200i/AX1600i/HX1500i SHIFT (power supply) | sensors only | 0 |
| 12 | iCUE LINK XD5 ELITE pump/reservoir | XD5ElitePumpRes | 22 |
| 13 | iCUE LINK GPU RGB (XG7) | XG7Rgb | 16 |
| 14 | iCUE LINK XD ELITE LCD | EliteLCDXD | 22 |
| 15 | iCUE LINK RX RGB fan | RxRgbFan | 8 |
| 16 | VRM COOLER MODULE | fan | 0 |
| 17, 273, 529, 785, 1041, 1297 | iCUE LINK TITAN 240/280/360/420 pump | LinkTitanPump | 20 |
| 19 | iCUE LINK RX fan | (no LEDs) | 0 |
| 20 | ARACHNID | OriginOA | 32 |
| 25 | iCUE LINK XD6 ELITE | XD6ElitePumpRes | 22 |
| 26, 28 | iCUE LINK FANS (generic, v2) | generic | varies |
| 27 | Commander DUO | lighting node | varies |
| 29 | HX1000i SHIFT CRYSTAL | RxRgbFan + sensors | 8 |
| 261 | iCUE LINK 9000D RGB AIRFLOW case | Link9000dRgbAirflowChassis | 22 |
| 282 | iCUE LINK GENERIC AIO PUMP | generic | varies |
| 517 | iCUE LINK 5000T RGB case | Link5000tRgbChassis | 204 |
| 2074 | iCUE LINK GENERIC LCD | dashboard LCD | – |

## 4. What supporting it in coreglow would take

1. **Model** `LinkHub` (`0c3f`) next to `CoreXt` and `Core` (`src/model.rs`), udev rule.
2. **Find the sub-device commands' numbers** (the one real unknown): from the `FileId`/command tables in
   `BragiProtocols.dll`, or better a usbmon capture of iCUE talking to a hub (the method used for the Core XT,
   §1 of the reverse-engineering log). Someone with a hub would need to run the capture.
3. **Layout from sub-devices** instead of ports: read the connected list, map each type with the table above to a
   kind and LED count, and build the frame in chain order (the layout code would grow a "chain" variant of `Layout`).
   Parts named by device ("QX fan 1", "H150i pump"), so profiles stay readable.
4. **Brightness cap** from `BrightnessReductionMap` applied to every frame.
5. **Cooling per sub-device**: fans and pumps found in the chain get duties; the hub's own sensors (liquid from the
   pump, fan rpm) go into the status.
6. **Lighting writes** in 4096-byte hunks if the capture confirms it.
7. Mark it experimental until an owner reports back, as with the Commander CORE.
