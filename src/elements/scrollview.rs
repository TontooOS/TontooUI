use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};

use super::layout::View;
use super::scrollbar::{SCROLLBAR_W_HOVER, Scrollbar};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Overlay bar width in logical px. Matches the hover width of
/// `Scrollbar` so the thumb never covers more content than needed.
pub const SCROLLVIEW_BAR_W: f32 = SCROLLBAR_W_HOVER;

/// Vertical scroll container. The child keeps its intrinsic height
/// (`total`), but the view only occupies the placed rect: content
/// outside is clipped and reached through the integrated `Scrollbar`
/// (wheel, thumb drag, track page jump) instead of growing the
/// window.
///
/// The bar is the existing `Scrollbar` from TontooUI: it overlays the
/// right edge, stays hidden until needed and fades out after
/// `SCROLLBAR_HIDE_DELAY` idle seconds.
///
/// Place the view with a bounded height (directly with the viewport
/// or as a flex child in a stack); `flex` is `1.0` so stacks hand it
/// the remaining space instead of its full content height.
pub struct ScrollView {
    child: Box<dyn View>,
    bar: Scrollbar,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    total: f32,
    content_width: f32,
    last_model: (f32, f32),
}

impl ScrollView {
    pub fn new(child: impl View + 'static) -> Self {
        Self {
            child: Box::new(child),
            bar: Scrollbar::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            total: 0.0,
            content_width: 0.0,
            last_model: (-1.0, -1.0),
        }
    }

    /// Typed access to the wrapped child for state updates.
    pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T> {
        self.child.as_any_mut().downcast_mut::<T>()
    }

    /// Full content height in logical px (last sync).
    pub fn total(&self) -> f32 {
        self.total
    }

    /// Viewport height in logical px (last place).
    pub fn visible(&self) -> f32 {
        self.height
    }

    pub fn offset(&self) -> f32 {
        self.bar.offset()
    }

    pub fn max_offset(&self) -> f32 {
        self.bar.max_offset()
    }

    pub fn scrollable(&self) -> bool {
        self.bar.scrollable()
    }

    /// Set the scroll offset immediately (clamped).
    pub fn set_offset(&mut self, offset: f32) {
        self.bar.set_offset(offset);
    }

    /// Move by `delta` in logical px (clamped).
    pub fn scroll_by(&mut self, delta: f32) {
        self.bar.scroll_by(delta);
    }

    /// Animated jump to `offset` (clamped), like a track click.
    pub fn scroll_to(&mut self, offset: f32) {
        self.bar.scroll_to(offset);
    }

    /// Flash the bar visible (e.g. after the content changed).
    pub fn flash(&mut self) {
        self.bar.flash();
    }

    /// Manual thumb color: wins over the theme accent until cleared.
    pub fn accent(mut self, color: Color) -> Self {
        let bar = std::mem::take(&mut self.bar).accent(color);
        self.bar = bar;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        let bar = std::mem::take(&mut self.bar).disabled(disabled);
        self.bar = bar;
        self
    }

    pub fn on_scroll(mut self, callback: impl FnMut(f32) + 'static) -> Self {
        let bar = std::mem::take(&mut self.bar).on_scroll(callback);
        self.bar = bar;
        self
    }

    /// Live theme forwarded to the bar (gray thumb on the default
    /// accent, theme accent otherwise). A manual `accent` wins.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.bar.set_theme(accent, dark);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.bar.set_focused(focused);
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// True while the pointer is over the overlay bar (with the same
    /// generous grab area the `Scrollbar` itself uses). Presses there
    /// drive the bar and never reach the child underneath.
    fn over_bar(&self, x: f32, y: f32) -> bool {
        if !self.bar.scrollable() || self.width <= 0.0 || self.height <= 0.0 {
            return false;
        }
        let bx = self.x + self.width - SCROLLVIEW_BAR_W;
        x >= bx - 2.0 && x <= self.x + self.width + 2.0 && y >= self.y && y <= self.y + self.height
    }

    /// Measure the child, update the bar model and place the child
    /// shifted up by the current offset. `set_content` flashes the
    /// bar, so it only runs when the model changed (same pattern as
    /// the scrollbar example); every-frame calls would keep the bar
    /// awake forever.
    fn sync(&mut self, fonts: &mut FontSystem) {
        let (cw, ch) = self.child.measure(fonts);
        self.content_width = cw;
        self.total = ch;
        let model = (self.total, self.height);
        if model != self.last_model {
            self.last_model = model;
            self.bar.set_content(self.total, self.height);
        }
        let bw = SCROLLVIEW_BAR_W.min(self.width).max(0.0);
        self.bar.set_rect(
            self.x + (self.width - bw).max(0.0),
            self.y,
            bw,
            self.height,
        );
        self.child.place(
            fonts,
            self.x,
            self.y - self.bar.offset(),
            self.width,
            self.total.max(0.0),
        );
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        self.bar.mouse_down(x, y);
        let (x32, y32) = (x as f32, y as f32);
        if self.hit(x32, y32) && !self.over_bar(x32, y32) {
            self.child.mouse_down(x, y);
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.mouse_move(x, y);
        // Raw coordinates: the child was placed shifted by the
        // offset, so scrolled-away content never hits. Forwarding
        // outside points clears stale hovers.
        self.child.set_hover(x as f32, y as f32);
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.bar.mouse_up(x, y);
        self.child.mouse_up(x, y);
    }

    /// Scroll wheel delta in logical px (right/down positive, like
    /// the shell). The child re-places on the next draw, so no
    /// fonts are needed here.
    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.bar.mouse_wheel(dx, dy);
    }
}

