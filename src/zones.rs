//! What's connected to the RGB ports (the "layout") and the named parts ("zones").
//!
//! LightingChannelsGeometries (Bragi file 0x1e): channel count, then per channel a
//! device count followed by one device-type byte per device. The type bytes and LED
//! counts come from iCUE's device manifest (commander_core_xt.rcc,
//! `LedChannelTypeToGeometryIndexMap` and `DynamicGeometryTemplates`). LED strips
//! (type 0x01) are written as one device whose LED total goes in file 0x1d.
//!
//! Without a [layout] in the config, the layout is this build's (what iCUE found
//! stored in the controller): external port = 6 QL-fan slots of which the 4 LED
//! strips use the first 40 LEDs, ports 1-2 = 8-LED fans, port 3 = pump cap. Its part
//! names were mapped with a colour test.
//!
//! On a Commander CORE (experimental) port 0 is the pump's own ring and ports 1-6 take
//! fans; without a [layout] the layout is built from what the controller detects.

use indexmap::IndexMap;

use crate::model::Model;
use std::ops::Range;
use std::sync::{Arc, RwLock};

/// Device kinds that can be connected to a lighting port.
pub struct Kind {
    pub id: &'static str,
    /// Geometry type byte (iCUE's LedChannelTypeToGeometryIndexMap).
    pub byte: u8,
    pub leds: usize,
    pub label: &'static str,
}

pub const KINDS: [Kind; 16] = [
    Kind { id: "rgb-strip", byte: 0x01, leds: 10, label: "RGB LED strip (10 LEDs)" },
    Kind { id: "strip-15", byte: 0x01, leds: 15, label: "LED strip, 15 LEDs" },
    Kind { id: "strip-21", byte: 0x01, leds: 21, label: "LED strip, 21 LEDs" },
    Kind { id: "strip-27", byte: 0x01, leds: 27, label: "LED strip, 27 LEDs" },
    Kind { id: "strip-40", byte: 0x01, leds: 40, label: "LED strip, 40 LEDs" },
    Kind { id: "strip-49", byte: 0x01, leds: 49, label: "LED strip, 49 LEDs" },
    Kind { id: "strip-84", byte: 0x01, leds: 84, label: "LED strip, 84 LEDs" },
    Kind { id: "triangle-9", byte: 0x01, leds: 9, label: "LED triangle (9 LEDs)" },
    Kind { id: "ll-fan", byte: 0x02, leds: 16, label: "LL fan (16 LEDs)" },
    Kind { id: "ml-fan", byte: 0x03, leds: 4, label: "ML fan (4 LEDs)" },
    Kind { id: "hd-fan", byte: 0x04, leds: 12, label: "HD fan (12 LEDs)" },
    Kind { id: "8-led-fan", byte: 0x05, leds: 8, label: "8-LED fan" },
    Kind { id: "ql-fan", byte: 0x06, leds: 34, label: "QL fan (34 LEDs)" },
    Kind { id: "pump-cap", byte: 0x0c, leds: 13, label: "13-LED pump cap" },
    // Commander CORE port 0 only. The CAPELLIX ring is CoolerPump21Leds: 21 lights on the
    // pump picture, 29 LEDs in the frame (the manifest maps the first 9 to the centre light).
    Kind { id: "capellix-pump", byte: 0x08, leds: 29, label: "ELITE CAPELLIX pump (29 LEDs)" },
    Kind { id: "lcd-pump-ring", byte: 0x0e, leds: 24, label: "ELITE LCD pump ring (24 LEDs)" },
];

const PUMP_RINGS: [u8; 2] = [0x08, 0x0e];

pub fn kind(id: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.id == id)
}

/// Best guess for an auto-detected LED count on ports 1-6.
pub fn kind_for_leds(leds: usize) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.byte != 0x01 && !PUMP_RINGS.contains(&k.byte) && k.leds == leds)
}

/// Can `k` go on port `i` of `model`?
fn allowed(model: Model, i: usize, k: &Kind) -> bool {
    match model {
        Model::CoreXt => !PUMP_RINGS.contains(&k.byte),
        Model::Core if i == 0 => PUMP_RINGS.contains(&k.byte),
        Model::Core => (0x02..=0x06).contains(&k.byte),
    }
}

