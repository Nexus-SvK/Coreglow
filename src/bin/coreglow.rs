//! coreglow: a window for choosing colours, effects and fan speeds.
//!
//! Edits ~/.config/coreglow/config.toml (saved automatically, checked first) and shows
//! the live values coreglowd publishes. The daemon applies changes within a second.

use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, Sense, Stroke, Vec2};

use coreglow::config::{
    color_hex, curve_of, parse_color, Config, Effect, EffectKind, Fans, Num, Profile, PRESET_BALANCED,
    PRESET_PERFORMANCE, PRESET_QUIET,
};
use coreglow::status::Status;
use coreglow::{paths, sensors, zones};

const SPEED_K: f64 = 16.0; // period (s) = SPEED_K / speed
const SAVE_DELAY: Duration = Duration::from_millis(400);

fn effect_label(k: EffectKind) -> (&'static str, &'static str) {
    match k {
        EffectKind::Static => ("Solid colour", "One colour, no movement"),
        EffectKind::Breathing => ("Breathing", "Slowly fades in and out"),
        EffectKind::Flash => ("Flashing", "Blinks on and off"),
        EffectKind::ColorShift => ("Colour cycle", "Fades through your colours"),
        EffectKind::Rainbow => ("Rainbow", "Everything changes colour together"),
        EffectKind::RainbowWave => ("Rainbow wave", "A rainbow moving along the lights"),
        EffectKind::ColorWave => ("Two-colour wave", "Two colours moving along the lights"),
        EffectKind::Temperature => ("Shows temperature", "Colour changes as it gets hotter"),
        EffectKind::Off => ("Off", "Lights off"),
    }
}

fn stamp() -> (Option<SystemTime>, Option<SystemTime>) {
    let m = |p: std::path::PathBuf| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    (m(paths::config_file()), m(paths::active_file()))
}

struct App {
    cfg: Config,
    selected: String,
    name_edit: String,
    status: Option<Status>,
    last_poll: Instant,
    changed_at: Option<Instant>,
    save_msg: (String, bool),
    file_stamp: (Option<SystemTime>, Option<SystemTime>),
    confirm_delete: bool,
    drag: Option<usize>,
    gpu: bool,
    load_error: Option<String>,
}

impl App {
    fn new() -> Self {
        let (cfg, load_error) = match Config::load() {
            Ok(mut c) => {
                c.active = c.active_name();
                (c, None)
            }
            Err(e) => (Config::default(), Some(e)),
        };
        let selected = cfg.active.clone();
        App {
            name_edit: selected.clone(),
            selected,
            cfg,
            status: Status::read(),
            last_poll: Instant::now(),
            changed_at: None,
            save_msg: (String::new(), false),
            file_stamp: stamp(),
            confirm_delete: false,
            drag: None,
            gpu: sensors::nvidia_available(),
            load_error,
        }
    }

    fn changed(&mut self) {
        self.changed_at = Some(Instant::now());
        self.save_msg = ("Saving…".into(), false);
    }

    fn select(&mut self, name: String) {
        self.name_edit = name.clone();
        self.selected = name;
        self.confirm_delete = false;
    }

    fn poll(&mut self) {
        if self.last_poll.elapsed() < Duration::from_millis(1000) {
            return;
        }
        self.last_poll = Instant::now();
        self.status = Status::read();
        // pick up changes made elsewhere (coreglowctl, editor) while nothing is pending here
        if self.changed_at.is_none() && stamp() != self.file_stamp {
            self.file_stamp = stamp();
            if let Ok(mut c) = Config::load() {
                c.active = c.active_name();
                self.cfg = c;
                if !self.cfg.profiles.contains_key(&self.selected) {
                    let a = self.cfg.active.clone();
                    self.select(a);
                }
            }
        }
    }

