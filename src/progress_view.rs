//! ProgressView — macOS-style loading indicator with two modes.
//!
//! A single element covering both SwiftUI `ProgressView` variants:
//! indeterminate (8-tick spinner) when no value is given, or a
//! determinate circular ring showing progress when `.value(v)` is set.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! // Indeterminate spinner
//! let spinner = ProgressView::new().label("Loading...");
//!
//! // Determinate ring (42%)
//! let ring = ProgressView::new()
//!     .value(0.42)
//!     .label("Foo")
//!     .sub_label("bar")
//!     .accent_color(Color::from_rgb(0, 122, 255));
//!
//! let view = View::new(spinner);
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

/// A macOS-style loading indicator (spinner or progress ring).
pub struct ProgressView {
    id: WidgetId,
    value: Option<f32>,
    label: Option<String>,
    sub_label: Option<String>,
    accent_color: Color,
    track_color: Color,
    size: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ProgressView {
    /// Create a new indeterminate progress indicator (spinner).
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            value: None,
            label: None,
            sub_label: None,
            accent_color: Color::new(0.047, 0.522, 0.937, 1.0), // TontooOS blue
            track_color: Color::from_rgb(44, 44, 46),
            size: 32.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Set a progress value `0.0..=1.0` to switch to the determinate ring.
    pub fn value(mut self, v: f32) -> Self {
        self.value = Some(v.clamp(0.0, 1.0));
        self
    }

    /// Set the main label below the indicator.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Set the secondary (smaller) label below the main label.
    pub fn sub_label(mut self, s: impl Into<String>) -> Self {
        self.sub_label = Some(s.into());
        self
    }

    /// Set the accent color of the spinner ticks / progress ring.
    pub fn accent_color(mut self, c: Color) -> Self {
        self.accent_color = c;
        self
    }

    /// Set the background track color of the determinate ring.
    pub fn track_color(mut self, c: Color) -> Self {
        self.track_color = c;
        self
    }

    /// Set the diameter of the indicator.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set the size using a frame (uses the smaller side).
    pub fn frame(mut self, w: f32, h: f32) -> Self {
        self.size = w.min(h);
        self
    }

    /// The current progress value, or `None` for the indeterminate spinner.
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

    /// Create a View wrapping this indicator.
    pub fn to_view(self) -> View {
        let s = self.size.max(1.0);
        View::new(self).with_frame(0.0, 0.0, s, s)
    }
}

impl Default for ProgressView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for ProgressView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let size = if self.size > 0.0 { self.size } else { frame.width.min(frame.height) };
        let size = size.max(1.0);

        let container = gtk::Box::new(Orientation::Vertical, 8);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let css = ".pv-label { color: rgba(235, 235, 245, 0.6); font-family: 'SF Pro Display'; font-size: 13px; }
                   .pv-sub { color: rgba(235, 235, 245, 0.4); font-family: 'SF Pro Display'; font-size: 11px; }";
        uikit::widget::apply_css(&container, &css);

        let da = DrawingArea::new();
        da.set_size_request(size as i32, size as i32);

        let accent = self.accent_color;
        let track = self.track_color;

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
            l.set_halign(gtk::Align::Center);
            container.append(&l);
        }
        if let Some(ref sub) = self.sub_label {
            let s = gtk::Label::new(Some(sub));
            s.add_css_class("pv-sub");
            s.set_halign(gtk::Align::Center);
            container.append(&s);
        }

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, _available: Size) -> Size {
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
}