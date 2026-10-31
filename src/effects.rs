//! Renders profiles into lighting frames.

use std::collections::HashMap;
use std::f64::consts::PI;

use crate::config::{parse_color, Effect, EffectKind, Fans, Profile};
use crate::zones;

pub type Rgb = [u8; 3];
pub type Temps = HashMap<String, f64>;

pub const FAILSAFE_DUTY: u8 = 100;

fn mix(a: Rgb, b: Rgb, f: f64) -> Rgb {
    let f = f.clamp(0.0, 1.0);
    [0, 1, 2].map(|i| (a[i] as f64 + (b[i] as f64 - a[i] as f64) * f).round() as u8)
}

fn scale(c: Rgb, k: f64) -> Rgb {
    c.map(|x| (x as f64 * k).round().clamp(0.0, 255.0) as u8)
}

fn hsv(h: f64) -> Rgb {
    let h = h.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    [r, g, b].map(|v: f64| (v * 255.0).round() as u8)
}

fn color(c: &Option<String>) -> Rgb {
    c.as_deref().and_then(|c| parse_color(c).ok()).unwrap_or([255, 255, 255])
}

/// Piecewise-linear lookup in sorted (x, y) points; flat outside the ends.
pub fn interp(points: &[(f64, f64)], x: f64) -> f64 {
    let Some(first) = points.first() else { return 0.0 };
    if x <= first.0 {
        return first.1;
    }
    for w in points.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            return if x1 > x0 { y0 + (y1 - y0) * (x - x0) / (x1 - x0) } else { y1 };
        }
    }
    points.last().unwrap().1
}

impl Effect {
    pub fn sensors(&self) -> Vec<String> {
        match self.effect {
            EffectKind::Temperature => vec![self.sensor.clone().unwrap_or_else(|| "cpu".into())],
            _ => vec![],
        }
    }

    pub fn animated(&self) -> bool {
        self.effect.moving() || (self.effect == EffectKind::Temperature && self.alarm.is_some())
    }

    /// Colours for `n` LEDs at time `t` seconds.
    pub fn render(&self, n: usize, t: f64, temps: &Temps) -> Vec<Rgb> {
        let period = self.period.map_or(4.0, |p| p.0.max(0.05));
        let phase = (t / period).rem_euclid(1.0);
        let waves = self.waves.map_or(1.0, |w| w.0);
        let colors: Vec<Rgb> = match &self.colors {
            Some(cs) if !cs.is_empty() => cs.iter().map(|c| parse_color(c).unwrap_or([255; 3])).collect(),
            _ => vec![color(&self.color)],
        };
        let pos = |i: usize| i as f64 / n.max(1) as f64;
        let mut px: Vec<Rgb> = match self.effect {
            EffectKind::Off => vec![[0; 3]; n],
            EffectKind::Static => vec![color(&self.color); n],
            EffectKind::Breathing => {
                let level = 0.05 + 0.95 * (1.0 - (2.0 * PI * phase).cos()) / 2.0;
                vec![scale(color(&self.color), level); n]
            }
            EffectKind::Flash => vec![if phase < 0.5 { color(&self.color) } else { [0; 3] }; n],
            EffectKind::ColorShift => {
                let p = phase * colors.len() as f64;
                let i = p as usize;
                vec![mix(colors[i % colors.len()], colors[(i + 1) % colors.len()], p - i as f64); n]
            }
            EffectKind::Rainbow => vec![hsv(phase); n],
            EffectKind::RainbowWave => (0..n).map(|i| hsv(phase + waves * pos(i))).collect(),
            EffectKind::ColorWave => {
                let (a, b) = (colors[0], colors[1 % colors.len()]);
                (0..n)
                    .map(|i| mix(a, b, (1.0 + (2.0 * PI * (phase + waves * pos(i))).sin()) / 2.0))
                    .collect()
            }
            EffectKind::Temperature => {
                let sensor = self.sensor.as_deref().unwrap_or("cpu");
                match temps.get(sensor) {
                    // sensor missing: plain white rather than a misleading colour
                    None => vec![[255; 3]; n],
                    Some(&temp) => {
                        let stops: Vec<(f64, Rgb)> = self
                            .stops
                            .clone()
                            .unwrap_or_else(crate::config::default_stops)
                            .iter()
                            .map(|(t, c)| (t.0, parse_color(c).unwrap_or([255; 3])))
                            .collect();
                        let mut c = stop_color(&stops, temp);
                        if self.alarm.is_some_and(|a| temp >= a.0) && phase >= 0.5 {
                            c = [0; 3];
                        }
                        vec![c; n]
                    }
                }
            }
        };
        let k = self.brightness.map_or(1.0, |b| (b.0 / 100.0).clamp(0.0, 1.0));
        if k < 1.0 {
            px.iter_mut().for_each(|p| *p = scale(*p, k));
        }
        px
    }
}