/// The Core XT's port names ("external", then "port1" … "port6"); see Model::ports.
pub const PORTS: [&str; 7] = ["external", "port1", "port2", "port3", "port4", "port5", "port6"];

/// Parts that aren't on the Corsair controller: "motherboard" is sent through OpenRGB.
pub const EXTERNAL_ZONES: [&str; 1] = ["motherboard"];

/// Parts shown in the window's picture of this build (one colour each).
pub const PICTURE_ZONES: [&str; 7] = ["strip1", "strip2", "strip3", "strip4", "right-fan", "left-fan", "pump"];

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub model: Model,
    /// Devices on each of the 7 ports.
    pub ports: Vec<Vec<&'static Kind>>,
    /// Part name → selector: "port1", "external2" (2nd device), "external:0-9" (LEDs of a port), joined with "+".
    pub names: IndexMap<String, String>,
}

impl PartialEq for Kind {
    fn eq(&self, other: &Kind) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.id)
    }
}

impl Layout {
    /// This build (see the module comment).
    pub fn this_build() -> Layout {
        let k = |id| kind(id).unwrap();
        let names = [
            ("strip1", "external:0-9"),
            ("strip2", "external:10-19"),
            ("strip3", "external:20-29"),
            ("strip4", "external:30-39"),
            ("strips", "external:0-39"),
            ("right-fan", "port1"),
            ("left-fan", "port2"),
            ("fans", "port1+port2"),
            ("pump", "port3"),
        ];
        Layout {
            model: Model::CoreXt,
            ports: vec![vec![k("ql-fan"); 6], vec![k("8-led-fan")], vec![k("8-led-fan")], vec![k("pump-cap")], vec![], vec![], vec![]],
            names: names.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
        }
    }

    /// What a Commander CORE reports (file 0x20: status, LED count per port; 2 = found).
    /// Port 0 holds the pump ring: 24 LEDs means the ELITE LCD one, anything else the CAPELLIX one.
    /// On the Core XT this is this build's layout (its external port isn't detected).
    pub fn detected(model: Model, found: &[(u16, u16)]) -> Layout {
        if model == Model::CoreXt {
            return Layout::this_build();
        }
        let get = |i: usize| found.get(i).copied().unwrap_or((0, 0));
        let mut ports = vec![Vec::new(); 7];
        match get(0) {
            (3, _) => {}
            (2, 24) => ports[0].push(kind("lcd-pump-ring").unwrap()),
            _ => ports[0].push(kind("capellix-pump").unwrap()),
        }
        for (i, port) in ports.iter_mut().enumerate().skip(1) {
            if let (2, n) = get(i) {
                port.extend(kind_for_leds(n as usize));
            }
        }
        let mut names = IndexMap::new();
        if !ports[0].is_empty() {
            names.insert("pump".to_string(), "pump".to_string());
        }
        let fans: Vec<&str> = (1..7).filter(|i| !ports[*i].is_empty()).map(|i| model.ports()[i]).collect();
        if fans.len() > 1 {
            names.insert("all-fans".to_string(), fans.join("+"));
        }
        for f in fans {
            names.insert(f.to_string(), f.to_string());
        }
        Layout { model, ports, names }
    }

    /// From the config's [layout] table (port name → device ids, plus `names`).
    pub fn from_config(model: Model, ports: &IndexMap<String, Vec<String>>, names: &IndexMap<String, String>) -> Result<Layout, String> {
        let port_names = model.ports();
        let mut out = vec![Vec::new(); port_names.len()];
        for (port, devices) in ports {
            let i = port_names
                .iter()
                .position(|p| p == port)
                .ok_or(format!("layout: unknown port '{port}' on a {} (use {})", model.name(), port_names.join(", ")))?;
            for d in devices {
                let k = kind(d).ok_or(format!(
                    "layout: unknown device '{d}' (use {})",
                    KINDS.iter().map(|k| k.id).collect::<Vec<_>>().join(", ")
                ))?;
                if !allowed(model, i, k) {
                    return Err(format!("layout: a {d} can't go on {port} of a {}", model.name()));
                }
                out[i].push(k);
            }
            let strips = out[i].iter().filter(|k| k.byte == 0x01).count();
            if strips > 0 && strips != out[i].len() {
                return Err(format!("layout: {port} mixes LED strips with other devices; the controller can't"));
            }
        }
        let layout = Layout { model, ports: out, names: names.clone() };
        for (name, sel) in &layout.names {
            // a port's own name may only stand for that port
            if (port_names.contains(&name.as_str()) && sel != name) || name == "all" {
                return Err(format!("layout: '{name}' is already a part name"));
            }
            layout.select(sel).map_err(|e| format!("layout name '{name}': {e}"))?;
        }
        Ok(layout)
    }

