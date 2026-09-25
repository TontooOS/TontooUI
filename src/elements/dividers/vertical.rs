use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::{DIVIDER_DARK, DIVIDER_FILL, DIVIDER_LIGHT, DIVIDER_THIN, DividerStyle};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Full-height vertical divider. The intrinsic height is the fill
/// extent (`DIVIDER_FILL`), so stacks hand it the whole parent height:
/// the line is full bleed with no edge gap. Display-only (no mouse
/// handling). Mirrors `HorizontalDivider` along the y axis.
pub struct VerticalDivider {
    thickness: f32,
    color: Color,
    color_manual: bool,
    inset: f32,
    rounded: bool,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl VerticalDivider {
    pub fn new() -> Self {
        Self {
            thickness: DIVIDER_THIN,
            color: DIVIDER_DARK,
            color_manual: false,
            inset: 0.0,
            rounded: false,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Preset look from `DividerStyle` (default, red, thick,
    /// blue thick, padded). Manual builder calls after this win.
    pub fn styled(style: DividerStyle) -> Self {
        Self::new().apply_style(style)
    }

    /// Preset look from `DividerStyle`, as a builder on `self`.
    pub fn apply_style(mut self, style: DividerStyle) -> Self {
        self.thickness = style.thickness();
        self.inset = style.inset();
        match style.color() {
            Some(color) => {
                self.color = color;
                self.color_manual = true;
            }
            None => {
                self.color_manual = false;
                self.color = if self.dark {
                    DIVIDER_DARK
                } else {
                    DIVIDER_LIGHT
                };
            }
        }
        self
    }

    /// Line thickness in logical px (clamped to >= 0).
    pub fn thickness(mut self, px: f32) -> Self {
        self.thickness = px.max(0.0);
        self
    }

    /// Manual line color: wins over the theme until cleared via
    /// `apply_style` with a theme-following preset or `set_theme`
    /// on a non-manual divider.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.color_manual = true;
        self
    }

    /// Symmetric vertical inset in logical px (the padded look).
    /// Clamped to >= 0.
    pub fn inset(mut self, px: f32) -> Self {
        self.inset = px.max(0.0);
        self
    }

    /// Rounded line ends (visible on thick dividers).
    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    /// Live theme: the line follows the theme divider color unless
    /// the dev set a manual color.
    pub fn set_theme(&mut self, divider: Color, dark: bool) {
        self.dark = dark;
        if !self.color_manual {
            self.color = divider;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Line thickness in logical px.
    pub fn set_thickness(&mut self, px: f32) {
        self.thickness = px.max(0.0);
    }

    /// Symmetric vertical inset in logical px.
    pub fn set_inset(&mut self, px: f32) {
        self.inset = px.max(0.0);
    }

    pub fn thickness_value(&self) -> f32 {
        self.thickness
    }

    pub fn inset_value(&self) -> f32 {
        self.inset
    }

    pub fn line_color(&self) -> Color {
        self.color
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }
}

impl Default for VerticalDivider {
    fn default() -> Self {
        Self::new()
    }
}

impl View for VerticalDivider {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.thickness, DIVIDER_FILL)
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
        _images: &mut ImageLoader<'_>,
    ) {
        if self.thickness <= 0.0 || self.width <= 0.0 || self.height <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        // Full bleed: inset only shrinks the line ends, never the
        // placed rect. The line stays horizontally centered if the
        // parent hands it extra width.
        let inset = self.inset.min(self.height / 2.0);
        let ly = self.y + inset;
        let lh = (self.height - inset * 2.0).max(0.0);
        let lw = self.thickness.min(self.width);
        let lx = self.x + ((self.width - lw) / 2.0).max(0.0);
        let brush = Brush::Solid(self.eff(self.color));
        if self.rounded && lw > 0.0 {
            let line = RoundedRect::new(
                px(lx),
                px(ly),
                px(lx + lw),
                px(ly + lh),
                px(lw / 2.0),
            );
            scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &line);
        } else {
            let line = Rect::new(px(lx), px(ly), px(lx + lw), px(ly + lh));
            scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &line);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn fills_parent_height() {
        let mut fonts = FontSystem::new();
        let mut line = VerticalDivider::new();
        let (w, h) = line.measure(&mut fonts);
        assert_eq!(w, DIVIDER_THIN);
        assert!(h >= DIVIDER_FILL);
        line.place(&mut fonts, 0.0, 0.0, DIVIDER_THIN, 600.0);
        assert_eq!(line.rect(), (0.0, 0.0, DIVIDER_THIN, 600.0));
    }

    #[test]
    fn style_presets_match_reference_rows() {
        let red = VerticalDivider::styled(DividerStyle::Red);
        assert_eq!(red.thickness_value(), DIVIDER_THIN);
        assert_eq!(red.line_color(), super::super::DIVIDER_RED);

        let blue = VerticalDivider::styled(DividerStyle::BlueThick);
        assert_eq!(blue.thickness_value(), super::super::DIVIDER_THICK);
        assert_eq!(blue.line_color(), super::super::DIVIDER_BLUE);

        let padded = VerticalDivider::styled(DividerStyle::Padded);
        assert_eq!(
            padded.inset_value(),
            super::super::DIVIDER_PADDED_INSET
        );
    }

    #[test]
    fn manual_color_wins_over_theme() {
        let mut line =
            VerticalDivider::new().color(Color::from_rgb8(0xff, 0x2d, 0x55));
        line.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), true);
        assert_eq!(line.line_color(), Color::from_rgb8(0xff, 0x2d, 0x55));
        let mut plain = VerticalDivider::new();
        plain.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), true);
        assert_eq!(plain.line_color(), Color::from_rgb8(0x00, 0x7a, 0xff));
    }

    #[test]
    fn clamps_negative_values() {
        let line = VerticalDivider::new().thickness(-4.0).inset(-8.0);
        assert_eq!(line.thickness_value(), 0.0);
        assert_eq!(line.inset_value(), 0.0);
    }
}