fn stop_color(stops: &[(f64, Rgb)], temp: f64) -> Rgb {
    let mut s = stops.to_vec();
    s.sort_by(|a, b| a.0.total_cmp(&b.0));
    if temp <= s[0].0 {
        return s[0].1;
    }
    for w in s.windows(2) {
        if temp <= w[1].0 {
            let f = if w[1].0 > w[0].0 { (temp - w[0].0) / (w[1].0 - w[0].0) } else { 1.0 };
            return mix(w[0].1, w[1].1, f);
        }
    }
    s.last().unwrap().1
}

impl Fans {
    pub fn sensor_name(&self) -> String {
        self.sensor.clone().unwrap_or_else(|| "probe1".into())
    }

    pub fn duty(&self, temps: &Temps) -> u8 {
        if let Some(f) = self.fixed {
            return f.0.clamp(0.0, 100.0).round() as u8;
        }
        let Some(&temp) = temps.get(&self.sensor_name()) else { return FAILSAFE_DUTY };
        let mut pts: Vec<(f64, f64)> = self
            .curve
            .clone()
            .unwrap_or_else(|| crate::config::curve_of(&crate::config::PRESET_BALANCED))
            .iter()
            .map(|(t, d)| (t.0, d.0))
            .collect();
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        interp(&pts, temp).clamp(0.0, 100.0).round() as u8
    }
}

impl Profile {
    pub fn fan_settings(&self) -> [Fans; 2] {
        [self.fans.merged(self.fan1.as_ref()), self.fans.merged(self.fan2.as_ref())]
    }

    pub fn animated(&self) -> bool {
        self.lighting.animated() || self.zones.values().any(Effect::animated)
    }

    /// Every sensor this profile reads.
    pub fn sensors(&self) -> Vec<String> {
        let mut s = self.lighting.sensors();
        for e in self.zones.values() {
            s.extend(e.sensors());
        }
        for f in self.fan_settings() {
            if f.fixed.is_none() {
                s.push(f.sensor_name());
            }
        }
        s.sort();
        s.dedup();
        s
    }

    /// Full lighting frame (RGB bytes); slots with nothing connected stay dark.
    pub fn frame(&self, t: f64, temps: &Temps) -> Vec<u8> {
        let mut px = vec![[0u8; 3]; zones::frame_len()];
        let all = zones::zone("all");
        for (i, c) in all.iter().zip(self.lighting.render(all.len(), t, temps)) {
            px[*i] = c;
        }
        for (name, effect) in &self.zones {
            let idx = zones::zone(name);
            for (i, c) in idx.iter().zip(effect.render(idx.len(), t, temps)) {
                px[*i] = c;
            }
        }
        px.concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn every_default_profile_renders_a_full_frame() {
        let cfg: Config = toml::from_str(crate::config::BUILTIN_DEFAULT).unwrap();
        let temps: Temps = [("cpu".into(), 50.0), ("probe1".into(), 34.0)].into();
        for (name, p) in &cfg.profiles {
            let f = p.frame(1.0, &temps);
            assert_eq!(f.len(), 233 * 3, "{name}");
            for fans in p.fan_settings() {
                assert!(fans.duty(&temps) <= 100);
            }
        }
    }

    #[test]
    fn curve_interpolates() {
        let pts = [(25.0, 30.0), (50.0, 100.0)];
        assert_eq!(interp(&pts, 20.0), 30.0);
        assert_eq!(interp(&pts, 37.5), 65.0);
        assert_eq!(interp(&pts, 60.0), 100.0);
    }
}