impl View for ScrollView {
    /// Intrinsic content size. Place with a bounded height: the view
    /// occupies the placed rect and scrolls inside it instead of
    /// growing the window.
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.child.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
        self.sync(fonts);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        // Re-sync every frame: wheel and thumb drags change the
        // offset without a new place call, and the child may have
        // grown (new rows, longer text).
        self.sync(fonts);
        if self.width <= 0.0 || self.height <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let clip = Rect::new(
            self.x as f64 * scale,
            self.y as f64 * scale,
            (self.x + self.width) as f64 * scale,
            (self.y + self.height) as f64 * scale,
        );
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        self.child.draw(scene, fonts, images);
        scene.pop_layer();
        self.bar.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        ScrollView::mouse_down(self, x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        ScrollView::mouse_up(self, x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.bar.mouse_move(x as f64, y as f64);
        self.child.set_hover(x, y);
    }

    /// Take the remaining stack space instead of the full content
    /// height, so surrounding windows keep their size.
    fn flex(&self) -> f32 {
        1.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::layout::{Spacer, VStack};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn fonts() -> FontSystem {
        FontSystem::new()
    }

    fn tall_stack(rows: usize) -> VStack {
        let mut stack = VStack::new().spacing(0.0);
        for _ in 0..rows {
            stack = stack.child(Spacer::new().min_size(20.0));
        }
        stack
    }

    /// Fixed-size hit recorder covering its whole placed rect.
    struct HitBox {
        w: f32,
        h: f32,
        hits: Rc<RefCell<u32>>,
    }

    impl View for HitBox {
        fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
            (self.w, self.h)
        }

        fn place(&mut self, _fonts: &mut FontSystem, _x: f32, _y: f32, _w: f32, _h: f32) {}

        fn draw(
            &mut self,
            _scene: &mut Scene,
            _fonts: &mut FontSystem,
            _images: &mut ImageLoader<'_>,
        ) {
        }

        fn mouse_down(&mut self, _x: f64, _y: f64) {
            *self.hits.borrow_mut() += 1;
        }

        fn mouse_up(&mut self, _x: f64, _y: f64) {}

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn flex_takes_remaining_stack_space() {
        let view = ScrollView::new(Spacer::new());
        assert_eq!(view.flex(), 1.0);
    }

    #[test]
    fn measure_reports_content_size() {
        let mut view = ScrollView::new(tall_stack(5));
        let (w, h) = view.measure(&mut fonts());
        assert_eq!((w, h), (20.0, 100.0));
    }

    #[test]
    fn overflow_clamps_and_scrolls() {
        let mut view = ScrollView::new(tall_stack(10));
        let mut f = fonts();
        // 10 rows of 20: total 200 in a 100 viewport.
        view.place(&mut f, 0.0, 0.0, 200.0, 100.0);
        assert_eq!(view.total(), 200.0);
        assert!(view.scrollable());
        assert_eq!(view.max_offset(), 100.0);
        view.mouse_wheel(0.0, -40.0);
        assert_eq!(view.offset(), 40.0);
        view.mouse_wheel(0.0, -1000.0);
        assert_eq!(view.offset(), 100.0);
        view.mouse_wheel(0.0, 1000.0);
        assert_eq!(view.offset(), 0.0);
    }

    #[test]
    fn fitting_content_never_scrolls() {
        let mut view = ScrollView::new(tall_stack(2));
        let mut f = fonts();
        view.place(&mut f, 0.0, 0.0, 200.0, 100.0);
        assert!(!view.scrollable());
        view.mouse_wheel(0.0, -50.0);
        assert_eq!(view.offset(), 0.0);
    }

    #[test]
    fn press_outside_viewport_misses_child() {
        let hits: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let mut view = ScrollView::new(HitBox {
            w: 200.0,
            h: 2000.0,
            hits: hits.clone(),
        });
        let mut f = fonts();
        view.place(&mut f, 10.0, 20.0, 200.0, 100.0);
        view.mouse_down(5000.0, 5000.0);
        view.mouse_up(5000.0, 5000.0);
        assert_eq!(*hits.borrow(), 0);
    }

    #[test]
    fn press_inside_viewport_reaches_child() {
        let hits: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let mut view = ScrollView::new(HitBox {
            w: 200.0,
            h: 2000.0,
            hits: hits.clone(),
        });
        let mut f = fonts();
        view.place(&mut f, 0.0, 0.0, 200.0, 400.0);
        view.mouse_down(50.0, 50.0);
        view.mouse_up(50.0, 50.0);
        assert_eq!(*hits.borrow(), 1);
    }

    #[test]
    fn press_on_bar_skips_child() {
        let hits: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let mut view = ScrollView::new(HitBox {
            w: 200.0,
            h: 2000.0,
            hits: hits.clone(),
        });
        let mut f = fonts();
        // Tall content in a short viewport: the bar is live and the
        // hit box spans the full width underneath it.
        view.place(&mut f, 0.0, 0.0, 200.0, 400.0);
        assert!(view.scrollable());
        view.mouse_down(199.0, 200.0);
        view.mouse_up(199.0, 200.0);
        assert_eq!(*hits.borrow(), 0);
    }

    #[test]
    fn child_mut_reaches_wrapped_child() {
        let mut view = ScrollView::new(VStack::new().spacing(0.0));
        assert!(view.child_mut::<VStack>().is_some());
        assert!(view.child_mut::<Spacer>().is_none());
    }
}
