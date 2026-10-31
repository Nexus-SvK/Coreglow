//! coreglow: a window for choosing colours, effects and fan speeds.
//!
//! Edits ~/.config/coreglow/config.toml (saved automatically, checked first) and shows
//! the live values coreglowd publishes. The daemon applies changes within a second.
//! Layout follows the "GUI updates" design: a sidebar with Home, Lighting, Fans and
//! Profiles pages.

use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

use coreglow::config::{
    color_hex, curve_of, parse_color, Config, Effect, EffectKind, Fans, Num, Profile, PRESET_BALANCED,
    PRESET_PERFORMANCE, PRESET_QUIET,
};
use coreglow::status::{Status, HISTORY_EVERY_S};
use coreglow::{paths, sensors, zones};

const SPEED_K: f64 = 16.0; // period (s) = SPEED_K / speed
const SAVE_DELAY: Duration = Duration::from_millis(400);
const GREEN: Color32 = Color32::from_rgb(76, 194, 127);
const CARD: Color32 = Color32::from_rgb(36, 36, 36);
const CARD_LINE: Color32 = Color32::from_rgb(46, 46, 46);
const SIDEBAR: Color32 = Color32::from_rgb(22, 22, 22);
const NAV_ON: Color32 = Color32::from_rgb(38, 48, 58);
const AMBER: Color32 = Color32::from_rgb(232, 163, 61);
const SKY: Color32 = Color32::from_rgb(127, 184, 255);

/// Quick buttons on Home switch to these profiles when they exist.
const QUICK: [(&str, &str); 4] = [("Lights off", "off"), ("Quiet fans", "quiet"), ("All white", "white"), ("Rainbow", "rainbow")];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Lighting,
    Fans,
    Profiles,
}

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
    /// Profile being edited on the Lighting and Fans pages (usually the one in use).
    selected: String,
    page: Page,
    /// Part chosen on the Lighting page; None = whole PC.
    part: Option<&'static str>,
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
    fn new(page: Page) -> Self {
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
            page,
            part: None,
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

    /// Put a profile in use and edit it from now on.
    fn activate(&mut self, name: &str) {
        if self.cfg.profiles.contains_key(name) {
            self.cfg.active = name.to_string();
            self.select(name.to_string());
            self.changed();
        }
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
                let followed_active = self.selected == self.cfg.active;
                self.cfg = c;
                if followed_active || !self.cfg.profiles.contains_key(&self.selected) {
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
                    Ok(()) => ("Saved".into(), false),
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

    fn live(&self) -> Option<&Status> {
        self.status.as_ref().filter(|s| s.fresh())
    }
}

// ---------- small building blocks ----------

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    card_min(ui, 0.0, add)
}

fn card_min(ui: &mut egui::Ui, min_height: f32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, CARD_LINE))
        .corner_radius(12.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(min_height);
            add(ui);
        });
}

/// Height that lines the four Home tiles up.
const TILE_H: f32 = 132.0;

fn title(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).strong().size(16.0));
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized([110.0, 20.0], egui::Label::new(RichText::new(label).weak()));
        add(ui);
    });
}

fn color32(hex: &str) -> Color32 {
    parse_color(hex).map_or(Color32::from_gray(60), |c| Color32::from_rgb(c[0], c[1], c[2]))
}

fn sparkline(ui: &mut egui::Ui, values: &[f64], color: Color32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    if values.len() < 2 {
        ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, "collecting…", FontId::proportional(11.0), ui.visuals().weak_text_color());
        return;
    }
    let lo = values.iter().copied().fold(f64::MAX, f64::min) - 1.0;
    let hi = values.iter().copied().fold(f64::MIN, f64::max) + 1.0;
    let n = values.len() - 1;
    let pts: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            Pos2::new(
                rect.left() + rect.width() * i as f32 / n as f32,
                rect.bottom() - rect.height() * ((v - lo) / (hi - lo)) as f32,
            )
        })
        .collect();
    ui.painter().add(egui::Shape::line(pts, Stroke::new(2.0, color)));
}

fn color_button(ui: &mut egui::Ui, hex: &mut String) -> bool {
    let mut c = parse_color(hex).unwrap_or([255; 3]);
    if egui::color_picker::color_edit_button_srgb(ui, &mut c).changed() {
        *hex = color_hex(c);
        return true;
    }
    false
}