    fn autosave(&mut self) {
        if let Some(t) = self.changed_at {
            if t.elapsed() >= SAVE_DELAY {
                self.changed_at = None;
                self.save_msg = match self.cfg.save() {
                    Ok(()) => ("Saved ✓".into(), false),
                    Err(e) => (format!("Not saved: {e}"), true),
                };
                self.file_stamp = stamp();
            }
        }
    }

    fn unique_name(&self, base: &str) -> String {
        let mut n = base.to_string();
        let mut i = 2;
        while self.cfg.profiles.contains_key(&n) {
            n = format!("{base} {i}");
            i += 1;
        }
        n
    }
}

// ---------- top bar ----------

fn chip(ui: &mut egui::Ui, text: impl Into<String>, value: impl Into<String>) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text.into()).weak());
            ui.label(RichText::new(value.into()).strong());
        });
}

fn top_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading(RichText::new("Coreglow").strong());
        ui.add_space(12.0);
        match &app.status {
            Some(s) if s.fresh() => {
                if let Some(t) = s.temps.get("cpu") {
                    chip(ui, "CPU", format!("{t:.0} °C"));
                }
                if let Some(t) = s.temps.get("probe1") {
                    chip(ui, "Case", format!("{t:.0} °C"));
                }
                let rpm = |k: &str| s.rpm.get(k).copied().unwrap_or(0);
                chip(ui, "Fans", format!("{} / {} rpm", rpm("fan1"), rpm("fan2")));
                chip(ui, "Pump", format!("{} rpm", rpm("pump")));
                if let Some(e) = &s.error {
                    ui.colored_label(ui.visuals().warn_fg_color, e);
                }
            }
            other => {
                let why = other.as_ref().and_then(|s| s.error.clone()).unwrap_or_else(|| "The lighting service isn't running.".into());
                ui.colored_label(ui.visuals().warn_fg_color, why);
                if ui.button("Start it").clicked() {
                    let _ = std::process::Command::new("systemctl").args(["--user", "start", "coreglowd.service"]).spawn();
                }
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (msg, err) = &app.save_msg;
            if *err {
                ui.colored_label(ui.visuals().error_fg_color, msg);
            } else {
                ui.label(RichText::new(msg).weak());
            }
        });
    });
}

// ---------- picture of the PC ----------

fn picture(ui: &mut egui::Ui, status: Option<&Status>) {
    let size = Vec2::new(ui.available_width().min(290.0), 200.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 10.0, Color32::from_rgb(20, 22, 25));
    let col = |z: &str| -> Color32 {
        status
            .filter(|s| s.fresh())
            .and_then(|s| s.zones.get(z))
            .and_then(|c| parse_color(c).ok())
            .map_or(Color32::from_gray(60), |c| Color32::from_rgb(c[0], c[1], c[2]))
    };
    let at = |x: f32, y: f32| Pos2::new(rect.left() + x / 300.0 * rect.width(), rect.top() + y);
    let label = |pos: Pos2, t: &str| {
        p.text(pos, Align2::CENTER_CENTER, t, FontId::proportional(11.0), Color32::from_gray(150));
    };
    for (x, z, t) in [(70.0, "left-fan", "left fan"), (150.0, "right-fan", "right fan")] {
        p.circle_stroke(at(x, 50.0), 32.0, Stroke::new(7.0, col(z)));
        p.circle_filled(at(x, 50.0), 10.0, Color32::from_gray(50));
        label(at(x, 98.0), t);
    }
    p.circle_stroke(at(80.0, 145.0), 20.0, Stroke::new(6.0, col("pump")));
    label(at(80.0, 182.0), "pump");
    for (i, (x, y)) in [(222.0, 20.0), (242.0, 20.0), (222.0, 80.0), (242.0, 80.0)].iter().enumerate() {
        let r = egui::Rect::from_min_size(at(*x, *y), Vec2::new(10.0, 40.0));
        p.rect_filled(r, 3.0, col(&format!("strip{}", i + 1)));
    }
    label(at(237.0, 140.0), "LED strips 1–4");
}

