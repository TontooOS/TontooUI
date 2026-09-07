//! ProgressView — macOS-style loading indicator with two styles.
//!
//! Covers SwiftUI `ProgressView` variants:
//! indeterminate spinner/linear when no value is given, determinate ring or
//! horizontal bar when `.value(v)` is set. Styles: `Circular` (spinner/ring)
//! and `Linear` (horizontal bar). Colors via `.accent_color`/`.tint` + Light/Dark.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//! use tontooui::ProgressViewStyle;
//!
//! // Indeterminate circular spinner
//! let spinner = ProgressView::new().label("Loading...");
//!
//! // Determinate circular ring 42%
//! let ring = ProgressView::new().value(0.42).label("Foo").sub_label("bar");
//!
//! // Linear determinate 60%
//! let bar = ProgressView::new().value(0.6).progress_view_style(ProgressViewStyle::Linear).tint(Color::from_rgb(255, 69, 58));
//! ```

use std::cell::Cell;
use std::rc::Rc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, DrawingArea};
use gtk::Orientation;

const SPIN_SEGMENTS: usize = 8;
const SPIN_STEP_DEG: f64 = 360.0 / SPIN_SEGMENTS as f64;
const SPIN_STEP_MS: u32 = 125;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressViewStyle {
    Circular,
    Linear,
}

impl Default for ProgressViewStyle {
    fn default() -> Self { Self::Circular }
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

fn draw_spinner(cr: &cairo::Context, w: i32, h: i32, angle_deg: f64, accent: Color) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let unit = (w as f64).min(h as f64) / 24.0;
    let opacities = [1.0, 0.875, 0.75, 0.625, 0.5, 0.375, 0.25, 0.125];
    let (r, g, b, a) = (accent.r as f64, accent.g as f64, accent.b as f64, accent.a as f64);

    for (i, &op) in opacities.iter().enumerate() {
        let angle = (angle_deg + i as f64 * (360.0 / SPIN_SEGMENTS as f64)).to_radians();
        let _ = cr.save();
        let _ = cr.translate(cx, cy);
        let _ = cr.rotate(angle);
        let inner = 4.0 * unit;
        let outer = 10.0 * unit;
        let tick_w = 2.0 * unit;
        let tick_h = outer - inner;
        set_rounded_rect(cr, -tick_w / 2.0, -outer, tick_w, tick_h, tick_w / 2.0);
        let _ = cr.set_source_rgba(r, g, b, a * op);
        let _ = cr.fill();
        let _ = cr.restore();
    }
}

fn draw_ring(cr: &cairo::Context, w: i32, h: i32, progress: f32, accent: Color, track: Color) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let unit = (w as f64).min(h as f64) / 36.0;
    let radius = 15.9155 * unit;
    let stroke = 4.0 * unit;
    let pi = std::f64::consts::PI;

    let _ = cr.set_source_rgba(track.r as f64, track.g as f64, track.b as f64, track.a as f64);
    let _ = cr.set_line_width(stroke);
    let _ = cr.arc(cx, cy, radius, 0.0, 2.0 * pi);
    let _ = cr.stroke();

    let p = progress.clamp(0.0, 1.0) as f64;
    if p > 0.0001 {
        let _ = cr.set_source_rgba(accent.r as f64, accent.g as f64, accent.b as f64, accent.a as f64);
        let _ = cr.set_line_width(stroke);
        let _ = cr.set_line_cap(cairo::LineCap::Round);
        let _ = cr.arc(cx, cy, radius, -pi / 2.0, -pi / 2.0 + p * 2.0 * pi);
        let _ = cr.stroke();
    }
}