const SWATCHES: [(&str, &str); 7] = [
    ("ffffff", "White"),
    ("ffd9a8", "Warm white"),
    ("ff2a2a", "Red"),
    ("ff8a00", "Orange"),
    ("21d25a", "Green"),
    ("2a6bff", "Blue"),
    ("a43bff", "Purple"),
];

/// Preset swatches plus the custom picker.
fn color_choice(ui: &mut egui::Ui, hex: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for (c, name) in SWATCHES {
            let sel = hex.eq_ignore_ascii_case(c);
            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
            let stroke = if sel { Stroke::new(3.0, ui.visuals().selection.bg_fill) } else { Stroke::new(1.0, Color32::from_gray(70)) };
            ui.painter().rect(rect, 7.0, color32(c), stroke, egui::StrokeKind::Inside);
            if resp.on_hover_text(name).clicked() && !sel {
                *hex = c.to_string();
                changed = true;
            }
        }
        ui.label(RichText::new("custom").weak());
        changed |= color_button(ui, hex);
    });
    changed
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

// ---------- picture of the PC ----------

/// Draws the PC; returns the part that was clicked (when `clickable`).
fn picture(ui: &mut egui::Ui, status: Option<&Status>, height: f32, clickable: bool) -> Option<&'static str> {
    let size = Vec2::new(ui.available_width(), height);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 10.0, Color32::from_rgb(18, 19, 20));
    let col = |z: &str| -> Color32 { status.and_then(|s| s.zones.get(z)).map_or(Color32::from_gray(60), |c| color32(c)) };
    let s = (height / 330.0).min(rect.width() / 520.0);
    let at = |x: f32, y: f32| Pos2::new(rect.left() + 30.0 * s + x * s, rect.top() + 24.0 * s + y * s);
    let weak = Color32::from_gray(160);
    let label = |pos: Pos2, t: &str| {
        p.text(pos, Align2::CENTER_CENTER, t, FontId::proportional(12.0), weak);
    };
    let mut clicked = None;
    let mut hit = |r: Rect, zone: &'static str, name: &str| {
        if !clickable {
            return;
        }
        let resp = ui.interact(r, ui.id().with(zone), Sense::click()).on_hover_text(format!("{name}: click to change its lighting"));
        if resp.hovered() {
            ui.painter().rect_stroke(r.expand(4.0), 8.0, Stroke::new(1.5, SKY), egui::StrokeKind::Outside);
        }
        if resp.clicked() {
            clicked = Some(zone);
        }
    };
    for (x, zone, name) in [(70.0, "left-fan", "Left fan"), (210.0, "right-fan", "Right fan")] {
        let c = at(x, 70.0);
        p.circle_stroke(c, 58.0 * s, Stroke::new(10.0 * s, col(zone)));
        p.circle_filled(c, 16.0 * s, Color32::from_gray(50));
        label(at(x, 152.0), name);
        hit(Rect::from_center_size(c, Vec2::splat(130.0 * s)), zone, name);
    }
    let pc = at(110.0, 225.0);
    p.circle_filled(pc, 40.0 * s, Color32::from_rgb(13, 14, 16));
    p.circle_stroke(pc, 40.0 * s, Stroke::new(8.0 * s, col("pump")));
    label(at(110.0, 285.0), "Pump");
    hit(Rect::from_center_size(pc, Vec2::splat(96.0 * s)), "pump", "Pump head");
    for (i, (x, y)) in [(360.0, 0.0), (395.0, 0.0), (360.0, 125.0), (395.0, 125.0)].into_iter().enumerate() {
        let r = Rect::from_min_size(at(x, y), Vec2::new(18.0 * s, 100.0 * s));
        p.rect_filled(r, 5.0, col(zones::PICTURE_ZONES[i]));
        hit(r, zones::PICTURE_ZONES[i], zones::label(zones::PICTURE_ZONES[i]));
    }
    label(at(386.0, 250.0), "LED strips 1–4");
    clicked
}

