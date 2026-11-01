//! coreglowd: drives the Commander CORE XT (or, experimentally, a Commander CORE) from the active profile.
//!
//! Keeps the controller in software mode like iCUE: streams the lighting and
//! sends fan duties (and, on a Core, the pump speed). On exit (or before taking over) it stores a
//! hardware fallback curve, so the fans stay sensible without this daemon.
//! Reconnects by itself when the controller disappears (unplugged, passed to a VM).

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use coreglow::auto::AutoState;
use coreglow::config::{Config, OpenRgb, Profile};
use coreglow::control::{self, Hub, Request};
use coreglow::device::{self, Device};
use coreglow::effects::Temps;
use coreglow::sensors::Sensors;
use coreglow::status::{self, Status};
use coreglow::model::Model;
use coreglow::{openrgb, paths, zones};

const FPS: u64 = 30;
const SENSOR_EVERY: Duration = Duration::from_secs(1);
const RESEND_EVERY: Duration = Duration::from_secs(10);
/// Unchanged frames are still re-sent this often (keeps software mode alive).
const STATIC_EVERY: Duration = Duration::from_secs(2);
const RETRY_EVERY: Duration = Duration::from_secs(3);

fn log(msg: impl AsRef<str>) {
    eprintln!("{}", msg.as_ref());
}

fn stamp() -> (Option<SystemTime>, Option<SystemTime>) {
    let m = |p: std::path::PathBuf| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    (m(paths::config_file()), m(paths::active_file()))
}

/// How long a "test fans" request runs them at full speed.
const TEST_FANS_FOR: Duration = Duration::from_secs(5);

struct State {
    cfg: Config,
    /// Profile chosen by hand (GUI, CLI, tray).
    manual: String,
    /// Profile in use: `manual`, or the one an automatic rule picked.
    name: String,
    profile: Profile,
    /// Index of the automatic rule in charge, if any.
    rule: Option<usize>,
    sensors: Sensors,
    fallback: Vec<(f64, f64)>,
}

fn load() -> Result<State, String> {
    let cfg = Config::load()?;
    let name = cfg.active_name();
    let profile = cfg.profiles[&name].clone();
    let sensors = Sensors::new(&cfg.sensors);
    let fallback = cfg.fallback_curve();
    Ok(State { manual: name.clone(), name, profile, rule: None, sensors, fallback, cfg })
}

impl State {
    /// Use the profile rule `rule` picks (or the manual one), logging the change.
    fn apply_rule(&mut self, rule: Option<usize>) {
        let target = match rule {
            Some(i) => self.cfg.auto.rules[i].profile.clone(),
            None => self.manual.clone(),
        };
        if target != self.name || rule != self.rule {
            match rule {
                Some(i) => log(format!("automatic: {} → profile {target}", self.cfg.auto.rules[i].describe())),
                None if self.rule.is_some() => log(format!("automatic rules no longer match → profile {target}")),
                None => {}
            }
        }
        self.rule = rule;
        if target != self.name {
            self.profile = self.cfg.profiles[&target].clone();
            self.name = target;
        }
    }
}

/// Sends the "motherboard" part of the profile to an OpenRGB server, when enabled.
struct MotherboardSync {
    client: Option<openrgb::Client>,
    next_try: Instant,
    last: Vec<[u8; 3]>,
    last_sent: Instant,
    /// For the status file: what's connected, or why not.
    state: Option<String>,
}

impl MotherboardSync {
    const RETRY: Duration = Duration::from_secs(10);

    fn new() -> Self {
        MotherboardSync { client: None, next_try: Instant::now(), last: Vec::new(), last_sent: Instant::now(), state: None }
    }

    fn tick(&mut self, cfg: &OpenRgb, profile: &Profile, t: f64, temps: &Temps, dim: f64) {
        if !cfg.enabled {
            if self.client.take().is_some() {
                log("motherboard sync off");
            }
            self.state = None;
            return;
        }
        let now = Instant::now();
        if self.client.is_none() {
            if now < self.next_try {
                return;
            }
            self.next_try = now + Self::RETRY;
            match openrgb::Client::connect(cfg.host(), cfg.port()) {
                Ok(mut c) => {
                    let names: Vec<String> = c.controllers.iter().map(|d| format!("{} ({} LEDs)", d.name, d.leds)).collect();
                    for i in 0..c.controllers.len() as u32 {
                        let _ = c.set_custom_mode(i);
                    }
                    log(format!("motherboard sync: {}", names.join(", ")));
                    self.state = Some(names.join(", "));
                    self.last.clear();
                    self.client = Some(c);
                }
                Err(e) => {
                    if self.state.as_deref() != Some(e.as_str()) {
                        log(format!("motherboard sync: {e}"));
                    }
                    self.state = Some(e);
                    return;
                }
            }
        }
        let Some(c) = self.client.as_mut() else { return };
        let colors: Vec<[u8; 3]> = profile.motherboard(t, temps, c.total_leds()).into_iter().map(|c| c.map(|v| dimmed(v, dim))).collect();
        if colors == self.last && now - self.last_sent < STATIC_EVERY {
            return;
        }
        let mut start = 0;
        let mut result = Ok(());
        for d in c.controllers.clone() {
            result = result.and_then(|_| c.set_leds(d.index, &colors[start..start + d.leds]));
            start += d.leds;
        }
        match result {
            Ok(()) => {
                self.last = colors;
                self.last_sent = now;
            }
            Err(e) => {
                log(format!("motherboard sync lost: {e}"));
                self.state = Some(e);
                self.client = None;
            }
        }
    }