// ---------- editors ----------

fn color_button(ui: &mut egui::Ui, hex: &mut String) -> bool {
    let mut c = parse_color(hex).unwrap_or([255; 3]);
    if egui::color_picker::color_edit_button_srgb(ui, &mut c).changed() {
        *hex = color_hex(c);
        return true;
    }
    false
}

fn sensor_combo(ui: &mut egui::Ui, id: &str, value: &mut String, gpu: bool) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id).selected_text(sensors::label(value)).width(220.0).show_ui(ui, |ui| {
        for (sid, label) in sensors::SENSORS {
            let enabled = sid != "gpu" || gpu;
            let text = if enabled { label.to_string() } else { format!("{label} — needs NVIDIA driver") };
            if ui.add_enabled(enabled, egui::SelectableLabel::new(value == sid, text)).clicked() {
                *value = sid.to_string();
                changed = true;
            }
        }
    });
    changed
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized([110.0, 20.0], egui::Label::new(RichText::new(label).weak()));
        add(ui);
    });
}

/// Returns true if anything changed.
fn effect_ui(ui: &mut egui::Ui, e: &mut Effect, id: &str, gpu: bool) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for k in EffectKind::ALL {
            let (name, desc) = effect_label(k);
            let sel = e.effect == k;
            let text = if sel { RichText::new(name).strong() } else { RichText::new(name) };
            if ui.add_sized([140.0, 30.0], egui::SelectableLabel::new(sel, text)).on_hover_text(desc).clicked() && !sel {
                e.set_kind(k);
                changed = true;
            }
        }
    });
    ui.add_space(4.0);
    let kind = e.effect;
    if matches!(kind, EffectKind::Static | EffectKind::Breathing | EffectKind::Flash) {
        let c = e.color.get_or_insert_with(|| "ffffff".into());
        row(ui, "Colour", |ui| changed |= color_button(ui, c));
    }
    if matches!(kind, EffectKind::ColorShift | EffectKind::ColorWave) {
        let max = if kind == EffectKind::ColorWave { 2 } else { 6 };
        let cs = e.colors.get_or_insert_with(|| vec!["ff0000".into(), "0000ff".into()]);
        row(ui, "Colours", |ui| {
            for c in cs.iter_mut() {
                changed |= color_button(ui, c);
            }
            if cs.len() < max && ui.small_button("+ add").clicked() {
                cs.push("ffffff".into());
                changed = true;
            }
            if cs.len() > 2 && ui.small_button("− remove").clicked() {
                cs.pop();
                changed = true;
            }
        });
    }
    if kind == EffectKind::Temperature {
        changed |= temperature_ui(ui, e, id, gpu);
    }
    if kind.moving() {
        let mut speed = SPEED_K / e.period.map_or(4.0, |p| p.0);
        row(ui, "Speed", |ui| {
            if ui.add(egui::Slider::new(&mut speed, 1.0..=10.0).step_by(0.5).show_value(false)).changed() {
                e.period = Some(Num((SPEED_K / speed * 100.0).round() / 100.0));
                changed = true;
            }
            ui.label(RichText::new(if speed <= 3.0 { "slow" } else if speed >= 8.0 { "fast" } else { "" }).weak());
        });
    }
    if matches!(kind, EffectKind::RainbowWave | EffectKind::ColorWave) {
        let mut w = e.waves.map_or(1.0, |w| w.0);
        row(ui, "Wave length", |ui| {
            if ui.add(egui::Slider::new(&mut w, 0.5..=4.0).step_by(0.5).show_value(false)).changed() {
                e.waves = Some(Num(w));
                changed = true;
            }
            ui.label(RichText::new(if w < 1.5 { "long" } else if w > 3.0 { "short" } else { "" }).weak());
        });
    }
    if kind != EffectKind::Off {
        let mut b = e.brightness.map_or(100.0, |b| b.0);
        row(ui, "Brightness", |ui| {
            if ui.add(egui::Slider::new(&mut b, 5.0..=100.0).step_by(5.0).suffix(" %")).changed() {
                e.brightness = Some(Num(b));
                changed = true;
            }
        });
    }
    changed
}

