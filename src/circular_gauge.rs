//! CircularGauge — iOS/macOS SwiftUI-style accessory circular gauge.
//!
//! Recreates the SwiftUI "Accessory Circular Gauge" look 1:1: a 270-degree
//! open ring with round line caps, a marker dot that travels clockwise from
//! the lower-left to the lower-right, an optional center readout (float,
//! percent or static text), optional min/max labels and an optional caption
//! below the gauge. Theme-aware (dark / light) with no manual toggle — it
//! follows the given or detected scheme.
//!
//! The displayed value can be updated at runtime through
//! [`CircularGauge::set_value`], which redraws the marker and the center
//! readout live.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let gauge = CircularGauge::new()
//!     .value(0.42)
//!     .center(CircularGaugeCenter::Percent)
//!     .min_label("0")
//!     .max_label("100")
//!     .label("Foo");
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use uikit::app::ColorScheme;
use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, DrawingArea, Label as GtkLabel, Overlay};
use gtk::Orientation;

/// Reference geometry from the 100x100 SwiftUI viewBox (scaled to widget size).
const RING_RADIUS: f64 = 35.0;
const RING_STROKE: f64 = 5.5;
const MARKER_RADIUS: f64 = 4.5;
const MARKER_STROKE: f64 = 3.0;
/// The open arc starts at the lower-left and sweeps clockwise to the
/// lower-right (270 degrees, leaving a 90-degree gap at the bottom).
const ARC_START_DEG: f64 = 135.0;
const ARC_SWEEP_DEG: f64 = 270.0;

const RING_LIGHT: (f64, f64, f64) = (0.0, 0.0, 0.0);
const RING_DARK: (f64, f64, f64) = (1.0, 1.0, 1.0);
const MARKER_FILL_LIGHT: (f64, f64, f64) = (1.0, 1.0, 1.0);
const MARKER_FILL_DARK: (f64, f64, f64) = (0.0, 0.0, 0.0);
const MARKER_STROKE_LIGHT: (f64, f64, f64) = (0.0, 0.0, 0.0);
const MARKER_STROKE_DARK: (f64, f64, f64) = (1.0, 1.0, 1.0);

/// What the gauge shows in its center.
#[derive(Debug, Clone, PartialEq)]
pub enum CircularGaugeCenter {
    /// No center readout.
    None,
    /// The raw value with the given number of decimals, e.g. `0.420000`.
    Float(u8),
    /// The value rounded to a whole percent, e.g. `42`.
    Percent,
    /// A fixed custom string.
    Text(String),
}

/// Live handles into the last rendered widget tree, used by
/// [`CircularGauge::set_value`] to redraw at runtime.
struct GaugeLive {
    area: DrawingArea,
    center_label: Option<GtkLabel>,
}

/// iOS/macOS SwiftUI-style accessory circular gauge.
pub struct CircularGauge {
    id: WidgetId,
    value: Rc<RefCell<f32>>,
    label: Option<String>,
    center: CircularGaugeCenter,
    min_label: Option<String>,
    max_label: Option<String>,
    size: f32,
    color_scheme: Option<ColorScheme>,
    live: Rc<RefCell<Option<GaugeLive>>>,
    position_mode: PositionMode,
    position: Position,
}

impl CircularGauge {
    /// Create a new gauge with value `0.0`, no readout and no labels.
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            value: Rc::new(RefCell::new(0.0)),
            label: None,
            center: CircularGaugeCenter::None,
            min_label: None,
            max_label: None,
            size: 96.0,
            color_scheme: None,
            live: Rc::new(RefCell::new(None)),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Set the value `0.0..=1.0` (clamped) that positions the marker.
    pub fn value(self, v: f32) -> Self {
        *self.value.borrow_mut() = v.clamp(0.0, 1.0);
        self
    }

    /// Set the caption below the gauge.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Set the center readout (default: none).
    pub fn center(mut self, c: CircularGaugeCenter) -> Self {
        self.center = c;
        self
    }

    /// Set the label at the lower-left inside the ring (e.g. the minimum).
    pub fn min_label(mut self, l: impl Into<String>) -> Self {
        self.min_label = Some(l.into());
        self
    }

    /// Set the label at the lower-right inside the ring (e.g. the maximum).
    pub fn max_label(mut self, l: impl Into<String>) -> Self {
        self.max_label = Some(l.into());
        self
    }

    /// Set the diameter of the gauge in pixels (default: 96).
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set the size using a frame (uses the smaller side).
    pub fn frame(mut self, w: f32, h: f32) -> Self {
        self.size = w.min(h);
        self
    }

