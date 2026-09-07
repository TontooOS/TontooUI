//! Gauge — SwiftUI-style gauge element for TontooOS.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//! use tontooui::GaugeStyle;
//!
//! // Plain gauge with label
//! let g = Gauge::new(42.0).label("Foo");
//!
//! // Min / Max / Current labels
//! let g2 = Gauge::new(42.0).in_range(0.0, 100.0)
//!     .label("Foo")
//!     .current_value_label("42")
//!     .minimum_value_label("0")
//!     .maximum_value_label("100");
//!
//! // Styled gauges
//! let circular = Gauge::new(0.42).label("Foo").gauge_style(GaugeStyle::Circular).tint(Color::from_rgb(10,132,255));
//! let linear = Gauge::new(0.42).gauge_style(GaugeStyle::Linear);
//! ```

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{DrawingArea, Orientation};
use crate::elements::resolve_scheme;

// ─────────────────────────────────────────────────────────────
// Style
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeStyle {
    /// Default linear capacity — fills from leading to trailing (used for the
    /// plain `Gauge` without an explicit style in the screenshots).
    Default,
    /// Circular open ring with a marker at the current value.
    Circular,
    /// Linear bar with a circular marker at the current value.
    Linear,
    /// Linear capacity — filled bar from leading to trailing.
    LinearCapacity,
    /// Small accessory linear with marker.
    AccessoryLinear,
    /// Small accessory linear capacity — thin filled bar.
    AccessoryLinearCapacity,
    /// Small accessory circular open ring with marker.
    AccessoryCircular,
    /// Small accessory circular closed ring partially filled.
    AccessoryCircularCapacity,
}

impl Default for GaugeStyle {
    fn default() -> Self { Self::Default }
}

// ─────────────────────────────────────────────────────────────
// Tint
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum GaugeTint {
    Single(Color),
    Gradient(Vec<Color>),
}

impl From<Color> for GaugeTint {
    fn from(c: Color) -> Self { Self::Single(c) }
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn clamp01(v: f32) -> f32 { v.clamp(0.0, 1.0) }

fn norm(value: f32, min: f32, max: f32) -> f32 {
    let r = max - min;
    if r.abs() < f32::EPSILON { 0.0 } else { clamp01((value - min) / r) }
}

fn set_rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, radius: f64) {
    let radius = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let pi = std::f64::consts::PI;
    let _ = cr.move_to(x + radius, y);
    let _ = cr.line_to(x + w - radius, y);
    let _ = cr.arc(x + w - radius, y + radius, radius, -pi / 2.0, 0.0);
    let _ = cr.line_to(x + w, y + h - radius);
    let _ = cr.arc(x + w - radius, y + h - radius, radius, 0.0, pi / 2.0);
    let _ = cr.line_to(x + radius, y + h);
    let _ = cr.arc(x + radius, y + h - radius, radius, pi / 2.0, pi);
    let _ = cr.line_to(x, y + radius);
    let _ = cr.arc(x + radius, y + radius, radius, pi, 3.0 * pi / 2.0);
    let _ = cr.close_path();
}

// ─────────────────────────────────────────────────────────────
// Drawing primitives
// ─────────────────────────────────────────────────────────────

fn draw_linear_capacity(cr: &cairo::Context, w: i32, h: i32, n: f64, tint: &GaugeTint, accessory: bool, is_dark: bool) {
    let track_h = if accessory { 4.0 } else { 6.0 };
    let y = (h as f64 - track_h) / 2.0;
    let x = 6.0;
    let ww = w as f64 - 12.0;
    let radius = track_h / 2.0;

    // track — dark #38383A vs light #D1D1D6 / #E5E5EA
    let _ = cr.save();
    set_rounded_rect(cr, x, y, ww, track_h, radius);
    if is_dark {
        let _ = cr.set_source_rgba(0.22, 0.22, 0.23, 1.0);
    } else {
        let _ = cr.set_source_rgba(0.82, 0.82, 0.84, 1.0);
    }
    let _ = cr.fill();
    let _ = cr.restore();

    if n <= 0.0001 { return; }
    let fill_w = (ww * n).max(radius * 2.0).min(ww);

    // handle gradient by splitting or using linear pattern
    match tint {
        GaugeTint::Single(c) => {
            let _ = cr.save();
            set_rounded_rect(cr, x, y, fill_w, track_h, radius);
            let _ = cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, c.a as f64);
            let _ = cr.fill();
            let _ = cr.restore();
        }
        GaugeTint::Gradient(colors) if colors.len() >= 2 => {
            let pat = cairo::LinearGradient::new(x, y, x + ww, y);
            for (i, col) in colors.iter().enumerate() {
                let offset = i as f64 / (colors.len() - 1) as f64;
                let _ = pat.add_color_stop_rgba(offset, col.r as f64, col.g as f64, col.b as f64, col.a as f64);
            }
            let _ = cr.save();
            set_rounded_rect(cr, x, y, fill_w, track_h, radius);
            let _ = cr.set_source(&pat);
            let _ = cr.fill();
            let _ = cr.restore();
        }
        _ => {}
    }
}