fn temperature_ui(ui: &mut egui::Ui, e: &mut Effect, id: &str, gpu: bool) -> bool {
    let mut changed = false;
    let sensor = e.sensor.get_or_insert_with(|| "cpu".into());
    row(ui, "Follows", |ui| changed |= sensor_combo(ui, &format!("{id}-tsensor"), sensor, gpu));
    let stops = e.stops.get_or_insert_with(coreglow::config::default_stops);
    let n = stops.len();
    let mut remove = None;
    for (i, (t, c)) in stops.iter_mut().enumerate() {
        let label = if i == 0 { "Cool" } else if i == n - 1 { "Hot" } else { "Warm" };
        row(ui, label, |ui| {
            changed |= ui.add(egui::DragValue::new(&mut t.0).range(0.0..=110.0).suffix(" °C")).changed();
            ui.label("→");
            changed |= color_button(ui, c);
            if n > 2 && ui.small_button("remove").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        stops.remove(i);
        changed = true;
    }
    if changed {
        stops.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0));
    }
    row(ui, "", |ui| {
        if ui.small_button("+ add a colour").clicked() {
            let last = stops.last().map_or(80.0, |s| s.0 .0);
            stops.push((Num((last + 10.0).min(110.0)), "ff00ff".into()));
            changed = true;
        }
    });
    let mut on = e.alarm.is_some();
    row(ui, "Warning", |ui| {
        if ui.checkbox(&mut on, "Flash when it gets too hot").changed() {
            e.alarm = on.then_some(Num(85.0));
            if on {
                e.period = Some(Num(1.0));
            }
            changed = true;
        }
        if let Some(a) = &mut e.alarm {
            ui.label("above");
            changed |= ui.add(egui::DragValue::new(&mut a.0).range(30.0..=110.0).suffix(" °C")).changed();
        }
    });
    changed
}

