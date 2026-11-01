# Corsair device catalogue

Facts extracted from iCUE 5's device packages (`dev-*.cuepkg`, downloaded from Corsair's public
module server): each package's `meta/auxiliary.json` and the device manifest inside its Qt
resource bundle. Only names, USB IDs and protocol/geometry facts are recorded here; no Corsair
files are redistributed. Regenerate with `tools/icue-catalogue/` (see its README).

385 device entries. Columns: **USB ID** (vendor:product), **type** (iCUE's device-type),
**HID protocol** (iCUE's `HidProto`, the driver flavour it picks).

| Protocol family | Devices |
|---|---|
| Bragi — the same protocol coreglow speaks to the Commander CORE XT | 195 |
| Legacy — older keyboards, mice and headsets | 85 |
| CLink — older Commander / Platinum coolers / PSUs / Lighting Node | 65 |
| LLAccess — memory (DRAM) via SMBus, needs a kernel driver | 14 |
| Touchscreen | 9 |
| Gip | 4 |
| Other | 3 |
| MccsUsb | 3 |
| DashAccessory | 1 |
| Govee | 1 |
| AvneraAccessory | 1 |
| Nanoleaf | 1 |
| PhilipsHue | 1 |
| Edid | 1 |
| Uniwill | 1 |

## Closest to coreglow

Bragi lighting nodes share the Commander CORE XT's command set (wake, open/read/write files,
geometry). Where the manifest lists LED channel kinds they are given as kind → geometry index,
with the LED count per kind.

### 5000T RGB — `1b1c:0c2a`

type `led-controller`, protocol `Bragi_lighting_node`, acts as CommanderCoreXt, bragi lister

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| Chassis5000T | 1 | 0 |
| Strip25Leds | 1 | 25 |
| Strip30Leds | 1 | 30 |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |

### ELITE LCD Cooler — `1b1c:0c32`

type `led-controller`, protocol `Bragi_lighting_node`, acts as CapellixLcd, bragi lister, 11 cooler models

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |
| CoolerScreen24Leds | 14 | 24 |

Coolers: H100i ELITE CAPELLIX (240 mm), H100i ELITE LCD (240 mm), H100i ELITE LCD XT (240 mm), H115i ELITE CAPELLIX (280 mm), H115i ELITE LCD XT (280 mm), H150i ELITE CAPELLIX (360 mm), H150i ELITE LCD (360 mm), H150i ELITE LCD XT (360 mm), H170i ELITE CAPELLIX (420 mm), H170i ELITE LCD (420 mm), H170i ELITE LCD XT (420 mm)

### ELITE LCD Cooler — `1b1c:0c1c`

type `led-controller`, protocol `Bragi_lighting_node`, acts as CapellixLcd, bragi lister, 11 cooler models

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |
| CoolerScreen24Leds | 14 | 24 |

Coolers: H100i ELITE CAPELLIX (240 mm), H100i ELITE LCD (240 mm), H100i ELITE LCD XT (240 mm), H115i ELITE CAPELLIX (280 mm), H115i ELITE LCD XT (280 mm), H150i ELITE CAPELLIX (360 mm), H150i ELITE LCD (360 mm), H150i ELITE LCD XT (360 mm), H170i ELITE CAPELLIX (420 mm), H170i ELITE LCD (420 mm), H170i ELITE LCD XT (420 mm)

### Commander CORE — `1b1c:0c32`, `1b1c:0c3c`

type `led-controller`, protocol `Bragi_lighting_node`, acts as Capellix, bragi lister, 8 cooler models

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |
| CoolerPump21Leds | 8 | 21 |

Coolers: H100i ELITE CAPELLIX (240 mm), H100i ELITE CAPELLIX XT (240 mm), H115i ELITE CAPELLIX (280 mm), H115i ELITE CAPELLIX XT (280 mm), H150i ELITE CAPELLIX (360 mm), H150i ELITE CAPELLIX XT (360 mm), H170i ELITE CAPELLIX (420 mm), H170i ELITE CAPELLIX XT (420 mm)

### Commander CORE — `1b1c:0c1c`, `1b1c:0c1d`

type `led-controller`, protocol `Bragi_lighting_node`, acts as Capellix, bragi lister, 8 cooler models

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |
| CoolerPump21Leds | 8 | 21 |

Coolers: H100i ELITE CAPELLIX (240 mm), H100i ELITE CAPELLIX XT (240 mm), H115i ELITE CAPELLIX (280 mm), H115i ELITE CAPELLIX XT (280 mm), H150i ELITE CAPELLIX (360 mm), H150i ELITE CAPELLIX XT (360 mm), H170i ELITE CAPELLIX (420 mm), H170i ELITE CAPELLIX XT (420 mm)

### Commander CORE XT — `1b1c:0c2a`

type `led-controller`, protocol `Bragi_lighting_node`, acts as CommanderCoreXt, bragi lister

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| CompanionTriangle9Leds | 1 | 9 |
| ExternalRGBLightStrip | 1 |  |
| LSAuroraSeries | 1 |  |
| MasterTriangle9Leds | 1 | 9 |
| RgbPowerExtensionCable | 1 | 25 |
| RgbStrip | 1 | 10 |
| Strip15Leds | 1 | 15 |
| Strip21Leds | 1 | 21 |
| Strip27Leds | 1 | 27 |
| Strip40Leds | 1 | 40 |
| Strip49Leds | 1 | 49 |
| Strip84Leds | 1 | 84 |
| LlFanHub | 2 | 16 |
| MlFanHub | 3 | 4 |
| HdFanHub | 4 | 12 |
| Fan8LedSeries | 5 | 8 |
| QlRgbFanHub | 6 | 34 |
| RgbPumpCap13Leds | 12 | 13 |

### iCUE LINK System Hub — `1b1c:0c3f`

type `led-controller`, protocol `Bragi_lighting_node`, acts as LinkSystemHub, bragi lister

| LED channel kind | geometry index | LEDs |
|---|---|---|
| Autodetect | 0 |  |
| GenericRgbFan | 0 |  |
| GenericRgbStrip | 0 |  |
| CompanionTriangle9Leds | 1 |  |
| CpuBlock | 1 |  |
| CpuBlockXC5 | 1 |  |
| ExternalRGBLightStrip | 1 |  |
| Fan8LedSeries | 1 |  |
| GpuBlock | 1 |  |
| GpuBlock_x2 | 1 |  |
| HydroXComplex | 1 |  |
| LSAuroraSeries | 1 |  |
| Link9000dRgbAirflowChassis | 1 |  |
| Link9000dRgbAirflowIoPanel | 1 |  |
| Link9000dRgbAirflowLogo | 1 |  |
| LinkGenericRGBStrip | 1 |  |
| LlFanHub | 1 |  |
| MasterTriangle9Leds | 1 |  |
| PumpResCombo | 1 |  |
| PumpResCombo_x2 | 1 |  |
| QlRgbFanHub | 1 |  |
| RgbPowerExtensionCable | 1 |  |
| RgbStrip | 1 |  |
| Strip15Leds | 1 |  |
| Strip21Leds | 1 |  |
| Strip27Leds | 1 |  |
| Strip40Leds | 1 |  |
| Strip49Leds | 1 |  |
| Strip84Leds | 1 |  |
| TriangleComplex | 1 |  |
| UsbArgbFanController | 1 | 0 |
| XD3PumpResCombo | 1 |  |
| XD3PumpResCombo_x2 | 1 |  |
| XD7PumpResCombo | 1 |  |
| QxRgbFanHub | 15 | 34 |
| CpuBlockXC7Elite | 16 | 24 |
| EliteLCDXD | 17 | 22 |
| XD5ElitePumpRes | 17 | 22 |
| XD6ElitePumpRes | 17 | 22 |
| XG3Hybrid | 18 | 18 |
| CoolerPump20Leds | 19 | 20 |
| LinkCoolerScreen40Leds | 20 | 40 |
| XG7Rgb | 22 | 16 |
| RxRgbFan | 23 | 8 |
| LxRgbFan | 24 | 18 |
| RxRgbMax | 25 | 8 |
| LinkTitanPump | 26 | 20 |
| OriginOA | 27 | 32 |
| Link5000tRgbChassis | 28 |  |
| Strip32Leds | 28 |  |
| Strip38Leds | 28 |  |
| LinkGenericCombinedFanRgb | 29 |  |
| LinkGenericDisplayRgb | 29 |  |
| LinkGenericFanRgb | 29 |  |
| LinkGenericPumpRgb | 29 |  |

## Bragi — the same protocol coreglow speaks to the Commander CORE XT

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| iCUE LINK GENERIC LCD | `-001:-001` | dashboardLCD | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| Fake Widget Display | `-064:-001` | display |  |  |
| DARK CORE PRO SE WIRELESS RECEIVER | `1b1c:1b7f` | dongle | Bragi_dongle | bragi lister |
| DARK CORE PRO WIRELESS RECEIVER | `1b1c:1b81` | dongle | Bragi_dongle | bragi lister |
| HARPOON WIRELESS RECEIVER | `1b1c:1b65` | dongle | Bragi_dongle | bragi lister |
| HS55 WIRELESS CORE RECEIVER | `1b1c:0a94`, `1b1c:0a95` | dongle | Bragi_dongle |  |
| HS55 WIRELESS RECEIVER | `1b1c:0a8e`, `1b1c:0a8f` | dongle | Bragi_dongle |  |
| HS65 WIRELESS RECEIVER | `1b1c:0a91`, `1b1c:0a92` | dongle | Bragi_dongle |  |
| HS80 MAX WIRELESS FOR XBOX RECEIVER | `1b1c:2a0b`, `1b1c:2a0c`, `1b1c:2a0d` | dongle | Bragi_dongle |  |
| HS80 MAX WIRELESS RECEIVER | `1b1c:0a96`, `1b1c:0a97` | dongle | Bragi_dongle |  |
| HS80 MAX WIRELESS RECEIVER | `1b1c:2a0e`, `1b1c:2a0f` | dongle | Bragi_dongle |  |
| HS80 V2 MAX WIRELESS RECEIVER | `1b1c:2a1a`, `1b1c:2a1b` | dongle | Bragi_dongle |  |
| HS80 WIRELESS RECEIVER | `1b1c:0a6b` | dongle | Bragi_dongle | bragi lister |
| HS80 WIRELESS RECEIVER | `1b1c:0a73` | dongle | Bragi_dongle | bragi lister |
| IRONCLAW WIRELESS RECEIVER | `1b1c:1b66` | dongle | Bragi_dongle | bragi lister |
| K57 HARPOON RGB WIRELESS BUNDLE | `1b1c:1bb0` | dongle | Bragi_dongle | bragi lister |
| K57 WIRELESS RECEIVER | `1b1c:1b62` | dongle | Bragi_dongle | bragi lister |
| K65 PLUS WIRELESS RECEIVER | `1b1c:2b07` | dongle | Bragi_dongle | bragi lister |
| KATAR PRO WIRELESS RECEIVER | `1b1c:1b94`, `1b1c:1bae` | dongle | Bragi_dongle |  |
| SCUF ENVISION PRO WIRELESS RECEIVER | `2e95:434e`, `2e95:5046` | dongle | Bragi_dongle |  |
| SCUF ENVISION PRO WIRELESS RECEIVER | `1b1c:3a08`, `1b1c:3a09` | dongle | Bragi_dongle |  |
| SLIPSTREAM v2 WIRELESS USB Receiver | `1b1c:2b3c` | dongle | Bragi_dongle | bragi lister |
| SLIPSTREAM v2 WIRELESS USB Receiver | `1b1c:2b1b` | dongle | Bragi_dongle | bragi lister |
| SLIPSTREAM WIRELESS USB Receiver | `1b1c:1bdc` | dongle | Bragi_dongle | bragi lister |
| SLIPSTREAM WIRELESS USB Receiver | `1b1c:2b00` | dongle | Bragi_dongle | bragi lister |
| SLIPSTREAM WIRELESS USB Receiver | `1b1c:1ba6` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO MAX WIRELESS FOR XBOX RECEIVER | `1b1c:2a01`, `1b1c:2a03`, `1b1c:2a04` | dongle | Bragi_dongle |  |
| VIRTUOSO MAX WIRELESS RECEIVER | `1b1c:2a00`, `1b1c:2a02` | dongle | Bragi_dongle |  |
| VIRTUOSO SE WIRELESS RECEIVER | `1b1c:0a48`, `1b1c:0a40` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO SE WIRELESS RECEIVER | `1b1c:0a46`, `1b1c:0a3e` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO WIRELESS RECEIVER | `1b1c:0a4a`, `1b1c:0a42` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO WIRELESS RECEIVER | `1b1c:0a67`, `1b1c:0a5c` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO WIRELESS RECEIVER | `1b1c:0a4c`, `1b1c:0a44` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO XT WIRELESS RECEIVER | `1b1c:0a64` | dongle | Bragi_dongle | bragi lister |
| VIRTUOSO XT WIRELESS RECEIVER | `1b1c:0a78` | dongle | Bragi_dongle | bragi lister |
| VOID MAX WIRELESS FOR XBOX RECEIVER | `1b1c:2a05`, `1b1c:2a06`, `1b1c:2a07` | dongle | Bragi_dongle |  |
| VOID MAX WIRELESS RECEIVER | `1b1c:2a10`, `1b1c:2a12` | dongle | Bragi_dongle |  |
| VOID WIRELESS V2 RECEIVER | `1b1c:2a08`, `1b1c:2a09` | dongle | Bragi_dongle |  |
| VOYAGER A1600 SLIPSTREAM MODULE | `1b1c:0d01` | dongle | Bragi_dongle | bragi lister |
| iCUE LINK FANS | `-001:-001` | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK FANS v2 | `-001:-001` | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK LX RGB | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK QX RGB | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK RX | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK RX MAX | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK RX RGB | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK RX RGB MAX | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK XG3 HYBRID | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| VRM COOLER MODULE | — | fan | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| SCUF ENVISION | `2e95:434b` | gamepad | Bragi_gamepad |  |
| SCUF ENVISION PRO | `2e95:434d`, `2e95:434e` | gamepad | Bragi_gamepad | wireless |
| SCUF ENVISION PRO V2 | `1b1c:3a05`, `1b1c:3a08` | gamepad | Bragi_gamepad | wireless |
| SCUF ENVISION V2 | `1b1c:3a04` | gamepad | Bragi_gamepad |  |
| HS55 WIRELESS | `1b1c:0a8e`, `1b1c:0a8f` | headset | Bragi_headset | wireless |
| HS55 WIRELESS CORE | `1b1c:0a94`, `1b1c:0a95` | headset | Bragi_headset | wireless |
| HS65 WIRELESS | `1b1c:0a91`, `1b1c:0a92` | headset | Bragi_headset | wireless |
| HS80 | `1b1c:0a71` | headset | Bragi_headset | wireless |
| HS80 | `1b1c:0a69` | headset | Bragi_headset | wireless |
| HS80 MAX WIRELESS | `1b1c:2a0e`, `1b1c:2a0f` | headset | Bragi_headset | wireless |
| HS80 MAX WIRELESS | `1b1c:0a96`, `1b1c:0a97` | headset | Bragi_headset | wireless |
| HS80 V2 MAX WIRELESS | `1b1c:2a1a`, `1b1c:2a1b` | headset | Bragi_headset | wireless |
| VIRTUOSO | `1b1c:0a41` | headset | Bragi_headset | wireless |
| VIRTUOSO | `1b1c:0a5a` | headset | Bragi_headset | wireless |
| VIRTUOSO | `1b1c:0a43` | headset | Bragi_headset | wireless |
| VIRTUOSO MAX WIRELESS | `1b1c:2a00`, `1b1c:2a02` | headset | Bragi_headset | wireless |
| VIRTUOSO MAX WIRELESS FOR XBOX | `1b1c:2a01`, `1b1c:2a03`, `1b1c:2a04` | headset | Bragi_headset | wireless |
| VIRTUOSO MAX WIRELESS FOR XBOX | `1b1c:2a01`, `1b1c:2a03`, `1b1c:2a04` | headset | Bragi_headset | wireless |
| VIRTUOSO SE | `1b1c:0a3f` | headset | Bragi_headset | wireless |
| VIRTUOSO SE | `1b1c:0a3d` | headset | Bragi_headset | wireless |
| VIRTUOSO XT | `1b1c:0a62` | headset | Bragi_headset | wireless |
| VIRTUOSO XT | `1b1c:0a76` | headset | Bragi_headset | wireless |
| VOID MAX WIRELESS | `1b1c:2a12`, `1b1c:2a10` | headset | Bragi_headset | wireless |
| VOID WIRELESS V2 | `1b1c:2a08`, `1b1c:2a09` | headset | Bragi_headset | wireless |
| K100 RGB | `1b1c:1bc5` | keyboard | Bragi_kbd |  |
| K100 RGB | `1b1c:1b7c` | keyboard | Bragi_kbd |  |
| K100 RGB | `1b1c:1b7d` | keyboard | Bragi_kbd |  |
| K100 RGB Air Wireless | `1b1c:1bab` | keyboard | Bragi_kbd | wireless |
| K55 CORE RGB | `1b1c:2b05` | keyboard | Bragi_kbd |  |
| K55 CORE RGB | `1b1c:1bfe` | keyboard | Bragi_kbd |  |
| K55 CORE RGB TKL | `1b1c:2b20` | keyboard | Bragi_kbd |  |
| K55 RGB PRO | `1b1c:1ba4` | keyboard | Bragi_kbd |  |
| K55 RGB PRO XT | `1b1c:1ba1` | keyboard | Bragi_kbd |  |
| K57 RGB WIRELESS | `1b1c:1b6e` | keyboard | Bragi_kbd | wireless |
| K60 PRO | `1b1c:1b83` | keyboard | Bragi_kbd |  |
| K60 PRO TKL RGB | `1b1c:1bed` | keyboard | Bragi_kbd |  |
| K60 PRO TKL RGB | `1b1c:1bc7` | keyboard | Bragi_kbd |  |
| K60 RGB PRO | `1b1c:1ba0` | keyboard | Bragi_kbd |  |
| K60 RGB PRO LP | `1b1c:1bad` | keyboard | Bragi_kbd |  |
| K60 RGB PRO SE | `1b1c:1b8d` | keyboard | Bragi_kbd |  |
| K65 PLUS WIRELESS | `1b1c:2b12` | keyboard | Bragi_kbd | wireless |
| K65 PLUS WIRELESS | `1b1c:2b10` | keyboard | Bragi_kbd | wireless |
| K65 PLUS WIRELESS | `1b1c:2b11` | keyboard | Bragi_kbd | wireless |
| K65 PLUS WIRELESS | `1b1c:2b37` | keyboard | Bragi_kbd | wireless |
| K65 PLUS WIRELESS | `1b1c:2b36` | keyboard | Bragi_kbd | wireless |
| K65 PLUS WIRELESS | `1b1c:2b23` | keyboard | Bragi_kbd | wireless |
| K65 PRO Mini | `1b1c:1bd7` | keyboard | Bragi_kbd |  |
| K65 RGB Mini | `1b1c:1bc3` | keyboard | Bragi_kbd |  |
| K65 RGB Mini | `1b1c:1bcf` | keyboard | Bragi_kbd |  |
| K65 RGB Mini | `1b1c:1bbd` | keyboard | Bragi_kbd |  |
| K65 RGB Mini | `1b1c:1baf` | keyboard | Bragi_kbd |  |
| K70 CORE RGB | `1b1c:2b0a` | keyboard | Bragi_kbd |  |
| K70 CORE RGB | `1b1c:1bff` | keyboard | Bragi_kbd |  |
| K70 CORE RGB | `1b1c:1bfd` | keyboard | Bragi_kbd |  |
| K70 CORE RGB TKL | `1b1c:2b01` | keyboard | Bragi_kbd |  |
| K70 CORE RGB TKL WIRELESS | `1b1c:2b02` | keyboard | Bragi_kbd | wireless |
| K70 CORE SE RGB | `1b1c:2b09` | keyboard | Bragi_kbd |  |
| K70 MAX | `1b1c:1bc0` | keyboard | Bragi_kbd |  |
| K70 PRO RGB TKL | `1b1c:2b14` | keyboard | Bragi_kbd |  |
| K70 RGB PRO | `1b1c:1bd4` | keyboard | Bragi_kbd |  |
| K70 RGB PRO | `1b1c:1bc6` | keyboard | Bragi_kbd |  |
| K70 RGB PRO | `1b1c:1bc4` | keyboard | Bragi_kbd |  |
| K70 RGB PRO | `1b1c:1bb3` | keyboard | Bragi_kbd |  |
| K70 RGB PRO Mini Wireless | `1b1c:1bd3` | keyboard | Bragi_kbd | wireless |
| K70 RGB PRO Mini Wireless | `1b1c:1bb6` | keyboard | Bragi_kbd | wireless |
| K70 RGB TKL Champion Series | `1b1c:1b73` | keyboard | Bragi_kbd |  |
| K70 RGB TKL Champion Series | `1b1c:1bb9` | keyboard | Bragi_kbd |  |
| K95 RGB PLATINUM XT | `1b1c:1b89` | keyboard | Bragi_kbd |  |
| MAKR 75 | `1b1c:2b04` | keyboard | Bragi_kbd | wireless |
| MAKR PRO 75 | `1b1c:2b5b` | keyboard | Bragi_kbd | wireless |
| MAKR PRO 75 | `1b1c:2b1a` | keyboard | Bragi_kbd | wireless |
| SKIFF 100 | `1b1c:2b3d` | keyboard | Bragi_kbd |  |
| SKIFF 100 PLUS | `1b1c:2b3e` | keyboard | Bragi_kbd |  |
| VANGUARD 96 | `1b1c:2b0d` | keyboard | Bragi_kbd |  |
| VANGUARD 96 WIRELESS | `1b1c:2b0f` | keyboard | Bragi_kbd | wireless |
| VANGUARD AIR 99 | `1b1c:2b21` | keyboard | Bragi_kbd | wireless |
| VANGUARD PRO 96 | `1b1c:2b54` | keyboard | Bragi_kbd |  |
| VANGUARD PRO 96 | `1b1c:2b0e` | keyboard | Bragi_kbd |  |
| VOYAGER A1600 KEYBOARD | `1b1c:1bd0` | keyboard | Bragi_kbd |  |
| 5000T RGB | `1b1c:0c2a` | led-controller | Bragi_lighting_node | acts as CommanderCoreXt, bragi lister |
| ARACHNID | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| Commander CORE | `1b1c:0c32`, `1b1c:0c3c` | led-controller | Bragi_lighting_node | acts as Capellix, bragi lister, 8 cooler models |
| Commander CORE | `1b1c:0c1c`, `1b1c:0c1d` | led-controller | Bragi_lighting_node | acts as Capellix, bragi lister, 8 cooler models |
| Commander CORE XT | `1b1c:0c2a` | led-controller | Bragi_lighting_node | acts as CommanderCoreXt, bragi lister |
| Commander DUO | `1b1c:0c56` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| Commander DUO | `1b1c:0c56` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| ELITE LCD Cooler | `1b1c:0c32` | led-controller | Bragi_lighting_node | acts as CapellixLcd, bragi lister, 11 cooler models |
| ELITE LCD Cooler | `1b1c:0c1c` | led-controller | Bragi_lighting_node | acts as CapellixLcd, bragi lister, 11 cooler models |
| iCUE LINK 5000T RGB | `-001:-001` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK 9000D RGB AIRFLOW | `-001:-001` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK ADAPTER | `-001:-001` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK COOLER PUMP LCD | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdeviceLcd |
| iCUE LINK GENERIC AIO PUMP | `-001:-001` | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK GPU RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H100i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H100i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H115i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H150i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H150i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK H170i RGB | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK System Hub | `1b1c:0c3f` | led-controller | Bragi_lighting_node | acts as LinkSystemHub, bragi lister |
| iCUE LINK TITAN 240 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK TITAN 240 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK TITAN 280 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK TITAN 360 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK TITAN 360 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK TITAN 420 | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK XC7 ELITE | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK XD ELITE LCD | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdeviceLcd |
| iCUE LINK XD5 ELITE | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| iCUE LINK XD6 ELITE | — | led-controller | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| DARK CORE PRO | `1b1c:1b80` | mouse | Bragi_mouse | wireless |
| DARK CORE PRO SE | `1b1c:1b7e` | mouse | Bragi_mouse | wireless |
| DARKSTAR WIRELESS | `1b1c:1bb2` | mouse | Bragi_mouse | wireless |
| HARPOON RGB WIRELESS | `1b1c:1b5e` | mouse | Bragi_mouse | wireless |
| IRONCLAW RGB WIRELESS | `1b1c:1b4c` | mouse | Bragi_mouse | wireless |
| IRONCLAW WIRELESS SE | `1b1c:2b32` | mouse | Bragi_mouse | wireless |
| KATAR ELITE Wireless | `1b1c:1bba` | mouse | Bragi_mouse | wireless |
| KATAR PRO | `1b1c:1b93` | mouse | Bragi_mouse |  |
| KATAR PRO WIRELESS | `1b1c:1b94`, `1b1c:1bae` | mouse | Bragi_mouse | wireless |
| KATAR PRO XT | `1b1c:1bac` | mouse | Bragi_mouse |  |
| M55 | `1b1c:2b03` | mouse | Bragi_mouse |  |
| M55 RGB PRO | `1b1c:1b70` | mouse | Bragi_mouse |  |
| M55 WIRELESS | `1b1c:2b1e` | mouse | Bragi_mouse | wireless |
| M55 WIRELESS | `1b1c:1bfb` | mouse | Bragi_mouse | wireless |
| M65 RGB ULTRA | `1b1c:1bef` | mouse | Bragi_mouse |  |
| M65 RGB ULTRA | `1b1c:1b9e` | mouse | Bragi_mouse |  |
| M65 RGB ULTRA Wireless | `1b1c:1bd6` | mouse | Bragi_mouse | wireless |
| M65 RGB ULTRA Wireless | `1b1c:1bb5` | mouse | Bragi_mouse | wireless |
| M75 | `1b1c:1bf0` | mouse | Bragi_mouse |  |
| M75 AIR WIRELESS | `1b1c:1bf2` | mouse | Bragi_mouse | wireless |
| M75 WIRELESS | `1b1c:2b08` | mouse | Bragi_mouse | wireless |
| M75 WIRELESS | `1b1c:1bf1` | mouse | Bragi_mouse | wireless |
| Nightsabre Wireless | `1b1c:1bb8` | mouse | Bragi_mouse | wireless |
| SABRE PRO CHAMPION SERIES | `1b1c:1b7a` | mouse | Bragi_mouse |  |
| SABRE RGB PRO CHAMPION SERIES | `1b1c:1b79` | mouse | Bragi_mouse |  |
| SABRE RGB PRO WIRELESS | `1b1c:1b98` | mouse | Bragi_mouse | wireless |
| SCIMITAR ELITE WIRELESS SE | `1b1c:2b22` | mouse | Bragi_mouse | wireless |
| SCIMITAR RGB ELITE | `1b1c:1be3` | mouse | Bragi_mouse |  |
| SCIMITAR RGB ELITE WIRELESS | `1b1c:1bdb` | mouse | Bragi_mouse | wireless |
| MM700 RGB | `1b1c:1bc9` | mouse-pad | Bragi_mousemat |  |
| MM700 RGB | `1b1c:1b9b` | mouse-pad | Bragi_mousemat |  |
| HX1000i SHIFT | `-001:-001` | power-supply | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| HX1000i SHIFT CRYSTAL | `-001:-001` | power-supply | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| HX1200i SHIFT | `-001:-001` | power-supply | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |
| HX1500i SHIFT | `-001:-001` | power-supply | Bragi_link_ecosystem_subdevice | acts as LinkSystemSubdevice |

## Legacy — older keyboards, mice and headsets

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| GENERIC HEADSET | `170d:0100` | headset | Generic_wireless_headset |  |
| HS35 SURROUND V2 | `1b1c:2a0a` | headset | Headset |  |
| HS45 SURROUND | `1b1c:0a58` | headset | Headset |  |
| HS55 SURROUND | `1b1c:0a86` | headset | Headset |  |
| HS60 HAPTIC | `1b1c:0a5f` | headset | Headset |  |
| HS60 PRO SURROUND | `1b1c:0a4d` | headset | Headset |  |
| HS60 SURROUND | `1b1c:0a3b` | headset | Headset |  |
| HS65 SURROUND | `1b1c:0a87` | headset | Headset |  |
| HS70 BLUETOOTH | `1b1c:0a61` | headset | Headset |  |
| HS70 PRO WIRELESS | `1b1c:0a4f` | headset | Headset | wireless |
| HS70 WIRELESS | `1b1c:0a38` | headset | Headset | wireless |
| HS80 RGB USB | `1b1c:0a88` | headset | Headset |  |
| VOID ELITE SURROUND | `1b1c:0a53` | headset | Headset |  |
| VOID ELITE SURROUND | `1b1c:0a57` | headset | Headset |  |
| VOID ELITE USB | `1b1c:0a52` | headset | Headset |  |
| VOID ELITE USB | `1b1c:0a56` | headset | Headset |  |
| VOID ELITE WIRELESS | `1b1c:0a51` | headset | Headset | wireless |
| VOID ELITE WIRELESS | `1b1c:0a75` | headset | Headset | wireless |
| VOID ELITE WIRELESS | `1b1c:0a55` | headset | Headset | wireless |
| VOID PRO SURROUND | `1b1c:0a18` | headset | Headset |  |
| VOID PRO SURROUND | `1b1c:0a1e` | headset | Headset |  |
| VOID PRO USB | `1b1c:0a17` | headset | Headset |  |
| VOID PRO USB | `1b1c:0a1d` | headset | Headset |  |
| VOID PRO USB IG | `1b1c:0a1f` | headset | Headset |  |
| VOID PRO WIRELESS | `1b1c:0a14` | headset | Headset | wireless |
| VOID PRO WIRELESS | `1b1c:0a16` | headset | Headset | wireless |
| VOID PRO WIRELESS | `1b1c:0a1a` | headset | Headset | wireless |
| VOID SURROUND | `1b1c:0a31` | headset | Headset |  |
| VOID SURROUND | `1b1c:0a30` | headset | Headset |  |
| VOID USB | `1b1c:1b2a` | headset | Headset |  |
| VOID USB | `1b1c:1b29` | headset | Headset |  |
| VOID WIRELESS | `1b1c:1b27` | headset | Headset | wireless |
| VOID WIRELESS | `1b1c:0a2b` | headset | Headset | wireless |
| VOID WIRELESS | `1b1c:1b23` | headset | Headset | wireless |
| ST100 RGB | `1b1c:0a36` | headset-stand | Headset_stand |  |
| K55 RGB | `1b1c:1b3d` | keyboard | Nxp2000_kbd |  |
| K63 | `1b1c:1b40` | keyboard | Nxp2000_kbd |  |
| K63 WIRELESS | `1b1c:1b8c`, `1b1c:1b8f` | keyboard | Nxp2000_kbd_rf | wireless |
| K63 WIRELESS | `1b1c:1b45`, `1b1c:1b50` | keyboard | Nxp2000_kbd_rf | wireless |
| K65 LUX RGB | `1b1c:1b37` | keyboard | Nxp2000_kbd |  |
| K65 RGB | `1b1c:1b17` | keyboard | Nxp2000_kbd |  |
| K65 RGB RAPIDFIRE | `1b1c:1b39` | keyboard | Nxp2000_kbd |  |
| K66 | `1b1c:1b41` | keyboard | Nxp2000_kbd |  |
| K68 | `1b1c:1b3f` | keyboard | Nxp2000_kbd |  |
| K68 RGB | `1b1c:1b4f` | keyboard | Nxp2000_kbd |  |
| K68 SE | `1b1c:1ba5` | keyboard | Nxp2000_kbd |  |
| K70 LUX | `1b1c:1b36` | keyboard | Nxp2000_kbd |  |
| K70 LUX RGB | `1b1c:1b33` | keyboard | Nxp2000_kbd |  |
| K70 RAPIDFIRE | `1b1c:1b3a` | keyboard | Nxp2000_kbd |  |
| K70 RGB | `1b1c:1b13` | keyboard | Nxp2000_kbd |  |
| K70 RGB MK.2 | `1b1c:1b49` | keyboard | Nxp2000_kbd |  |
| K70 RGB MK.2 IG | `1b1c:1b87` | keyboard | Nxp2000_kbd |  |
| K70 RGB MK.2 LP | `1b1c:1b55` | keyboard | Nxp2000_kbd |  |
| K70 RGB MK.2 SE | `1b1c:1b6b` | keyboard | Nxp2000_kbd |  |
| K70 RGB RAPIDFIRE | `1b1c:1b38` | keyboard | Nxp2000_kbd |  |
| K70 RGB SE RAPIDFIRE | `1b1c:1b59` | keyboard | Nxp2000_kbd |  |
| K83 WIRELESS | `1b1c:1b42`, `1b1c:1b6d` | keyboard | Nxp2000_kbd_rf | wireless |
| K95 RGB | `1b1c:1b11` | keyboard | Nxp2000_kbd |  |
| K95 RGB PLATINUM | `1b1c:1b2d` | keyboard | Nxp2000_kbd |  |
| K95 RGB PLATINUM SE | `1b1c:1b82` | keyboard | Nxp2000_kbd |  |
| STRAFE | `1b1c:1b15` | keyboard | Nxp2000_kbd |  |
| STRAFE | `1b1c:1b44` | keyboard | Nxp2000_kbd |  |
| STRAFE RGB | `1b1c:1b20` | keyboard | Nxp2000_kbd |  |
| STRAFE RGB MK.2 | `1b1c:1b48` | keyboard | Nxp2000_kbd |  |
| DARK CORE RGB | `1b1c:1b35`, `1b1c:1b64` | mouse | Nxp2000_mouse_rf | wireless |
| DARK CORE SE | `1b1c:1b4b`, `1b1c:1b51` | mouse | Nxp2000_mouse_rf | wireless |
| GLAIVE RGB | `1b1c:1b34` | mouse | Nxp2000_mouse |  |
| GLAIVE RGB PRO | `1b1c:1b74` | mouse | Nxp2000_mouse |  |
| HARPOON RGB | `1b1c:1b3c` | mouse | Nxp2000_mouse |  |
| HARPOON RGB PRO | `1b1c:1b75` | mouse | Nxp2000_mouse |  |
| IRONCLAW RGB | `1b1c:1b5d` | mouse | Nxp2000_mouse |  |
| KATAR | `1b1c:1b22` | mouse | Nxp2000_mouse |  |
| M65 PRO RGB | `1b1c:1b2e` | mouse | Nxp2000_mouse |  |
| M65 RGB | `1b1c:1b12` | mouse | Nxp2000_mouse |  |
| M65 RGB ELITE | `1b1c:1b5a` | mouse | Nxp2000_mouse |  |
| M65 RGB ELITE IG | `1b1c:1b88` | mouse | Nxp2000_mouse |  |
| NIGHTSWORD RGB | `1b1c:1b5c` | mouse | Nxp2000_mouse |  |
| SABRE | `1b1c:1b32` | mouse | Nxp2000_mouse |  |
| SABRE RGB | `1b1c:1b2f` | mouse | Nxp2000_mouse |  |
| SABRE RGB LASER | `1b1c:1b19` | mouse | Nxp2000_mouse |  |
| SABRE RGB OPTICAL | `1b1c:1b14` | mouse | Nxp2000_mouse |  |
| SCIMITAR | `1b1c:1b1e` | mouse | Nxp2000_mouse |  |
| SCIMITAR PRO RGB | `1b1c:1b3e` | mouse | Nxp2000_mouse |  |
| SCIMITAR RGB ELITE | `1b1c:1b8b` | mouse | Nxp2000_mouse |  |
| MM800 RGB POLARIS | `1b1c:1b3b` | mouse-pad | Nxp2000_mousemat |  |

## CLink — older Commander / Platinum coolers / PSUs / Lighting Node

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| Commander PRO | `1b1c:0c10` | led-controller | ClinkLightingNode | iCUE LINK, acts as CommanderPro |
| COMMANDER PRO | `1b1c:0c11` | led-controller | ClinkLightingNode | iCUE LINK |
| LIGHTING NODE CORE | `1b1c:0c1b` | led-controller | ClinkLightingNode | iCUE LINK |
| LIGHTING NODE CORE | `1b1c:210000`, `1b1c:0c1a` | led-controller | ClinkLightingNode | iCUE LINK |
| LIGHTING NODE PRO | `1b1c:0c0c` | led-controller | ClinkLightingNode | iCUE LINK |
| LIGHTING NODE PRO | `1b1c:0c0b` | led-controller | ClinkLightingNode | iCUE LINK |
| LS100 STARTER KIT | `1b1c:0c1f` | led-controller | ClinkLightingNode | iCUE LINK |
| LS100 STARTER KIT | `1b1c:0c1e` | led-controller | ClinkLightingNode | iCUE LINK |
| LT100 STARTER KIT | `1b1c:0c24` | led-controller | ClinkLightingNode | iCUE LINK |
| LT100 STARTER KIT | `1b1c:0c23` | led-controller | ClinkLightingNode | iCUE LINK, acts as LT100Tower |
| OBSIDIAN 1000D | `1b1c:1d01` | led-controller | ClinkLightingNode | iCUE LINK |
| OBSIDIAN 1000D | `1b1c:1d00` | led-controller | ClinkLightingNode | iCUE LINK, acts as CommanderPro |
| ORIGIN PC CHRONOS | `1b1c:220001` | led-controller | ClinkLightingNode | iCUE LINK |
| ORIGIN PC PRIME | `1b1c:220002` | led-controller | ClinkLightingNode | iCUE LINK |
| SPEC-OMEGA RGB | `1b1c:1d05` | led-controller | ClinkLightingNode | iCUE LINK |
| SPEC-OMEGA RGB | `1b1c:1d04` | led-controller | ClinkLightingNode | iCUE LINK |
| VENGEANCE PC | `1b1c:210002` | led-controller | ClinkLightingNode | iCUE LINK |
| VENGEANCE PC | `1b1c:210001` | led-controller | ClinkLightingNode | iCUE LINK |
| CORSAIR ONE | `1b1c:200001` | liquid-cooler | ClinkCorsairOne | iCUE LINK |
| CORSAIR ONE PRO | `1b1c:200002` | liquid-cooler | ClinkCorsairOne | iCUE LINK |
| H100i ELITE | `1b1c:0c35` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i ELITE WHITE | `1b1c:0c40` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i PLATINUM | `1b1c:0c18` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i PLATINUM SE | `1b1c:0c19` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i PRO | `1b1c:0c15` | liquid-cooler | ClinkStarCoolers | iCUE LINK |
| H100i RGB PRO XT | `1b1c:0c2d` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i RGB PRO XT | `1b1c:0c20` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H100i v2 | `1b1c:0c09` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H100iGTX | `1b1c:0c03` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H110i | `1b1c:c0442` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H110iGT | `1b1c:c0441` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H110iGTX | `1b1c:0c07` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H115i | `1b1c:0c0a` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H115i ELITE | `1b1c:0c36` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H115i PLATINUM | `1b1c:0c17` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H115i PRO | `1b1c:0c13` | liquid-cooler | ClinkStarCoolers | iCUE LINK |
| H115i RGB PRO XT | `1b1c:0c2e` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H115i RGB PRO XT | `1b1c:0c21` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H150i ELITE | `1b1c:0c37` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H150i ELITE WHITE | `1b1c:0c41` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H150i PRO | `1b1c:0c12` | liquid-cooler | ClinkStarCoolers | iCUE LINK |
| H150i RGB PRO XT | `1b1c:0c2f` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H150i RGB PRO XT | `1b1c:0c22` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H60i RGB PRO XT | `1b1c:0c30` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H60i RGB PRO XT | `1b1c:0c29` | liquid-cooler | ClinkPlatinumCoolers | iCUE LINK |
| H80i v2 | `1b1c:0c08` | liquid-cooler | ClinkGeneric | iCUE LINK |
| H80iGT | `1b1c:0c02` | liquid-cooler | ClinkGeneric | iCUE LINK |
| AX1200i | `1b1c:1c0002`, `1b1c:1c00` | power-supply | ClinkPsu | iCUE LINK, clink lister |
| AX1500i | `1b1c:1c02`, `1b1c:1c0001`, `1b1c:1c0201`, `1b1c:1c00` | power-supply | ClinkPsu | iCUE LINK, clink lister |
| AX1600i | `1b1c:1c11`, `1b1c:1c1108` | power-supply | ClinkPsu | iCUE LINK, clink lister |
| AX1600i SHIFT | `-001:-001`, `1b1c:1c25` | power-supply | ClinkPsu | iCUE LINK |
| AX760i | `1b1c:1c0004`, `1b1c:1c00` | power-supply | ClinkPsu | iCUE LINK, clink lister |
| AX860i | `1b1c:1c0003`, `1b1c:1c00` | power-supply | ClinkPsu | iCUE LINK, clink lister |
| HX1000i | `1b1c:1c1e` | power-supply | ClinkPsu | iCUE LINK |
| HX1000i | `1b1c:1c07` | power-supply | ClinkPsu | iCUE LINK |
| HX1200i | `1b1c:1c23` | power-supply | ClinkPsu | iCUE LINK |
| HX1200i | `1b1c:1c27` | power-supply | ClinkPsu | iCUE LINK |
| HX1200i | `1b1c:1c08` | power-supply | ClinkPsu | iCUE LINK |
| HX1500i | `1b1c:1c1f` | power-supply | ClinkPsu | iCUE LINK |
| HX750i | `1b1c:1c05` | power-supply | ClinkPsu | iCUE LINK |
| HX850i | `1b1c:1c06` | power-supply | ClinkPsu | iCUE LINK |
| RM1000i | `1b1c:1c0d` | power-supply | ClinkPsu | iCUE LINK |
| RM650i | `1b1c:1c0a` | power-supply | ClinkPsu | iCUE LINK |
| RM750i | `1b1c:1c0b` | power-supply | ClinkPsu | iCUE LINK |
| RM850i | `1b1c:1c0c` | power-supply | ClinkPsu | iCUE LINK |

## LLAccess — memory (DRAM) via SMBus, needs a kernel driver

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| DOMINATOR PLATINUM RGB | `1b1c:0404`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| DOMINATOR PLATINUM RGB DDR5 | `1b1c:0408`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| DOMINATOR TITANIUM | `1b1c:040b`, `1b1c:0006` | dram | Dram | iCUE LINK |
| SHUGO SERIES DDR5 | `1b1c:040c`, `1b1c:0006` | dram | Dram | iCUE LINK |
| VENGEANCE DDR4 BLANK | `1b1c:03ff`, `1b1c:0006` | dram | Dram | llaccess lister |
| VENGEANCE DDR5 | `1b1c:0409`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| VENGEANCE LED | `1b1c:0401`, `1b1c:0006` | dram | Dram | iCUE LINK |
| VENGEANCE RGB | `1b1c:0402`, `1b1c:0006` | dram | Dram | iCUE LINK |
| VENGEANCE RGB DDR5 | `1b1c:040a`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| VENGEANCE RGB PRO | `1b1c:0403`, `1b1c:0006` | dram | Dram | iCUE LINK |
| VENGEANCE RGB PRO SL | `1b1c:0405`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| VENGEANCE RGB RS | `1b1c:0406`, `1b1c:0006` | dram | Dram | iCUE LINK |
| VENGEANCE RGB RS DDR5 | `1b1c:040d`, `1b1c:0006` | dram | Dram | iCUE LINK, llaccess lister |
| VENGEANCE RGB RT | `1b1c:0407`, `1b1c:0006` | dram | Dram | iCUE LINK |

## Touchscreen

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| Elite LCD | `1b1c:0c39` | pump-display | PumpDisplay |  |
| Elite LCD | `1b1c:0c33` | pump-display | PumpDisplay |  |
| iCUE LINK LCD Cap for LINK coolers | `1b1c:0c4e` | pump-display | PumpDisplay |  |
| iCUE LINK XC7 ELITE LCD | `1b1c:0c42` | pump-display | PumpDisplay_al021 |  |
| iCUE LINK XD ELITE LCD SCREEN | `1b1c:0c43` | pump-display | PumpDisplay |  |
| NAUTILUS LCD CAP | `1b1c:0c57` | pump-display | PumpDisplay_al021 |  |
| NAUTILUS LCD CAP | `1b1c:0c55` | pump-display | PumpDisplay_al021 |  |
| iCUE NEXUS | `1b1c:1b8e` | touchscreen | Touchscreen |  |
| S-Key Macro Bar | `1b1c:0d02` | touchscreen | Touchscreen_bs010 |  |

## Gip

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| HS80 MAX WIRELESS FOR XBOX | `1b1c:2a0b`, `1b1c:2a0c`, `1b1c:2a0d` | headset |  |  |
| HS80 MAX WIRELESS FOR XBOX | `1b1c:2a0b`, `1b1c:2a0c`, `1b1c:2a0d` | headset |  |  |
| VOID MAX WIRELESS FOR XBOX | `1b1c:2a05`, `1b1c:2a06`, `1b1c:2a07` | headset |  |  |
| VOID MAX WIRELESS FOR XBOX | `1b1c:2a05`, `1b1c:2a06`, `1b1c:2a07` | headset |  |  |

## Other

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| iCUE LINK XD5 ELITE LCD | — | led-controller |  |  |
| ORIGIN OA | — | led-controller |  |  |
| iCUE LINK XD5 ELITE LCD SCREEN | `1b1c:0c43` | pump-display |  |  |

## MccsUsb

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| XENEON 32QHD165 | `1b1c:f10e` | display | MccsUsb |  |
| XENEON 32QHD240 | `1b1c:0d08` | display | MccsUsb |  |
| XENEON 32UHD144 | `1b1c:0d05` | display | MccsUsb |  |

## DashAccessory

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| XENEON EDGE | `1b1c:1d0d` | dashboardLCD | DashAccessory |  |

## Govee

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| GOVEE | `-402:-402` | govee | Govee |  |

## AvneraAccessory

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| HS75 XB WIRELESS | `1b1c:0a5d`, `170d:0526` | headset | AvneraAccessory |  |

## Nanoleaf

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| NANOLEAF | `-401:-401` | nanoleaf | Nanoleaf |  |

## PhilipsHue

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| PHILIPS HUE | `-400:-400` | philips-hue | PhilipsHue |  |

## Edid

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| NAUTILUS II LCD | `-001:-001` | dashboardLCD |  |  |

## Uniwill

| Device | USB ID | type | HID protocol | notes |
|---|---|---|---|---|
| VOYAGER A1600 COOLER | `1b1c:0d19` | cooler | Uniwill_cooler |  |
