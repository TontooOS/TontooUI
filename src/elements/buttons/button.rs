use std::any::Any;

use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::desaturate;

/// Light mode button fill.
pub const BUTTON_BG_LIGHT: Color = Color::from_rgb8(0xe9, 0xe9, 0xeb);
/// Dark mode button fill.
pub const BUTTON_BG_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
/// Default accent for prominent/tinted styles.
pub const BUTTON_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Corner radius in logical px.
pub const BUTTON_RADIUS: f32 = 8.0;
/// Label size in logical px (macOS control size).
pub const BUTTON_FONT_SIZE: f32 = 13.0;
/// Horizontal/vertical padding in logical px.
pub const BUTTON_PAD_X: f32 = 12.0;
pub const BUTTON_PAD_Y: f32 = 6.0;
/// Gap between icon and label in logical px.
pub const BUTTON_GAP: f32 = 6.0;
/// Default icon box in logical px.
pub const BUTTON_ICON_SIZE: f32 = 16.0;

/// Button style (SwiftUI `buttonStyle`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Gray fill (default via `Automatic`).
    #[default]
    Automatic,
    Bordered,
    BorderedProminent,
    BorderedTinted,
    Plain,
}

/// Button shape (SwiftUI `buttonBorderShape`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    #[default]
    Automatic,
    RoundedRectangle,
    Capsule,
    Circle,
}

/// Standard button: rounded fill, optional SF Symbol icon plus label,
/// hover/pressed/disabled states and a press callback. Icon artwork comes
/// from CoreIcon (`COREICON_ASSETS_DIR` override or the system resources on
/// TontooOS); a missing icon simply draws the label.
pub struct Button {
    label: String,
    icon: Option<String>,
    icon_size: f32,
    style: ButtonStyle,
    shape: ButtonShape,
    accent: Color,
    dark: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    bg: Color,
    text_color: Color,
    hovered: bool,
    hover_effect: bool,
    pressed: bool,
    disabled: bool,
    focused: bool,
    on_press: Option<Box<dyn FnMut()>>,
    layout: Option<Layout<SolidBrush>>,
    layout_color: Color,
    layout_scale: f32,
    dirty: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            icon_size: BUTTON_ICON_SIZE,
            style: ButtonStyle::Automatic,
            shape: ButtonShape::Automatic,
            accent: BUTTON_ACCENT,
            dark: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            bg: BUTTON_BG_DARK,
            text_color: Color::WHITE,
            hovered: false,
            hover_effect: true,
            pressed: false,
            disabled: false,
            focused: true,
            on_press: None,
            layout: None,
            layout_color: Color::WHITE,
            layout_scale: 0.0,
            dirty: true,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn accent(mut self, accent: Color) -> Self {
        self.accent = accent;
        self
    }

