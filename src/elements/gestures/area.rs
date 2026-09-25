use std::any::Any;
use std::time::Instant;

use vello::Scene;

use super::super::layout::View;
use super::{
    GESTURE_LONG_PRESS_SECONDS, GESTURE_MAGNIFY_MAX, GESTURE_MAGNIFY_MIN,
    GESTURE_MAGNIFY_STEP, GESTURE_MOVE_SLOP,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Gesture area over any child element: tracks tap, long press, drag
/// and magnify (mouse wheel) inside the placed rect and reports them
/// through callbacks, like an action back to the app. The child fills
/// the area; with `draggable` the child follows the drag offset and
/// with `zoomable` it scales around the center, so pads can move and
/// zoom their content directly.
pub struct GestureArea<V> {
    child: V,
    on_tap: Option<Box<dyn FnMut()>>,
    on_long_press: Option<Box<dyn FnMut()>>,
    on_drag: Option<Box<dyn FnMut(f32, f32)>>,
    on_magnify: Option<Box<dyn FnMut(f32)>>,
    draggable: bool,
    zoomable: bool,
    pressed: Option<(Instant, f32, f32)>,
    long_fired: bool,
    dragging: bool,
    drag: (f32, f32),
    scale: f32,
    hovered: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl<V: View> GestureArea<V> {
    pub fn new(child: V) -> Self {
        Self {
            child,
            on_tap: None,
            on_long_press: None,
            on_drag: None,
            on_magnify: None,
            draggable: false,
            zoomable: false,
            pressed: None,
            long_fired: false,
            dragging: false,
            drag: (0.0, 0.0),
            scale: 1.0,
            hovered: false,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Quick press-and-release inside the area.
    pub fn on_tap(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_tap = Some(Box::new(callback));
        self
    }

    /// Hold inside the area past the long-press time without
    /// wandering. Fires while holding (checked every draw), so no
    /// release is needed.
    pub fn on_long_press(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_long_press = Some(Box::new(callback));
        self
    }

    /// Press-move with the total `(dx, dy)` offset from the press
    /// start in logical px, fired on every move past the slop.
    pub fn on_drag(mut self, callback: impl FnMut(f32, f32) + 'static) -> Self {
        self.on_drag = Some(Box::new(callback));
        self
    }

    /// Mouse wheel over the area with the current zoom scale
    /// (clamped, starts at 1.0).
    pub fn on_magnify(mut self, callback: impl FnMut(f32) + 'static) -> Self {
        self.on_magnify = Some(Box::new(callback));
        self
    }

    /// The child follows the drag offset (its placed origin shifts).
    pub fn draggable(mut self, enabled: bool) -> Self {
        self.draggable = enabled;
        self
    }

    /// The child scales around the area center (placed size scales).
    /// Rect-filling children scale truly; baked glyphs keep size.
    pub fn zoomable(mut self, enabled: bool) -> Self {
        self.zoomable = enabled;
        self
    }

    /// Wrapped child for state updates.
    pub fn child_mut(&mut self) -> &mut V {
        &mut self.child
    }

    /// Current drag offset `(dx, dy)` in logical px.
    pub fn drag_offset(&self) -> (f32, f32) {
        self.drag
    }

    /// Current zoom scale (starts at 1.0).
    pub fn scale_value(&self) -> f32 {
        self.scale
    }

    /// Reset drag offset and zoom scale to rest.
    pub fn reset(&mut self) {
        self.drag = (0.0, 0.0);
        self.scale = 1.0;
        self.pressed = None;
        self.long_fired = false;
        self.dragging = false;
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.placed_w && y >= self.y && y <= self.y + self.placed_h
    }

    /// Long-press check at `now` (pure helper for tests; draw calls
    /// it with the current time every frame).
    pub(crate) fn poll_long_press(&mut self, now: Instant) {
        if self.long_fired || self.dragging {
            return;
        }
        if let Some((start, _, _)) = self.pressed {
            if now.saturating_duration_since(start).as_secs_f64() >= GESTURE_LONG_PRESS_SECONDS {
                self.long_fired = true;
                if let Some(callback) = self.on_long_press.as_mut() {
                    callback();
                }
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        if !self.hit(x, y) {
            return;
        }
        self.pressed = Some((Instant::now(), x, y));
        self.long_fired = false;
        self.dragging = false;
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        let press = self.pressed.take();
        let was_dragging = self.dragging;
        let long_fired = self.long_fired;
        self.dragging = false;
        // Tap: quick release inside without wandering (wandering
        // sets `dragging` in `mouse_move`) and without a long press
        // having fired.
        if let Some((start, _, _)) = press {
            let quick = start.elapsed().as_secs_f64() < GESTURE_LONG_PRESS_SECONDS;
            if !long_fired && !was_dragging && quick && self.hit(x, y) {
                if let Some(callback) = self.on_tap.as_mut() {
                    callback();
                }
            }
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        self.hovered = self.hit(x, y);
        if let Some((_, sx, sy)) = self.pressed {
            if (x - sx).abs() > GESTURE_MOVE_SLOP || (y - sy).abs() > GESTURE_MOVE_SLOP {
                self.dragging = true;
                self.drag = (x - sx, y - sy);
                if let Some(callback) = self.on_drag.as_mut() {
                    callback(self.drag.0, self.drag.1);
                }
            }
        }
    }

    /// Mouse wheel delta in logical px (right/down positive, like the
    /// shell). Zooms while hovered and reports the scale.
    pub fn mouse_wheel(&mut self, _dx: f64, dy: f64) {
        if !self.hovered {
            return;
        }
        self.scale = (self.scale * (1.0 + dy as f32 * GESTURE_MAGNIFY_STEP))
            .clamp(GESTURE_MAGNIFY_MIN, GESTURE_MAGNIFY_MAX);
        if let Some(callback) = self.on_magnify.as_mut() {
            callback(self.scale);
        }
    }

    fn place_child(&mut self, fonts: &mut FontSystem) {
        let s = if self.zoomable { self.scale.max(0.0) } else { 1.0 };
        let (w, h) = (self.placed_w * s, self.placed_h * s);
        let (ox, oy) = if self.draggable {
            (self.x + self.drag.0, self.y + self.drag.1)
        } else {
            (self.x, self.y)
        };
        // Zoom keeps the area center fixed.
        let (ox, oy) = (
            ox + (self.placed_w - w) / 2.0,
            oy + (self.placed_h - h) / 2.0,
        );
        self.child.place(fonts, ox, oy, w, h);
    }
}

impl<V: View + 'static> View for GestureArea<V> {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.child.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.place_child(fonts);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        // Hold-to-long-press needs no events: poll every frame.
        self.poll_long_press(Instant::now());
        // Drag and zoom move the child live.
        if self.draggable || self.zoomable {
            self.place_child(fonts);
        }
        self.child.draw(scene, fonts, images);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Spacer;
    use crate::renderer::text::FontSystem;

    fn area() -> GestureArea<Spacer> {
        GestureArea::new(Spacer::new())
    }

    fn placed() -> GestureArea<Spacer> {
        let mut area = area();
        let mut fonts = FontSystem::new();
        area.place(&mut fonts, 0.0, 0.0, 200.0, 100.0);
        area
    }

    #[test]
    fn tap_fires_on_quick_release_inside() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let taps = count.clone();
        let mut area = placed().on_tap(move || {
            *taps.borrow_mut() += 1;
        });
        area.mouse_down(50.0, 50.0);
        area.mouse_up(52.0, 51.0);
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn tap_ignored_outside() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let taps = count.clone();
        let mut area = placed().on_tap(move || {
            *taps.borrow_mut() += 1;
        });
        area.mouse_down(500.0, 500.0);
        area.mouse_up(500.0, 500.0);
        area.mouse_down(50.0, 50.0);
        area.mouse_up(500.0, 500.0);
        assert_eq!(*count.borrow(), 0);
    }

    #[test]
    fn drag_tracks_offset_and_kills_tap() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let taps: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let drags: Rc<RefCell<Vec<(f32, f32)>>> = Rc::new(RefCell::new(Vec::new()));
        let t = taps.clone();
        let d = drags.clone();
        let mut area = placed()
            .on_tap(move || {
                *t.borrow_mut() += 1;
            })
            .on_drag(move |dx, dy| {
                d.borrow_mut().push((dx, dy));
            });
        area.mouse_down(50.0, 50.0);
        area.mouse_move(52.0, 51.0);
        // Inside the slop: no drag yet.
        assert_eq!(area.drag_offset(), (0.0, 0.0));
        area.mouse_move(80.0, 50.0);
        assert_eq!(area.drag_offset(), (30.0, 0.0));
        assert_eq!(drags.borrow().len(), 1);
        area.mouse_up(80.0, 50.0);
        assert_eq!(*taps.borrow(), 0);
    }

    #[test]
    fn long_press_fires_at_threshold() {
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::time::Duration;

        let count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let longs = count.clone();
        let mut area = placed().on_long_press(move || {
            *longs.borrow_mut() += 1;
        });
        area.mouse_down(50.0, 50.0);
        let start = Instant::now();
        area.poll_long_press(start);
        assert_eq!(*count.borrow(), 0);
        area.poll_long_press(start + Duration::from_secs_f64(GESTURE_LONG_PRESS_SECONDS));
        assert_eq!(*count.borrow(), 1);
        // Fires only once.
        area.poll_long_press(start + Duration::from_secs_f64(5.0));
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn wander_cancels_long_press() {
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::time::Duration;

        let count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let longs = count.clone();
        let mut area = placed().on_long_press(move || {
            *longs.borrow_mut() += 1;
        });
        area.mouse_down(50.0, 50.0);
        area.mouse_move(100.0, 50.0);
        area.poll_long_press(Instant::now() + Duration::from_secs_f64(5.0));
        assert_eq!(*count.borrow(), 0);
    }

    #[test]
    fn magnify_clamps_and_reports() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let last: Rc<RefCell<f32>> = Rc::new(RefCell::new(1.0));
        let seen = last.clone();
        let mut area = placed().on_magnify(move |s| {
            *seen.borrow_mut() = s;
        });
        // Not hovered: wheel ignored.
        area.mouse_wheel(0.0, 20.0);
        assert_eq!(area.scale_value(), 1.0);
        area.mouse_move(50.0, 50.0);
        area.mouse_wheel(0.0, 20.0);
        assert!(area.scale_value() > 1.0);
        assert_eq!(*last.borrow(), area.scale_value());
        area.mouse_wheel(0.0, -100000.0);
        assert_eq!(area.scale_value(), GESTURE_MAGNIFY_MIN);
        area.mouse_wheel(0.0, 100000.0);
        assert_eq!(area.scale_value(), GESTURE_MAGNIFY_MAX);
    }

    #[test]
    fn reset_clears_state() {
        let mut area = placed();
        area.mouse_down(50.0, 50.0);
        area.mouse_move(80.0, 50.0);
        area.mouse_move(50.0, 50.0);
        area.mouse_wheel(0.0, 20.0);
        area.reset();
        assert_eq!(area.drag_offset(), (0.0, 0.0));
        assert_eq!(area.scale_value(), 1.0);
    }
}