/// One bar per part, in the order the lights run (strips, right fan, left fan, pump).
fn led_strip(ui: &mut egui::Ui, status: Option<&Status>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 8.0, Color32::from_rgb(18, 19, 20));
    let parts = [("strip1", 10.0), ("strip2", 10.0), ("strip3", 10.0), ("strip4", 10.0), ("right-fan", 8.0), ("left-fan", 8.0), ("pump", 13.0)];
    let total: f32 = parts.iter().map(|p| p.1).sum();
    let inner = rect.shrink(10.0);
    let mut x = inner.left();
    for (z, n) in parts {
        let w = inner.width() * n / total;
        let c = status.and_then(|s| s.zones.get(z)).map_or(Color32::from_gray(60), |c| color32(c));
        p.rect_filled(Rect::from_min_size(Pos2::new(x + 1.5, inner.top()), Vec2::new(w - 3.0, inner.height())), 4.0, c);
        x += w;
    }
}

// ---------- lighting editor ----------

/// Returns true if anything changed.
fn effect_ui(ui: &mut egui::Ui, e: &mut Effect, id: &str, gpu: bool) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for k in EffectKind::ALL {
            let (name, desc) = effect_label(k);
            let sel = e.effect == k;
            let text = if sel { RichText::new(name).strong() } else { RichText::new(name) };
            if ui.add_sized([150.0, 34.0], egui::SelectableLabel::new(sel, text)).on_hover_text(desc).clicked() && !sel {
                e.set_kind(k);
                changed = true;
            }
        }
    });
    ui.add_space(6.0);
    let kind = e.effect;
    if matches!(kind, EffectKind::Static | EffectKind::Breathing | EffectKind::Flash) {
        let c = e.color.get_or_insert_with(|| "ffffff".into());
        row(ui, "Colour", |ui| changed |= color_choice(ui, c));
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

// ---------- fan curve ----------

fn curve_ui(ui: &mut egui::Ui, curve: &mut Vec<(Num, Num)>, now: Option<f64>, recent: Option<(f64, f64)>, drag: &mut Option<usize>) -> bool {
    const X0: f32 = 20.0;
    const X1: f32 = 100.0;
    let mut changed = false;
    let size = Vec2::new(ui.available_width().min(640.0), 280.0);
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
    if let Some((lo, hi)) = recent {
        let band = Rect::from_min_max(Pos2::new(x((lo as f32).clamp(X0, X1)), t), Pos2::new(x((hi as f32).clamp(X0, X1)), b));
        p.rect_filled(band, 0.0, AMBER.gamma_multiply(0.12));
        p.text(Pos2::new(band.center().x, t + 10.0), Align2::CENTER_CENTER, format!("last 10 min: {lo:.0}–{hi:.0} °C"), FontId::proportional(11.0), AMBER);
    }
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
        p.extend(egui::Shape::dashed_line(&[Pos2::new(xx, t), Pos2::new(xx, b)], Stroke::new(1.5, AMBER), 4.0, 3.0));
    }
    for (i, pt) in pts.iter().enumerate() {
        let hot = *drag == Some(i);
        p.circle(*pt, if hot { 9.0 } else { 7.0 }, vis.extreme_bg_color, Stroke::new(2.5, accent));
    }
    let nearest = |pos: Pos2| -> Option<usize> {
        pts.iter().enumerate().map(|(i, q)| (i, q.distance(pos))).filter(|(_, d)| *d < 14.0).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
    };
    if resp.drag_started() || resp.clicked() {
        *drag = resp.interact_pointer_pos().and_then(nearest).or(if resp.clicked() { None } else { *drag });
    }
    if let (true, Some(i), Some(pos)) = (resp.dragged(), *drag, resp.interact_pointer_pos()) {
        let lo = if i > 0 { curve[i - 1].0 .0 + 1.0 } else { X0 as f64 };
        let hi = if i + 1 < curve.len() { curve[i + 1].0 .0 - 1.0 } else { X1 as f64 };
        curve[i].0 = Num((tx(pos.x) as f64).round().clamp(lo, hi));
        curve[i].1 = Num((ty(pos.y) as f64).round().clamp(0.0, 100.0));
        changed = true;
    }
    // arrow keys move the selected dot by 1 (left/right = temperature, up/down = speed)
    if let Some(i) = *drag {
        if i < curve.len() && !resp.dragged() {
            let (mut dx, mut dy) = (0.0, 0.0);
            ui.input(|inp| {
                if inp.key_pressed(egui::Key::ArrowLeft) { dx -= 1.0 }
                if inp.key_pressed(egui::Key::ArrowRight) { dx += 1.0 }
                if inp.key_pressed(egui::Key::ArrowUp) { dy += 1.0 }
                if inp.key_pressed(egui::Key::ArrowDown) { dy -= 1.0 }
            });
            if dx != 0.0 || dy != 0.0 {
                let lo = if i > 0 { curve[i - 1].0 .0 + 1.0 } else { X0 as f64 };
                let hi = if i + 1 < curve.len() { curve[i + 1].0 .0 - 1.0 } else { X1 as f64 };
                curve[i].0 = Num((curve[i].0 .0 + dx).clamp(lo, hi));
                curve[i].1 = Num((curve[i].1 .0 + dy).clamp(0.0, 100.0));
                changed = true;
            }
        }
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
            *drag = None;
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
    let now = status.and_then(|s| s.temps.get(sensor.as_str()).copied());
    let recent = status.and_then(|s| s.history.get(sensor.as_str())).filter(|h| h.len() > 1).map(|h| {
        (h.iter().copied().fold(f64::MAX, f64::min), h.iter().copied().fold(f64::MIN, f64::max))
    });
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
    changed |= curve_ui(ui, curve, now, recent, drag);
    ui.label(RichText::new("Drag a dot to change it · double-click to add · right-click to remove · click a dot, then use the arrow keys for fine steps").weak().small());
    if full {
        f.fixed = Some(Num(100.0));
        changed = true;
    }
    changed
}

// ---------- pages ----------

fn sidebar(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
        ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(4.0, ui.visuals().selection.bg_fill));
        ui.painter().circle_filled(r.center(), 3.0, Color32::from_gray(230));
        ui.label(RichText::new("Coreglow").strong().size(17.0));
    });
    ui.add_space(14.0);
    for (page, name) in [(Page::Home, "Home"), (Page::Lighting, "Lighting"), (Page::Fans, "Fans"), (Page::Profiles, "Profiles")] {
        let on = app.page == page;
        let text = if on { RichText::new(name).strong().color(Color32::WHITE) } else { RichText::new(name) };
        let btn = egui::Button::new(text).fill(if on { NAV_ON } else { Color32::TRANSPARENT }).min_size(Vec2::new(ui.available_width(), 40.0));
        if ui.add(btn).clicked() {
            app.page = page;
        }
    }
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        ui.add_space(12.0);
        egui::Frame::new().fill(Color32::from_rgb(32, 32, 32)).corner_radius(10.0).inner_margin(10.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            match app.live() {
                Some(s) => {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 4.0, GREEN);
                        ui.label(RichText::new("Service running").color(GREEN));
                    });
                    if let Some(e) = &s.error {
                        ui.colored_label(ui.visuals().warn_fg_color, e);
                    }
                }
                None => {
                    let why = app.status.as_ref().and_then(|s| s.error.clone()).unwrap_or_else(|| "Service isn't running".into());
                    ui.colored_label(ui.visuals().warn_fg_color, why);
                    if ui.button("Start it").clicked() {
                        let _ = std::process::Command::new("systemctl").args(["--user", "start", "coreglowd.service"]).spawn();
                    }
                }
            }
            let (msg, err) = &app.save_msg;
            if !msg.is_empty() {
                if *err {
                    ui.colored_label(ui.visuals().error_fg_color, msg);
                } else {
                    ui.label(RichText::new(msg).weak().small());
                }
            }
        });
    });
}

