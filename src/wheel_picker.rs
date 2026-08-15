//! WheelPicker — macOS/iOS style scroll wheel picker with spring physics.
//!
//! A vertically scrollable wheel with snap-to-center spring animation and a
//! smooth 3D fade/scale effect, matching the macOS/iOS `Picker` drum. Items
//! are positioned absolutely around a fixed center selection bar, so the
//! wheel animates cleanly while dragging, flinging or scrolling.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let picker = WheelPicker::new()
//!     .items(["1", "2", "3", "4", "5", "6"])
//!     .selected("3")
//!     .on_change(|value| println!("Selected: {}", value));
//!
//! let view = View::new(picker).with_frame(0.0, 0.0, 280.0, 176.0);
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, pango, Label};
use gtk::Orientation;

// ═══════════════════════════════════════════════════════════════
// Physics + visual constants
// ═══════════════════════════════════════════════════════════════

/// Height of each item row in the wheel drum (pt).
const ITEM_HEIGHT: f32 = 40.0;

/// Spring stiffness — higher -> snappier response.
const SPRING_STIFFNESS: f32 = 280.0;

/// Spring damping — controls overshoot.
const SPRING_DAMPING: f32 = 26.0;

/// Pixel distance from the center at which an item is fully faded.
const FADE_DISTANCE: f32 = 70.0;

/// Font size of the item sitting in the center of the wheel (pt).
const BASE_FONT_SIZE: f32 = 18.0;

/// Font scale of an item at the very edge of the wheel (min scale).
const MIN_SCALE: f32 = 0.85;

// ═══════════════════════════════════════════════════════════════
// WheelState — spring physics
// ═══════════════════════════════════════════════════════════════

/// Mutable spring state for the wheel. Drives the snap-to-center lock.
#[derive(Debug, Clone)]
struct WheelState {
    /// Current scroll position (fractional item index).
    scroll_offset: f32,
    /// Velocity of the scroll (items / second).
    scroll_velocity: f32,
    /// The offset the spring pulls toward (nearest item index).
    target_offset: f32,
    /// Whether the user's mouse button is held down on the wheel.
    is_dragging: bool,
    /// Whether scroll events have arrived recently (drum is gliding).
    is_scrolling: bool,
    /// Y widget-coordinate where the current drag began.
    drag_start_y: f32,
    /// `scroll_offset` at the moment the drag started.
    drag_start_offset: f32,
    /// `scroll_offset` from the previous tick, used to measure velocity.
    last_offset: f32,
}

impl WheelState {
    fn new(initial: usize) -> Self {
        Self {
            scroll_offset: initial as f32,
            scroll_velocity: 0.0,
            target_offset: initial as f32,
            is_dragging: false,
            is_scrolling: false,
            drag_start_y: 0.0,
            drag_start_offset: 0.0,
            last_offset: initial as f32,
        }
    }

    /// Advance the spring simulation by `dt` seconds.
    ///
    /// While dragging, only the finger velocity is tracked so releasing the
    /// wheel produces a natural fling before the spring locks it into place.
    fn tick(&mut self, dt: f32) {
        if self.is_dragging {
            self.scroll_velocity = ((self.scroll_offset - self.last_offset) / dt.max(1e-4))
                .clamp(-40.0, 40.0);
            self.last_offset = self.scroll_offset;
            return;
        }

        let displacement = self.scroll_offset - self.target_offset;
        let spring_force = -SPRING_STIFFNESS * displacement;
        let damping_force = -SPRING_DAMPING * self.scroll_velocity;

        self.scroll_velocity += (spring_force + damping_force) * dt;
        self.scroll_offset += self.scroll_velocity * dt;

        // Settle when very close — avoids perpetual micro-oscillation.
        if displacement.abs() < 0.001 && self.scroll_velocity.abs() < 0.01 {
            self.scroll_offset = self.target_offset;
            self.scroll_velocity = 0.0;
        }
    }

    /// Snap `target_offset` to the nearest item index (clamped).
    ///
    /// Call once at the end of a drag gesture to initiate the lock animation.
    fn snap(&mut self, total: usize) {
        if total == 0 {
            return;
        }
        self.target_offset = self.scroll_offset.round().clamp(0.0, (total as f32) - 1.0);
    }

    /// Integer index of the item closest to the current scroll position.
    fn selected_index(&self, total: usize) -> usize {
        if total == 0 {
            return 0;
        }
        self.scroll_offset.round().clamp(0.0, (total as f32) - 1.0) as usize
    }
}

// ═══════════════════════════════════════════════════════════════
// Visual helpers — 3D fade / scale
// ═══════════════════════════════════════════════════════════════

/// How far (0..=1) an item at index distance `dist` sits from the center.
fn item_factor(dist: f32) -> f32 {
    (dist * ITEM_HEIGHT / FADE_DISTANCE).clamp(0.0, 1.0)
}