    fn first_color(&self) -> Option<[u8; 3]> {
        self.client.as_ref().and(self.last.first().copied())
    }
}

fn dimmed(v: u8, factor: f64) -> u8 {
    (v as f64 * factor).round() as u8
}

/// The layout for this controller: the config's [layout] if it fits, this build's on a
/// Core XT, else what the controller detects. True when detected.
fn choose_layout(dev: &mut Device, cfg: &Config, found: &[(u16, u16)]) -> (zones::Layout, bool) {
    if let Some(l) = &cfg.layout {
        match zones::Layout::from_config(dev.model, &l.ports, &l.names) {
            Ok(layout) => return (layout, false),
            Err(e) => log(format!("[layout] doesn't fit this {}: {e}; using what it detects", dev.model.name())),
        }
    } else if dev.model == Model::CoreXt {
        return (zones::Layout::this_build(), false);
    }
    (zones::Layout::detected(dev.model, found), true)
}

/// What the controller reports on each RGB port, next to what the layout says.
fn detected_devices(found: &[(u16, u16)], layout: &zones::Layout) -> Vec<status::PortInfo> {
    layout
        .model
        .ports()
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let (state, leds) = found.get(i).copied().unwrap_or((0, 0));
            status::PortInfo {
                port: port.to_string(),
                configured: layout.ports[i].iter().map(|k| k.id.to_string()).collect(),
                detected_leds: (state == 2).then_some(leds),
                detected_kind: (state == 2).then(|| zones::kind_for_leds(leds as usize).map(|k| k.id.to_string())).flatten(),
            }
        })
        .collect()
}

/// True while a "test fans" request (the request file touched by the GUI or CLI) is recent.
fn testing_fans() -> bool {
    std::fs::metadata(paths::test_fans_file())
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < TEST_FANS_FOR)
}

/// Fan and pump duties to send: (channel, %). The Core XT gets this build's two fans
/// (its pump on port 6 can't be slowed); the Core gets fan1/fan2, the shared setting on
/// ports 3-6, and the profile's pump speed.
fn duties(model: Model, profile: &Profile, temps: &Temps, testing: bool) -> Vec<(usize, u8)> {
    let fans = profile.fan_settings();
    let duty = |i: usize| if testing { 100 } else { fans[i].duty(temps) };
    let mut out = vec![(model.fan_channel(1), duty(0)), (model.fan_channel(2), duty(1))];
    if model.pump_settable() {
        let shared = if testing { 100 } else { profile.fans.duty(temps) };
        out.extend((3..=6).map(|n| (model.fan_channel(n), shared)));
        out.push((model.pump_channel(), profile.pump_duty()));
    }
    out
}

