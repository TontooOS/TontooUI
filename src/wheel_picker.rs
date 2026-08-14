//! WheelPicker — macOS/iOS style scroll wheel picker with spring physics.
//!
//! A scrollable wheel with snap-to-center spring animation and smooth
//! 3D fading effect, matching the compositor's WheelPickerState.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let picker = WheelPicker::new()
//!     .items(["1", "2", "3", "4", "5", "6"])
//!     .selected("3")
//!     .on_change(|value| println!("Selected: {}", value));
//!
//! let view = View::new(picker).with_frame(0.0, 0.0, 280.0, 200.0);
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Label, PolicyType};
use gtk::Orientation;

const ITEM_HEIGHT: f32 = 40.0;
const SPRING_STIFFNESS: f32 = 280.0;
const SPRING_DAMPING: f32 = 26.0;

/// Mutable spring state for the wheel.
#[derive(Debug, Clone)]
struct WheelState {
    scroll_offset: f32,
    scroll_velocity: f32,
    target_offset: f32,
    is_dragging: bool,
    drag_start_y: f32,
    drag_start_offset: f32,
}

impl WheelState {
    fn new(initial: usize) -> Self {
        Self {
            scroll_offset: initial as f32,
            scroll_velocity: 0.0,
            target_offset: initial as f32,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_offset: 0.0,
        }
    }

    fn tick(&mut self, dt: f32) {
        if self.is_dragging {
            return;
        }
        let displacement = self.scroll_offset - self.target_offset;
        let spring_force = -SPRING_STIFFNESS * displacement;
        let damping_force = -SPRING_DAMPING * self.scroll_velocity;
        self.scroll_velocity += (spring_force + damping_force) * dt;
        self.scroll_offset += self.scroll_velocity * dt;

        if displacement.abs() < 0.001 && self.scroll_velocity.abs() < 0.01 {
            self.scroll_offset = self.target_offset;
            self.scroll_velocity = 0.0;
        }
    }

    fn snap(&mut self, total: usize) {
        if total == 0 { return; }
        self.target_offset = self.scroll_offset.round().clamp(0.0, (total as f32) - 1.0);
    }

    fn selected_index(&self, total: usize) -> usize {
        if total == 0 { return 0; }
        self.scroll_offset.round().clamp(0.0, (total as f32) - 1.0) as usize
    }
}

/// A macOS/iOS style wheel picker with spring physics.
pub struct WheelPicker {
    id: WidgetId,
    items: Vec<String>,
    selected_index: usize,
    accent_color: Color,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
    width: f32,
    height: f32,
}