/// Opacity for a center-distance factor (fades to 35% at the edges).
fn item_opacity(factor: f32) -> f32 {
    1.0 - factor * 0.65
}

/// Font scale for a center-distance factor (scales down at the edges).
fn item_scale(factor: f32) -> f32 {
    1.0 - factor * (1.0 - MIN_SCALE)
}

/// Apply the 3D fade/scale styling for an item `dist` rows from the center.
///
/// Uses native Pango attributes (family, base size, scale, weight) plus a
/// foreground-alpha fade, so the numbers fade and scale smoothly per frame.
fn style_item(label: &Label, dist: f32) {
    let factor = item_factor(dist);

    let attrs = pango::AttrList::new();
    attrs.insert(pango::AttrString::new_family("SF Pro Display"));
    attrs.insert(pango::AttrSize::new((BASE_FONT_SIZE * 1024.0) as i32));
    attrs.insert(pango::AttrFloat::new_scale(item_scale(factor) as f64));
    attrs.insert(pango::AttrInt::new_weight(if dist < 0.5 {
        pango::Weight::Bold
    } else {
        pango::Weight::Normal
    }));
    attrs.insert(pango::AttrInt::new_foreground_alpha(
        (item_opacity(factor) * 65535.0) as u16,
    ));
    label.set_attributes(Some(&attrs));
}

/// Position and style every label for the current wheel state.
fn layout_items(state: &WheelState, labels: &[Label], h: f32) {
    let center = state.scroll_offset;
    let mid = h / 2.0;
    for (i, label) in labels.iter().enumerate() {
        let dist = (i as f32 - center).abs();
        let y = mid + (i as f32 - center) * ITEM_HEIGHT - ITEM_HEIGHT / 2.0;
        label.set_margin_top(y.round() as i32);
        style_item(label, dist);
    }
}