fn draw_linear_marker(cr: &cairo::Context, w: i32, h: i32, n: f64, tint: &GaugeTint, accessory: bool, is_dark: bool) {
    let track_h = if accessory { 2.5 } else { 5.0 };
    let track_y = (h as f64 - track_h) / 2.0;
    let pad = if accessory { 8.0 } else { 14.0 };
    let x = pad;
    let ww = w as f64 - pad * 2.0;
    let radius = track_h / 2.0;

    // track — adaptive
    let _ = cr.save();
    set_rounded_rect(cr, x, track_y, ww, track_h, radius);
    if is_dark {
        if accessory {
            let _ = cr.set_source_rgba(0.35, 0.35, 0.36, 1.0);
        } else {
            let _ = cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        }
    } else {
        if accessory {
            let _ = cr.set_source_rgba(0.68, 0.68, 0.70, 1.0);
        } else {
            let _ = cr.set_source_rgba(0.56, 0.56, 0.58, 1.0);
        }
    }
    let _ = cr.fill();
    let _ = cr.restore();

    // marker dot position
    let cx = x + ww * n;
    let cy = h as f64 / 2.0;
    let r = if accessory { 3.5 } else { 6.0 };
    let border = if accessory { 1.0 } else { 1.6 };

    // outer white circle (for non-accessory) or tint for accessory
    let (mr, mg, mb) = match tint {
        GaugeTint::Single(c) => (c.r as f64, c.g as f64, c.b as f64),
        GaugeTint::Gradient(cs) => cs.first().map(|c| (c.r as f64, c.g as f64, c.b as f64)).unwrap_or((1.0, 1.0, 1.0)),
    };
    // For default linear, marker is white ring with dark center; for tinted, use tint
    let use_tint = match tint {
        GaugeTint::Single(c) => !(c.r == 0.016 && c.g == 0.525), // if custom tint, use it
        GaugeTint::Gradient(_) => true,
    };
    if accessory && use_tint {
        let _ = cr.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        let _ = cr.set_source_rgba(mr, mg, mb, 1.0);
        let _ = cr.fill();
    } else if accessory {
        // dot — white in dark, dark in light
        let _ = cr.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        if is_dark {
            let _ = cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        } else {
            let _ = cr.set_source_rgba(0.11, 0.11, 0.12, 1.0);
        }
        let _ = cr.fill();
    } else {
        // Normal linear: pill marker — adaptive outer/inner
        let _ = cr.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        if is_dark {
            let _ = cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        } else {
            let _ = cr.set_source_rgba(0.11, 0.11, 0.12, 1.0);
        }
        let _ = cr.fill();
        let _ = cr.arc(cx, cy, r - border, 0.0, std::f64::consts::PI * 2.0);
        if is_dark {
            let _ = cr.set_source_rgba(0.11, 0.11, 0.11, 1.0);
        } else {
            let _ = cr.set_source_rgba(0.96, 0.96, 0.96, 1.0);
        }
        let _ = cr.fill();
        if use_tint && !matches!(tint, GaugeTint::Single(c) if c.r == 0.016) {
            // thin tint ring around marker for colored variant
            let _ = cr.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
            let _ = cr.set_source_rgba(mr, mg, mb, 1.0);
            let _ = cr.set_line_width(1.2);
            let _ = cr.stroke();
        }
    }
}