    /// Force a color scheme (defaults to detecting the system scheme).
    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    /// The current value (`0.0..=1.0`).
    pub fn current_value(&self) -> f32 {
        *self.value.borrow()
    }

    /// The caption below the gauge, if set.
    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// The center readout configuration.
    pub fn center_readout(&self) -> &CircularGaugeCenter {
        &self.center
    }

    /// The lower-left label, if set.
    pub fn min_label_text(&self) -> Option<&str> {
        self.min_label.as_deref()
    }

    /// The lower-right label, if set.
    pub fn max_label_text(&self) -> Option<&str> {
        self.max_label.as_deref()
    }

    /// Update the displayed value at runtime (clamped to `0.0..=1.0`). The
    /// rendered marker moves and the center readout updates live. Safe to
    /// call after the gauge was rendered with [`Widget::to_gtk`].
    pub fn set_value(&self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        *self.value.borrow_mut() = v;
        if let Some(live) = self.live.borrow().as_ref() {
            if let Some(ref lbl) = live.center_label {
                lbl.set_text(&center_text(&self.center, v));
            }
            live.area.queue_draw();
        }
    }

    /// Create a View wrapping this gauge.
    pub fn to_view(self) -> View {
        let s = self.size.max(1.0);
        View::new(self).with_frame(0.0, 0.0, s, s)
    }
}

impl Default for CircularGauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the center readout text for the given value.
fn center_text(center: &CircularGaugeCenter, value: f32) -> String {
    match center {
        CircularGaugeCenter::None => String::new(),
        CircularGaugeCenter::Float(d) => format!("{:.*}", *d as usize, value),
        CircularGaugeCenter::Percent => format!("{}", (value * 100.0).round() as i64),
        CircularGaugeCenter::Text(t) => t.clone(),
    }
}

