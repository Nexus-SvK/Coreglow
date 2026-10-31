//! Profiles file (~/.config/coreglow/config.toml): lighting effects, per-part
//! overrides and fan curves. The same format the earlier Python daemon used.

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use crate::paths;
use crate::zones::ZONE_NAMES;

pub const BUILTIN_DEFAULT: &str = include_str!("../assets/config.toml");

const HEADER: &str = "\
# Commander Core XT profiles (coreglow). Edited by coreglow; editing by hand works too.
# Switch with: coreglowctl set NAME    List: coreglowctl list    Live values: coreglowctl status
# Saving applies the change within a second.
#
# lighting effects: static (color), off, breathing (color, period), flash (color, period),
#   color_shift (colors, period), rainbow (period), rainbow_wave (period, waves),
#   color_wave (colors = [a, b], period, waves),
#   temperature (sensor, stops = [[temp, colour], ...], alarm = degrees C to flash above).
#   Every effect also takes brightness (0-100). Colours are \"RRGGBB\", period is seconds.
# zones: strip1 ... strip4, strips, right-fan, left-fan, fans, pump, all
# fans: sensor + curve = [[temp, duty], ...], or fixed = duty. fan1 / fan2 override one fan.
#   The pump always runs at full speed while the lighting is driven from the PC.
# sensors: probe1, probe2 (controller probes), cpu, igpu (Ryzen graphics), gpu (needs nvidia-smi).
";

/// A number that may be written as 45 or 45.0 in TOML.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Num(pub f64);

impl<'de> Deserialize<'de> for Num {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Num;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Num, E> {
                Ok(Num(v as f64))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Num, E> {
                Ok(Num(v as f64))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Num, E> {
                Ok(Num(v))
            }
        }
        d.deserialize_any(V)
    }
}

