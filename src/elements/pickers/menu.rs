use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::{BUTTON_BG_DARK, BUTTON_BG_LIGHT};
use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Menu button height in logical px.
pub const MENU_BUTTON_H: f32 = 24.0;
/// Menu button corner radius in logical px.
pub const MENU_BUTTON_RADIUS: f32 = 6.0;
/// Button, row and leading label size in logical px.
pub const MENU_FONT_SIZE: f32 = 13.0;
/// Gap between the leading label and the button in logical px.
pub const MENU_GAP: f32 = 9.0;
/// Horizontal text padding inside the button in logical px.
pub const MENU_BTN_PAD_X: f32 = 10.0;
/// Chevron box width inside the button in logical px.
pub const MENU_CHEV_W: f32 = 12.0;
/// Gap between button text and chevron in logical px.
pub const MENU_CHEV_GAP: f32 = 8.0;
/// Menu panel padding on every side in logical px.
pub const MENU_PAD: f32 = 6.0;
/// Menu row height in logical px.
pub const MENU_ROW_H: f32 = 26.0;
/// Vertical gap between menu rows in logical px.
pub const MENU_ROW_SPACING: f32 = 2.0;
/// Menu panel corner radius in logical px (slightly rounded only).
pub const MENU_RADIUS: f32 = 4.0;
/// Column reserved for the checkmark in logical px.
pub const MENU_CHECK_COL: f32 = 20.0;
/// Gap between checkmark column and row text in logical px.
pub const MENU_TEXT_GAP: f32 = 6.0;
/// Gap between button and menu panel in logical px.
pub const MENU_PANEL_GAP: f32 = 4.0;
/// Checkmark glyph width in logical px (fixed, independent of size).
pub const MENU_CHECK_W: f32 = 10.0;
/// Checkmark glyph height in logical px (fixed, independent of size).
pub const MENU_CHECK_H: f32 = 7.5;
/// Menu edge shadow blur in logical px (heavy, like the reference).
pub const MENU_SHADOW_BLUR: f32 = 24.0;
/// Default selected fill (theme accent blue).
pub const MENU_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Menu picker (SwiftUI `Picker` with `.menu` style): an optional
/// leading label plus a pop-up button. A click opens a frosted glass
/// menu under (or above) the button; hovering a row tints it with the
/// accent color and clicking it selects it.
///
/// The closed button has no hover state, like macOS. The menu panel
/// uses the `Frosted` glass finish (no clear background) with a heavy
/// edge shadow, and it is always clamped into the viewport passed via
/// `set_viewport`: glass samples the in-app backdrop, so the panel
/// must never leave the window. Apps must call `set_viewport` every
/// frame (see `examples/menu.rs`) and return `is_open()` from
/// `App::wants_backdrop` so the shell runs the blur pass.
pub struct MenuPicker {
    label: String,
    options: Vec<String>,
    selected: usize,
    open: bool,
    accent: Color,
    accent_manual: bool,
    hover: Color,
    hover_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
    glass: GlassContainer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    btn_x: f32,
    btn_y: f32,
    btn_w: f32,
    label_x: f32,
    label_y: f32,
    menu_x: f32,
    menu_y: f32,
    menu_w: f32,
    menu_h: f32,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    armed_button: bool,
    armed_row: Option<usize>,
    hovered: Option<usize>,
    disabled: bool,
    focused: bool,
    on_select: Option<Box<dyn FnMut(usize)>>,
}

impl MenuPicker {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        let mut glass = GlassContainer::new();
        glass.set_glass_type(GlassType::Frosted);
        Self {
            label: label.into(),
            options,
            selected: 0,
            open: false,
            accent: MENU_ACCENT,
            accent_manual: false,
            hover: MENU_ACCENT,
            hover_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            glass,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            btn_x: 0.0,
            btn_y: 0.0,
            btn_w: 0.0,
            label_x: 0.0,
            label_y: 0.0,
            menu_x: 0.0,
            menu_y: 0.0,
            menu_w: 0.0,
            menu_h: 0.0,
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: f32::MAX,
            vp_h: f32::MAX,
            armed_button: false,
            armed_row: None,
            hovered: None,
            disabled: false,
            focused: true,
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

    /// Initial selection without firing `on_select`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = self.clamp_index(index);
        self
    }

    /// Manual hover fill: wins over the system accent until cleared.
    /// The row hover follows the system accent unless set by hand.
    pub fn hover_fill(mut self, color: Color) -> Self {
        self.hover = color;
        self.hover_manual = true;
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
        self.disabled = disabled;
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
        if !self.hover_manual {
            self.hover = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
        }
    }