    /// As a [layout] table (port name → device ids, plus names).
    pub fn to_config(&self) -> crate::config::LayoutConfig {
        crate::config::LayoutConfig {
            names: self.names.clone(),
            ports: self
                .ports
                .iter()
                .enumerate()
                .filter(|(_, d)| !d.is_empty())
                .map(|(i, d)| (self.model.ports()[i].to_string(), d.iter().map(|k| k.id.to_string()).collect()))
                .collect(),
        }
    }

    /// Frame slice of each port, in frame order.
    pub fn port_ranges(&self) -> Vec<Range<usize>> {
        let mut start = 0;
        self.ports
            .iter()
            .map(|devs| {
                let n: usize = devs.iter().map(|k| k.leds).sum();
                start += n;
                start - n..start
            })
            .collect()
    }

    pub fn frame_len(&self) -> usize {
        self.port_ranges().last().map_or(0, |r| r.end)
    }

    /// File 0x1e data: geometry of every port.
    pub fn geometry(&self) -> Vec<u8> {
        let mut g = vec![self.ports.len() as u8];
        for devs in &self.ports {
            if devs.first().is_some_and(|k| k.byte == 0x01) {
                g.extend_from_slice(&[1, 0x01]); // a strip chain is one device; its LEDs go in 0x1d
            } else {
                g.push(devs.len() as u8);
                g.extend(devs.iter().map(|k| k.byte));
            }
        }
        g
    }

    /// File 0x1d data (LED totals of strip ports), only when a port has LED strips.
    pub fn strip_quantities(&self) -> Option<Vec<u8>> {
        if !self.ports.iter().any(|d| d.first().is_some_and(|k| k.byte == 0x01)) {
            return None;
        }
        let mut q = vec![self.ports.len() as u8];
        for devs in &self.ports {
            let n: usize = if devs.first().is_some_and(|k| k.byte == 0x01) { devs.iter().map(|k| k.leds).sum() } else { 0 };
            q.extend_from_slice(&(n as u16).to_le_bytes());
        }
        Some(q)
    }

    /// LED indices for one selector term: "port1", "external2", "external:0-9".
    fn term(&self, t: &str) -> Result<Vec<usize>, String> {
        let ranges = self.port_ranges();
        let ports = self.model.ports();
        if let Some((port, span)) = t.split_once(':') {
            let i = ports.iter().position(|p| *p == port).ok_or(format!("unknown port '{port}'"))?;
            let (a, b) = span.split_once('-').ok_or("use port:FIRST-LAST")?;
            let (a, b): (usize, usize) = (a.parse().map_err(|_| "bad LED number")?, b.parse().map_err(|_| "bad LED number")?);
            let r = &ranges[i];
            if a > b || r.start + b >= r.end {
                return Err(format!("{port} has only {} LEDs", r.len()));
            }
            return Ok((r.start + a..=r.start + b).collect());
        }
        if let Some(i) = ports.iter().position(|p| *p == t) {
            return Ok(ranges[i].clone().collect());
        }
        for (i, port) in ports.iter().enumerate() {
            if let Some(n) = t.strip_prefix(port).and_then(|n| n.parse::<usize>().ok()) {
                let devs = &self.ports[i];
                if n == 0 || n > devs.len() {
                    return Err(format!("{port} has {} devices", devs.len()));
                }
                let start = ranges[i].start + devs[..n - 1].iter().map(|k| k.leds).sum::<usize>();
                return Ok((start..start + devs[n - 1].leds).collect());
            }
        }
        Err(format!("unknown part '{t}'"))
    }