fn page_header(ui: &mut egui::Ui, heading: &str, profile: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(heading).strong().size(24.0));
        ui.label(RichText::new(format!("· profile “{profile}”")).weak().size(16.0));
    });
    ui.add_space(8.0);
}

fn home(app: &mut App, ui: &mut egui::Ui) {
    let status = app.live().cloned();
    let s = status.as_ref();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Your PC").strong().size(24.0));
            let line = match s {
                Some(st) if st.error.is_some() => "There's a problem — see the sidebar.",
                Some(_) => "Everything is running normally.",
                None => "The lighting service isn't running.",
            };
            ui.label(RichText::new(line).weak());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let names: Vec<String> = app.cfg.profiles.keys().cloned().collect();
            let mut pick = None;
            egui::ComboBox::from_id_salt("home-profile").selected_text(&app.cfg.active).width(200.0).show_ui(ui, |ui| {
                for n in &names {
                    if ui.selectable_label(*n == app.cfg.active, n).clicked() {
                        pick = Some(n.clone());
                    }
                }
            });
            ui.label(RichText::new("Profile").weak());
            if let Some(n) = pick {
                app.activate(&n);
            }
        });
    });
    ui.add_space(10.0);

    let hist = |k: &str| s.and_then(|st| st.history.get(k)).cloned().unwrap_or_default();
    let temp = |k: &str| s.and_then(|st| st.temps.get(k)).map_or("–".into(), |v| format!("{v:.0} °C"));
    let rpm = |k: &str| s.and_then(|st| st.rpm.get(k)).copied().unwrap_or(0);
    let duty = s.and_then(|st| st.fan_duty.get("fan1")).copied();
    let minutes = coreglow::status::HISTORY_LEN as u64 * HISTORY_EVERY_S / 60;
    ui.columns(4, |cols| {
        card_min(&mut cols[0], TILE_H, |ui| {
            ui.label(RichText::new("CPU").weak());
            ui.label(RichText::new(temp("cpu")).strong().size(28.0));
            sparkline(ui, &hist("cpu"), AMBER, 34.0);
            ui.label(RichText::new(format!("last {minutes} minutes")).weak().small());
        });
        card_min(&mut cols[1], TILE_H, |ui| {
            ui.label(RichText::new("Inside the case").weak());
            ui.label(RichText::new(temp("probe1")).strong().size(28.0));
            sparkline(ui, &hist("probe1"), SKY, 34.0);
            ui.label(RichText::new(format!("probe 2: {}", temp("probe2"))).weak().small());
        });
        card_min(&mut cols[2], TILE_H, |ui| {
            ui.label(RichText::new("Radiator fans").weak());
            ui.label(RichText::new(duty.map_or("–".into(), |d| format!("{d} %"))).strong().size(28.0));
            ui.label(format!("Right fan  {} rpm", rpm("fan1")));
            ui.label(format!("Left fan   {} rpm", rpm("fan2")));
        });
        card_min(&mut cols[3], TILE_H, |ui| {
            ui.label(RichText::new("Pump").weak());
            ui.label(RichText::new(format!("{} rpm", rpm("pump"))).strong().size(28.0));
            ui.label(RichText::new("Full speed while the lighting is controlled from the PC. That's normal.").weak().small());
        });
    });
    ui.add_space(12.0);

    let mut go_part = None;
    ui.columns(2, |cols| {
        card(&mut cols[0], |ui| {
            ui.horizontal(|ui| {
                title(ui, "Your PC right now");
                ui.label(RichText::new("Click a part to change its lighting").weak());
            });
            ui.add_space(6.0);
            go_part = picture(ui, s, 300.0, true);
        });
        card(&mut cols[1], |ui| {
            title(ui, "Quick changes");
            ui.add_space(6.0);
            let active = app.cfg.active.clone();
            let mut bright_changed = false;
            if let Some(p) = app.cfg.profiles.get_mut(&active) {
                if p.lighting.effect != EffectKind::Off {
                    let mut b = p.lighting.brightness.map_or(100.0, |b| b.0);
                    row(ui, "Brightness", |ui| {
                        if ui.add(egui::Slider::new(&mut b, 5.0..=100.0).step_by(5.0).suffix(" %")).changed() {
                            p.lighting.brightness = Some(Num(b));
                            bright_changed = true;
                        }
                    });
                }
            }
            if bright_changed {
                app.changed();
            }
            ui.add_space(8.0);
            let mut switch = None;
            egui::Grid::new("quick").num_columns(2).spacing([8.0, 8.0]).show(ui, |ui| {
                for (i, (label, profile)) in QUICK.iter().enumerate() {
                    let exists = app.cfg.profiles.contains_key(*profile);
                    let b = egui::Button::new(*label).min_size(Vec2::new(150.0, 40.0));
                    if ui.add_enabled(exists, b).on_disabled_hover_text(format!("Make a profile called “{profile}” to use this")).clicked() {
                        switch = Some(*profile);
                    }
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
            if let Some(p) = switch {
                app.activate(p);
            }
        });
    });
    if let Some(z) = go_part {
        app.select(app.cfg.active.clone());
        app.part = Some(z);
        app.page = Page::Lighting;
    }
}

fn lighting(app: &mut App, ui: &mut egui::Ui) -> bool {
    let mut changed = false;
    let status = app.live().cloned();
    page_header(ui, "Lighting", &app.selected);
    led_strip(ui, status.as_ref());
    ui.add_space(12.0);
    let gpu = app.gpu;
    let sel = app.selected.clone();
    let Some(p) = app.cfg.profiles.get_mut(&sel) else { return false };
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(250.0);
            card(ui, |ui| {
                title(ui, "What to change");
                ui.add_space(4.0);
                if ui.add(egui::SelectableLabel::new(app.part.is_none(), RichText::new("Whole PC").strong())).clicked() {
                    app.part = None;
                }
                ui.label(RichText::new("Or give one part its own lighting:").weak().small());
                for z in zones::ZONE_NAMES.iter().filter(|z| **z != "all") {
                    let own = p.zones.contains_key(*z);
                    ui.horizontal(|ui| {
                        let swatch = status.as_ref().and_then(|s| s.zones.get(*z).or_else(|| s.zones.get(match *z {
                            "strips" => "strip1",
                            "fans" => "right-fan",
                            _ => z,
                        }))).map_or(Color32::from_gray(60), |c| color32(c));
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        ui.painter().rect_filled(r, 3.0, swatch);
                        let indent = matches!(*z, "strip1" | "strip2" | "strip3" | "strip4" | "left-fan" | "right-fan");
                        let name = if indent { format!("   {}", zones::label(z)) } else { zones::label(z).to_string() };
                        if ui.add(egui::SelectableLabel::new(app.part == Some(*z), name)).clicked() {
                            app.part = Some(*z);
                        }
                        ui.label(RichText::new(if own { "own" } else { "same" }).small().color(if own { SKY } else { Color32::from_gray(140) }));
                    });
                }
            });
        });
        ui.vertical(|ui| {
            card(ui, |ui| match app.part {
                None => {
                    title(ui, "Effect for the whole PC");
                    ui.add_space(6.0);
                    changed |= effect_ui(ui, &mut p.lighting, "main", gpu);
                }
                Some(z) => {
                    title(ui, zones::label(z));
                    ui.add_space(6.0);
                    let own = p.zones.contains_key(z);
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!own, "Same as the whole PC").clicked() && own {
                            p.zones.shift_remove(z);
                            changed = true;
                        }
                        if ui.selectable_label(own, "Its own lighting").clicked() && !own {
                            p.zones.insert(z.to_string(), Effect::solid("ffffff"));
                            changed = true;
                        }
                    });
                    ui.add_space(6.0);
                    if let Some(e) = p.zones.get_mut(z) {
                        changed |= effect_ui(ui, e, z, gpu);
                        ui.add_space(4.0);
                        ui.label(RichText::new("If two choices cover the same part (for example “Both fans” and “Left fan”), the one lower in the list wins.").weak().small());
                    } else {
                        ui.label(RichText::new("This part follows the whole-PC lighting.").weak());
                    }
                }
            });
        });
    });
    changed
}