/// Draw the gauge ring and marker into a cairo context.
///
/// `w`/`h` define the canvas (normally square), `value` is clamped to
/// `0.0..=1.0` and `dark` selects the light/dark color pair.
pub fn draw_gauge(cr: &cairo::Context, w: i32, h: i32, value: f32, dark: bool) {
    let s = w.min(h) as f64 / 100.0;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let r = RING_RADIUS * s;

    let (ring_r, ring_g, ring_b) = if dark { RING_DARK } else { RING_LIGHT };
    let (fill_r, fill_g, fill_b) = if dark { MARKER_FILL_DARK } else { MARKER_FILL_LIGHT };
    let (mk_r, mk_g, mk_b) = if dark { MARKER_STROKE_DARK } else { MARKER_STROKE_LIGHT };

    // 270-degree open ring with round caps, lower-left to lower-right.
    let _ = cr.set_line_width(RING_STROKE * s);
    let _ = cr.set_line_cap(cairo::LineCap::Round);
    let _ = cr.set_source_rgb(ring_r, ring_g, ring_b);
    let a1 = ARC_START_DEG.to_radians();
    let a2 = (ARC_START_DEG + ARC_SWEEP_DEG).to_radians();
    let _ = cr.arc(cx, cy, r, a1, a2);
    let _ = cr.stroke();

    // Marker dot at the value position on the arc.
    let v = value.clamp(0.0, 1.0) as f64;
    let ang = (ARC_START_DEG + v * ARC_SWEEP_DEG).to_radians();
    let mx = cx + r * ang.cos();
    let my = cy + r * ang.sin();
    let _ = cr.set_source_rgb(fill_r, fill_g, fill_b);
    let _ = cr.arc(mx, my, MARKER_RADIUS * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();
    let _ = cr.set_line_width(MARKER_STROKE * s);
    let _ = cr.set_source_rgb(mk_r, mk_g, mk_b);
    let _ = cr.arc(mx, my, MARKER_RADIUS * s, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.stroke();
}

impl ViewContent for CircularGauge {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let scheme = self.color_scheme.unwrap_or_else(ColorScheme::detect_system);
        let dark = scheme == ColorScheme::Dark;
        let size = if self.size > 0.0 { self.size } else { frame.width.min(frame.height) };
        let size = size.max(1.0);
        let s = size / 96.0; // reference canvas is 96 px

        let (text_c, small_c) = if dark {
            ("#ffffff", "#a1a1aa")
        } else {
            ("#000000", "#71717a")
        };

        let container = gtk::Box::new(Orientation::Vertical, 8);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let css = format!(
            ".cg-center-big {{ font-family: 'SF Pro Display'; font-size: 22px; font-weight: 700; letter-spacing: -0.5px; color: {text}; }}
             .cg-center-small {{ font-family: 'SF Pro Display'; font-size: 9px; color: {text}; }}
             .cg-minmax {{ font-family: 'SF Pro Display'; font-size: 11px; font-weight: 500; color: {small}; }}
             .cg-label {{ font-family: 'SF Pro Display'; font-size: 13px; font-weight: 500; letter-spacing: 0.3px; color: {text}; }}",
            text = text_c,
            small = small_c,
        );
        uikit::widget::apply_css(&container, &css);

        let overlay = Overlay::new();
        overlay.set_size_request(size as i32, size as i32);

        let area = DrawingArea::new();
        let value_cell = self.value.clone();
        area.set_draw_func(move |_w, cr, w, h| {
            draw_gauge(&cr, w, h, *value_cell.borrow(), dark);
        });
        overlay.set_child(Some(&area));

        let mut center_label = None;
        if self.center != CircularGaugeCenter::None {
            let lbl = GtkLabel::new(Some(&center_text(&self.center, *self.value.borrow())));
            match &self.center {
                CircularGaugeCenter::Float(_) => lbl.add_css_class("cg-center-small"),
                _ => lbl.add_css_class("cg-center-big"),
            }
            lbl.set_halign(gtk::Align::Center);
            lbl.set_valign(gtk::Align::Center);
            overlay.add_overlay(&lbl);
            center_label = Some(lbl);
        }

        if let Some(ref m) = self.min_label {
            let lbl = GtkLabel::new(Some(m));
            lbl.add_css_class("cg-minmax");
            lbl.set_halign(gtk::Align::Start);
            lbl.set_valign(gtk::Align::End);
            lbl.set_margin_start((14.0 * s) as i32);
            lbl.set_margin_bottom((4.0 * s) as i32);
            overlay.add_overlay(&lbl);
        }
        if let Some(ref m) = self.max_label {
            let lbl = GtkLabel::new(Some(m));
            lbl.add_css_class("cg-minmax");
            lbl.set_halign(gtk::Align::End);
            lbl.set_valign(gtk::Align::End);
            lbl.set_margin_end((8.0 * s) as i32);
            lbl.set_margin_bottom((4.0 * s) as i32);
            overlay.add_overlay(&lbl);
        }

        *self.live.borrow_mut() = Some(GaugeLive { area, center_label });

        container.append(&overlay);

        if let Some(ref lbl_text) = self.label {
            let lbl = GtkLabel::new(Some(lbl_text));
            lbl.add_css_class("cg-label");
            lbl.set_halign(gtk::Align::Center);
            container.append(&lbl);
        }

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let mut h = self.size.max(1.0);
        if self.label.is_some() {
            h += 8.0 + 18.0;
        }
        Size::new(self.size.max(1.0), h)
    }
}

impl Widget for CircularGauge {
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
    fn gauge_builder() {
        let g = CircularGauge::new();
        assert_eq!(g.current_value(), 0.0);
        assert_eq!(g.size, 96.0);
        assert_eq!(g.center_readout(), &CircularGaugeCenter::None);
        assert_eq!(g.label_text(), None);
        assert_eq!(g.min_label_text(), None);
        assert_eq!(g.max_label_text(), None);
    }

    #[test]
    fn gauge_value_clamped() {
        assert_eq!(CircularGauge::new().value(1.7).current_value(), 1.0);
        assert_eq!(CircularGauge::new().value(-0.5).current_value(), 0.0);
        let g = CircularGauge::new().value(0.3);
        g.set_value(-0.2);
        assert_eq!(g.current_value(), 0.0);
        g.set_value(2.5);
        assert_eq!(g.current_value(), 1.0);
        g.set_value(0.66);
        assert_eq!(g.current_value(), 0.66);
    }

    #[test]
    fn gauge_center_text() {
        assert_eq!(center_text(&CircularGaugeCenter::Float(6), 0.42), "0.420000");
        assert_eq!(center_text(&CircularGaugeCenter::Float(2), 0.42), "0.42");
        assert_eq!(center_text(&CircularGaugeCenter::Percent, 0.42), "42");
        assert_eq!(center_text(&CircularGaugeCenter::Percent, 0.999), "100");
        assert_eq!(center_text(&CircularGaugeCenter::Text("42%".into()), 0.42), "42%");
        assert_eq!(center_text(&CircularGaugeCenter::None, 0.42), "");
    }

    #[test]
    fn gauge_labels() {
        let g = CircularGauge::new()
            .label("Foo")
            .min_label("0")
            .max_label("100");
        assert_eq!(g.label_text(), Some("Foo"));
        assert_eq!(g.min_label_text(), Some("0"));
        assert_eq!(g.max_label_text(), Some("100"));
    }
}