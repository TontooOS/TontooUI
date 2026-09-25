use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::menu::Menu;
use super::nested::NestedMenu;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::TouchPhase;
use crate::theme::{GlassAmount, ThemeMode};

/// Long-press hold time in seconds before the menu opens.
pub const CONTEXT_LONG_PRESS_SECONDS: f64 = 0.6;
/// Pointer travel in logical px cancelling a long-press.
pub const CONTEXT_LONG_PRESS_MOVE: f32 = 10.0;

/// Menu behind a context area: plain or nested dropdown.
pub enum ContextKind {
    Basic(Menu),
    Nested(NestedMenu),
}

/// Context menu: opens any menu at the pointer on right-click or
/// touch long-press inside a fixed area. The menu floats anchored at
/// the press point (no button) and clamps into the viewport, so the
/// glass never samples outside the window. Apps forward
/// `context_click`/`touch` plus the usual mouse methods and return
/// `is_open()` from `App::wants_backdrop` (see `examples/menu.rs`).
pub struct ContextMenu {
    area: (f32, f32, f32, f32),
    kind: ContextKind,
    touch: Option<(Instant, f32, f32)>,
}

impl ContextMenu {
    pub fn new(area: (f32, f32, f32, f32), kind: ContextKind) -> Self {
        Self {
            area,
            kind,
            touch: None,
        }
    }

    pub fn basic(area: (f32, f32, f32, f32), menu: Menu) -> Self {
        Self::new(area, ContextKind::Basic(menu))
    }

    pub fn nested(area: (f32, f32, f32, f32), menu: NestedMenu) -> Self {
        Self::new(area, ContextKind::Nested(menu))
    }

    /// Activation area (x, y, width, height) in logical px.
    pub fn set_area(&mut self, area: (f32, f32, f32, f32)) {
        self.area = area;
    }

    pub fn area(&self) -> (f32, f32, f32, f32) {
        self.area
    }

    pub fn is_open(&self) -> bool {
        match &self.kind {
            ContextKind::Basic(menu) => menu.is_open(),
            ContextKind::Nested(menu) => menu.is_open(),
        }
    }

    pub fn close(&mut self) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.close(),
            ContextKind::Nested(menu) => menu.close(),
        }
        self.touch = None;
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.set_viewport(x, y, w, h),
            ContextKind::Nested(menu) => menu.set_viewport(x, y, w, h),
        }
    }

    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.set_theme(accent, dark),
            ContextKind::Nested(menu) => menu.set_theme(accent, dark),
        }
    }

    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.set_glass(mode, amount),
            ContextKind::Nested(menu) => menu.set_glass(mode, amount),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.set_focused(focused),
            ContextKind::Nested(menu) => menu.set_focused(focused),
        }
    }

    fn in_area(&self, x: f32, y: f32) -> bool {
        let (ax, ay, aw, ah) = self.area;
        x >= ax && x <= ax + aw && y >= ay && y <= ay + ah
    }

    fn open_at(&mut self, x: f32, y: f32) {
        match &mut self.kind {
            ContextKind::Basic(menu) => {
                menu.set_anchor(Some((x, y)));
                menu.open();
            }
            ContextKind::Nested(menu) => {
                menu.set_anchor(Some((x, y)));
                menu.open();
            }
        }
    }

    /// Right-click press: opens the anchored menu inside the area.
    pub fn context_click(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        if self.in_area(x, y) {
            self.touch = None;
            self.open_at(x, y);
        }
    }

    /// Touch long-press: hold inside the area without moving opens
    /// the anchored menu on release.
    pub fn touch(&mut self, phase: TouchPhase, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        match phase {
            TouchPhase::Started => {
                if self.in_area(x, y) && !self.is_open() {
                    self.touch = Some((Instant::now(), x, y));
                }
            }
            TouchPhase::Moved => {
                if let Some((_, sx, sy)) = self.touch {
                    if (x - sx).abs() > CONTEXT_LONG_PRESS_MOVE
                        || (y - sy).abs() > CONTEXT_LONG_PRESS_MOVE
                    {
                        self.touch = None;
                    }
                }
            }
            TouchPhase::Ended => {
                if let Some((start, sx, sy)) = self.touch.take() {
                    if start.elapsed().as_secs_f64() >= CONTEXT_LONG_PRESS_SECONDS
                        && self.in_area(x, y)
                    {
                        let _ = (sx, sy);
                        self.open_at(x, y);
                    }
                }
            }
            TouchPhase::Cancelled => {
                self.touch = None;
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.mouse_down(x, y),
            ContextKind::Nested(menu) => menu.mouse_down(x, y),
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.mouse_move(x, y),
            ContextKind::Nested(menu) => menu.mouse_move(x, y),
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.mouse_up(x, y),
            ContextKind::Nested(menu) => menu.mouse_up(x, y),
        }
    }

    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.mouse_wheel(dx, dy),
            ContextKind::Nested(menu) => menu.mouse_wheel(dx, dy),
        }
    }
}