fn draw_circular_marker(cr: &cairo::Context, w: i32, h: i32, n: f64, tint: &GaugeTint, accessory: bool, is_dark: bool) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let size = w.min(h) as f64;
    let stroke = if accessory { 3.0 } else { 5.0 };
    let radius = if accessory { size * 0.36 } else { size * 0.38 };
    let start = -210.0_f64.to_radians();
    let end = 30.0_f64.to_radians();
    let sweep = end - start; // 240°

    // track arc (open) — adaptive
    let _ = cr.set_line_width(stroke);
    let _ = cr.set_line_cap(cairo::LineCap::Round);
    if is_dark {
        let _ = cr.set_source_rgba(0.22, 0.22, 0.23, 1.0);
    } else {
        let _ = cr.set_source_rgba(0.82, 0.82, 0.84, 1.0);
    }
    let _ = cr.arc(cx, cy, radius, start, end);
    let _ = cr.stroke();

    // marker at position
    let angle = start + n * sweep;
    let mx = cx + radius * angle.cos();
    let my = cy + radius * angle.sin();
    let r = if accessory { 3.2 } else { 4.5 };
    let (mr, mg, mb) = match tint {
        GaugeTint::Single(c) => (c.r as f64, c.g as f64, c.b as f64),
        GaugeTint::Gradient(cs) => cs.first().map(|c| (c.r as f64, c.g as f64, c.b as f64)).unwrap_or((0.016, 0.525, 0.941)),
    };
    // outer + tint inner — outer adapts for contrast
    let _ = cr.arc(mx, my, r + 1.2, 0.0, std::f64::consts::PI * 2.0);
    if is_dark {
        let _ = cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    } else {
        let _ = cr.set_source_rgba(0.11, 0.11, 0.12, 1.0);
    }
    let _ = cr.fill();
    let _ = cr.arc(mx, my, r, 0.0, std::f64::consts::PI * 2.0);
    let _ = cr.set_source_rgba(mr, mg, mb, 1.0);
    let _ = cr.fill();
}

fn draw_circular_capacity(cr: &cairo::Context, w: i32, h: i32, n: f64, tint: &GaugeTint, accessory: bool, is_dark: bool) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let size = w.min(h) as f64;
    let stroke = if accessory { 5.0 } else { 7.0 };
    let radius = if accessory { size * 0.34 } else { size * 0.36 };

    // For accessoryCircularCapacity: closed ring, partially filled via arc + track
    // Use full circle for capacity
    let pi = std::f64::consts::PI;
    // track — adaptive
    let _ = cr.set_line_width(stroke);
    let _ = cr.set_line_cap(cairo::LineCap::Butt);
    if is_dark {
        let _ = cr.set_source_rgba(0.22, 0.22, 0.23, 1.0);
    } else {
        let _ = cr.set_source_rgba(0.82, 0.82, 0.84, 1.0);
    }
    let _ = cr.arc(cx, cy, radius, 0.0, 2.0 * pi);
    let _ = cr.stroke();

    if n > 0.0001 {
        let (mr, mg, mb, ma) = match tint {
            GaugeTint::Single(c) => (c.r as f64, c.g as f64, c.b as f64, c.a as f64),
            GaugeTint::Gradient(cs) => cs.first().map(|c| (c.r as f64, c.g as f64, c.b as f64, c.a as f64)).unwrap_or((0.016, 0.525, 0.941, 1.0)),
        };
        let _ = cr.set_source_rgba(mr, mg, mb, ma);
        let _ = cr.set_line_width(stroke);
        let _ = cr.set_line_cap(cairo::LineCap::Round);
        // start at top (-90deg)
        let start = -pi / 2.0;
        let end = start + n * 2.0 * pi;
        let _ = cr.arc(cx, cy, radius, start, end);
        let _ = cr.stroke();
    }
}

// ─────────────────────────────────────────────────────────────
// Gauge
// ─────────────────────────────────────────────────────────────

pub struct Gauge {
    id: WidgetId,
    value: f32,
    min: f32,
    max: f32,
    label: Option<String>,
    current_value_label: Option<String>,
    minimum_value_label: Option<String>,
    maximum_value_label: Option<String>,
    style: GaugeStyle,
    tint: GaugeTint,
    width: f32,
    height: f32,
    position_mode: PositionMode,
    position: Position,
}

