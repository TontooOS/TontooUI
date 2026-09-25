use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use vello::Scene;

use super::super::layout::View;
use super::super::menu::{Menu, MenuChevron};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{GlassAmount, ThemeMode};
use vello::peniko::Color;

// Geometry tokens live on the shared menu base; re-exported here so
// existing `pickers::MENU_*` paths keep working.
pub use super::super::menu::{
    MENU_ACCENT, MENU_BUTTON_H, MENU_BUTTON_RADIUS, MENU_BTN_PAD_X, MENU_CHECK_COL,
    MENU_CHECK_H, MENU_CHECK_W, MENU_CHEV_GAP, MENU_CHEV_W, MENU_FONT_SIZE, MENU_GAP,
    MENU_PAD, MENU_PANEL_GAP, MENU_RADIUS, MENU_ROW_H, MENU_ROW_SPACING, MENU_SHADOW_BLUR,
    MENU_TEXT_GAP,
};

/// Menu picker (SwiftUI `Picker` with `.menu` style): an optional
/// leading label plus a pop-up button showing the current value. A
/// click opens the frosted glass menu; hovering a row tints it with
/// the accent color and clicking it selects it.
///
/// Same function as before, now running on the shared menu base
/// (`menu::Menu`): the base owns button, frosted panel, rows, hover
/// and viewport clamping, while the picker only tracks the selection
/// (button text plus checkmark) and fires `on_select` on change. The
/// closed button has no hover state, like macOS. Apps must call
/// `set_viewport` every frame (see `examples/menu.rs`) and return
/// `is_open()` from `App::wants_backdrop` so the shell runs the blur
/// pass.
pub struct MenuPicker {
    menu: Menu,
    selected: usize,
    clicked: Rc<Cell<Option<usize>>>,
    accent: Color,
    accent_manual: bool,
    on_select: Option<Box<dyn FnMut(usize)>>,
}

impl MenuPicker {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        let clicked = Rc::new(Cell::new(None));
        let sink = clicked.clone();
        let selected = 0;
        let button = options.first().cloned().unwrap_or_default();
        let menu = Menu::new(button, options)
            .label(label)
            .chevron(MenuChevron::Both)
            .checked(Some(selected))
            .on_action(move |row| sink.set(Some(row)));
        Self {
            menu,
            selected,
            clicked,
            accent: MENU_ACCENT,
            accent_manual: false,
            on_select: None,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Red", "Green"]`).
    pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self {
        Self::new(
            label,
            options.iter().map(|s| s.to_string()).collect(),
        )
    }

    fn clamp_index(&self, index: usize) -> usize {
        if self.menu.is_empty() {
            0
        } else {
            index.min(self.menu.len() - 1)
        }
    }

    fn sync_base(&mut self) {
        let label = self.menu.option(self.selected).unwrap_or("").to_string();
        self.menu.set_button(label);
        self.menu.set_checked(Some(self.selected));
    }

    /// Initial selection without firing `on_select`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = self.clamp_index(index);
        self.sync_base();
        self
    }

    /// Manual hover fill: wins over the system accent until cleared.
    /// The row hover follows the system accent unless set by hand.
    pub fn hover_fill(mut self, color: Color) -> Self {
        self.menu = self.menu.hover_fill(color);
        self
    }

    /// Manual check/dot accent: wins over the system accent until
    /// cleared. Kept for API parity with the other pickers.
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = color;
        self.accent_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.menu = self.menu.disabled(disabled);
        self
    }

    pub fn on_select(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Live theme: hover fill plus mode grays and label colors. A
    /// manually set hover/accent wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.accent_manual {
            self.accent = accent;
        }
        self.menu.set_theme(accent, dark);
    }

    /// Glass stage for the menu panel (frost tint per setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.menu.set_glass(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.menu.set_focused(focused);
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.menu.set_label(label);
    }

    /// Window bounds the menu is clamped into. Apps must call this
    /// every frame with the current viewport; without it the panel
    /// cannot be kept inside the window.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.menu.set_viewport(x, y, w, h);
    }

    pub fn is_open(&self) -> bool {
        self.menu.is_open()
    }

    pub fn open(&mut self) {
        self.menu.open();
    }

    pub fn close(&mut self) {
        self.menu.close();
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.menu.option(self.selected)
    }

    pub fn len(&self) -> usize {
        self.menu.len()
    }

    pub fn is_empty(&self) -> bool {
        self.menu.is_empty()
    }

    /// Select and close. Fires `on_select` when the selection changed.
    pub fn select(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.sync_base();
            self.notify();
        }
        self.menu.close();
    }

    /// Set the selection without opening. Fires `on_select` when the
    /// selection changed.
    pub fn set_selected(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.sync_base();
            self.notify();
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected);
        }
    }

    /// Row clicks recorded by the base: adopt on change, ignore
    /// re-clicks of the current value.
    fn adopt_click(&mut self) {
        if let Some(row) = self.clicked.take() {
            let clamped = self.clamp_index(row);
            if clamped != self.selected {
                self.selected = clamped;
                self.sync_base();
                self.notify();
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        self.menu.mouse_down(x, y);
    }

    /// Hover only exists inside the open menu; the closed button
    /// never highlights, like macOS.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.menu.mouse_move(x, y);
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.menu.mouse_up(x, y);
        self.adopt_click();
    }
}

impl View for MenuPicker {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.menu.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.menu.place(fonts, x, y, w, h);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
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
    use super::super::super::menu::MENU_BUTTON_H as BTN_H;
    use super::super::super::menu::MENU_ROW_H as ROW_H;