fn draw_linear_determinate(cr: &cairo::Context, w: i32, h: i32, progress: f32, accent: Color, track: Color, is_dark: bool) {
    let track_h: f64 = 6.0;
    let y = (h as f64 - track_h) / 2.0;
    let x = 0.0;
    let ww = w as f64;
    let r = track_h / 2.0;
    let track_c = if is_dark { track } else {
        // Light: lighter track if default dark track is used — caller passes already resolved track
        track
    };
    let _ = cr.save();
    set_rounded_rect(cr, x, y, ww, track_h, r);
    let _ = cr.set_source_rgba(track_c.r as f64, track_c.g as f64, track_c.b as f64, track_c.a as f64);
    let _ = cr.fill();
    let _ = cr.restore();

    let p = progress.clamp(0.0, 1.0) as f64;
    if p > 0.0001 {
        let fw = (ww * p).max(r * 2.0).min(ww);
        let _ = cr.save();
        set_rounded_rect(cr, x, y, fw, track_h, r);
        let _ = cr.set_source_rgba(accent.r as f64, accent.g as f64, accent.b as f64, accent.a as f64);
        let _ = cr.fill();
        let _ = cr.restore();
    }
}

fn draw_linear_indeterminate(cr: &cairo::Context, w: i32, h: i32, offset: f64, accent: Color, track: Color) {
    let track_h: f64 = 6.0;
    let y = (h as f64 - track_h) / 2.0;
    let ww = w as f64;
    let r = track_h / 2.0;
    let _ = cr.save();
    set_rounded_rect(cr, 0.0, y, ww, track_h, r);
    let _ = cr.set_source_rgba(track.r as f64, track.g as f64, track.b as f64, track.a as f64);
    let _ = cr.fill();
    let _ = cr.restore();
    // sliding 35% bar
    let bar_w = ww * 0.35;
    let x = (offset * (ww + bar_w) - bar_w).clamp(-bar_w, ww);
    let _ = cr.save();
    set_rounded_rect(cr, x, y, bar_w, track_h, r);
    let _ = cr.set_source_rgba(accent.r as f64, accent.g as f64, accent.b as f64, accent.a as f64);
    let _ = cr.fill();
    let _ = cr.restore();
}

/// A macOS-style loading indicator (spinner/ring or linear bar).
pub struct ProgressView {
    id: WidgetId,
    value: Option<f32>,
    label: Option<String>,
    sub_label: Option<String>,
    accent_color: Color,
    track_color: Option<Color>,
    style: ProgressViewStyle,
    size: f32,
    width: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ProgressView {
    /// Create a new indeterminate progress indicator (spinner or linear bar depending on style).
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            value: None,
            label: None,
            sub_label: None,
            accent_color: Color::new(0.047, 0.522, 0.937, 1.0),
            track_color: None,
            style: ProgressViewStyle::Circular,
            size: 32.0,
            width: 180.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Set a progress value `0.0..=1.0` to switch to the determinate variant.
    pub fn value(mut self, v: f32) -> Self {
        self.value = Some(v.clamp(0.0, 1.0));
        self
    }

    /// Set the main label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Set the secondary (smaller) label below the main label.
    pub fn sub_label(mut self, s: impl Into<String>) -> Self {
        self.sub_label = Some(s.into());
        self
    }

    /// Set the accent color of the spinner ticks / progress.
    pub fn accent_color(mut self, c: Color) -> Self {
        self.accent_color = c;
        self
    }

    /// Alias for `accent_color` — SwiftUI `.tint`.
    pub fn tint(mut self, c: Color) -> Self {
        self.accent_color = c;
        self
    }

    /// Set the background track color of the determinate variant.
    pub fn track_color(mut self, c: Color) -> Self {
        self.track_color = Some(c);
        self
    }

    /// Set the progress view style (Circular or Linear).
    pub fn progress_view_style(mut self, s: ProgressViewStyle) -> Self {
        self.style = s;
        self
    }

    /// Alias for `progress_view_style`.
    pub fn style(mut self, s: ProgressViewStyle) -> Self {
        self.style = s;
        self
    }

    /// Set the diameter (circular) or keep for view sizing.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set linear width (also used for `View` frame).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set the size using a frame (uses the smaller side for circular).
    pub fn frame(mut self, w: f32, h: f32) -> Self {
        self.size = w.min(h);
        self.width = w;
        self
    }

    /// The current progress value, or `None` for indeterminate.
    pub fn progress(&self) -> Option<f32> {
        self.value
    }

    /// The main label, if set.
    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// The secondary label, if set.
    pub fn sub_label_text(&self) -> Option<&str> {
        self.sub_label.as_deref()
    }

    pub fn view_style(&self) -> ProgressViewStyle {
        self.style
    }

    /// Create a View wrapping this indicator.
    pub fn to_view(self) -> View {
        if self.style == ProgressViewStyle::Linear {
            let w = self.width.max(40.0);
            let h = 28.0;
            View::new(self).with_frame(0.0, 0.0, w, h)
        } else {
            let s = self.size.max(1.0);
            View::new(self).with_frame(0.0, 0.0, s, s)
        }
    }
}

impl Default for ProgressView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for ProgressView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        use crate::elements::resolve_scheme;
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let lbl_color = if is_dark { "rgba(235, 235, 245, 0.6)" } else { "rgba(60, 60, 67, 0.85)" };
        let sub_color = if is_dark { "rgba(235, 235, 245, 0.4)" } else { "rgba(60, 60, 67, 0.55)" };
        let track_resolved = self.track_color.unwrap_or_else(|| {
            if is_dark { Color::from_rgb(44, 44, 46) } else { Color::from_rgb(209, 209, 214) }
        });