impl Gauge {
    /// Create a new gauge displaying `value` in the default `0.0..=1.0` range.
    /// Use `.in_range(min, max)` and `.label("...")` to configure.
    pub fn new(value: f32) -> Self {
        Self {
            id: next_widget_id(),
            value,
            min: 0.0,
            max: 1.0,
            label: None,
            current_value_label: None,
            minimum_value_label: None,
            maximum_value_label: None,
            style: GaugeStyle::Default,
            tint: GaugeTint::Single(Color::new(0.016, 0.525, 0.941, 1.0)),
            width: 220.0,
            height: 0.0, // auto
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Convenience: `Gauge::with_label("Foo", 42.0)` — mirrors SwiftUI `Gauge(value: ...) { Text("Foo") }`.
    pub fn with_label(label: impl Into<String>, value: f32) -> Self {
        Self::new(value).label(label)
    }

    pub fn value(mut self, v: f32) -> Self { self.value = v; self }
    pub fn in_range(mut self, min: f32, max: f32) -> Self { self.min = min; self.max = max; self }
    pub fn range(mut self, min: f32, max: f32) -> Self { self.min = min; self.max = max; self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn current_value_label(mut self, l: impl Into<String>) -> Self { self.current_value_label = Some(l.into()); self }
    pub fn minimum_value_label(mut self, l: impl Into<String>) -> Self { self.minimum_value_label = Some(l.into()); self }
    pub fn maximum_value_label(mut self, l: impl Into<String>) -> Self { self.maximum_value_label = Some(l.into()); self }
    pub fn gauge_style(mut self, s: GaugeStyle) -> Self { self.style = s; self }
    pub fn style(mut self, s: GaugeStyle) -> Self { self.style = s; self }
    pub fn tint(mut self, c: Color) -> Self { self.tint = GaugeTint::Single(c); self }
    pub fn tint_gradient(mut self, colors: Vec<Color>) -> Self { self.tint = GaugeTint::Gradient(colors); self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn height(mut self, h: f32) -> Self { self.height = h; self }
    pub fn frame(mut self, w: f32, h: f32) -> Self { self.width = w; self.height = h; self }

    // getters
    pub fn current_value(&self) -> f32 { self.value }
    pub fn normalized(&self) -> f32 { norm(self.value, self.min, self.max) }

    pub fn to_view(self) -> View {
        let w = self.width.max(1.0);
        let h = self.height;
        // estimate height if not set based on style
        let hh = if h > 0.0 { h } else {
            match self.style {
                GaugeStyle::Circular => 110.0,
                GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity => 56.0,
                GaugeStyle::AccessoryLinear | GaugeStyle::AccessoryLinearCapacity => 34.0,
                _ => 42.0,
            }
        };
        View::new(self).with_frame(0.0, 0.0, w, hh)
    }
}

impl Default for Gauge {
    fn default() -> Self { Self::new(0.5) }
}

impl ViewContent for Gauge {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let w = w.max(40.0);
        let n = norm(self.value, self.min, self.max) as f64;
        let tint = self.tint.clone();
        let scheme = resolve_scheme(None);
        let is_dark = scheme == uikit::app::ColorScheme::Dark;
        // palette per AGENTS.md spec: Dark #1d1d1d, Light #ececec — text adapts
        let lbl_color = if is_dark { "rgba(235,235,245,0.85)" } else { "rgba(28,28,30,0.92)" };
        let sec_color = if is_dark { "rgba(235,235,245,0.55)" } else { "rgba(60,60,67,0.60)" };
        let strong_color = if is_dark { "white" } else { "#1d1d1d" };

        let outer = gtk::Box::new(Orientation::Vertical, 4);
        outer.set_hexpand(true);
        outer.set_width_request(w as i32);

        // Header: label centered / with value? For linear gauges label is above bar centered.
        let is_circular = matches!(self.style, GaugeStyle::Circular | GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity);

        // For linear styles, show label on top if present
        if !is_circular {
            if let Some(ref lbl) = self.label {
                let row = gtk::Box::new(Orientation::Horizontal, 6);
                row.set_hexpand(true);
                row.set_halign(gtk::Align::Center);
                // For plain Gauge (third card) the label is centered above the bar
                let l = gtk::Label::new(Some(lbl));
                l.add_css_class("gauge-lbl");
                uikit::widget::apply_css(&l, &format!(".gauge-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}", lbl_color));
                row.append(&l);
                outer.append(&row);
            }
        }

        // Drawing area
        let (da_w, da_h) = match self.style {
            GaugeStyle::Circular => (w as i32, 86),
            GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity => (56, 56),
            GaugeStyle::AccessoryLinear | GaugeStyle::AccessoryLinearCapacity => (w as i32, 16),
            _ => (w as i32, 18),
        };
        let da = DrawingArea::new();
        da.set_size_request(da_w, da_h);
        da.set_hexpand(true);
        da.set_halign(gtk::Align::Fill);
        let style = self.style;
        da.set_draw_func(move |_w, cr, ww, hh| {
            match style {
                GaugeStyle::Default | GaugeStyle::LinearCapacity => draw_linear_capacity(cr, ww, hh, n, &tint, false, is_dark),
                GaugeStyle::Linear => draw_linear_marker(cr, ww, hh, n, &tint, false, is_dark),
                GaugeStyle::AccessoryLinearCapacity => draw_linear_capacity(cr, ww, hh, n, &tint, true, is_dark),
                GaugeStyle::AccessoryLinear => draw_linear_marker(cr, ww, hh, n, &tint, true, is_dark),
                GaugeStyle::Circular => draw_circular_marker(cr, ww, hh, n, &tint, false, is_dark),
                GaugeStyle::AccessoryCircular => draw_circular_marker(cr, ww, hh, n, &tint, true, is_dark),
                GaugeStyle::AccessoryCircularCapacity => draw_circular_capacity(cr, ww, hh, n, &tint, true, is_dark),
            }
        });

        // For circular, wrap with overlay to show center labels
        if is_circular {
            let overlay = gtk::Overlay::new();
            overlay.set_halign(gtk::Align::Center);
            overlay.set_size_request(da_w, da_h);
            overlay.set_child(Some(&da));

            // center labels box
            let center_box = gtk::Box::new(Orientation::Vertical, 0);
            center_box.set_halign(gtk::Align::Center);
            center_box.set_valign(gtk::Align::Center);
            center_box.set_can_target(false);

            // For circular, current value inside the ring, label below
            // If current_value_label present, show it large centered; otherwise show value formatted
            let show_current_inside = self.current_value_label.is_some() || self.label.is_none();
            if show_current_inside {
                if let Some(ref cv) = self.current_value_label {
                    let l = gtk::Label::new(Some(cv));
                    l.add_css_class("gauge-cv");
                    uikit::widget::apply_css(&l, &format!(".gauge-cv {{ color: {}; font-family: 'SF Pro Display'; font-size: 14px; font-weight: 700; }}", strong_color));
                    l.set_halign(gtk::Align::Center);
                    center_box.append(&l);
                } else if self.label.is_none() {
                    // fallback: show value integer
                    let txt = if self.max <= 1.0 { format!("{:.2}", self.value) } else { format!("{:.0}", self.value) };
                    let l = gtk::Label::new(Some(&txt));
                    l.add_css_class("gauge-cv");
                    uikit::widget::apply_css(&l, &format!(".gauge-cv {{ color: {}; font-family: 'SF Pro Display'; font-size: 14px; font-weight: 700; }}", strong_color));
                    center_box.append(&l);
                }
            }
            if let Some(ref lbl) = self.label {
                // For circular styles, label appears below center value
                // In accessory circular, label is not inside but the accessories in screenshot show "Foo" on left small + "42" on right etc via external layout; we just show label small below if circular normal
                let l = gtk::Label::new(Some(lbl));
                l.add_css_class("gauge-lbl-sm");
                let sz = if matches!(self.style, GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity) { 7 } else { 9 };
                uikit::widget::apply_css(&l, &format!(".gauge-lbl-sm {{ color: {}; font-family: 'SF Pro Display'; font-size: {}px; }}", sec_color, sz));
                l.set_halign(gtk::Align::Center);
                center_box.append(&l);
            }
            // For normal Circular, also handle min/max below ring? Not inside overlay, handled outside
            overlay.add_overlay(&center_box);
            outer.append(&overlay);
        } else {
            outer.append(&da);
        }

        // Footer row for linear: min / current / max labels
        if !is_circular {
            let has_footer = self.minimum_value_label.is_some() || self.maximum_value_label.is_some() || self.current_value_label.is_some();
            if has_footer {
                let row = gtk::Box::new(Orientation::Horizontal, 6);
                row.set_hexpand(true);
                // left: min
                if let Some(ref mn) = self.minimum_value_label {
                    let l = gtk::Label::new(Some(mn));
                    l.add_css_class("gauge-mm");
                    uikit::widget::apply_css(&l, &format!(".gauge-mm {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", sec_color));
                    l.set_halign(gtk::Align::Start);
                    l.set_hexpand(true);
                    row.append(&l);
                } else {
                    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
                    spacer.set_hexpand(true);
                    row.append(&spacer);
                }
                // center: current value label (styled slightly larger/accent)
                if let Some(ref cv) = self.current_value_label {
                    let l = gtk::Label::new(Some(cv));
                    l.add_css_class("gauge-cv2");
                    uikit::widget::apply_css(&l, &format!(".gauge-cv2 {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", lbl_color));
                    l.set_halign(gtk::Align::Center);
                    l.set_hexpand(true);
                    row.append(&l);
                } else {
                    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
                    spacer.set_hexpand(true);
                    row.append(&spacer);
                }
                // right: max
                if let Some(ref mx) = self.maximum_value_label {
                    let l = gtk::Label::new(Some(mx));
                    l.add_css_class("gauge-mm");
                    uikit::widget::apply_css(&l, &format!(".gauge-mm {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", sec_color));
                    l.set_halign(gtk::Align::End);
                    l.set_hexpand(true);
                    row.append(&l);
                } else {
                    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
                    spacer.set_hexpand(true);
                    row.append(&spacer);
                }
                outer.append(&row);
                // Additionally, if style has current value but no min/max row centered single label below bar (like second card 0.42000)
                // That case is already covered: min/max None but current Some -> centered label appears in footer row center
            } else {
                // If no footer but gauge had standalone current display below bar like screenshot second card?
                // That is actually current_value_label centered below bar, which we handled via footer already when only current is Some.
                // No extra.
            }
            // For the pure "Foo 0.42000" case where current is below bar but we want it centered without min/max row, ensure we have a centered label row when only current is set? Already appended above.
        } else {
            // Circular footer for min/max labels below ring (like third circular example shows 0 and 100)
            let has_mm = self.minimum_value_label.is_some() || self.maximum_value_label.is_some();
            if has_mm {
                let row = gtk::Box::new(Orientation::Horizontal, 0);
                row.set_hexpand(true);
                row.set_halign(gtk::Align::Center);
                if let Some(ref mn) = self.minimum_value_label {
                    let l = gtk::Label::new(Some(mn));
                    l.add_css_class("gauge-mm");
                    uikit::widget::apply_css(&l, &format!(".gauge-mm {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", sec_color));
                    row.append(&l);
                }
                let spacer = gtk::Box::new(Orientation::Horizontal, 0);
                spacer.set_width_request(32);
                row.append(&spacer);
                if let Some(ref mx) = self.maximum_value_label {
                    let l = gtk::Label::new(Some(mx));
                    l.add_css_class("gauge-mm");
                    uikit::widget::apply_css(&l, &format!(".gauge-mm {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", sec_color));
                    row.append(&l);
                }
                outer.append(&row);
            }
        }

        outer.upcast()
    }

    fn can_become_first_responder(&self) -> bool { false }

    fn size_that_fits(&self, _available: Size) -> Size {
        let w = self.width.max(1.0);
        let h = if self.height > 0.0 { self.height } else {
            match self.style {
                GaugeStyle::Circular => 120.0,
                GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity => 72.0,
                GaugeStyle::AccessoryLinear | GaugeStyle::AccessoryLinearCapacity => 40.0,
                _ => 54.0,
            }
        };
        Size::new(w, h)
    }
}

impl Widget for Gauge {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        let w = self.width.max(1.0);
        let hh = if self.height > 0.0 { self.height } else {
            match self.style { GaugeStyle::Circular => 110.0, GaugeStyle::AccessoryCircular | GaugeStyle::AccessoryCircularCapacity => 56.0, _ => 42.0 }
        };
        self.render(Rect::new(0.0, 0.0, w, hh))
    }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_builder() {
        let g = Gauge::new(0.42).label("Foo").in_range(0.0, 1.0).gauge_style(GaugeStyle::Circular);
        assert_eq!(g.label.as_deref(), Some("Foo"));
        assert_eq!(g.style, GaugeStyle::Circular);
        assert!((g.normalized() - 0.42).abs() < 0.001);
    }

    #[test]
    fn gauge_min_max_current_labels() {
        let g = Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").current_value_label("42");
        assert_eq!(g.minimum_value_label.as_deref(), Some("0"));
        assert_eq!(g.maximum_value_label.as_deref(), Some("100"));
        assert_eq!(g.current_value_label.as_deref(), Some("42"));
    }

    #[test]
    fn gauge_norm_clamped() {
        assert_eq!(norm(150.0, 0.0, 100.0), 1.0);
        assert_eq!(norm(-10.0, 0.0, 100.0), 0.0);
    }

    #[test]
    fn gauge_tint_gradient() {
        let g = Gauge::new(0.5).tint_gradient(vec![Color::from_rgb(10,132,255), Color::from_rgb(255,69,58)]);
        assert!(matches!(g.tint, GaugeTint::Gradient(_)));
    }

    #[test]
    fn gauge_styles_distinct() {
        assert_ne!(GaugeStyle::Linear, GaugeStyle::LinearCapacity);
        assert_ne!(GaugeStyle::Circular, GaugeStyle::AccessoryCircular);
    }
}