    fn picker() -> MenuPicker {
        MenuPicker::from_slice("Color", &["Red", "Green", "Blue", "Yellow", "Purple"])
    }

    fn placed() -> (MenuPicker, FontSystem) {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        (p, fonts)
    }

    fn button_center(p: &MenuPicker) -> (f64, f64) {
        let (x, y, w, _) = p.menu.button_rect();
        ((x + w / 2.0) as f64, (y + BTN_H / 2.0) as f64)
    }

    fn row_center(p: &MenuPicker, row: usize) -> (f64, f64) {
        let (x, y, w, h) = p.menu.row_rect(row).expect("row exists");
        ((x + w / 2.0) as f64, (y + h / 2.0) as f64)
    }

    #[test]
    fn defaults_to_first_option_closed() {
        let p = picker();
        assert_eq!(p.selected_index(), 0);
        assert_eq!(p.selected_label(), Some("Red"));
        assert!(!p.is_open());
    }

    #[test]
    fn click_button_opens_and_row_click_selects() {
        let (mut p, _) = placed();
        assert_eq!(p.len(), 5);
        assert!(!p.is_empty());
        // Release without press does nothing.
        p.mouse_up(1000.0, 1000.0);
        assert!(!p.is_open());
        // Click on the button opens the menu.
        let (bx, by) = button_center(&p);
        p.mouse_down(bx, by);
        p.mouse_up(bx, by);
        assert!(p.is_open());
        // Moving over a row highlights it in the base.
        let (rx, ry) = row_center(&p, 2);
        p.mouse_move(rx, ry);
        // Press + release on the row selects and closes.
        p.mouse_down(rx, ry);
        p.mouse_up(rx, ry);
        assert_eq!(p.selected_index(), 2);
        assert_eq!(p.selected_label(), Some("Blue"));
        assert!(!p.is_open());
        // Button text follows the selection.
        assert_eq!(p.menu.button_text(), "Blue");
    }

    #[test]
    fn reclick_same_value_fires_nothing() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let (mut p, _) = placed();
        p.on_select = Some(Box::new(move |_| count.set(count.get() + 1)));
        let (bx, by) = button_center(&p);
        p.mouse_down(bx, by);
        p.mouse_up(bx, by);
        let (rx, ry) = row_center(&p, 0);
        p.mouse_down(rx, ry);
        p.mouse_up(rx, ry);
        assert_eq!(fires.get(), 0);
        assert!(!p.is_open());
    }

    #[test]
    fn closed_button_has_no_hover() {
        let (mut p, _) = placed();
        let (bx, by) = button_center(&p);
        p.mouse_move(bx, by);
        assert_eq!(p.menu.hovered, None);
        // Rows highlight once open.
        p.open();
        let (rx, ry) = row_center(&p, 1);
        p.mouse_move(rx, ry);
        assert_eq!(p.menu.hovered, Some(1));
    }

    #[test]
    fn outside_click_closes_without_selecting() {
        let (mut p, _) = placed();
        let (bx, by) = button_center(&p);
        p.mouse_down(bx, by);
        p.mouse_up(bx, by);
        assert!(p.is_open());
        // Press inside a row, release far outside: no change.
        let (rx, ry) = row_center(&p, 3);
        p.mouse_down(rx, ry);
        p.mouse_up(700.0, 500.0);
        assert!(!p.is_open());
        assert_eq!(p.selected_index(), 0);
    }

    #[test]
    fn menu_stays_inside_viewport() {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        // Tiny window with the button at the bottom-right corner.
        p.set_viewport(0.0, 0.0, 200.0, 120.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 150.0, 90.0, w, h);
        let (x, y, mw, mh) = p.menu.menu_rect();
        assert!(x >= 0.0 && y >= 0.0 && x + mw <= 200.0 && y + mh <= 120.0);
    }

    #[test]
    fn select_fires_callback_only_on_change() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut p = picker().on_select(move |_| count.set(count.get() + 1));
        p.select(0);
        assert_eq!(fires.get(), 0);
        p.select(4);
        assert_eq!(p.selected_index(), 4);
        assert_eq!(p.selected_label(), Some("Purple"));
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn set_selected_is_immediate_and_clamped() {
        let mut p = picker();
        p.set_selected(99);
        assert_eq!(p.selected_index(), 4);
        assert!(!p.is_open());
        assert_eq!(p.menu.button_text(), "Purple");
    }

    #[test]
    fn manual_hover_wins_over_theme_accent() {
        let mut p = MenuPicker::from_slice("Color", &["Red", "Green"])
            .hover_fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        p.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(p.menu.hover, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = MenuPicker::from_slice("Color", &["Red", "Green"]);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.menu.hover, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn empty_options_never_open() {
        let mut p = MenuPicker::new("Color", Vec::new());
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w.max(1.0), h.max(1.0));
        p.mouse_down(2.0, 2.0);
        p.mouse_up(2.0, 2.0);
        assert_eq!(p.selected_label(), None);
    }

    #[test]
    fn row_geometry_matches_base() {
        let (p, _) = placed();
        let (x, y, w, h) = p.menu.row_rect(2).expect("row exists");
        let (mx, my, mw, _) = p.menu.menu_rect();
        assert_eq!((w, h), (mw, ROW_H));
        assert!(x >= mx && y >= my);
        assert!(p.menu.row_rect(99).is_none());
    }
}