fn fans_page(app: &mut App, ui: &mut egui::Ui) -> bool {
    let mut changed = false;
    let status = app.live().cloned();
    page_header(ui, "Fan speed", &app.selected);
    let gpu = app.gpu;
    let sel = app.selected.clone();
    let Some(p) = app.cfg.profiles.get_mut(&sel) else { return false };
    let s = status.as_ref();
    ui.horizontal_top(|ui| {
        let w = (ui.available_width() - 290.0).max(380.0);
        ui.vertical(|ui| {
            ui.set_width(w);
            card(ui, |ui| {
                title(ui, "Both radiator fans");
                ui.add_space(6.0);
                changed |= fans_ui(ui, &mut p.fans, gpu, s, &mut app.drag);
                if p.fan1.is_some() || p.fan2.is_some() {
                    ui.label(RichText::new("This profile also has separate settings for one fan, set in the profiles file.").weak());
                }
            });
        });
        ui.vertical(|ui| {
            card(ui, |ui| {
                title(ui, "Right now");
                let rpm = |k: &str| s.and_then(|st| st.rpm.get(k)).copied().unwrap_or(0);
                let duty = |k: &str| s.and_then(|st| st.fan_duty.get(k)).copied().unwrap_or(0);
                ui.label(format!("Right fan   {} rpm · {} %", rpm("fan1"), duty("fan1")));
                ui.label(format!("Left fan    {} rpm · {} %", rpm("fan2"), duty("fan2")));
                ui.add_space(4.0);
                sparkline(ui, &s.and_then(|st| st.history.get("fan1_rpm")).cloned().unwrap_or_default(), SKY, 30.0);
                ui.label(RichText::new("right fan, last 10 minutes").weak().small());
            });
            ui.add_space(12.0);
            card(ui, |ui| {
                title(ui, "Pump");
                ui.label(RichText::new(format!("{} rpm", s.and_then(|st| st.rpm.get("pump")).copied().unwrap_or(0))).strong().size(22.0));
                ui.label(RichText::new("Runs at full speed while the lighting is controlled from the PC. That keeps the CPU cool and is normal.").weak());
            });
            ui.add_space(12.0);
            card(ui, |ui| {
                title(ui, "When Coreglow is closed");
                ui.label(RichText::new("The controller keeps its own backup curve: fans follow case probe 1, pump at full speed.").weak());
            });
        });
    });
    changed
}