impl View for ContextMenu {
    /// Overlay element: floats above content, takes no layout space.
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.place(fonts, x, y, w, h),
            ContextKind::Nested(menu) => menu.place(fonts, x, y, w, h),
        }
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        match &mut self.kind {
            ContextKind::Basic(menu) => menu.draw(scene, fonts, images),
            ContextKind::Nested(menu) => menu.draw(scene, fonts, images),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn basic() -> ContextMenu {
        ContextMenu::basic(
            (10.0, 10.0, 200.0, 150.0),
            Menu::from_slice("Edit", &["Cut", "Copy", "Paste"]),
        )
    }

    fn pump_viewport(ctx: &mut ContextMenu) {
        let mut fonts = FontSystem::new();
        ctx.set_viewport(0.0, 0.0, 800.0, 600.0);
        ctx.place(&mut fonts, 0.0, 0.0, 0.0, 0.0);
    }

    #[test]
    fn right_click_inside_opens_at_point() {
        let mut ctx = basic();
        pump_viewport(&mut ctx);
        // Outside the area: nothing.
        ctx.context_click(500.0, 500.0);
        assert!(!ctx.is_open());
        // Inside: anchored menu opens near the point.
        ctx.context_click(100.0, 100.0);
        assert!(ctx.is_open());
        pump_viewport(&mut ctx);
        let (x, y, w, h) = match &ctx.kind {
            ContextKind::Basic(menu) => menu.menu_rect(),
            ContextKind::Nested(_) => unreachable!(),
        };
        assert!(x <= 100.0 && 100.0 <= x + w);
        assert!(y >= 100.0 || y + h <= 100.0);
        assert!(x >= 0.0 && y >= 0.0 && x + w <= 800.0 && y + h <= 600.0);
    }

    #[test]
    fn long_press_opens_on_release() {
        let mut ctx = basic();
        pump_viewport(&mut ctx);
        // Hold still for over a second, release inside.
        ctx.touch(TouchPhase::Started, 50.0, 50.0);
        ctx.touch = Some((Instant::now() - Duration::from_secs_f64(1.2), 50.0, 50.0));
        ctx.touch(TouchPhase::Ended, 52.0, 51.0);
        assert!(ctx.is_open());
    }

    #[test]
    fn quick_tap_and_move_cancel() {
        let mut ctx = basic();
        pump_viewport(&mut ctx);
        // Quick tap: no menu.
        ctx.touch(TouchPhase::Started, 50.0, 50.0);
        ctx.touch(TouchPhase::Ended, 50.0, 50.0);
        assert!(!ctx.is_open());
        // Moving far cancels the hold.
        ctx.touch(TouchPhase::Started, 50.0, 50.0);
        ctx.touch(TouchPhase::Moved, 200.0, 200.0);
        ctx.touch = Some((Instant::now() - Duration::from_secs_f64(1.2), 50.0, 50.0));
        ctx.touch(TouchPhase::Ended, 200.0, 200.0);
        assert!(!ctx.is_open());
    }

    #[test]
    fn nested_kind_opens_too() {
        use super::super::nested::{MenuItem, NestedMenu};
        let mut ctx = ContextMenu::nested(
            (10.0, 10.0, 200.0, 150.0),
            NestedMenu::new(
                "Share",
                vec![
                    MenuItem::action("Mail"),
                    MenuItem::submenu("More", vec![MenuItem::action("SMS")]),
                ],
            ),
        );
        pump_viewport(&mut ctx);
        ctx.context_click(100.0, 100.0);
        assert!(ctx.is_open());
        ctx.close();
        assert!(!ctx.is_open());
    }

    #[test]
    fn overlay_takes_no_layout_space() {
        let mut ctx = basic();
        let mut fonts = FontSystem::new();
        assert_eq!(ctx.measure(&mut fonts), (0.0, 0.0));
    }
}