    fn select(&self, sel: &str) -> Result<Vec<usize>, String> {
        let mut out = Vec::new();
        for t in sel.split('+') {
            out.extend(self.term(t.trim())?);
        }
        Ok(out)
    }

    /// Generic part names: each port with devices, and each device on ports with several.
    fn generic(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (i, devs) in self.ports.iter().enumerate() {
            if devs.is_empty() {
                continue;
            }
            let port = self.model.ports()[i];
            out.push(port.to_string());
            if devs.len() > 1 {
                out.extend((1..=devs.len()).map(|n| format!("{port}{n}")));
            }
        }
        out
    }

    /// Every part name, "all" last. With `names` set, those replace the generic ones.
    pub fn zone_names(&self) -> Vec<String> {
        let mut out: Vec<String> = if self.names.is_empty() { self.generic() } else { self.names.keys().cloned().collect() };
        out.extend(EXTERNAL_ZONES.iter().map(|s| s.to_string()));
        out.push("all".into());
        out
    }

    /// LED indices of a part. "all" runs through the named parts in order (no gaps,
    /// no repeats), or through every port when there are no names.
    pub fn zone(&self, name: &str) -> Vec<usize> {
        if name == "all" {
            let mut out = Vec::new();
            let parts: Vec<String> =
                if self.names.is_empty() { self.model.ports().iter().map(|p| p.to_string()).collect() } else { self.names.keys().cloned().collect() };
            for p in parts {
                for i in self.zone(&p) {
                    if !out.contains(&i) {
                        out.push(i);
                    }
                }
            }
            return out;
        }
        if let Some(sel) = self.names.get(name) {
            return self.select(sel).unwrap_or_default();
        }
        if self.names.is_empty() {
            return self.select(name).unwrap_or_default();
        }
        Vec::new()
    }
}

static CURRENT: RwLock<Option<Arc<Layout>>> = RwLock::new(None);

/// The layout in use (set when the config is loaded; this build's until then).
pub fn current() -> Arc<Layout> {
    if let Some(l) = CURRENT.read().unwrap().as_ref() {
        return l.clone();
    }
    Arc::new(Layout::this_build())
}

pub fn set_current(layout: Layout) {
    *CURRENT.write().unwrap() = Some(Arc::new(layout));
}

pub fn frame_len() -> usize {
    current().frame_len()
}

pub fn zone(name: &str) -> Vec<usize> {
    current().zone(name)
}

pub fn zone_names() -> Vec<String> {
    current().zone_names()
}