    /// Glass stage for the menu panel (frost tint per setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Window bounds the menu is clamped into. Apps must call this
    /// every frame with the current viewport; without it the panel
    /// cannot be kept inside the window.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vp_x = x;
        self.vp_y = y;
        self.vp_w = w.max(0.0);
        self.vp_h = h.max(0.0);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        if !self.disabled {
            self.open = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.armed_button = false;
        self.armed_row = None;
        self.hovered = None;
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    fn clamp_index(&self, index: usize) -> usize {
        if self.options.is_empty() {
            0
        } else {
            index.min(self.options.len() - 1)
        }
    }

    /// Select and close. Fires `on_select` when the selection changed.
    pub fn select(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.notify();
        }
        self.close();
    }

    /// Set the selection without opening. Fires `on_select` when the
    /// selection changed.
    pub fn set_selected(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.notify();
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected);
        }
    }

    fn button_bg(&self) -> Color {
        if self.dark {
            BUTTON_BG_DARK
        } else {
            BUTTON_BG_LIGHT
        }
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

    fn label_size(&self, fonts: &mut FontSystem) -> (f32, f32) {
        if self.label.is_empty() {
            return (0.0, 0.0);
        }
        let layout = fonts.layout_text(&self.label, MENU_FONT_SIZE, Color::WHITE, None);
        let (tw, th) = FontSystem::layout_size(&layout);
        (tw / fonts.scale, th / fonts.scale)
    }

    fn text_w(&self, fonts: &mut FontSystem, text: &str) -> f32 {
        let layout = fonts.layout_text(text, MENU_FONT_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn button_w(&self, fonts: &mut FontSystem) -> f32 {
        let mut widest: f32 = 0.0;
        for option in &self.options {
            widest = widest.max(self.text_w(fonts, option));
        }
        // Wide enough for the longest option: opening the menu must
        // not resize the button.
        widest + MENU_BTN_PAD_X * 2.0 + MENU_CHEV_GAP + MENU_CHEV_W
    }

    fn row_stride(&self) -> f32 {
        MENU_ROW_H + MENU_ROW_SPACING
    }

    fn row_y(&self, row: usize) -> f32 {
        self.menu_y + MENU_PAD + row as f32 * self.row_stride()
    }

    /// Menu panel rect, clamped into the viewport so the glass never
    /// samples outside the window. Prefers below the button, falls
    /// back above it, then clamps.
    fn layout_menu(&mut self, fonts: &mut FontSystem) {
        if self.options.is_empty() {
            self.menu_w = 0.0;
            self.menu_h = 0.0;
            return;
        }
        let mut content_w: f32 = 0.0;
        for option in &self.options {
            content_w = content_w.max(MENU_CHECK_COL + MENU_TEXT_GAP + self.text_w(fonts, option));
        }
        let mut w = (content_w + MENU_PAD * 2.0).max(self.btn_w);
        let mut h = MENU_PAD * 2.0
            + self.options.len() as f32 * MENU_ROW_H
            + (self.options.len() as f32 - 1.0) * MENU_ROW_SPACING;
        // Never larger than the window itself.
        w = w.min(self.vp_w).max(0.0);
        h = h.min(self.vp_h).max(0.0);
        let mut x = self.btn_x;
        if x + w > self.vp_x + self.vp_w {
            x = self.vp_x + self.vp_w - w;
        }
        if x < self.vp_x {
            x = self.vp_x;
        }
        let below = self.btn_y + MENU_BUTTON_H + MENU_PANEL_GAP;
        let mut y = below;
        if y + h > self.vp_y + self.vp_h {
            y = self.btn_y - MENU_PANEL_GAP - h;
        }
        if y < self.vp_y {
            y = self.vp_y;
        }
        self.menu_x = x;
        self.menu_y = y;
        self.menu_w = w;
        self.menu_h = h;
        self.glass.set_bounds(x, y, w, h);
    }

    fn button_hit(&self, x: f32, y: f32) -> bool {
        x >= self.btn_x
            && x <= self.btn_x + self.btn_w
            && y >= self.btn_y
            && y <= self.btn_y + MENU_BUTTON_H
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.open || self.options.is_empty() {
            return None;
        }
        if x < self.menu_x
            || x > self.menu_x + self.menu_w
            || y < self.menu_y
            || y > self.menu_y + self.menu_h
        {
            return None;
        }
        let rel = y - (self.menu_y + MENU_PAD);
        if rel < 0.0 {
            return None;
        }
        let row = (rel / self.row_stride()).floor() as usize;
        if row >= self.options.len() {
            return None;
        }
        // Ignore hits in padding and row gaps.
        let within = rel - row as f32 * self.row_stride();
        if within > MENU_ROW_H {
            return None;
        }
        Some(row)
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if !self.open {
            if self.button_hit(x, y) {
                self.armed_button = true;
            }
            return;
        }
        // Open menu: arm the row under the pointer, if any. A press
        // anywhere else closes the menu on release.
        self.armed_button = false;
        self.armed_row = self.row_at(x, y);
    }

    /// Hover only exists inside the open menu; the closed button
    /// never highlights, like macOS.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.open {
            self.hovered = self.row_at(x as f32, y as f32);
        } else {
            self.hovered = None;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if !self.open {
            // Click (press + release) on the button opens the menu.
            if self.armed_button && self.button_hit(x, y) {
                self.open();
            }
            self.armed_button = false;
            return;
        }
        let armed = self.armed_row.take();
        self.armed_button = false;
        if let Some(row) = armed {
            // Press + release on the same row selects and closes.
            if self.row_at(x, y) == Some(row) {
                self.select(row);
                return;
            }
        }
        // Any other release (button toggle, outside click) closes
        // without changing the selection.
        self.close();
    }

    fn draw_check(&self, scene: &mut Scene, x: f32, y: f32, scale: f32, color: Color) {
        // Fixed-size checkmark, independent of the menu size.
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        path.move_to((px(x), px(y + MENU_CHECK_H * 0.55)));
        path.line_to((px(x + MENU_CHECK_W * 0.38), px(y + MENU_CHECK_H)));
        path.line_to((px(x + MENU_CHECK_W), px(y)));
        let mut stroke = Stroke::new(2.0 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &path,
        );
    }

    fn draw_chevron(&self, scene: &mut Scene, x: f32, cy: f32, scale: f32, color: Color) {
        // Up/down chevron pair, like the macOS pop-up button.
        let px = |v: f32| v as f64 * scale as f64;
        let w = 7.0;
        let h = 4.0;
        let gap = 2.0;
        let cx = x + MENU_CHEV_W / 2.0;
        let mut up = BezPath::new();
        up.move_to((px(cx - w / 2.0), px(cy - gap / 2.0)));
        up.line_to((px(cx), px(cy - gap / 2.0 - h)));
        up.line_to((px(cx + w / 2.0), px(cy - gap / 2.0)));
        let mut down = BezPath::new();
        down.move_to((px(cx - w / 2.0), px(cy + gap / 2.0)));
        down.line_to((px(cx), px(cy + gap / 2.0 + h)));
        down.line_to((px(cx + w / 2.0), px(cy + gap / 2.0)));
        let mut stroke = Stroke::new(1.5 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        for path in [&up, &down] {
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &Brush::Solid(color),
                None,
                path,
            );
        }
    }
}

impl View for MenuPicker {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (label_w, label_h) = self.label_size(fonts);
        let label_part = if label_w > 0.0 {
            label_w + MENU_GAP
        } else {
            0.0
        };
        (
            label_part + self.button_w(fonts),
            MENU_BUTTON_H.max(label_h),
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let (label_w, label_h) = self.label_size(fonts);
        self.label_x = x;
        self.label_y = y + (h - label_h) / 2.0;
        self.btn_x = if label_w > 0.0 { x + label_w + MENU_GAP } else { x };
        self.btn_y = y + (h - MENU_BUTTON_H) / 2.0;
        self.btn_w = self.button_w(fonts).min((w - (self.btn_x - x)).max(0.0));
        self.layout_menu(fonts);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        // Capture pass: the glass skips itself so the blur sees only
        // what sits behind the menu; rows must skip too.
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        if self.options.is_empty() {
            return;
        }
        // Viewport may change between frames: re-clamp every draw.
        self.layout_menu(fonts);

        if !self.label.is_empty() {
            let layout = fonts.layout_text_weighted(
                &self.label,
                MENU_FONT_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            draw_layout(scene, &layout, self.label_x, self.label_y, fonts.scale);
        }

        // Pop-up button (no hover state).
        let button = RoundedRect::new(
            px(self.btn_x),
            px(self.btn_y),
            px(self.btn_x + self.btn_w),
            px(self.btn_y + MENU_BUTTON_H),
            px(MENU_BUTTON_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.button_bg())),
            None,
            &button,
        );
        if self.armed_button && !self.disabled {
            let press = if self.dark {
                Color::from_rgba8(255, 255, 255, 24)
            } else {
                Color::from_rgba8(0, 0, 0, 24)
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(press)),
                None,
                &button,
            );
        }
        let current = self.selected_label().unwrap_or("");
        let layout = fonts.layout_text_weighted(
            current,
            MENU_FONT_SIZE,
            self.eff(self.text_color),
            400.0,
            None,
        );
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            self.btn_x + MENU_BTN_PAD_X,
            self.btn_y + (MENU_BUTTON_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        self.draw_chevron(
            scene,
            self.btn_x + self.btn_w - MENU_BTN_PAD_X - MENU_CHEV_W,
            self.btn_y + MENU_BUTTON_H / 2.0,
            fonts.scale,
            self.eff(self.text_dim),
        );

        if !self.open {
            return;
        }

        // Heavy edge shadow, then the frosted glass panel (opaque
        // finish, no clear background).
        let rect = vello::kurbo::Rect::new(
            px(self.menu_x),
            px(self.menu_y),
            px(self.menu_x + self.menu_w),
            px(self.menu_y + self.menu_h),
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 70),
            px(MENU_RADIUS),
            MENU_SHADOW_BLUR as f64 * scale,
        );
        self.glass.draw(scene, fonts, images);

        // Rows on top of the glass.
        for (i, option) in self.options.clone().iter().enumerate() {
            let ry = self.row_y(i);
            let is_hovered = Some(i) == self.hovered && !self.disabled;
            if is_hovered {
                let hl = RoundedRect::new(
                    px(self.menu_x + MENU_PAD / 2.0),
                    px(ry),
                    px(self.menu_x + self.menu_w - MENU_PAD / 2.0),
                    px(ry + MENU_ROW_H),
                    px(MENU_BUTTON_RADIUS),
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.hover)),
                    None,
                    &hl,
                );
            }
            let color = if is_hovered {
                Color::WHITE
            } else {
                self.text_color
            };
            if i == self.selected {
                self.draw_check(
                    scene,
                    self.menu_x + MENU_PAD + (MENU_CHECK_COL - MENU_CHECK_W) / 2.0,
                    ry + (MENU_ROW_H - MENU_CHECK_H) / 2.0,
                    fonts.scale,
                    self.eff(color),
                );
            }
            let layout = fonts.layout_text_weighted(
                option,
                MENU_FONT_SIZE,
                self.eff(color),
                400.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.menu_x + MENU_PAD + MENU_CHECK_COL + MENU_TEXT_GAP,
                ry + (MENU_ROW_H - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let bx = (p.btn_x + p.btn_w / 2.0) as f64;
        let by = (p.btn_y + MENU_BUTTON_H / 2.0) as f64;
        p.mouse_down(bx, by);
        p.mouse_up(bx, by);
        assert!(p.is_open());
        // Hovering the closed button is impossible now, but moving
        // over a row highlights it.
        let ry = (p.row_y(2) + MENU_ROW_H / 2.0) as f64;
        let rx = (p.menu_x + p.menu_w / 2.0) as f64;
        p.mouse_move(rx, ry);
        assert_eq!(p.hovered, Some(2));
        // Press + release on the row selects and closes.
        p.mouse_down(rx, ry);
        p.mouse_up(rx, ry);
        assert_eq!(p.selected_index(), 2);
        assert_eq!(p.selected_label(), Some("Blue"));
        assert!(!p.is_open());
    }

    #[test]
    fn closed_button_has_no_hover() {
        let (mut p, _) = placed();
        let bx = (p.btn_x + p.btn_w / 2.0) as f64;
        let by = (p.btn_y + MENU_BUTTON_H / 2.0) as f64;
        p.mouse_move(bx, by);
        assert_eq!(p.hovered, None);
    }

    #[test]
    fn outside_click_closes_without_selecting() {
        let (mut p, _) = placed();
        let bx = (p.btn_x + p.btn_w / 2.0) as f64;
        let by = (p.btn_y + MENU_BUTTON_H / 2.0) as f64;
        p.mouse_down(bx, by);
        p.mouse_up(bx, by);
        assert!(p.is_open());
        // Press inside a row, release far outside: no change.
        let ry = (p.row_y(3) + MENU_ROW_H / 2.0) as f64;
        let rx = (p.menu_x + p.menu_w / 2.0) as f64;
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
        assert!(p.menu_x >= 0.0);
        assert!(p.menu_y >= 0.0);
        assert!(p.menu_x + p.menu_w <= 200.0);
        assert!(p.menu_y + p.menu_h <= 120.0);
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
    }

    #[test]
    fn manual_hover_wins_over_theme_accent() {
        let mut p = MenuPicker::from_slice("Color", &["Red", "Green"])
            .hover_fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        p.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(p.hover, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = MenuPicker::from_slice("Color", &["Red", "Green"]);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.hover, Color::from_rgb8(0xff, 0x2d, 0x55));
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
}