// ═══════════════════════════════════════════════════════════════
// WheelPicker — builder API
// ═══════════════════════════════════════════════════════════════

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
            height: 176.0,
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
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for WheelPicker {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };
        let item_h = ITEM_HEIGHT;
        let num_items = self.items.len();

        // Root overlay sized w x h; items are positioned absolutely on top.
        let overlay = gtk::Overlay::new();
        overlay.set_width_request(w as i32);
        overlay.set_height_request(h as i32);

        let css = format!(
            ".wheel-picker {{
                background: rgba(28, 28, 30, 0.6);
                border-radius: 20px;
                border: 1px solid #3a3a3c;
                overflow: hidden;
            }}
            .wheel-highlight {{
                background: rgba(255, 255, 255, 0.10);
                border-top: 1px solid rgba(255, 255, 255, 0.04);
                border-bottom: 1px solid rgba(255, 255, 255, 0.04);
                border-radius: 12px;
            }}"
        );
        uikit::widget::apply_css(&overlay, &css);
        overlay.add_css_class("wheel-picker");
        // Clip labels to the drum so no text leaks outside the element.
        overlay.set_overflow(gtk::Overflow::Hidden);

        // Main child: transparent box so the overlay has a real size.
        let base = gtk::Box::new(Orientation::Vertical, 0);
        base.set_width_request(w as i32);
        base.set_height_request(h as i32);
        overlay.set_child(Some(&base));

        // Item labels (absolutely positioned overlay children).
        let mut labels: Vec<Label> = Vec::with_capacity(num_items);
        for item_text in self.items.iter() {
            let label = Label::new(Some(item_text));
            label.set_width_request(w as i32);
            label.set_height_request(item_h as i32);
            label.set_xalign(0.5);
            label.set_yalign(0.5);
            label.set_halign(gtk::Align::Fill);
            label.set_valign(gtk::Align::Start);
            overlay.add_overlay(&label);
            labels.push(label);
        }

        // Center selection "lock" bar.
        let highlight = gtk::Box::new(Orientation::Vertical, 0);
        highlight.set_height_request(item_h as i32);
        highlight.set_halign(gtk::Align::Fill);
        highlight.set_valign(gtk::Align::Center);
        highlight.add_css_class("wheel-highlight");
        overlay.add_overlay(&highlight);

        // ── Spring physics state ──
        let state = Rc::new(RefCell::new(WheelState::new(self.selected_index)));

        // Initial layout so nothing flashes before the first tick.
        if num_items > 0 {
            layout_items(&state.borrow(), &labels, h);
        }

        // Timestamp of the last scroll event, used to detect scroll-stop.
        let last_scroll = Rc::new(std::cell::Cell::new(std::time::Instant::now()));

        // ── Tick loop: spring + item layout ──
        {
            let state = state.clone();
            let labels = labels.clone();
            let items = self.items.clone();
            let cb = self.on_change.clone();
            let last_scroll = last_scroll.clone();
            let mut last_fired_idx = self.selected_index;

            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                {
                    let mut s = state.borrow_mut();
                    // Lock onto the nearest item shortly after scrolling stops.
                    // The accumulated target (not the lagging offset) decides
                    // the lock, so a fast scroll keeps its momentum.
                    if s.is_scrolling && last_scroll.get().elapsed() > std::time::Duration::from_millis(120)
                    {
                        s.is_scrolling = false;
                        s.target_offset = s
                            .target_offset
                            .round()
                            .clamp(0.0, (num_items as f32) - 1.0);
                    }
                    s.tick(1.0 / 60.0);
                }

                let s = state.borrow();
                let center = s.scroll_offset;
                let new_idx = s.selected_index(num_items);
                drop(s);

                let mid = h / 2.0;
                for (i, label) in labels.iter().enumerate() {
                    let dist = (i as f32 - center).abs();
                    let y = mid + (i as f32 - center) * item_h - item_h / 2.0;
                    label.set_margin_top(y.round() as i32);
                    style_item(label, dist);
                }

                // Fire callback only when the selected index changes.
                if new_idx != last_fired_idx && new_idx < items.len() {
                    last_fired_idx = new_idx;
                    if let Some(ref cb) = cb {
                        cb(items[new_idx].clone());
                    }
                }

                glib::ControlFlow::Continue
            });
        }

        // ── Drag gesture: lock the wheel while the mouse is held ──
        let press = gtk::GestureClick::new();
        press.set_button(1);
        {
            let state = state.clone();
            press.connect_pressed(move |_g, _n, _x, y| {
                let mut s = state.borrow_mut();
                s.is_dragging = true;
                s.drag_start_y = y as f32;
                s.drag_start_offset = s.scroll_offset;
                s.scroll_velocity = 0.0;
                s.last_offset = s.scroll_offset;
            });
        }
        {
            let state = state.clone();
            press.connect_released(move |_g, _n, _x, _y| {
                let mut s = state.borrow_mut();
                s.is_dragging = false;
                s.snap(num_items);
            });
        }
        overlay.add_controller(press);

        // ── Drag motion: items follow the pointer 1:1 ──
        let motion = gtk::EventControllerMotion::new();
        {
            let state = state.clone();
            motion.connect_motion(move |_m, _x, y| {
                let mut s = state.borrow_mut();
                if !s.is_dragging {
                    return;
                }
                let dy = y as f32 - s.drag_start_y;
                let delta_items = -dy / item_h;
                let max = (num_items as f32) - 1.0;
                s.scroll_offset = (s.drag_start_offset + delta_items).clamp(0.0, max.max(0.0));
            });
        }
        overlay.add_controller(motion);

        // ── Mouse wheel / trackpad scroll ──
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
        {
            let state = state.clone();
            let last_scroll = last_scroll.clone();
            scroll.connect_scroll(move |_c, dx, dy| {
                let mut s = state.borrow_mut();
                if num_items == 0 {
                    return glib::Propagation::Stop;
                }
                last_scroll.set(std::time::Instant::now());
                // Scrolling up (positive dy) reveals earlier items. The spring
                // glides the drum toward this moving target, then locks.
                let delta = if dy.abs() >= dx.abs() { dy } else { dx };
                let max = (num_items as f32) - 1.0;
                s.is_scrolling = true;
                s.target_offset = (s.target_offset - delta as f32).clamp(0.0, max);
                glib::Propagation::Stop
            });
        }
        overlay.add_controller(scroll);

        overlay.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for WheelPicker {
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
        self.render(Rect::new(0.0, 0.0, self.width, self.height))
    }
    fn is_interactive(&self) -> bool {
        true
    }
    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
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

    #[test]
    fn wheel_state_fling_locks() {
        // Release with velocity should lock onto the nearest item.
        let mut s = WheelState::new(0);
        s.scroll_offset = 2.4;
        s.scroll_velocity = 25.0;
        s.snap(6);
        for _ in 0..240 {
            s.tick(1.0 / 60.0);
        }
        assert!((s.scroll_offset - 2.0).abs() < 0.01);
    }

    #[test]
    fn wheel_visual_helpers() {
        assert_eq!(item_factor(0.0), 0.0);
        assert!((item_opacity(item_factor(0.0)) - 1.0).abs() < 1e-4);
        // An item fully faded (1.75 rows away) keeps 35% opacity.
        assert!((item_opacity(item_factor(1.75)) - 0.35).abs() < 1e-4);
        // Center scale is 1.0, edge scale the minimum.
        assert!((item_scale(item_factor(0.0)) - 1.0).abs() < 1e-4);
        assert!((item_scale(item_factor(1.75)) - MIN_SCALE).abs() < 1e-4);
    }
}