    /// Live theme: accent color plus dark mode flag (press lightens in
    /// dark mode and darkens in light mode).
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
    }

    /// SF Symbol name for the leading icon (e.g. `"hand.tap"`).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    pub fn icon_size(mut self, px: f32) -> Self {
        self.icon_size = px.max(0.0);
        self
    }

    pub fn on_press(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_press = Some(Box::new(callback));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Hover highlight on/off. Alerts disable it: their buttons react
    /// to clicks only, hover does nothing.
    pub fn hover_effect(mut self, enabled: bool) -> Self {
        self.hover_effect = enabled;
        if !enabled {
            self.hovered = false;
        }
        self
    }

    pub fn set_hover_effect(&mut self, enabled: bool) {
        self.hover_effect = enabled;
        if !enabled {
            self.hovered = false;
        }
    }

    /// Live theme colors (background fill and label/icon color).
    pub fn set_palette(&mut self, bg: Color, text: Color) {
        if text != self.text_color {
            self.dirty = true;
        }
        self.bg = bg;
        self.text_color = text;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        let label = label.into();
        if label != self.label {
            self.label = label;
            self.dirty = true;
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if !self.disabled && self.hit(x as f32, y as f32) {
            self.pressed = true;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn finish_press(&mut self, x: f64, y: f64) {
        let was_pressed = self.pressed;
        self.pressed = false;
        if was_pressed && !self.disabled && self.hit(x as f32, y as f32) {
            if let Some(callback) = self.on_press.as_mut() {
                callback();
            }
        }
    }

    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.hover_effect && self.hit(x, y);
    }

    fn resolved_style(&self) -> ButtonStyle {
        match self.style {
            ButtonStyle::Automatic => ButtonStyle::Bordered,
            style => style,
        }
    }

    fn corner_radius(&self) -> f32 {
        match self.shape {
            ButtonShape::Capsule | ButtonShape::Circle => {
                self.width.min(self.height) / 2.0
            }
            _ => BUTTON_RADIUS,
        }
    }

    /// (Background fill or `None` for plain, label/icon color).
    fn effective(&self) -> (Option<Color>, Color) {
        let (mut bg, mut text) = match self.resolved_style() {
            ButtonStyle::BorderedProminent => (Some(self.accent), Color::WHITE),
            ButtonStyle::BorderedTinted => {
                (Some(with_alpha(self.accent, 0.2)), self.accent)
            }
            ButtonStyle::Plain => (None, self.text_color),
            _ => (Some(self.bg), self.text_color),
        };
        if !self.focused {
            bg = bg.map(desaturate);
            text = desaturate(text);
        }
        if self.disabled {
            bg = bg.map(|c| with_alpha(c, 0.4));
            text = with_alpha(text, 0.4);
        }
        if self.resolved_style() == ButtonStyle::Plain && self.pressed && !self.disabled {
            text = with_alpha(text, 0.6);
        }
        (bg, text)
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem, text: Color) {
        if self.dirty
            || self.layout.is_none()
            || text != self.layout_color
            || self.layout_scale != fonts.scale
        {
            self.layout = Some(fonts.layout_text(
                &self.label,
                BUTTON_FONT_SIZE,
                text,
                None,
            ));
            self.layout_color = text;
            self.layout_scale = fonts.scale;
            self.dirty = false;
        }
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let (bg, text) = self.effective();

        let shape = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + self.height),
            self.corner_radius() as f64 * scale,
        );
        if let Some(bg) = bg {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(bg),
                None,
                &shape,
            );
            // Press lightens in dark mode, darkens in light mode.
            let (press, hover) = if self.dark {
                (
                    Color::from_rgba8(255, 255, 255, 36),
                    Color::from_rgba8(255, 255, 255, 20),
                )
            } else {
                (
                    Color::from_rgba8(0, 0, 0, 36),
                    Color::from_rgba8(0, 0, 0, 15),
                )
            };
            if self.pressed && !self.disabled {
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(press), None, &shape);
            } else if self.hovered && self.hover_effect && !self.disabled {
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(hover), None, &shape);
            }
        }

        self.ensure_layout(fonts, text);
        let layout = self.layout.as_ref().expect("layout built");
        let (tw, th) = FontSystem::layout_size(layout);
        let tw = tw / fonts.scale;
        let th = th / fonts.scale;

        let has_icon = self.icon.is_some();
        let icon_box = if has_icon { self.icon_size } else { 0.0 };
        let gap = if has_icon { BUTTON_GAP } else { 0.0 };
        let content_w = icon_box + gap + tw;
        let mut cx = self.x + (self.width - content_w) / 2.0;
        let cy = self.y + (self.height - th.max(self.icon_size)) / 2.0;

        if let Some(name) = self.icon.clone() {
            let target = (self.icon_size * fonts.scale * 2.0).ceil().max(1.0) as u32;
            if let Some((image, iw, ih)) = images.get(&name, text, target) {
                let s = (self.icon_size / iw as f32).min(self.icon_size / ih as f32);
                let ix = cx + (self.icon_size - iw as f32 * s) / 2.0;
                let iy = cy + (th.max(self.icon_size) - ih as f32 * s) / 2.0;
                let transform = Affine::translate((ix as f64 * scale, iy as f64 * scale))
                    * Affine::scale(s as f64 * scale);
                scene.draw_image(&image, transform);
            }
            cx += icon_box + gap;
        }
        draw_layout(scene, layout, cx, self.y + (self.height - th) / 2.0, fonts.scale);
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let c = color.to_rgba8();
    Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha).round() as u8)
}

impl View for Button {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (_, text) = (self.bg, self.text_color);
        self.ensure_layout(fonts, text);
        let layout = self.layout.as_ref().expect("layout built");
        let (tw, th) = FontSystem::layout_size(layout);
        let scale = fonts.scale;
        let icon_box = if self.icon.is_some() {
            self.icon_size
        } else {
            0.0
        };
        let gap = if self.icon.is_some() {
            BUTTON_GAP
        } else {
            0.0
        };
        (
            tw / scale + icon_box + gap + BUTTON_PAD_X * 2.0,
            th.max(self.icon_size) / scale + BUTTON_PAD_Y * 2.0,
        )
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        if !self.disabled && self.hit(x as f32, y as f32) {
            self.pressed = true;
        }
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.hover_effect && self.hit(x, y);
    }

    fn set_focused(&mut self, focused: bool) {
        self.set_focused(focused);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_effect_off_ignores_hover() {
        let mut button = Button::new("OK").hover_effect(false);
        button.place(
            &mut crate::renderer::text::FontSystem::new(),
            0.0,
            0.0,
            200.0,
            44.0,
        );
        button.set_hover(100.0, 22.0);
        assert!(!button.hovered);
        button.set_hover_effect(true);
        button.set_hover(100.0, 22.0);
        assert!(button.hovered);
    }
}