fn curve_ui(ui: &mut egui::Ui, curve: &mut Vec<(Num, Num)>, now: Option<f64>, drag: &mut Option<usize>) -> bool {
    const X0: f32 = 20.0;
    const X1: f32 = 100.0;
    let mut changed = false;
    let size = Vec2::new(ui.available_width().min(560.0), 240.0);
    let (resp, p) = ui.allocate_painter(size, Sense::click_and_drag());
    let r = resp.rect;
    let (l, rr, t, b) = (r.left() + 40.0, r.right() - 12.0, r.top() + 10.0, r.bottom() - 26.0);
    let x = |temp: f32| l + (temp - X0) / (X1 - X0) * (rr - l);
    let y = |duty: f32| t + (1.0 - duty / 100.0) * (b - t);
    let tx = |px: f32| X0 + (px - l) / (rr - l) * (X1 - X0);
    let ty = |py: f32| 100.0 * (1.0 - (py - t) / (b - t));
    let vis = ui.visuals();
    let grid = Stroke::new(1.0, vis.widgets.noninteractive.bg_stroke.color);
    let text = vis.weak_text_color();
    let accent = vis.selection.bg_fill;
    let mut tmp = 20;
    while tmp <= 100 {
        let xx = x(tmp as f32);
        p.line_segment([Pos2::new(xx, t), Pos2::new(xx, b)], grid);
        p.text(Pos2::new(xx, b + 12.0), Align2::CENTER_CENTER, format!("{tmp}°"), FontId::proportional(11.0), text);
        tmp += 10;
    }
    for d in [0, 25, 50, 75, 100] {
        let yy = y(d as f32);
        p.line_segment([Pos2::new(l, yy), Pos2::new(rr, yy)], grid);
        p.text(Pos2::new(l - 6.0, yy), Align2::RIGHT_CENTER, format!("{d}%"), FontId::proportional(11.0), text);
    }
    curve.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0));
    let pts: Vec<Pos2> = curve.iter().map(|(tt, dd)| Pos2::new(x(tt.0 as f32), y(dd.0 as f32))).collect();
    let mut line = vec![Pos2::new(x(X0), pts[0].y)];
    line.extend(pts.iter().copied());
    line.push(Pos2::new(x(X1), pts[pts.len() - 1].y));
    p.add(egui::Shape::line(line, Stroke::new(2.5, accent)));
    if let Some(now) = now {
        let xx = x((now as f32).clamp(X0, X1));
        p.extend(egui::Shape::dashed_line(&[Pos2::new(xx, t), Pos2::new(xx, b)], Stroke::new(1.5, vis.warn_fg_color), 4.0, 3.0));
    }
    for (i, pt) in pts.iter().enumerate() {
        let hot = *drag == Some(i);
        p.circle(*pt, if hot { 9.0 } else { 7.0 }, vis.extreme_bg_color, Stroke::new(2.5, accent));
    }
    let nearest = |pos: Pos2| -> Option<usize> {
        pts.iter().enumerate().map(|(i, q)| (i, q.distance(pos))).filter(|(_, d)| *d < 14.0).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
    };
    if resp.drag_started() {
        *drag = resp.interact_pointer_pos().and_then(nearest);
    }
    if let (true, Some(i), Some(pos)) = (resp.dragged(), *drag, resp.interact_pointer_pos()) {
        let lo = if i > 0 { curve[i - 1].0 .0 + 1.0 } else { X0 as f64 };
        let hi = if i + 1 < curve.len() { curve[i + 1].0 .0 - 1.0 } else { X1 as f64 };
        curve[i].0 = Num((tx(pos.x) as f64).round().clamp(lo, hi));
        curve[i].1 = Num((ty(pos.y) as f64).round().clamp(0.0, 100.0));
        changed = true;
    }
    if resp.drag_stopped() {
        *drag = None;
    }
    if resp.double_clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let (tt, dd) = ((tx(pos.x) as f64).round(), (ty(pos.y) as f64).round().clamp(0.0, 100.0));
            if tt > X0 as f64 && tt < X1 as f64 && !curve.iter().any(|c| c.0 .0 == tt) {
                curve.push((Num(tt), Num(dd)));
                changed = true;
            }
        }
    }
    if resp.secondary_clicked() && curve.len() > 2 {
        if let Some(i) = resp.interact_pointer_pos().and_then(nearest) {
            curve.remove(i);
            changed = true;
        }
    }
    if let Some(i) = resp.hover_pos().and_then(nearest) {
        resp.on_hover_text(format!("{} °C → {} %", curve[i].0 .0, curve[i].1 .0));
    }
    changed
}