fn profiles_page(app: &mut App, ui: &mut egui::Ui) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Profiles").strong().size(24.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(RichText::new("+ New profile").strong()).min_size(Vec2::new(0.0, 36.0))).clicked() {
                let n = app.unique_name("My profile");
                app.cfg.profiles.insert(n.clone(), Profile {
                    lighting: Effect::solid("ffffff"),
                    fans: Fans { sensor: Some("cpu".into()), curve: Some(curve_of(&PRESET_BALANCED)), fixed: None },
                    ..Default::default()
                });
                app.select(n);
                changed = true;
            }
        });
    });
    ui.label(RichText::new("A profile is a saved set of colours and fan speeds. Click one to edit it.").weak());
    ui.add_space(10.0);

    let names: Vec<String> = app.cfg.profiles.keys().cloned().collect();
    let mut pick = None;
    // card = 196 content + 2 x 12 margin + stroke; laid out in rows that fit the window
    let per_row = (((ui.available_width() + 12.0) / 234.0).floor() as usize).max(1);
    for chunk in names.chunks(per_row) {
      ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(12.0);
        for name in chunk {
            let p = &app.cfg.profiles[name];
            let active = *name == app.cfg.active;
            let sel = *name == app.selected;
            let frame = egui::Frame::new()
                .fill(if sel { NAV_ON } else { CARD })
                .stroke(if sel { Stroke::new(2.0, ui.visuals().selection.bg_fill) } else { Stroke::new(1.0, CARD_LINE) })
                .corner_radius(12.0)
                .inner_margin(12.0);
            let resp = frame
                .show(ui, |ui| ui.vertical(|ui| {
                    ui.set_width(196.0);
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                    let preview = p.frame(1.0, &Default::default());
                    let (r, _) = ui.allocate_exact_size(Vec2::new(196.0, 10.0), Sense::hover());
                    let all = zones::zone("all");
                    let n = all.len() as f32;
                    for (k, idx) in all.iter().enumerate() {
                        let c = Color32::from_rgb(preview[idx * 3], preview[idx * 3 + 1], preview[idx * 3 + 2]);
                        let x0 = r.left() + r.width() * k as f32 / n;
                        ui.painter().rect_filled(Rect::from_min_max(Pos2::new(x0, r.top()), Pos2::new(x0 + r.width() / n + 0.5, r.bottom())), 0.0, c);
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).strong());
                        if active {
                            ui.label(RichText::new("in use").small().color(GREEN));
                        }
                    });
                    let fans = match p.fans.fixed {
                        Some(f) => format!("Fans: fixed {} %", f.0),
                        None => format!("Fans: follow {}", sensors::label(&p.fans.sensor_name())),
                    };
                    ui.label(RichText::new(fans).weak().small());
                }))
                .response
                .interact(Sense::click());
            if resp.clicked() {
                pick = Some(name.clone());
            }
        }
      });
      ui.add_space(12.0);
    }
    if let Some(n) = pick {
        app.select(n);
    }

    ui.add_space(14.0);
    let sel = app.selected.clone();
    if app.cfg.profiles.contains_key(&sel) {
        card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Name").weak());
                let r = ui.add(egui::TextEdit::singleline(&mut app.name_edit).desired_width(200.0));
                if r.lost_focus() {
                    let nn = app.name_edit.trim().to_string();
                    if !nn.is_empty() && nn != sel && !app.cfg.profiles.contains_key(&nn) {
                        if let Some(i) = app.cfg.profiles.get_index_of(&sel) {
                            let p = app.cfg.profiles.shift_remove(&sel).unwrap();
                            app.cfg.profiles.shift_insert(i, nn.clone(), p);
                            if app.cfg.active == sel {
                                app.cfg.active = nn.clone();
                            }
                            app.select(nn);
                            changed = true;
                        }
                    } else {
                        app.name_edit = sel.clone();
                    }
                }
                if app.selected == app.cfg.active {
                    ui.label(RichText::new("in use").color(GREEN).strong());
                } else if ui.add(egui::Button::new(RichText::new("Use this profile").strong())).clicked() {
                    app.cfg.active = app.selected.clone();
                    changed = true;
                }
                if ui.button("Edit lighting").clicked() {
                    app.part = None;
                    app.page = Page::Lighting;
                }
                if ui.button("Edit fans").clicked() {
                    app.page = Page::Fans;
                }
                if ui.button("Duplicate").clicked() {
                    let n = app.unique_name(&format!("{} copy", app.selected));
                    let copy = app.cfg.profiles[&app.selected].clone();
                    app.cfg.profiles.insert(n.clone(), copy);
                    app.select(n);
                    changed = true;
                }
                let can_delete = app.cfg.profiles.len() > 1 && app.selected != app.cfg.active;
                let label = if app.confirm_delete { "Click again to delete" } else { "Delete" };
                if ui.add_enabled(can_delete, egui::Button::new(label)).on_disabled_hover_text("Switch to another profile first").clicked() {
                    if app.confirm_delete {
                        app.cfg.profiles.shift_remove(&app.selected);
                        let a = app.cfg.active.clone();
                        app.select(a);
                        changed = true;
                    } else {
                        app.confirm_delete = true;
                    }
                }
            });
            if app.selected != app.cfg.active {
                ui.label(RichText::new("You're editing a profile that isn't in use. Click “Use this profile” to see it on your PC.").weak());
            }
        });
    }
    changed
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.autosave();
        ctx.request_repaint_after(Duration::from_millis(500));

        if let Some(e) = &self.load_error {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("The profiles file couldn't be read");
                ui.label(e);
                ui.label(format!("Fix or delete {} and reopen this window.", paths::config_file().display()));
            });
            return;
        }

        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(200.0)
            .frame(egui::Frame::new().fill(SIDEBAR).inner_margin(egui::Margin::symmetric(12, 0)))
            .show(ctx, |ui| sidebar(self, ui));

        let mut changed = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin::symmetric(24, 20)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| match self.page {
                    Page::Home => home(self, ui),
                    Page::Lighting => changed |= lighting(self, ui),
                    Page::Fans => changed |= fans_page(self, ui),
                    Page::Profiles => changed |= profiles_page(self, ui),
                });
            });
        if changed {
            self.changed();
        }
    }
}

fn main() -> eframe::Result {
    // `coreglow --page lighting|fans|profiles` opens on that page (handy for screenshots).
    let args: Vec<String> = std::env::args().collect();
    let page = match args.iter().position(|a| a == "--page").and_then(|i| args.get(i + 1)).map(String::as_str) {
        Some("lighting") => Page::Lighting,
        Some("fans") => Page::Fans,
        Some("profiles") => Page::Profiles,
        _ => Page::Home,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Coreglow — PC Lighting & Fans")
            .with_app_id("coreglow")
            .with_inner_size([1180.0, 840.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("Coreglow", options, Box::new(move |_cc| Ok(Box::new(App::new(page)))))
}