/// Runs until `stop` is set or the device fails. Err = device error (reconnect).
fn run(dev: &mut Device, st: &mut State, stop: &AtomicBool, hub: &Hub, requests: &Receiver<Request>) -> Result<(), String> {
    let model = dev.model;
    dev.wake()?;
    device::store_fallback(dev, &st.fallback)?;
    let found = dev.autodetected().unwrap_or_default();
    let (layout0, mut is_detected) = choose_layout(dev, &st.cfg, &found);
    zones::set_current(layout0);
    let mut layout = zones::current();
    dev.init_lighting(&layout)?;
    log(format!("lighting layout: {} LEDs{}", layout.frame_len(), if is_detected { " (detected)" } else { "" }));
    let mut devices = detected_devices(&found, &layout);
    let mut all_parts: Vec<(&str, usize)> = zones::PICTURE_ZONES.iter().filter_map(|z| zones::zone(z).first().map(|i| (*z, *i))).collect();
    let t0 = Instant::now();
    let mut cfg_stamp = stamp();
    let mut temps = Temps::new();
    let mut sent: Option<Vec<(usize, u8)>> = None;
    let mut night = false;
    let mut test_until = Instant::now();
    let (mut last_fan, mut last_light) = (Instant::now() - RESEND_EVERY, Instant::now() - STATIC_EVERY);
    let mut next_sensor = Instant::now();
    let mut last_frame: Vec<u8> = Vec::new();
    let mut error = None;
    let mut history: BTreeMap<String, VecDeque<f64>> = BTreeMap::new();
    let mut next_history = Instant::now();
    let mut auto = AutoState::default();
    let mut board = MotherboardSync::new();

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        for r in requests.try_iter() {
            match r {
                Request::TestFans => {
                    test_until = now + TEST_FANS_FOR;
                    next_sensor = now; // fans and status right away
                }
            }
        }
        // Checked every frame (two stat calls), so edits and profile switches show at once.
        if stamp() != cfg_stamp {
            cfg_stamp = stamp();
            match load() {
                Ok(mut new) => {
                    if new.manual != st.manual {
                        log(format!("switched to profile {}", new.manual));
                    } else {
                        log(format!("profile {} changed", new.manual));
                    }
                    if new.fallback != st.fallback {
                        device::store_fallback(dev, &new.fallback)?;
                        log("backup curve stored in the controller");
                    }
                    // keep the rule in charge until the next check, if it still exists
                    let rule = st.rule.filter(|i| *i < new.cfg.auto.rules.len() && new.cfg.auto.enabled);
                    new.rule = st.rule;
                    new.apply_rule(rule);
                    *st = new;
                    error = None;
                    next_sensor = now; // publish the new profile right away
                    let found = dev.autodetected().unwrap_or_default();
                    let (new_layout, d) = choose_layout(dev, &st.cfg, &found);
                    is_detected = d;
                    zones::set_current(new_layout);
                    if *zones::current() != *layout {
                        layout = zones::current();
                        devices = detected_devices(&found, &layout);
                        dev.init_lighting(&layout)?;
                        all_parts = zones::PICTURE_ZONES.iter().filter_map(|z| zones::zone(z).first().map(|i| (*z, *i))).collect();
                        log(format!("lighting layout changed: {} LEDs", layout.frame_len()));
                    }
                }
                Err(e) => {
                    log(format!("config not loaded, keeping profile {}: {e}", st.name));
                    error = Some(e);
                }
            }
        }
        if now >= next_sensor {
            next_sensor = now + SENSOR_EVERY;
            let readings = dev.temps()?;
            let (probes, liquid) = match model {
                Model::CoreXt => (readings, None),
                Model::Core => (Vec::new(), readings.first().copied().flatten()),
            };
            let mut wanted = st.profile.sensors();
            for p in st.cfg.profiles.values() {
                wanted.extend(p.sensors());
            }
            wanted.extend(st.cfg.auto.rules.iter().filter_map(|r| r.sensor.clone()));
            wanted.extend(["probe1", "probe2", "liquid", "cpu"].map(String::from));
            wanted.sort();
            wanted.dedup();
            temps = st.sensors.read(&probes, liquid, &wanted);
            let rule = auto.evaluate(&st.cfg.auto, &temps);
            st.apply_rule(rule);
            let testing = testing_fans() || now < test_until;
            let duties = duties(model, &st.profile, &temps, testing);
            let moved = sent.as_ref().is_none_or(|s| s.iter().zip(&duties).any(|(a, b)| a.1.abs_diff(b.1) >= 2));
            if moved || now - last_fan >= RESEND_EVERY {
                dev.set_duties(&duties)?;
                if sent.as_ref() != Some(&duties) {
                    let pump = if model.pump_settable() { format!(", pump {} %", st.profile.pump_duty()) } else { String::new() };
                    log(format!("fans {}/{} %{pump}", duties[0].1, duties[1].1));
                }
                sent = Some(duties.clone());
                last_fan = now;
            }
            let dim_now = st.cfg.night_factor() < 1.0;
            if dim_now != night {
                log(if dim_now { "night dimming on" } else { "night dimming off" });
                night = dim_now;
            }
            let rpm = dev.speeds()?;
            if now >= next_history {
                next_history = now + Duration::from_secs(status::HISTORY_EVERY_S);
                let r0 = rpm.first().copied().unwrap_or(0) as f64;
                let samples = [("cpu", temps.get("cpu").copied()), ("probe1", temps.get("probe1").copied()), ("fan1_rpm", Some(r0))];
                for (key, value) in samples {
                    if let Some(v) = value {
                        let h = history.entry(key.to_string()).or_default();
                        h.push_back((v * 10.0).round() / 10.0);
                        while h.len() > status::HISTORY_LEN {
                            h.pop_front();
                        }
                    }
                }
            }
            let r = |i: usize| rpm.get(i).copied().unwrap_or(0);
            let mut fan_duty: BTreeMap<String, u8> = [("fan1".into(), duties[0].1), ("fan2".into(), duties[1].1)].into();
            if model.pump_settable() {
                fan_duty.insert("pump".into(), st.profile.pump_duty());
            }
            let status = Status {
                profile: st.name.clone(),
                time: status::now(),
                temps: temps.iter().map(|(k, v)| (k.clone(), (v * 10.0).round() / 10.0)).collect(),
                fan_duty,
                rpm: [
                    ("fan1".into(), r(model.fan_channel(1))),
                    ("fan2".into(), r(model.fan_channel(2))),
                    ("pump".into(), r(model.pump_channel())),
                ]
                .into(),
                zones: if last_frame.is_empty() {
                    Default::default()
                } else {
                    all_parts
                        .iter()
                        .map(|(z, i)| (z.to_string(), coreglow::config::color_hex([last_frame[i * 3], last_frame[i * 3 + 1], last_frame[i * 3 + 2]])))
                        .chain(board.first_color().map(|c| ("motherboard".to_string(), coreglow::config::color_hex(c))))
                        .collect()
                },
                openrgb: board.state.clone(),
                devices: devices.clone(),
                error: error.clone(),
                history: history.iter().map(|(k, v)| (k.clone(), v.iter().copied().collect())).collect(),
                testing_fans: testing,
                manual_profile: st.manual.clone(),
                auto_rule: st.rule.map(|i| st.cfg.auto.rules[i].describe()),
                controller: Some(model.id().to_string()),
                detected: is_detected.then(|| layout.to_config()),
                night,
            };
            status.write();
            hub.publish(&status);
        }

        let dim = st.cfg.night_factor();
        let mut frame = st.profile.frame((now - t0).as_secs_f64(), &temps);
        if dim < 1.0 {
            frame.iter_mut().for_each(|v| *v = dimmed(*v, dim));
        }
        if frame != last_frame || now - last_light >= STATIC_EVERY {
            dev.write_frame(&frame)?;
            last_frame = frame;
            last_light = now;
        }
        board.tick(&st.cfg.openrgb, &st.profile, (now - t0).as_secs_f64(), &temps, dim);
        let pause = if st.profile.animated() { Duration::from_millis(1000 / FPS) } else { Duration::from_millis(200) };
        std::thread::sleep(pause);
    }
    Ok(())
}