fn fans_ui(ui: &mut egui::Ui, f: &mut Fans, gpu: bool, status: Option<&Status>, drag: &mut Option<usize>) -> bool {
    let mut changed = false;
    let fixed = f.fixed.is_some();
    ui.horizontal(|ui| {
        if ui.selectable_label(!fixed, "Follow temperature").clicked() && fixed {
            f.fixed = None;
            f.sensor.get_or_insert_with(|| "cpu".into());
            f.curve.get_or_insert_with(|| curve_of(&PRESET_BALANCED));
            changed = true;
        }
        if ui.selectable_label(fixed, "Fixed speed").clicked() && !fixed {
            f.fixed = Some(Num(50.0));
            changed = true;
        }
    });
    if let Some(fx) = &mut f.fixed {
        row(ui, "Speed", |ui| {
            changed |= ui.add(egui::Slider::new(&mut fx.0, 20.0..=100.0).step_by(5.0).suffix(" %")).changed();
        });
        return changed;
    }
    let sensor = f.sensor.get_or_insert_with(|| "probe1".into());
    row(ui, "Follows", |ui| changed |= sensor_combo(ui, "fan-sensor", sensor, gpu));
    let now = status.filter(|s| s.fresh()).and_then(|s| s.temps.get(sensor.as_str()).copied());
    let curve = f.curve.get_or_insert_with(|| curve_of(&PRESET_BALANCED));
    let mut full = false;
    row(ui, "Quick choice", |ui| {
        for (name, preset) in [("Quiet", &PRESET_QUIET[..]), ("Balanced", &PRESET_BALANCED[..]), ("Performance", &PRESET_PERFORMANCE[..])] {
            if ui.button(name).clicked() {
                *curve = curve_of(preset);
                changed = true;
            }
        }
        full = ui.button("Full speed").clicked();
    });
    changed |= curve_ui(ui, curve, now, drag);
    ui.label(RichText::new("Drag the dots to change how fast the fans spin at each temperature. Double-click to add a dot, right-click a dot to remove it.").weak().small());
    if full {
        f.fixed = Some(Num(100.0));
        changed = true;
    }
    changed
}