        // Linear style renders as horizontal bar directly on background (#1d1d1d / #ececec)
        if self.style == ProgressViewStyle::Linear {
            let w = if self.width > 0.0 { self.width } else { frame.width.max(180.0) };
            let w = w.max(40.0);
            let container = gtk::Box::new(Orientation::Vertical, 6);
            container.set_hexpand(true);
            container.set_width_request(w as i32);

            if let Some(ref lbl) = self.label {
                let l = gtk::Label::new(Some(lbl));
                l.set_halign(gtk::Align::Start);
                uikit::widget::apply_css(&l, &format!(".pv-label {{ color: {lbl_color}; font-family: 'SF Pro Display'; font-size: 11px; }}"));
                container.append(&l);
            }

            let da = DrawingArea::new();
            da.set_size_request(w as i32, 12);
            da.set_hexpand(true);
            let accent = self.accent_color;
            let track = track_resolved;
            match self.value {
                Some(v) => {
                    let is_dark_c = is_dark;
                    da.set_draw_func(move |_w, cr, ww, hh| {
                        draw_linear_determinate(cr, ww, hh, v, accent, track, is_dark_c);
                    });
                }
                None => {
                    let off = Rc::new(Cell::new(0.0f64));
                    {
                        let off = off.clone();
                        da.set_draw_func(move |_w, cr, ww, hh| {
                            draw_linear_indeterminate(cr, ww, hh, off.get(), accent, track);
                        });
                    }
                    {
                        let off = off.clone();
                        let da = da.clone();
                        // ~60fps slide
                        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                            let mut o = off.get() + 0.015;
                            if o > 1.2 { o = -0.2; }
                            off.set(o);
                            da.queue_draw();
                            glib::ControlFlow::Continue
                        });
                    }
                }
            }
            container.append(&da);

            if let Some(ref sub) = self.sub_label {
                let s = gtk::Label::new(Some(sub));
                s.set_halign(gtk::Align::Start);
                uikit::widget::apply_css(&s, &format!(".pv-sub {{ color: {sub_color}; font-family: 'SF Pro Display'; font-size: 9px; }}"));
                container.append(&s);
            } else if self.sub_label.is_none() && self.label.is_none() && self.value.is_none() {
                // keep size for indeterminate without labels
            }

            return container.upcast();
        }

        // Circular: spinner or ring, centered
        let size = if self.size > 0.0 { self.size } else { frame.width.min(frame.height) };
        let size = size.max(1.0);

        let container = gtk::Box::new(Orientation::Vertical, 8);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let da = DrawingArea::new();
        da.set_size_request(size as i32, size as i32);

        let accent = self.accent_color;
        let track = track_resolved;

        match self.value {
            None => {
                let angle = Rc::new(Cell::new(0.0f64));
                {
                    let angle = angle.clone();
                    da.set_draw_func(move |_w, cr, w, h| {
                        draw_spinner(cr, w, h, angle.get(), accent);
                    });
                }
                {
                    let angle = angle.clone();
                    let da = da.clone();
                    glib::timeout_add_local(std::time::Duration::from_millis(SPIN_STEP_MS as u64), move || {
                        let a = angle.get() + SPIN_STEP_DEG;
                        angle.set(if a >= 360.0 { a - 360.0 } else { a });
                        da.queue_draw();
                        glib::ControlFlow::Continue
                    });
                }
            }
            Some(v) => {
                da.set_draw_func(move |_w, cr, w, h| {
                    draw_ring(cr, w, h, v, accent, track);
                });
            }
        }

        container.append(&da);

        if let Some(ref lbl) = self.label {
            let l = gtk::Label::new(Some(lbl));
            l.add_css_class("pv-label");
            uikit::widget::apply_css(&l, &format!(".pv-label {{ color: {lbl_color}; font-family: 'SF Pro Display'; font-size: 13px; }}"));
            l.set_halign(gtk::Align::Center);
            container.append(&l);
        }
        if let Some(ref sub) = self.sub_label {
            let s = gtk::Label::new(Some(sub));
            s.add_css_class("pv-sub");
            uikit::widget::apply_css(&s, &format!(".pv-sub {{ color: {sub_color}; font-family: 'SF Pro Display'; font-size: 11px; }}"));
            s.set_halign(gtk::Align::Center);
            container.append(&s);
        }

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        if self.style == ProgressViewStyle::Linear {
            let w = self.width.max(40.0);
            let mut h: f32 = 14.0;
            if self.label.is_some() { h += 14.0; }
            if self.sub_label.is_some() { h += 12.0; }
            return Size::new(w, h);
        }
        let size = self.size.max(1.0);
        let mut h = size;
        if self.label.is_some() {
            h += 8.0 + 16.0;
            if self.sub_label.is_some() {
                h += 14.0;
            }
        }
        Size::new(size, h)
    }
}