fn main() {
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed)).expect("signal handler");
    }
    let mut st = match load() {
        Ok(s) => s,
        Err(e) => {
            log(format!("can't start: {e}"));
            std::process::exit(1);
        }
    };
    log(format!(
        "coreglowd {}: profile {}, {} LEDs ({} connected)",
        env!("CARGO_PKG_VERSION"),
        st.name,
        zones::frame_len(),
        zones::zone("all").len()
    ));

    let hub = Arc::new(Hub::default());
    let (tx, requests) = std::sync::mpsc::channel();
    match control::serve(hub.clone(), tx) {
        Ok(()) => log(format!("listening on {}", control::socket_path().display())),
        Err(e) => log(format!("control socket not available ({e}); the files still work")),
    }

    let mut waiting_logged = false;
    while !stop.load(Ordering::Relaxed) {
        let mut dev = match Device::open() {
            Ok(d) => d,
            Err(e) => {
                if !waiting_logged {
                    log(format!("waiting for the controller: {e}"));
                    waiting_logged = true;
                }
                let status = Status { profile: st.name.clone(), time: status::now(), error: Some(e), ..Default::default() };
                status.write();
                hub.publish(&status);
                std::thread::sleep(RETRY_EVERY);
                continue;
            }
        };
        waiting_logged = false;
        let note = if dev.model.experimental() { " (experimental support: please report how it works)" } else { "" };
        log(format!("{} at {}{note}", dev.model.name(), dev.path.display()));
        let result = run(&mut dev, &mut st, &stop, &hub, &requests);
        // Hand the controller back to its own fallback curve, whatever happened.
        let _ = dev.wake().and_then(|_| device::store_fallback(&mut dev, &st.fallback)).and_then(|_| dev.sleep());
        match result {
            Ok(()) => log("controller back in hardware mode"),
            Err(e) => {
                log(format!("controller error: {e}; reconnecting"));
                std::thread::sleep(RETRY_EVERY);
            }
        }
    }
    let _ = std::fs::remove_file(control::socket_path());
}
