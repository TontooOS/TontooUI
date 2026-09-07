//! Toggle — SwiftUI-style boolean toggle.
//!
//! Mirrors the SwiftUI `Toggle` with the two non-list styles from the macOS 26
//! dumps: `SwitchToggleStyle` (leading label, trailing switch) and
//! `CheckboxToggleStyle` (checkbox followed by its label). Pressing the control
//! makes only the white knob grow 0.25× and go translucent, staying in place.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use gtk::prelude::*;
use gtk::{Label as GtkLabel, Orientation};

use super::common::{ToggleStyle, label_color, paint_checkbox, paint_switch};

/// SwiftUI-style toggle with switch or checkbox appearance.
pub struct Toggle {
    id: WidgetId,
    label: String,
    image: Option<String>,
    system_image: Option<String>,
    custom_label: Option<Box<dyn Widget>>,
    style: ToggleStyle,
    initial_value: bool,
    color_scheme: Option<ColorScheme>,
    width: f32,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Toggle {
    /// Create a toggle with the given label (switch style by default).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            label: label.into(),
            image: None,
            system_image: None,
            custom_label: None,
            style: ToggleStyle::Switch,
            initial_value: false,
            color_scheme: None,
            width: 0.0,
            on_change: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    /// Attach a custom image asset next to the label (mirrors `image:` param).
    pub fn image(mut self, name: impl Into<String>) -> Self {
        self.image = Some(name.into());
        self
    }
    /// Attach a system image (SF Symbol) next to the label.
    pub fn system_image(mut self, name: impl Into<String>) -> Self {
        self.system_image = Some(name.into());
        self
    }
    /// Replace the label area with a custom widget (mirrors `Toggle { Label(...) }`).
    pub fn custom_label(mut self, w: impl Widget + 'static) -> Self {
        self.custom_label = Some(Box::new(w));
        self
    }
    /// Set the toggle style (switch or checkbox).
    pub fn style(mut self, style: ToggleStyle) -> Self {
        self.style = style;
        self
    }
    /// Set the initial state.
    pub fn value(mut self, on: bool) -> Self {
        self.initial_value = on;
        self
    }
    /// Force a color scheme (defaults to the running app's scheme).
    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }
    /// Force a row width (label leading, control trailing).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Set the state-change handler.
    pub fn on_change(mut self, handler: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 200.0, 32.0)
    }
}

