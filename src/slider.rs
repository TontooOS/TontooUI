//! Slider — macOS-style slider using pure CSS (no cairo).
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let slider = Slider::new(0.0, 100.0)
//!     .value(50.0)
//!     .step(1.0)
//!     .label("Volume")
//!     .on_change(|val| println!("Value: {}", val));
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Label as GtkLabel};
use gtk::Orientation;

const SPRING_K: f32 = 35.0 * 1.35;
const DAMP: f32 = 8.0 * 1.35;
const VALUE_SPEED: f32 = 18.0;
const WOBBLE_DURATION: f32 = 0.259;

fn ease_out_elastic(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 { return t; }
    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * std::f32::consts::PI * 2.0 / 3.0).sin() + 1.0
}

#[derive(Debug, Clone, Copy)]
struct SliderPhysics {
    is_dragging: bool,
    squish_x: f32,
    squish_y: f32,
    squish_vel_x: f32,
    squish_vel_y: f32,
    drag_vel: f32,
    prev_mouse_x: f32,
    wobbling: bool,
    wobble_time: f32,
    display_value: f32,
    target_value: f32,
    min: f32,
    max: f32,
    step: f32,
    grow: f32,
}

impl SliderPhysics {
    fn new(initial: f32) -> Self {
        Self {
            is_dragging: false, squish_x: 1.0, squish_y: 1.0,
            squish_vel_x: 0.0, squish_vel_y: 0.0,
            drag_vel: 0.0, prev_mouse_x: 0.0,
            wobbling: false, wobble_time: 0.0,
            display_value: initial, target_value: initial,
            min: 0.0, max: 100.0, step: 0.0, grow: 1.0,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.display_value += (self.target_value - self.display_value) * VALUE_SPEED * dt;
        if (self.display_value - self.target_value).abs() < 0.01 {
            self.display_value = self.target_value;
        }

        let target_grow = if self.is_dragging { 1.25 }
            else if self.wobbling {
                let t = (self.wobble_time / WOBBLE_DURATION).min(1.0);
                1.25 - 0.25 * ease_out_elastic(t)
            } else { 1.0 };
        self.grow += (target_grow - self.grow) * 12.0 * dt;

        let mut tsx = 1.0_f32;
        let mut tsy = 1.0_f32;
        if self.is_dragging {
            let stretch = (self.drag_vel.abs() * 0.0375).min(0.56);
            tsx = 1.0 + stretch;
            tsy = 1.0 - stretch * 0.8;
            self.drag_vel *= 0.85;
        } else if self.wobbling {
            self.wobble_time += dt;
            let t = (self.wobble_time / WOBBLE_DURATION).min(1.0);
            let eased = ease_out_elastic(t);
            let wobble = (t * std::f32::consts::PI * 5.0).sin() * (1.0 - eased) * 0.44;
            tsx = 1.0 + wobble;
            tsy = 1.0 - wobble;
            if t >= 1.0 { self.wobbling = false; }
        }

        let dx = tsx - self.squish_x;
        let dy = tsy - self.squish_y;
        self.squish_vel_x += dx * SPRING_K * dt;
        self.squish_vel_y += dy * SPRING_K * dt;
        self.squish_vel_x *= (1.0 - DAMP * dt).max(0.0);
        self.squish_vel_y *= (1.0 - DAMP * dt).max(0.0);
        self.squish_x += self.squish_vel_x * dt;
        self.squish_y += self.squish_vel_y * dt;
    }

    fn value_norm(&self) -> f32 {
        let r = self.max - self.min;
        if r > 0.0 { ((self.display_value - self.min) / r).clamp(0.0, 1.0) } else { 0.0 }
    }

    fn x_to_value(&self, x: f32, track_width: f32) -> f32 {
        let thumb_r = 15.0;
        let t = ((x - thumb_r) / (track_width - thumb_r * 2.0)).clamp(0.0, 1.0);
        let raw = self.min + t * (self.max - self.min);
        if self.step > 0.0 { (raw / self.step).round() * self.step } else { raw }
    }
}

pub struct Slider {
    id: WidgetId,
    min: f32,
    max: f32,
    initial_value: f32,
    step: f32,
    accent_color: Color,
    track_color: Color,
    label: Option<String>,
    width: f32,
    on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Slider {
    pub fn new(min: f32, max: f32) -> Self {
        Self { id: next_widget_id(), min, max, initial_value: min, step: 0.0,
            accent_color: Color::new(0.016, 0.525, 0.941, 1.0), track_color: Color::from_rgb(51, 51, 51),
            label: None, width: 300.0,
            on_change: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn value(mut self, v: f32) -> Self { self.initial_value = v; self }
    pub fn step(mut self, s: f32) -> Self { self.step = s; self }
    pub fn accent_color(mut self, c: Color) -> Self { self.accent_color = c; self }
    pub fn track_color(mut self, c: Color) -> Self { self.track_color = c; self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn on_change(mut self, h: impl Fn(f32) + Send + Sync + 'static) -> Self { self.on_change = Some(Arc::new(h)); self }
    pub fn frame(mut self, w: f32, _h: f32) -> Self { self.width = w; self }

    pub fn value_text(&self, v: f32) -> String {
        if self.step > 0.0 && self.step.fract() == 0.0 { format!("{:.0}", v) }
        else if self.step > 0.0 {
            let s = format!("{:.10}", self.step);
            let dec = s.trim_end_matches('0').split('.').last().unwrap_or("0").len();
            format!("{:.prec$}", v, prec = dec)
        }
        else { format!("{:.1}", v) }
    }

    pub fn to_view(self) -> View { let w = self.width; View::new(self).with_frame(0.0, 0.0, w, 80.0) }
}

impl Default for Slider { fn default() -> Self { Self::new(0.0, 100.0) } }

impl ViewContent for Slider {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let accent = self.accent_color;
        let accent_hex = format!("#{:02x}{:02x}{:02x}",
            (accent.r * 255.0) as u8, (accent.g * 255.0) as u8, (accent.b * 255.0) as u8);
        let track = self.track_color;
        let track_hex = format!("#{:02x}{:02x}{:02x}",
            (track.r * 255.0) as u8, (track.g * 255.0) as u8, (track.b * 255.0) as u8);

        let container = gtk::Box::new(Orientation::Vertical, 8);
        container.set_width_request(w as i32);

        let css = format!(
            ".sl-c {{ background: transparent; }}
             .sl-lbl {{ color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 13px; }}
             .sl-val {{ color: {accent_hex}; font-family: 'SF Pro Display'; font-size: 28px; font-weight: 600; }}
             .sl-track {{ background: {track_hex}; border-radius: 3px; min-height: 4px; }}
             .sl-fill {{ background: {accent_hex}; border-radius: 3px; min-height: 4px; }}
             .sl-thumb {{ background: white; border-radius: 15px; min-width: 30px; min-height: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.3); }}"
        );
        uikit::widget::apply_css(&container, &css);
        container.add_css_class("sl-c");

        // Header
        let header = gtk::Box::new(Orientation::Horizontal, 8);
        header.set_hexpand(true);
        if let Some(ref lbl) = self.label {
            let l = GtkLabel::new(Some(lbl)); l.add_css_class("sl-lbl"); header.append(&l);
        }
        let spacer = gtk::Box::new(Orientation::Horizontal, 0); spacer.set_hexpand(true); header.append(&spacer);
        let val_label = GtkLabel::new(Some(&self.value_text(self.initial_value)));
        val_label.add_css_class("sl-val");
        header.append(&val_label);
        container.append(&header);

        // Track container
        let track_outer = gtk::Box::new(Orientation::Vertical, 0);
        track_outer.set_hexpand(true);
        track_outer.set_valign(gtk::Align::Center);
        track_outer.set_height_request(30);

        // Track bar
        let track_bar = gtk::Overlay::new();
        track_bar.set_hexpand(true);
        track_bar.set_height_request(4);

        let track_bg = gtk::Box::new(Orientation::Horizontal, 0);
        track_bg.set_hexpand(true); track_bg.set_height_request(4);
        track_bg.add_css_class("sl-track");
        track_bar.set_child(Some(&track_bg));

        let fill_bar = gtk::Box::new(Orientation::Horizontal, 0);
        fill_bar.set_hexpand(false); fill_bar.set_height_request(4);
        fill_bar.set_halign(gtk::Align::Start);
        fill_bar.add_css_class("sl-fill");
        track_bar.add_overlay(&fill_bar);

        // Thumb (overlays on track)
        let thumb = gtk::Box::new(Orientation::Vertical, 0);
        thumb.set_width_request(30); thumb.set_height_request(20);
        thumb.set_halign(gtk::Align::Start);
        thumb.set_valign(gtk::Align::Center);
        thumb.add_css_class("sl-thumb");
        track_bar.add_overlay(&thumb);

        track_outer.append(&track_bar);
        container.append(&track_outer);

        // State
        let state = Rc::new(RefCell::new(SliderPhysics::new(self.initial_value)));
        state.borrow_mut().min = self.min;
        state.borrow_mut().max = self.max;
        state.borrow_mut().step = self.step;

        // Initial position
        {
            let s = state.borrow();
            let norm = s.value_norm();
            let track_w = w - 30.0;
            let fill_px = norm * track_w;
            let thumb_px = norm * track_w;
            fill_bar.set_width_request(fill_px.max(0.0) as i32);
            thumb.set_margin_start(thumb_px.max(0.0) as i32);
        }

        // Tick loop
        {
            let state = state.clone();
            let val_label = val_label.clone();
            let fill_bar = fill_bar.clone();
            let thumb = thumb.clone();
            let cb = self.on_change.clone();
            let step_c = self.step;
            let tw = w - 30.0;
            let mut last_fired = self.initial_value;

            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                let mut s = state.borrow_mut();
                s.tick(1.0 / 60.0);
                let new_v = s.display_value;
                let norm = s.value_norm();
                let grow = s.grow;
                let sx = s.squish_x;
                let sy = s.squish_y;
                let is_dragging = s.is_dragging;
                drop(s);

                // Position fill + thumb
                let fill_px = norm * tw;
                let thumb_px = norm * tw;
                fill_bar.set_width_request(fill_px.max(0.0) as i32);
                thumb.set_margin_start(thumb_px.max(0.0) as i32);

                // Thumb size (grow + squish)
                let thumb_w = (30.0 * grow * sx) as i32;
                let thumb_h = (20.0 * grow * sy) as i32;
                thumb.set_size_request(thumb_w.max(10), thumb_h.max(6));
                let radius = (thumb_h / 2).min(thumb_w / 2);
                let tcss = format!(".sl-thumb {{ border-radius: {}px; }}", radius);
                uikit::widget::apply_css(&thumb, &tcss);

                // Value label
                let text = if step_c > 0.0 && step_c.fract() == 0.0 { format!("{:.0}", new_v) }
                    else if step_c > 0.0 {
                        let st = format!("{:.10}", step_c);
                        let dec = st.trim_end_matches('0').split('.').last().unwrap_or("0").len();
                        format!("{:.prec$}", new_v, prec = dec)
                    }
                    else { format!("{:.1}", new_v) };
                val_label.set_text(&text);

                // Only fire callback when user is dragging AND value changed meaningfully
                let threshold = if step_c > 0.0 { step_c } else { 0.1 };
                if is_dragging && (new_v - last_fired).abs() >= threshold {
                    last_fired = new_v;
                    if let Some(ref h) = cb { h(new_v); }
                }
                glib::ControlFlow::Continue
            });
        }

        // Click + drag on track
        {
            let state = state.clone();
            let press = gtk::GestureClick::new();
            press.set_button(1);

            {
                let state = state.clone();
                let tw = w - 30.0;
                press.connect_pressed(move |_g, _n, x, _y| {
                    let mut s = state.borrow_mut();
                    s.is_dragging = true;
                    s.wobbling = false;
                    s.drag_vel = 0.0;
                    s.prev_mouse_x = x as f32;
                    s.target_value = s.x_to_value(x as f32, tw + 30.0);
                    s.display_value = s.target_value;
                });
            }
            {
                let state = state.clone();
                press.connect_released(move |_g, _n, _x, _y| {
                    let mut s = state.borrow_mut();
                    s.is_dragging = false;
                    s.wobbling = true;
                    s.wobble_time = 0.0;
                });
            }
            track_outer.add_controller(press);
        }

        {
            let state = state.clone();
            let motion = gtk::EventControllerMotion::new();
            let tw = w - 30.0;
            motion.connect_motion(move |_m, x, _y| {
                let mut s = state.borrow_mut();
                if !s.is_dragging { return; }
                s.drag_vel = x as f32 - s.prev_mouse_x;
                s.prev_mouse_x = x as f32;
                s.target_value = s.x_to_value(x as f32, tw + 30.0);
                s.display_value = s.target_value;
            });
            track_outer.add_controller(motion);
        }

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool { true }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(self.width, 80.0) }
}

impl Widget for Slider {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, self.width, 80.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_builder() {
        let s = Slider::new(0.0, 100.0).value(50.0).step(1.0).label("Vol")
            .track_color(Color::from_rgb(10, 20, 30));
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 100.0);
        assert_eq!(s.initial_value, 50.0);
        assert_eq!(s.track_color, Color::from_rgb(10, 20, 30));
    }

    #[test]
    fn slider_physics_norm() {
        let mut p = SliderPhysics::new(50.0);
        p.min = 0.0; p.max = 100.0;
        assert!((p.value_norm() - 0.5).abs() < 0.01);
    }

    #[test]
    fn slider_physics_settle() {
        let mut p = SliderPhysics::new(0.0);
        p.target_value = 50.0;
        for _ in 0..300 { p.tick(1.0 / 60.0); }
        assert!((p.display_value - 50.0).abs() < 0.5);
    }
}