fn section(ui: &mut egui::Ui, title: &str, hint: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style()).inner_margin(14.0).corner_radius(10.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(title).strong().size(16.0));
        if !hint.is_empty() {
            ui.label(RichText::new(hint).weak());
        }
        ui.add_space(6.0);
        add(ui);
    });
    ui.add_space(12.0);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.autosave();
        ctx.request_repaint_after(Duration::from_millis(500));

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(6.0);
            top_bar(self, ui);
            ui.add_space(6.0);
        });

        if let Some(e) = &self.load_error {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("The profiles file couldn't be read");
                ui.label(e);
                ui.label(format!("Fix or delete {} and reopen this window.", paths::config_file().display()));
            });
            return;
        }

        let mut changed = false;
        egui::SidePanel::left("left").resizable(false).exact_width(310.0).show(ctx, |ui| {
            ui.add_space(8.0);
            section(ui, "Your PC right now", "The colours update every second.", |ui| picture(ui, self.status.as_ref()));
            let names: Vec<String> = self.cfg.profiles.keys().cloned().collect();
            let mut pick = None;
            section(ui, "Profiles", "A saved set of colours and fan speeds.", |ui| {
                for name in &names {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(*name == self.selected, name).clicked() {
                            pick = Some(name.clone());
                        }
                        if *name == self.cfg.active {
                            ui.label(RichText::new("in use").color(Color32::from_rgb(70, 190, 120)).small());
                        }
                    });
                }
                ui.add_space(6.0);
                if ui.button("+ New profile").clicked() {
                    let n = self.unique_name("My profile");
                    self.cfg.profiles.insert(n.clone(), Profile {
                        lighting: Effect::solid("ffffff"),
                        fans: Fans { sensor: Some("cpu".into()), curve: Some(curve_of(&PRESET_BALANCED)), fixed: None },
                        ..Default::default()
                    });
                    pick = Some(n);
                    changed = true;
                }
            });
            if let Some(p) = pick {
                self.select(p);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.add_space(8.0);
                let gpu = self.gpu;
                let status = self.status.clone();
                let sel = self.selected.clone();
                let Some(_) = self.cfg.profiles.get(&sel) else { return };

                section(ui, "Profile", "", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Name").weak());
                        let r = ui.add(egui::TextEdit::singleline(&mut self.name_edit).desired_width(200.0));
                        if r.lost_focus() {
                            let nn = self.name_edit.trim().to_string();
                            if !nn.is_empty() && nn != sel && !self.cfg.profiles.contains_key(&nn) {
                                if let Some(i) = self.cfg.profiles.get_index_of(&sel) {
                                    let p = self.cfg.profiles.shift_remove(&sel).unwrap();
                                    self.cfg.profiles.shift_insert(i, nn.clone(), p);
                                    if self.cfg.active == sel {
                                        self.cfg.active = nn.clone();
                                    }
                                    self.select(nn);
                                    changed = true;
                                }
                            } else {
                                self.name_edit = sel.clone();
                            }
                        }
                        if self.selected == self.cfg.active {
                            ui.label(RichText::new("in use").color(Color32::from_rgb(70, 190, 120)).strong());
                        } else if ui.add(egui::Button::new(RichText::new("Use this profile").strong())).clicked() {
                            self.cfg.active = self.selected.clone();
                            changed = true;
                        }
                        if ui.button("Duplicate").clicked() {
                            let n = self.unique_name(&format!("{} copy", self.selected));
                            let copy = self.cfg.profiles[&self.selected].clone();
                            self.cfg.profiles.insert(n.clone(), copy);
                            self.select(n);
                            changed = true;
                        }
                        let can_delete = self.cfg.profiles.len() > 1 && self.selected != self.cfg.active;
                        let label = if self.confirm_delete { "Click again to delete" } else { "Delete" };
                        if ui.add_enabled(can_delete, egui::Button::new(label)).on_disabled_hover_text("Switch to another profile first").clicked() {
                            if self.confirm_delete {
                                self.cfg.profiles.shift_remove(&self.selected);
                                let a = self.cfg.active.clone();
                                self.select(a);
                                changed = true;
                            } else {
                                self.confirm_delete = true;
                            }
                        }
                    });
                    if self.selected != self.cfg.active {
                        ui.label(RichText::new("You're editing a profile that isn't in use. Click “Use this profile” to see it on your PC.").weak());
                    }
                });

                let sel = self.selected.clone();
                let Some(p) = self.cfg.profiles.get_mut(&sel) else { return };
                section(ui, "Lighting", "How the whole PC lights up.", |ui| {
                    changed |= effect_ui(ui, &mut p.lighting, "main", gpu);
                });
                section(ui, "Lighting for single parts",
                    "Optional: give a fan, a strip or the pump its own colour or effect. If two choices cover the same part, the lower one wins.",
                    |ui| {
                        for z in zones::ZONE_NAMES.iter().filter(|z| **z != "all") {
                            let own = p.zones.contains_key(*z);
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.add_sized([180.0, 22.0], egui::Label::new(RichText::new(zones::label(z)).strong()));
                                if ui.selectable_label(!own, "Same as above").clicked() && own {
                                    p.zones.shift_remove(*z);
                                    changed = true;
                                }
                                if ui.selectable_label(own, "Its own lighting").clicked() && !own {
                                    p.zones.insert(z.to_string(), Effect::solid("ffffff"));
                                    changed = true;
                                }
                            });
                            if let Some(e) = p.zones.get_mut(*z) {
                                ui.indent(*z, |ui| changed |= effect_ui(ui, e, z, gpu));
                            }
                        }
                    });
                section(ui, "Radiator fan speed", "Applies to both radiator fans.", |ui| {
                    changed |= fans_ui(ui, &mut p.fans, gpu, status.as_ref(), &mut self.drag);
                    if p.fan1.is_some() || p.fan2.is_some() {
                        ui.label(RichText::new("This profile also has separate settings for one fan, set in the profiles file.").weak());
                    }
                    ui.add_space(6.0);
                    egui::Frame::new().fill(ui.visuals().faint_bg_color).corner_radius(8.0).inner_margin(8.0).show(ui, |ui| {
                        ui.label(RichText::new("The pump always runs at full speed while the lights are controlled from the PC. That's normal for a water-cooling pump and keeps the CPU cool.").weak());
                    });
                });
            });
        });

        if changed {
            self.changed();
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Coreglow — PC Lighting & Fans")
            .with_app_id("coreglow")
            .with_inner_size([1150.0, 820.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native("Coreglow", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}