impl ViewContent for Toggle {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let value = Rc::new(RefCell::new(self.initial_value));
        let row = gtk::Box::new(Orientation::Horizontal, 10);
        if self.width > 0.0 {
            row.set_width_request(self.width as i32);
        }
        row.set_valign(gtk::Align::Center);
        row.set_overflow(gtk::Overflow::Visible);
        let label_area: gtk::Widget = if let Some(ref custom) = self.custom_label {
            custom.to_gtk()
        } else {
            let area = gtk::Box::new(Orientation::Horizontal, 6);
            area.set_valign(gtk::Align::Center);
            let icon_name = self.system_image.as_ref().or(self.image.as_ref());
            if let Some(name) = icon_name {
                let tint = if dark { (10u8, 132u8, 255u8) } else { (0u8, 122u8, 255u8) };
                if let Some(path) = crate::elements::buttons::common::sf_icon_path(name, tint) {
                    let img = gtk::Image::from_file(&path);
                    img.set_pixel_size(18);
                    img.set_valign(gtk::Align::Center);
                    area.append(&img);
                }
            }
            if !self.label.is_empty() {
                let lbl = GtkLabel::new(Some(&self.label));
                uikit::widget::apply_css(&lbl, &format!("label {{ color: {c}; font-family: 'SF Pro Text'; font-size: 15px; }}", c = label_color(dark),));
                area.append(&lbl);
            }
            area.upcast()
        };
        const TRACK_W: f32 = 28.0;
        const TRACK_H: f32 = 23.0;
        const KNOB_W_REST: f32 = 30.0;
        const KNOB_H: f32 = 20.0;
        const KNOB_W_PRESSED: f32 = 40.0;
        const CHECK_S: f32 = 16.0;
        match self.style {
            ToggleStyle::Switch => {
                let track_bg = gtk::Box::new(Orientation::Horizontal, 0);
                track_bg.set_hexpand(true);
                track_bg.set_vexpand(true);
                let track = gtk::Overlay::new();
                track.set_size_request(TRACK_W as i32, TRACK_H as i32);
                track.set_valign(gtk::Align::Center);
                track.set_overflow(gtk::Overflow::Visible);
                let knob = gtk::Box::new(Orientation::Horizontal, 0);
                knob.set_width_request(KNOB_W_REST as i32);
                knob.set_height_request(KNOB_H as i32);
                knob.set_halign(gtk::Align::Start);
                knob.set_valign(gtk::Align::Center);
                knob.set_overflow(gtk::Overflow::Visible);
                uikit::widget::apply_css(&knob, "box { background: white; border-radius: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.25); transition: background 120ms ease, box-shadow 120ms ease; }");
                track.set_child(Some(&track_bg));
                track.add_overlay(&knob);
                paint_switch(&track_bg, &knob, self.initial_value, dark);
                let travel: f32 = 30.0;
                knob.set_margin_start((0.0 + if self.initial_value { travel } else { 0.0 }) as i32);
                row.append(&label_area);
                let spacer = gtk::Box::new(Orientation::Horizontal, 0);
                spacer.set_hexpand(true);
                row.append(&spacer);
                row.append(&track);
                // ── smooth animation helper ──
                // Animiert margin_start über 220ms mit easeOutCubic (iOS-like spring-feel).
                let anim_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
                let animate_knob = {
                    let anim_id = anim_id.clone();
                    move |knob_w: gtk::Box, target_margin: i32| {
                        let start = knob_w.margin_start();
                        if start == target_margin {
                            return;
                        }
                        // Cancel laufende Animation
                        if let Some(old) = anim_id.borrow_mut().take() {
                            old.remove();
                        }
                        let start_f = start as f32;
                        let end_f = target_margin as f32;
                        let start_t = std::time::Instant::now();
                        let dur_ms = 220.0_f32;
                        let knob_c = knob_w.clone();
                        let anim_id_c = anim_id.clone();
                        let id = glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                            let elapsed = start_t.elapsed().as_millis() as f32;
                            let t = (elapsed / dur_ms).clamp(0.0, 1.0);
                            // easeOutCubic = 1 - (1-t)^3  → natürliches Auslaufen wie iOS
                            let eased = 1.0 - (1.0 - t).powi(3);
                            let cur = start_f + (end_f - start_f) * eased;
                            knob_c.set_margin_start(cur.round() as i32);
                            if t >= 1.0 {
                                knob_c.set_margin_start(target_margin);
                                *anim_id_c.borrow_mut() = None;
                                glib::ControlFlow::Break
                            } else {
                                glib::ControlFlow::Continue
                            }
                        });
                        *anim_id.borrow_mut() = Some(id);
                    }
                };
                let drag = gtk::GestureDrag::new();
                drag.set_button(1);
                let start_x = Rc::new(RefCell::new(0.0f32));
                let start_x_c1 = start_x.clone();
                let knob_c1 = knob.clone();
                let anim_id_c1 = anim_id.clone();
                drag.connect_drag_begin(move |_g, x, _y| {
                    *start_x_c1.borrow_mut() = x as f32;
                    // Laufende Snap-Animation abbrechen damit Drag direkt folgt
                    if let Some(old) = anim_id_c1.borrow_mut().take() {
                        old.remove();
                    }
                    // 0.25 Breite + 0.25 Höhe = in allen Richtungen etwas größer, poppt über Rand.
                    knob_c1.set_size_request(KNOB_W_PRESSED as i32, (KNOB_H * 1.33) as i32);
                    uikit::widget::apply_css(&knob_c1, "box { background: rgba(255,255,255,0.22); border-radius: 16px; box-shadow: 0 4px 16px rgba(0,0,0,0.38), inset 0 1px 0 rgba(255,255,255,0.45); border: 1px solid rgba(255,255,255,0.22); }");
                });
                let track_bg_c2 = track_bg.clone();
                let knob_c2 = knob.clone();
                // Während Drag direkt folgen (kein Smooth), nur visuelles Track-Preview.
                drag.connect_drag_update(move |_g, dx, _dy| {
                    if (dx as f32).abs() < 5.0 { return; }
                    let target_on = (dx as f32) > 0.0;
                    paint_switch(&track_bg_c2, &knob_c2, target_on, dark);
                    let travel2: f32 = TRACK_W - KNOB_W_REST - -24.0;
                    knob_c2.set_margin_start((2.0 + if target_on { travel2 } else { 0.0 }) as i32);
                });
                let track_bg_c3 = track_bg.clone();
                let knob_c3 = knob.clone();
                let value_c3 = value.clone();
                let on_change_c = self.on_change.clone();
                let animate_c3 = animate_knob.clone();
                drag.connect_drag_end(move |_g, dx, _dy| {
                    let dragged = (dx.abs() as f32) as f32;
                    let prev_on = *value_c3.borrow();
                    let mut target_on = prev_on;
                    if dragged < 5.0 {
                        target_on = !prev_on;
                    } else if (dx as f32) > 10.0 {
                        target_on = true;
                    } else if (dx as f32) < -10.0 {
                        target_on = false;
                    }
                    knob_c3.set_size_request(KNOB_W_REST as i32, KNOB_H as i32);
                    uikit::widget::apply_css(&knob_c3, "box { background: white; border-radius: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.25); transition: background 120ms ease, box-shadow 120ms ease; }");
                    paint_switch(&track_bg_c3, &knob_c3, target_on, dark);
                    let travel2: f32 = TRACK_W - KNOB_W_REST - -30.0;
                    let target_margin = (2.0 + if target_on { travel2 } else { 0.0 }) as i32;
                    animate_c3(knob_c3.clone(), target_margin);
                    if target_on != *value_c3.borrow() {
                        *value_c3.borrow_mut() = target_on;
                        if let Some(ref h) = on_change_c { h(target_on); }
                    }
                });
                knob.set_can_target(true);
                knob.add_controller(drag);
            }
            ToggleStyle::Checkbox => {
                let box_w = gtk::Box::new(Orientation::Horizontal, 0);
                box_w.set_width_request(CHECK_S as i32);
                box_w.set_height_request(CHECK_S as i32);
                box_w.set_valign(gtk::Align::Center);
                box_w.set_overflow(gtk::Overflow::Visible);
                let mark = GtkLabel::new(Some("✓"));
                mark.set_halign(gtk::Align::Center);
                mark.set_valign(gtk::Align::Center);
                mark.set_xalign(0.5);
                mark.set_yalign(0.5);
                mark.set_justify(gtk::Justification::Center);
                mark.set_margin_start(2);
                uikit::widget::apply_css(&mark, "label { color: white; font-size: 15px; font-weight: 900; }");
                box_w.append(&mark);
                paint_checkbox(&box_w, &mark, self.initial_value, dark);
                row.append(&box_w);
                row.append(&label_area);
                let value_c = value.clone();
                let box_c = box_w.clone();
                let mark_c = mark.clone();
                let on_change_c = self.on_change.clone();
                let tap = gtk::GestureClick::new();
                tap.set_button(1);
                tap.connect_released(move |_g, _n, _x, _y| {
                    let now = !*value_c.borrow();
                    *value_c.borrow_mut() = now;
                    paint_checkbox(&box_c, &mark_c, now, dark);
                    if let Some(ref h) = on_change_c { h(now); }
                });
                row.add_controller(tap);
            }
        }
        row.upcast()
    }
    fn can_become_first_responder(&self) -> bool { true }
    fn size_that_fits(&self, _available: Size) -> Size {
        let text_w = self.label.chars().count() as f32 * 8.0;
        let w = if self.width > 0.0 { self.width } else { text_w + 60.0 };
        Size::new(w, 32.0)
    }
}
impl Widget for Toggle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 0.0, 0.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> Padding { Padding::ZERO }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toggle_builder() {
        let t = Toggle::new("Foo1").style(ToggleStyle::Checkbox).value(true).width(240.0);
        assert_eq!(t.style, ToggleStyle::Checkbox);
        assert!(t.initial_value);
        assert_eq!(t.label, "Foo1");
        assert_eq!(t.width, 240.0);
    }
    #[test]
    fn toggle_defaults() {
        let t = Toggle::new("Foo");
        assert_eq!(t.style, ToggleStyle::Switch);
        assert!(!t.initial_value);
    }
}
