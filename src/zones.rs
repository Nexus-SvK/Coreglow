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

use indexmap::IndexMap;
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

pub const KINDS: [Kind; 14] = [
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
];

pub fn kind(id: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.id == id)
}

/// Best guess for an auto-detected LED count on ports 1-6.
pub fn kind_for_leds(leds: usize) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.byte != 0x01 && k.leds == leds)
}

/// "external", then "port1" … "port6" (controller order).
pub const PORTS: [&str; 7] = ["external", "port1", "port2", "port3", "port4", "port5", "port6"];

/// Parts that aren't on the Corsair controller: "motherboard" is sent through OpenRGB.
pub const EXTERNAL_ZONES: [&str; 1] = ["motherboard"];

/// Parts shown in the window's picture of this build (one colour each).
pub const PICTURE_ZONES: [&str; 7] = ["strip1", "strip2", "strip3", "strip4", "right-fan", "left-fan", "pump"];

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
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
            ports: vec![vec![k("ql-fan"); 6], vec![k("8-led-fan")], vec![k("8-led-fan")], vec![k("pump-cap")], vec![], vec![], vec![]],
            names: names.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
        }
    }

    /// From the config's [layout] table (port name → device ids, plus `names`).
    pub fn from_config(ports: &IndexMap<String, Vec<String>>, names: &IndexMap<String, String>) -> Result<Layout, String> {
        let mut out = vec![Vec::new(); PORTS.len()];
        for (port, devices) in ports {
            let i = PORTS.iter().position(|p| p == port).ok_or(format!("layout: unknown port '{port}' (use {})", PORTS.join(", ")))?;
            for d in devices {
                let k = kind(d).ok_or(format!(
                    "layout: unknown device '{d}' (use {})",
                    KINDS.iter().map(|k| k.id).collect::<Vec<_>>().join(", ")
                ))?;
                out[i].push(k);
            }
            let strips = out[i].iter().filter(|k| k.byte == 0x01).count();
            if strips > 0 && strips != out[i].len() {
                return Err(format!("layout: {port} mixes LED strips with other devices; the controller can't"));
            }
        }
        let layout = Layout { ports: out, names: names.clone() };
        for (name, sel) in &layout.names {
            if PORTS.contains(&name.as_str()) || name == "all" {
                return Err(format!("layout: '{name}' is already a part name"));
            }
            layout.select(sel).map_err(|e| format!("layout name '{name}': {e}"))?;
        }
        Ok(layout)
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
        if let Some((port, span)) = t.split_once(':') {
            let i = PORTS.iter().position(|p| *p == port).ok_or(format!("unknown port '{port}'"))?;
            let (a, b) = span.split_once('-').ok_or("use port:FIRST-LAST")?;
            let (a, b): (usize, usize) = (a.parse().map_err(|_| "bad LED number")?, b.parse().map_err(|_| "bad LED number")?);
            let r = &ranges[i];
            if a > b || r.start + b >= r.end {
                return Err(format!("{port} has only {} LEDs", r.len()));
            }
            return Ok((r.start + a..=r.start + b).collect());
        }
        if let Some(i) = PORTS.iter().position(|p| *p == t) {
            return Ok(ranges[i].clone().collect());
        }
        for (i, port) in PORTS.iter().enumerate() {
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
            out.push(PORTS[i].to_string());
            if devs.len() > 1 {
                out.extend((1..=devs.len()).map(|n| format!("{}{n}", PORTS[i])));
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
            let parts: Vec<String> = if self.names.is_empty() { PORTS.iter().map(|p| p.to_string()).collect() } else { self.names.keys().cloned().collect() };
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
        "pump" => "Pump head",
        "motherboard" => "Motherboard (via OpenRGB)",
        "all" => "Everything",
        "external" => "External RGB port",
        _ => "",
    };
    if !known.is_empty() {
        return crate::i18n::tr(known).to_string();
    }
    for (i, p) in PORTS.iter().enumerate().skip(1) {
        if name == *p {
            return crate::i18n::trf("RGB port {}", &[&i]);
        }
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
        let l = Layout::from_config(&ports, &IndexMap::new()).unwrap();
        assert_eq!(l.geometry(), vec![7, 1, 0x01, 2, 0x02, 0x02, 0, 0, 0, 0, 0]);
        assert_eq!(l.strip_quantities().unwrap()[..3], [7, 40, 0]);
        assert_eq!(l.frame_len(), 72);
        assert_eq!(l.zone("port12"), (56..72).collect::<Vec<_>>());
        assert!(l.zone_names().contains(&"external3".to_string()));
        let bad: IndexMap<String, Vec<String>> = [("port1".to_string(), vec!["rgb-strip".to_string(), "ll-fan".to_string()])].into();
        assert!(Layout::from_config(&bad, &IndexMap::new()).is_err());
    }
}