impl Serialize for Num {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if self.0.fract() == 0.0 && self.0.abs() < 1e12 {
            s.serialize_i64(self.0 as i64)
        } else {
            s.serialize_f64((self.0 * 1000.0).round() / 1000.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    #[default]
    Static,
    Off,
    Breathing,
    Flash,
    ColorShift,
    Rainbow,
    RainbowWave,
    ColorWave,
    Temperature,
}

impl EffectKind {
    pub const ALL: [EffectKind; 9] = [
        EffectKind::Static,
        EffectKind::Breathing,
        EffectKind::Flash,
        EffectKind::ColorShift,
        EffectKind::Rainbow,
        EffectKind::RainbowWave,
        EffectKind::ColorWave,
        EffectKind::Temperature,
        EffectKind::Off,
    ];

    /// Effects that change over time on their own.
    pub fn moving(self) -> bool {
        matches!(
            self,
            EffectKind::Breathing
                | EffectKind::Flash
                | EffectKind::ColorShift
                | EffectKind::Rainbow
                | EffectKind::RainbowWave
                | EffectKind::ColorWave
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Effect {
    #[serde(default)]
    pub effect: EffectKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Num>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<Num>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stops: Option<Vec<(Num, String)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alarm: Option<Num>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waves: Option<Num>,
}

impl Effect {
    pub fn solid(color: &str) -> Self {
        Effect { effect: EffectKind::Static, color: Some(color.into()), ..Default::default() }
    }

    /// Switch effect kind with sensible defaults for the new one (keeps brightness).
    pub fn set_kind(&mut self, kind: EffectKind) {
        let brightness = self.brightness;
        let color = self.color.clone();
        *self = Effect { effect: kind, brightness, ..Default::default() };
        match kind {
            EffectKind::Static | EffectKind::Breathing | EffectKind::Flash => {
                self.color = Some(color.unwrap_or_else(|| "ffffff".into()))
            }
            EffectKind::ColorShift => self.colors = Some(vec!["ff0000".into(), "00ff00".into(), "0000ff".into()]),
            EffectKind::ColorWave => self.colors = Some(vec!["0040ff".into(), "00ffd0".into()]),
            EffectKind::Temperature => {
                self.sensor = Some("cpu".into());
                self.stops = Some(default_stops());
            }
            _ => {}
        }
        if kind.moving() {
            self.period = Some(Num(if kind == EffectKind::Flash { 1.0 } else { 4.0 }));
        }
    }
}

pub fn default_stops() -> Vec<(Num, String)> {
    vec![(Num(45.0), "00a0ff".into()), (Num(65.0), "ffff00".into()), (Num(80.0), "ff0000".into())]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Fans {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curve: Option<Vec<(Num, Num)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<Num>,
}

impl Fans {
    /// Fields set in `over` replace the ones here (per-fan overrides).
    pub fn merged(&self, over: Option<&Fans>) -> Fans {
        let Some(o) = over else { return self.clone() };
        Fans {
            sensor: o.sensor.clone().or_else(|| self.sensor.clone()),
            curve: o.curve.clone().or_else(|| self.curve.clone()),
            fixed: o.fixed.or(self.fixed),
        }
    }
}

pub const PRESET_QUIET: [(f64, f64); 5] = [(50.0, 25.0), (65.0, 35.0), (75.0, 55.0), (85.0, 80.0), (90.0, 100.0)];
pub const PRESET_BALANCED: [(f64, f64); 4] = [(45.0, 30.0), (60.0, 45.0), (75.0, 75.0), (85.0, 100.0)];
pub const PRESET_PERFORMANCE: [(f64, f64); 4] = [(40.0, 50.0), (60.0, 70.0), (75.0, 90.0), (80.0, 100.0)];

pub fn curve_of(points: &[(f64, f64)]) -> Vec<(Num, Num)> {
    points.iter().map(|&(t, d)| (Num(t), Num(d))).collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(default)]
    pub lighting: Effect,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub zones: IndexMap<String, Effect>,
    #[serde(default)]
    pub fans: Fans,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fan1: Option<Fans>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fan2: Option<Fans>,
    /// Accepted for compatibility and ignored: the pump can't be slowed in software mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pump: Option<toml::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub active: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sensors: BTreeMap<String, String>,
    /// Curve the controller uses on its own when coreglowd isn't running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
    /// Rules that switch profiles automatically.
    #[serde(default, skip_serializing_if = "Auto::is_empty")]
    pub auto: Auto,
    /// Drive other RGB devices (the motherboard) through an OpenRGB server.
    #[serde(default, skip_serializing_if = "OpenRgb::is_default")]
    pub openrgb: OpenRgb,
    #[serde(default)]
    pub profiles: IndexMap<String, Profile>,
}

/// Stored in the controller (hardware mode): fans 1 and 2 follow case probe 1, the pump stays at 100 %.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fallback {
    pub curve: Vec<(Num, Num)>,
}

/// Motherboard sync: coreglowd sends the "motherboard" part of each profile to every
/// device the OpenRGB server lists (keep OpenRGB's Corsair detector off).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct OpenRgb {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

impl OpenRgb {
    pub fn is_default(&self) -> bool {
        *self == OpenRgb::default()
    }
    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(crate::openrgb::DEFAULT_PORT)
    }
}

/// Automatic profile switching: the first rule that matches decides the profile;
/// with none matching (or switched off) the profile chosen by hand is used.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Auto {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Auto {
    pub fn is_empty(&self) -> bool {
        !self.enabled && self.rules.is_empty()
    }
}

/// `when` is one of: "temp-above" (sensor, temp), "process" (name), "time" (from, to), "locked".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub when: String,
    pub profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp: Option<Num>,
    /// Part of a running program's command line, e.g. "steamapps" for Steam games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

pub const RULE_KINDS: [(&str, &str); 4] = [
    ("temp-above", "A temperature is above"),
    ("process", "A program is running"),
    ("time", "During a time of day"),
    ("locked", "The screen is locked"),
];

/// "HH:MM" → minutes after midnight.
pub fn parse_hm(s: &str) -> Option<u32> {
    let (h, m) = s.trim().split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

impl Rule {
    /// A new rule of the given kind with sensible values.
    pub fn new(when: &str, profile: &str) -> Rule {
        let mut r = Rule { when: when.into(), profile: profile.into(), sensor: None, temp: None, name: None, from: None, to: None };
        match when {
            "temp-above" => {
                r.sensor = Some("cpu".into());
                r.temp = Some(Num(80.0));
            }
            "process" => r.name = Some("steamapps".into()),
            "time" => {
                r.from = Some("23:00".into());
                r.to = Some("07:00".into());
            }
            _ => {}
        }
        r
    }

    /// Plain-language condition, e.g. "Processor (CPU) is above 85 °C".
    pub fn describe(&self) -> String {
        use crate::i18n::{tr, trf};
        match self.when.as_str() {
            "temp-above" => trf(
                "{} is above {} °C",
                &[&crate::sensors::label(self.sensor.as_deref().unwrap_or("cpu")), &self.temp.map_or(0.0, |t| t.0)],
            ),
            "process" => trf("a program matching “{}” is running", &[&self.name.as_deref().unwrap_or("")]),
            "time" => trf("it's between {} and {}", &[&self.from.as_deref().unwrap_or("?"), &self.to.as_deref().unwrap_or("?")]),
            "locked" => tr("the screen is locked").into(),
            other => format!("unknown condition “{other}”"),
        }
    }

    fn validate(&self, profiles: &IndexMap<String, Profile>) -> Result<(), String> {
        if !profiles.contains_key(&self.profile) {
            return Err(format!("automatic rule uses profile '{}', which doesn't exist", self.profile));
        }
        match self.when.as_str() {
            "temp-above" if self.temp.is_none() => Err("“temperature above” rule needs a temp".into()),
            "process" if self.name.as_deref().is_none_or(|n| n.trim().is_empty()) => Err("“program running” rule needs a name".into()),
            "time" if self.from.as_deref().and_then(parse_hm).is_none() || self.to.as_deref().and_then(parse_hm).is_none() => {
                Err("“time of day” rule needs from and to as HH:MM".into())
            }
            "temp-above" | "process" | "time" | "locked" => Ok(()),
            other => Err(format!("unknown automatic rule '{other}' (use temp-above, process, time or locked)")),
        }
    }
}

pub const FALLBACK_DEFAULT: [(f64, f64); 6] = [(25.0, 30.0), (30.0, 35.0), (35.0, 45.0), (40.0, 60.0), (45.0, 80.0), (50.0, 100.0)];
/// The controller's curve format holds at most this many points (liquidctl's limit).
pub const FALLBACK_MAX_POINTS: usize = 7;

impl Config {
    /// The backup curve, sorted, as (°C, %) pairs.
    pub fn fallback_curve(&self) -> Vec<(f64, f64)> {
        let mut c: Vec<(f64, f64)> = match &self.fallback {
            Some(f) => f.curve.iter().map(|(t, d)| (t.0, d.0)).collect(),
            None => FALLBACK_DEFAULT.to_vec(),
        };
        c.sort_by(|a, b| a.0.total_cmp(&b.0));
        c
    }
}

pub fn parse_color(c: &str) -> Result<[u8; 3], String> {
    let s = c.trim_start_matches('#');
    if s.len() != 6 || !s.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(format!("'{c}' isn't a colour (use RRGGBB)"));
    }
    let v = u32::from_str_radix(s, 16).unwrap();
    Ok([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

pub fn color_hex(c: [u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

impl Effect {
    fn validate(&self, what: &str) -> Result<(), String> {
        for c in self.color.iter().chain(self.colors.iter().flatten()) {
            parse_color(c).map_err(|e| format!("{what}: {e}"))?;
        }
        if let Some(stops) = &self.stops {
            if stops.is_empty() {
                return Err(format!("{what}: temperature needs at least one colour"));
            }
            for (_, c) in stops {
                parse_color(c).map_err(|e| format!("{what}: {e}"))?;
            }
        }
        if let Some(p) = self.period {
            if p.0 <= 0.0 {
                return Err(format!("{what}: period must be above 0"));
            }
        }
        if self.effect == EffectKind::ColorShift || self.effect == EffectKind::ColorWave {
            if self.colors.as_ref().is_none_or(|c| c.is_empty()) {
                return Err(format!("{what}: needs colours"));
            }
        }
        Ok(())
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.profiles.is_empty() {
            return Err("there must be at least one profile".into());
        }
        for r in &self.auto.rules {
            r.validate(&self.profiles)?;
        }
        if let Some(f) = &self.fallback {
            if f.curve.len() < 2 || f.curve.len() > FALLBACK_MAX_POINTS {
                return Err(format!("backup curve needs 2 to {FALLBACK_MAX_POINTS} points"));
            }
            if f.curve.iter().any(|(t, d)| !(0.0..=100.0).contains(&t.0) || !(0.0..=100.0).contains(&d.0)) {
                return Err("backup curve points must be 0-100 °C and 0-100 %".into());
            }
        }
        for (name, p) in &self.profiles {
            if name.is_empty() || name.len() > 40 || name.chars().any(|c| c.is_control() || c == '"') {
                return Err(format!("profile name '{name}' isn't allowed (1-40 characters, no quotes)"));
            }
            p.lighting.validate(&format!("{name}: lighting"))?;
            for (zone, e) in &p.zones {
                if !ZONE_NAMES.contains(&zone.as_str()) {
                    return Err(format!("{name}: unknown part '{zone}' (use {})", ZONE_NAMES.join(", ")));
                }
                e.validate(&format!("{name}: {zone}"))?;
            }
            for f in [Some(&p.fans), p.fan1.as_ref(), p.fan2.as_ref()].into_iter().flatten() {
                if let Some(c) = &f.curve {
                    if c.is_empty() {
                        return Err(format!("{name}: fan curve needs at least one point"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Profile currently selected: the `active` file wins over the config's `active` key.
    pub fn active_name(&self) -> String {
        let from_file = std::fs::read_to_string(paths::active_file()).ok().map(|s| s.trim().to_string());
        match from_file {
            Some(n) if self.profiles.contains_key(&n) => n,
            _ if self.profiles.contains_key(&self.active) => self.active.clone(),
            _ => self.profiles.keys().next().cloned().unwrap_or_default(),
        }
    }

    pub fn load_from(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let cfg: Config = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load() -> Result<Config, String> {
        paths::ensure_config().map_err(|e| e.to_string())?;
        Self::load_from(&paths::config_file())
    }

    /// Validate, then write atomically (temp file + rename) and record the active profile.
    pub fn save(&self) -> Result<(), String> {
        self.validate()?;
        let body = toml::to_string(self).map_err(|e| e.to_string())?;
        let text = format!("{HEADER}\n{body}");
        toml::from_str::<Config>(&text).map_err(|e| format!("internal error, not saved: {e}"))?;
        let path = paths::config_file();
        std::fs::create_dir_all(paths::config_dir()).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        std::fs::write(paths::active_file(), format!("{}\n", self.active)).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_rules_are_read_checked_and_kept() {
        let text = format!(
            "{BUILTIN_DEFAULT}\n[auto]\nenabled = true\nrules = [\n  {{ when = \"temp-above\", sensor = \"cpu\", temp = 85, profile = \"cpu-temp\" }},\n  {{ when = \"time\", from = \"23:00\", to = \"07:00\", profile = \"quiet\" }},\n  {{ when = \"locked\", profile = \"off\" }},\n]\n"
        );
        let mut cfg: Config = toml::from_str(&text).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.auto.rules.len(), 3);
        let again: Config = toml::from_str(&toml::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, again);
        cfg.auto.rules[1].from = Some("25:00".into());
        assert!(cfg.validate().is_err(), "bad time");
        cfg.auto.rules[1] = Rule::new("time", "no-such-profile");
        assert!(cfg.validate().is_err(), "unknown profile");
        assert_eq!(parse_hm("07:30"), Some(450));
    }

    #[test]
    fn fallback_curve_is_read_checked_and_kept() {
        let mut cfg: Config = toml::from_str(BUILTIN_DEFAULT).unwrap();
        assert_eq!(cfg.fallback_curve(), FALLBACK_DEFAULT.to_vec());
        let text = format!("{BUILTIN_DEFAULT}\n[fallback]\ncurve = [[50, 100], [20, 25]]\n");
        cfg = toml::from_str(&text).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.fallback_curve(), vec![(20.0, 25.0), (50.0, 100.0)]);
        let again: Config = toml::from_str(&toml::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, again);
        cfg.fallback = Some(Fallback { curve: vec![(Num(30.0), Num(50.0))] });
        assert!(cfg.validate().is_err(), "one point isn't a curve");
    }

    #[test]
    fn default_config_parses_and_round_trips() {
        let cfg: Config = toml::from_str(BUILTIN_DEFAULT).unwrap();
        cfg.validate().unwrap();
        let again: Config = toml::from_str(&toml::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, again);
    }
}