impl Widget for ProgressView {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }

    fn position(&self) -> Position {
        self.position
    }

    fn to_gtk(&self) -> gtk::Widget {
        if self.style == ProgressViewStyle::Linear {
            let w = self.width.max(40.0);
            return self.render(Rect::new(0.0, 0.0, w, 28.0));
        }
        let s = self.size.max(1.0);
        self.render(Rect::new(0.0, 0.0, s, s))
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_view_default_is_indeterminate() {
        let pv = ProgressView::new();
        assert!(pv.progress().is_none());
        assert_eq!(pv.size, 32.0);
        assert_eq!(pv.view_style(), ProgressViewStyle::Circular);
    }

    #[test]
    fn progress_view_determinate() {
        let pv = ProgressView::new().value(0.42);
        assert_eq!(pv.progress(), Some(0.42));
    }

    #[test]
    fn progress_view_value_clamped() {
        assert_eq!(ProgressView::new().value(1.7).progress(), Some(1.0));
        assert_eq!(ProgressView::new().value(-0.5).progress(), Some(0.0));
    }

    #[test]
    fn progress_view_labels() {
        let pv = ProgressView::new().value(0.5).label("Foo").sub_label("bar");
        assert_eq!(pv.label_text(), Some("Foo"));
        assert_eq!(pv.sub_label_text(), Some("bar"));
    }

    #[test]
    fn progress_view_size() {
        let pv = ProgressView::new().size(48.0);
        assert_eq!(pv.size, 48.0);
        let pv = ProgressView::new().frame(60.0, 40.0);
        assert_eq!(pv.size, 40.0);
    }

    #[test]
    fn progress_view_styles() {
        let pv = ProgressView::new().progress_view_style(ProgressViewStyle::Linear);
        assert_eq!(pv.view_style(), ProgressViewStyle::Linear);
        let pv2 = ProgressView::new().tint(Color::RED);
        assert_eq!(pv2.accent_color, Color::RED);
    }
}