/// A part's name for the interface.
pub fn label(name: &str) -> String {
    let known = match name {
        "strip1" => "LED strip 1",
        "strip2" => "LED strip 2",
        "strip3" => "LED strip 3",
        "strip4" => "LED strip 4",
        "strips" => "All LED strips",
        "right-fan" => "Right fan",
        "left-fan" => "Left fan",
        "fans" => "Both radiator fans",
        "all-fans" => "All fans",
        "pump" => "Pump head",
        "motherboard" => "Motherboard (via OpenRGB)",
        "all" => "Everything",
        "external" => "External RGB port",
        _ => "",
    };
    if !known.is_empty() {
        return crate::i18n::tr(known).to_string();
    }
    if let Some(n) = name.strip_prefix("port").and_then(|n| n.parse::<usize>().ok()) {
        return crate::i18n::trf("RGB port {}", &[&n]);
    }
    if let Some(n) = name.strip_prefix("external") {
        return crate::i18n::trf("External device {}", &[&n]);
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The geometry and parts this build has used since 0.1 (verified on the hardware).
    const PORT_CONFIG: [u8; 17] = [
        0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x01, 0x05, 0x01, 0x05, 0x01, 0x0c, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn default_layout_is_unchanged() {
        let l = Layout::this_build();
        assert_eq!(l.geometry(), PORT_CONFIG.to_vec());
        assert_eq!(l.strip_quantities(), None);
        assert_eq!(l.frame_len(), 233);
        assert_eq!(l.zone("all").len(), 69);
        assert_eq!(l.zone("all")[..40], (0..40).collect::<Vec<_>>()[..]);
        assert_eq!(l.zone("strip2"), (10..20).collect::<Vec<_>>());
        assert_eq!(l.zone("right-fan"), (204..212).collect::<Vec<_>>());
        assert_eq!(l.zone("left-fan"), (212..220).collect::<Vec<_>>());
        assert_eq!(l.zone("fans"), (204..220).collect::<Vec<_>>());
        assert_eq!(l.zone("pump"), (220..233).collect::<Vec<_>>());
        assert_eq!(
            l.zone_names(),
            ["strip1", "strip2", "strip3", "strip4", "strips", "right-fan", "left-fan", "fans", "pump", "motherboard", "all"]
        );
    }

    #[test]
    fn custom_layout_with_strips() {
        let ports: IndexMap<String, Vec<String>> = [
            ("external".to_string(), vec!["rgb-strip".to_string(); 4]),
            ("port1".to_string(), vec!["ll-fan".to_string(), "ll-fan".to_string()]),
        ]
        .into();
        let l = Layout::from_config(Model::CoreXt, &ports, &IndexMap::new()).unwrap();
        assert_eq!(l.geometry(), vec![7, 1, 0x01, 2, 0x02, 0x02, 0, 0, 0, 0, 0]);
        assert_eq!(l.strip_quantities().unwrap()[..3], [7, 40, 0]);
        assert_eq!(l.frame_len(), 72);
        assert_eq!(l.zone("port12"), (56..72).collect::<Vec<_>>());
        assert!(l.zone_names().contains(&"external3".to_string()));
        let bad: IndexMap<String, Vec<String>> = [("port1".to_string(), vec!["rgb-strip".to_string(), "ll-fan".to_string()])].into();
        assert!(Layout::from_config(Model::CoreXt, &bad, &IndexMap::new()).is_err());
    }

    #[test]
    fn commander_core_layouts() {
        // pump ring found, 8-LED fans on ports 1 and 2, nothing on 3-6
        let found = [(2, 29), (2, 8), (2, 8), (3, 0), (3, 0), (3, 0), (3, 0)];
        let l = Layout::detected(Model::Core, &found);
        assert_eq!(l.geometry(), vec![7, 1, 0x08, 1, 0x05, 1, 0x05, 0, 0, 0, 0]);
        assert_eq!(l.strip_quantities(), None);
        assert_eq!(l.frame_len(), 45);
        assert_eq!(l.zone("pump"), (0..29).collect::<Vec<_>>());
        assert_eq!(l.zone("port2"), (37..45).collect::<Vec<_>>());
        assert_eq!(l.zone("all-fans"), (29..45).collect::<Vec<_>>());
        assert_eq!(l.zone_names(), ["pump", "all-fans", "port1", "port2", "motherboard", "all"]);
        assert_eq!(l.zone("all").len(), 45);
        assert_eq!(l.zone("fans"), Vec::<usize>::new());
        let lcd = Layout::detected(Model::Core, &[(2, 24)]);
        assert_eq!(lcd.geometry()[..3], [7, 1, 0x0e]);

        let ports: IndexMap<String, Vec<String>> =
            [("pump".to_string(), vec!["lcd-pump-ring".to_string()]), ("port1".to_string(), vec!["ql-fan".to_string(); 3])].into();
        let names: IndexMap<String, String> = [("pump".to_string(), "pump".to_string())].into();
        assert_eq!(Model::for_ports(ports.keys()), Model::Core);
        let l = Layout::from_config(Model::Core, &ports, &names).unwrap();
        assert_eq!(l.frame_len(), 24 + 3 * 34);
        let bad: IndexMap<String, String> = [("pump".to_string(), "port1".to_string())].into();
        assert!(Layout::from_config(Model::Core, &ports, &bad).is_err());
        let strip: IndexMap<String, Vec<String>> = [("port1".to_string(), vec!["rgb-strip".to_string()])].into();
        assert!(Layout::from_config(Model::Core, &strip, &IndexMap::new()).is_err());
        let ring: IndexMap<String, Vec<String>> = [("port1".to_string(), vec!["capellix-pump".to_string()])].into();
        assert!(Layout::from_config(Model::CoreXt, &ring, &IndexMap::new()).is_err());
    }
}
