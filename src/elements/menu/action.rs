use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use vello::Scene;
use vello::kurbo::{Affine, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::{BUTTON_BG_DARK, BUTTON_BG_LIGHT};
use super::super::layout::View;
use super::menu::{
    Menu, MENU_BTN_PAD_X, MENU_BUTTON_H, MENU_BUTTON_RADIUS, MENU_CHEV_GAP, MENU_FONT_SIZE,
    MENU_ROW_H,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Chevron zone width in logical px (chevron plus padding).
pub const MENUBTN_CHEV_ZONE: f32 = 32.0;
/// Divider line inset from the button edges in logical px.
pub const MENUBTN_DIV_INSET: f32 = 6.0;

/// Menu action button: a normal button with a divider plus chevron
/// zone on the right, like the reference. Clicking the main zone
/// acts as a plain button (`on_press`); clicking the chevron zone
/// opens the anchored dropdown menu whose rows fire `on_action`.
///
/// The dropdown reuses the shared menu base in anchor mode (no
/// button of its own): same frosted panel, hover, shadow and
/// viewport clamping. While the menu is open, clicks only select or
/// dismiss. Apps must call `set_viewport` every frame and return
/// `is_open()` from `App::wants_backdrop` (see `examples/menu.rs`).
pub struct MenuButton {
    label: String,
    menu: Menu,
    clicked: Rc<Cell<Option<usize>>>,
    text_color: Color,
    text_dim: Color,
    divider: Color,
    dark: bool,
    focused: bool,
    disabled: bool,
    hovered_main: bool,
    armed_main: bool,
    armed_chev: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    on_press: Option<Box<dyn FnMut()>>,
    on_action: Option<Box<dyn FnMut(usize)>>,
}

impl MenuButton {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        let clicked = Rc::new(Cell::new(None));
        let sink = clicked.clone();
        // Anchor mode: the panel floats at the chevron zone, the
        // menu draws no button of its own.
        let menu = Menu::new("", options).on_action(move |row| sink.set(Some(row)));
        Self {
            label: label.into(),
            menu,
            clicked,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            divider: Color::from_rgba8(255, 255, 255, 40),
            dark: true,
            focused: true,
            disabled: false,
            hovered_main: false,
            armed_main: false,
            armed_chev: false,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            on_press: None,
            on_action: None,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Edit", "Delete"]`).
    pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self {
        Self::new(
            label,
            options.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Plain button press (press plus release on the main zone).
    pub fn on_press(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_press = Some(Box::new(callback));
        self
    }

    /// Dropdown row click (press plus release on the same row).
    pub fn on_action(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    /// Manual hover fill for the dropdown rows (system accent unless
    /// set by hand, like the menu base).
    pub fn hover_fill(mut self, color: Color) -> Self {
        self.menu = self.menu.hover_fill(color);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.menu = self.menu.disabled(disabled);
        self
    }

    /// Live theme: button grays plus the menu base theme.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
            self.divider = Color::from_rgba8(255, 255, 255, 40);
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
            self.divider = Color::from_rgba8(0, 0, 0, 36);
        }
        self.menu.set_theme(accent, dark);
    }

    /// Glass stage for the dropdown panel (frost tint per setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.menu.set_glass(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.menu.set_focused(focused);
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Window bounds the dropdown clamps into. Apps must call this
    /// every frame with the current viewport.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.menu.set_viewport(x, y, w, h);
    }

    pub fn is_open(&self) -> bool {
        self.menu.is_open()
    }

    pub fn close(&mut self) {
        self.menu.close();
        self.armed_main = false;
        self.armed_chev = false;
    }

    pub fn len(&self) -> usize {
        self.menu.len()
    }

    pub fn is_empty(&self) -> bool {
        self.menu.is_empty()
    }

    /// Last clicked dropdown row, if any.
    pub fn last_action(&self) -> Option<usize> {
        self.menu.last_action()
    }

    fn eff(&self, color: Color) -> Color {
        let mut color = if self.focused {
            color
        } else {
            desaturate(color)
        };
        if self.disabled {
            let c = color.to_rgba8();
            color = Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * 0.4).round() as u8);
        }
        color
    }

    fn button_bg(&self) -> Color {
        if self.dark {
            BUTTON_BG_DARK
        } else {
            BUTTON_BG_LIGHT
        }
    }

    fn chev_x(&self) -> f32 {
        self.x + self.width - MENUBTN_CHEV_ZONE
    }

    fn main_hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.chev_x() && y >= self.y && y <= self.y + MENU_BUTTON_H
    }

    fn chev_hit(&self, x: f32, y: f32) -> bool {
        x >= self.chev_x()
            && x <= self.x + self.width
            && y >= self.y
            && y <= self.y + MENU_BUTTON_H
    }

    fn open_at_chevron(&mut self) {
        let cx = self.chev_x() + MENUBTN_CHEV_ZONE / 2.0;
        self.menu.set_anchor(Some((cx, self.y + MENU_BUTTON_H)));
        self.menu.open();
    }

    /// Row clicks recorded by the base menu.
    fn adopt_click(&mut self) {
        if let Some(row) = self.clicked.take() {
            if let Some(callback) = self.on_action.as_mut() {
                callback(row);
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if self.menu.is_open() {
            self.menu.mouse_down(x, y);
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.chev_hit(x, y) {
            self.armed_chev = true;
            self.armed_main = false;
        } else if self.main_hit(x, y) {
            self.armed_main = true;
            self.armed_chev = false;
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.menu.is_open() {
            self.menu.mouse_move(x, y);
            return;
        }
        if self.disabled {
            self.hovered_main = false;
            return;
        }
        let (x, y) = (x as f32, y as f32);
        self.hovered_main = self.main_hit(x, y);
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if self.menu.is_open() {
            // Open menu: clicks only select or dismiss, never press.
            self.menu.mouse_up(x, y);
            self.adopt_click();
            self.armed_main = false;
            self.armed_chev = false;
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.armed_main && self.main_hit(x, y) {
            if let Some(callback) = self.on_press.as_mut() {
                callback();
            }
        }
        if self.armed_chev && self.chev_hit(x, y) {
            self.open_at_chevron();
        }
        self.armed_main = false;
        self.armed_chev = false;
    }

    /// Scrolling is not needed: panels cap at the window size.
    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.menu.mouse_wheel(dx, dy);
    }

    fn draw_down_chevron(&self, scene: &mut Scene, cx: f32, cy: f32, scale: f32, color: Color) {
        let px = |v: f32| v as f64 * scale as f64;
        let w = 7.0;
        let h = 4.5;
        let mut path = vello::kurbo::BezPath::new();
        path.move_to((px(cx - w / 2.0), px(cy - h / 2.0)));
        path.line_to((px(cx), px(cy + h / 2.0)));
        path.line_to((px(cx + w / 2.0), px(cy - h / 2.0)));
        let mut stroke = Stroke::new(1.5 * scale as f64);
        stroke.start_cap = vello::kurbo::Cap::Round;
        stroke.end_cap = vello::kurbo::Cap::Round;
        stroke.join = vello::kurbo::Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

impl View for MenuButton {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let layout = fonts.layout_text(&self.label, MENU_FONT_SIZE, Color::WHITE, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        (
            tw / fonts.scale + MENU_BTN_PAD_X + MENU_CHEV_GAP + MENUBTN_CHEV_ZONE,
            MENU_BUTTON_H,
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        // Anchor the dropdown panel at the chevron zone.
        self.menu.place(fonts, x, y, w, h);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;

        // Button body.
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + MENU_BUTTON_H),
            px(MENU_BUTTON_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.button_bg())),
            None,
            &body,
        );
        // Main zone hover/press, like a plain button.
        let (hover, press) = if self.dark {
            (
                Color::from_rgba8(255, 255, 255, 20),
                Color::from_rgba8(255, 255, 255, 36),
            )
        } else {
            (
                Color::from_rgba8(0, 0, 0, 15),
                Color::from_rgba8(0, 0, 0, 36),
            )
        };
        let main = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.chev_x()),
            px(self.y + MENU_BUTTON_H),
            px(MENU_BUTTON_RADIUS),
        );
        if self.armed_main && !self.disabled {
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(self.eff(press)), None, &main);
        } else if self.hovered_main && !self.disabled {
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(self.eff(hover)), None, &main);
        }

        // Label left, divider, chevron right.
        let layout = fonts.layout_text_weighted(
            &self.label,
            MENU_FONT_SIZE,
            self.eff(self.text_color),
            400.0,
            None,
        );
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            self.x + MENU_BTN_PAD_X,
            self.y + (MENU_BUTTON_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        let div = vello::kurbo::Line::new(
            (px(self.chev_x()), px(self.y + MENUBTN_DIV_INSET)),
            (px(self.chev_x()), px(self.y + MENU_BUTTON_H - MENUBTN_DIV_INSET)),
        );
        scene.stroke(
            &Stroke::new(1.0 * scale),
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.divider)),
            None,
            &div,
        );
        self.draw_down_chevron(
            scene,
            self.chev_x() + MENUBTN_CHEV_ZONE / 2.0,
            self.y + MENU_BUTTON_H / 2.0,
            fonts.scale,
            self.eff(self.text_dim),
        );

        // Anchored dropdown panel on top.
        self.menu.draw(scene, fonts, images);
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
    use std::cell::Cell;
    use std::rc::Rc;

    fn button() -> MenuButton {
        MenuButton::from_slice("Menu Label", &["Edit", "Delete"])
    }

    fn placed() -> (MenuButton, FontSystem) {
        let mut b = button();
        let mut fonts = FontSystem::new();
        b.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = b.measure(&mut fonts);
        b.place(&mut fonts, 0.0, 0.0, w, h);
        (b, fonts)
    }

    fn main_center(b: &MenuButton) -> (f64, f64) {
        ((b.x + (b.chev_x() - b.x) / 2.0) as f64, (b.y + MENU_BUTTON_H / 2.0) as f64)
    }

    fn chev_center(b: &MenuButton) -> (f64, f64) {
        ((b.chev_x() + MENUBTN_CHEV_ZONE / 2.0) as f64, (b.y + MENU_BUTTON_H / 2.0) as f64)
    }

    #[test]
    fn main_click_presses_without_menu() {
        let presses = Rc::new(Cell::new(0));
        let count = presses.clone();
        let mut b = button().on_press(move || count.set(count.get() + 1));
        let mut fonts = FontSystem::new();
        b.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = b.measure(&mut fonts);
        b.place(&mut fonts, 0.0, 0.0, w, h);
        let (mx, my) = main_center(&b);
        b.mouse_down(mx, my);
        b.mouse_up(mx, my);
        assert_eq!(presses.get(), 1);
        assert!(!b.is_open());
    }

    #[test]
    fn chevron_click_opens_and_row_fires() {
        let actions = Rc::new(Cell::new(99usize));
        let fired = actions.clone();
        let mut b = button().on_action(move |i| fired.set(i));
        let mut fonts = FontSystem::new();
        b.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = b.measure(&mut fonts);
        b.place(&mut fonts, 0.0, 0.0, w, h);
        // Main press, chevron release: neither fires.
        let (mx, my) = main_center(&b);
        let (cx, cy) = chev_center(&b);
        b.mouse_down(mx, my);
        b.mouse_up(cx, cy);
        assert!(!b.is_open());
        // Chevron click opens at the chevron zone.
        b.mouse_down(cx, cy);
        b.mouse_up(cx, cy);
        assert!(b.is_open());
        // Next redraw anchors the panel at the chevron zone.
        let (w, h) = b.measure(&mut fonts);
        b.place(&mut fonts, 0.0, 0.0, w, h);
        let (x, y, w, h) = b.menu.menu_rect();
        assert!(x <= cx as f32 && (cx as f32) <= x + w);
        assert!(y >= 0.0 && x >= 0.0 && x + w <= 800.0 && y + h <= 600.0);
        // Row click fires the action and closes.
        let (rx, ry, rw, rh) = b.menu.row_rect(1).expect("row exists");
        let rx = (rx + rw / 2.0) as f64;
        let ry = (ry + rh / 2.0) as f64;
        b.mouse_down(rx, ry);
        b.mouse_up(rx, ry);
        assert_eq!(actions.get(), 1);
        assert_eq!(b.last_action(), Some(1));
        assert!(!b.is_open());
    }

    #[test]
    fn main_click_while_open_only_closes() {
        let presses = Rc::new(Cell::new(0));
        let count = presses.clone();
        let (mut b, _) = placed();
        b.on_press = Some(Box::new(move || count.set(count.get() + 1)));
        let (cx, cy) = chev_center(&b);
        b.mouse_down(cx, cy);
        b.mouse_up(cx, cy);
        assert!(b.is_open());
        let (mx, my) = main_center(&b);
        b.mouse_down(mx, my);
        b.mouse_up(mx, my);
        assert_eq!(presses.get(), 0);
        assert!(!b.is_open());
    }

    #[test]
    fn zones_split_button_width() {
        let (b, _) = placed();
        assert!(b.chev_x() > b.x);
        assert_eq!(b.chev_x(), b.x + b.width - MENUBTN_CHEV_ZONE);
    }
}