impl WheelPicker {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            items: Vec::new(),
            selected_index: 0,
            accent_color: Color::new(0.047, 0.522, 0.937, 1.0),
            on_change: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
            width: 280.0,
            height: 200.0,
        }
    }

    pub fn items<I, S>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.items = iter.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn item(mut self, text: impl Into<String>) -> Self {
        self.items.push(text.into());
        self
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        let val = value.into();
        self.selected_index = self.items.iter().position(|i| *i == val).unwrap_or(0);
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    pub fn frame(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn selected_value(&self) -> &str {
        self.items.get(self.selected_index).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn to_view(self) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl Default for WheelPicker {
    fn default() -> Self { Self::new() }
}

impl ViewContent for WheelPicker {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };
        let item_h = ITEM_HEIGHT;
        let num_items = self.items.len();

        // Main container
        let container = gtk::Box::new(Orientation::Vertical, 0);
        container.set_width_request(w as i32);
        container.set_height_request(h as i32);

        let border_color = "#3a3a3c";
        let highlight_bg = "rgba(255, 255, 255, 0.08)";
        let highlight_border = "rgba(255, 255, 255, 0.05)";

        let css = format!(
            ".wheel-picker {{
                background: rgba(28, 28, 30, 0.6);
                border-radius: 20px;
                border: 1px solid {border_color};
            }}
            .wheel-scroll {{
                background: transparent;
            }}
            .wheel-scroll > scrollbar {{ background: transparent; border: none; }}
            .wheel-scroll > scrollbar slider {{ background: transparent; border: none; }}
            .wheel-highlight {{
                background: {highlight_bg};
                border-top: 1px solid {highlight_border};
                border-bottom: 1px solid {highlight_border};
                border-radius: 10px;
            }}",
            border_color = border_color,
            highlight_bg = highlight_bg,
            highlight_border = highlight_border,
        );
        uikit::widget::apply_css(&container, &css);
        container.add_css_class("wheel-picker");

        // Items box
        let items_box = gtk::Box::new(Orientation::Vertical, 0);
        items_box.set_halign(gtk::Align::Center);
        items_box.set_valign(gtk::Align::Start);

        // Top spacer
        let top_pad = ((h - item_h) / 2.0).max(0.0) as i32;
        let spacer_top = gtk::Box::new(Orientation::Vertical, 0);
        spacer_top.set_height_request(top_pad);
        items_box.append(&spacer_top);

        let mut labels: Vec<gtk::Label> = Vec::new();
        for (i, item_text) in self.items.iter().enumerate() {
            let label = Label::new(Some(item_text));
            label.set_height_request(item_h as i32);
            label.set_width_request(w as i32);
            label.set_valign(gtk::Align::Center);
            label.set_halign(gtk::Align::Center);
            // Initial opacity based on distance from selected
            let dist = (i as f32 - self.selected_index as f32).abs();
            let max_dist = (h / item_h / 2.0).max(1.0);
            let factor = (dist / max_dist).min(1.0);
            let opacity = 1.0 - factor * 0.65;
            label.set_opacity(opacity as f64);
            if i == self.selected_index {
                label.set_markup(&format!(
                    "<span font='SF Pro Display 18' weight='bold'>{}</span>", item_text
                ));
            } else {
                label.set_markup(&format!(
                    "<span font='SF Pro Display 18' weight='normal'>{}</span>", item_text
                ));
            }
            labels.push(label.clone());
            items_box.append(&label);
        }

        // Bottom spacer
        let spacer_bottom = gtk::Box::new(Orientation::Vertical, 0);
        spacer_bottom.set_height_request(top_pad);
        items_box.append(&spacer_bottom);

        // Scrolled window
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(&items_box));
        scroll.set_size_request(w as i32, h as i32);
        scroll.add_css_class("wheel-scroll");

        // Highlight bar
        let highlight = gtk::Box::new(Orientation::Vertical, 0);
        highlight.set_height_request(item_h as i32);
        highlight.set_valign(gtk::Align::Center);
        highlight.set_halign(gtk::Align::Fill);
        highlight.set_hexpand(true);
        highlight.add_css_class("wheel-highlight");

        let overlay = gtk::Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        overlay.set_child(Some(&scroll));
        overlay.add_overlay(&highlight);
        container.append(&overlay);

        // ── Spring physics state ──
        let state = Rc::new(RefCell::new(WheelState::new(self.selected_index)));
        let callback = self.on_change.clone();
        let items_clone = self.items.clone();
        let num = num_items;

        // Initial scroll position
        {
            let adj = scroll.vadjustment();
            let target = self.selected_index as f64 * item_h as f64;
            adj.set_value(target);
        }

        // ── Tick loop: spring + update labels ──
        let state_tick = state.clone();
        let labels_tick = labels.clone();
        let items_tick = items_clone.clone();
        let adj_tick = scroll.vadjustment();
        let cb_tick = callback.clone();
        let h_tick = h;
        let mut last_fired_idx = self.selected_index;

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let mut s = state_tick.borrow_mut();
            s.tick(1.0 / 60.0);
            drop(s);

            let s = state_tick.borrow();
            let scroll_y = s.scroll_offset as f64 * item_h as f64;
            adj_tick.set_value(scroll_y);

            // Update each label's opacity + weight based on distance from center
            let center_idx = s.scroll_offset;
            for (i, label) in labels_tick.iter().enumerate() {
                let dist = (i as f32 - center_idx).abs();
                let max_dist = (h_tick / item_h / 2.0).max(1.0);
                let factor = (dist / max_dist).min(1.0);
                let opacity = 1.0 - factor * 0.65;
                label.set_opacity(opacity as f64);

                if dist < 0.5 {
                    label.set_markup(&format!(
                        "<span font='SF Pro Display 18' weight='bold'>{}</span>",
                        items_tick[i]
                    ));
                } else {
                    label.set_markup(&format!(
                        "<span font='SF Pro Display 18' weight='normal'>{}</span>",
                        items_tick[i]
                    ));
                }
            }

            // Fire callback only when selected index changes
            let new_idx = s.selected_index(num);
            if new_idx != last_fired_idx && new_idx < items_tick.len() {
                last_fired_idx = new_idx;
                if let Some(ref cb) = cb_tick {
                    cb(items_tick[new_idx].clone());
                }
            }
            drop(s);

            glib::ControlFlow::Continue
        });

        // ── Drag gesture: track mouse for scroll ──
        let press = gtk::GestureClick::new();
        press.set_button(1);
        {
            let state_drag = state.clone();
            press.connect_pressed(move |_g, _n, _x, y| {
                let mut s = state_drag.borrow_mut();
                s.is_dragging = true;
                s.drag_start_y = y as f32;
                s.drag_start_offset = s.scroll_offset;
                s.scroll_velocity = 0.0;
            });
        }
        {
            let state_drag = state.clone();
            press.connect_released(move |_g, _n, _x, _y| {
                let mut s = state_drag.borrow_mut();
                s.is_dragging = false;
                s.snap(num);
            });
        }
        items_box.add_controller(press);

        // Motion controller for dragging
        let motion = gtk::EventControllerMotion::new();
        {
            let state_motion = state.clone();
            motion.connect_motion(move |_m, _x, y| {
                let mut s = state_motion.borrow_mut();
                if !s.is_dragging { return; }
                let dy = y as f32 - s.drag_start_y;
                let delta_items = -dy / item_h;
                s.scroll_offset = (s.drag_start_offset + delta_items)
                    .clamp(0.0, (num as f32) - 1.0);
            });
        }
        items_box.add_controller(motion);

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool { true }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for WheelPicker {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, self.width, self.height))
    }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_picker_builder() {
        let picker = WheelPicker::new()
            .items(["A", "B", "C", "D"])
            .selected("C")
            .frame(300.0, 250.0);
        assert_eq!(picker.items.len(), 4);
        assert_eq!(picker.selected_index, 2);
        assert_eq!(picker.selected_value(), "C");
    }

    #[test]
    fn wheel_picker_default() {
        let picker = WheelPicker::new().items(["1", "2", "3"]);
        assert_eq!(picker.selected_index, 0);
        assert_eq!(picker.selected_value(), "1");
    }

    #[test]
    fn wheel_state_snap() {
        let mut s = WheelState::new(2);
        s.scroll_offset = 2.4;
        s.snap(6);
        assert_eq!(s.target_offset, 2.0);
        s.scroll_offset = 2.6;
        s.snap(6);
        assert_eq!(s.target_offset, 3.0);
    }

    #[test]
    fn wheel_state_settle() {
        let mut s = WheelState::new(0);
        s.target_offset = 3.0;
        for _ in 0..300 {
            s.tick(1.0 / 60.0);
        }
        assert!((s.scroll_offset - 3.0).abs() < 0.01);
    }
}
