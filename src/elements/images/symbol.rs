use std::any::Any;

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::Color;

use super::super::animation::Spin;
use super::super::layout::View;
use super::{IMAGE_SYMBOL_SIZE, spin_transform};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// SF Symbol from CoreIcon (`COREICON_ASSETS_DIR` override or the
/// system resources on TontooOS), sized by `size` with an optional
/// hand-set color. Without `.color()` the glyph follows the theme
/// text color via `set_theme`. Display-only (no mouse handling); a
/// missing symbol draws nothing but keeps its box.
pub struct SFSymbolImage {
    name: String,
    size: f32,
    color: Option<Color>,
    theme_text: Color,
    dark: bool,
    focused: bool,
    spin_deg: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl SFSymbolImage {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: IMAGE_SYMBOL_SIZE,
            color: None,
            theme_text: Color::from_rgb8(0xd8, 0xd9, 0xd9),
            dark: true,
            focused: true,
            spin_deg: 0.0,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Glyph box in logical px (clamped to >= 0). The artwork keeps
    /// its aspect ratio and centers inside.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(0.0);
        self
    }

    /// Optional hand-set glyph color. Wins over the theme until
    /// cleared by building without `.color()` again.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Swap the glyph name (CoreIcon lookup on next draw).
    pub fn set_symbol(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Swap the hand-set glyph color (`None` follows the theme again).
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Live theme: the glyph follows the theme text color unless the
    /// dev set a manual color.
    pub fn set_theme(&mut self, text: Color, dark: bool) {
        self.dark = dark;
        self.theme_text = text;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Active glyph color (manual color or theme text).
    pub fn tint(&self) -> Color {
        let base = self.color.unwrap_or(self.theme_text);
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }

    pub fn name_value(&self) -> &str {
        &self.name
    }

    pub fn size_value(&self) -> f32 {
        self.size
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl View for SFSymbolImage {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.size, self.size)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        // Crisp supersampling: upload at ~2x the display size.
        let target = (self.size * fonts.scale * 2.0).ceil().max(1.0) as u32;
        let name = self.name.clone();
        let tint = self.tint();
        if let Some((image, iw, ih)) = images.get(&name, tint, target) {
            let s = (self.placed_w / iw as f32).min(self.placed_h / ih as f32);
            let ix = self.x + (self.placed_w - iw as f32 * s) / 2.0;
            let iy = self.y + (self.placed_h - ih as f32 * s) / 2.0;
            let base = Affine::translate((ix as f64 * scale, iy as f64 * scale))
                * Affine::scale(s as f64 * scale);
            let (bw, bh) = (iw as f64 * s as f64 * scale, ih as f64 * s as f64 * scale);
            let transform = spin_transform(
                base,
                ix as f64 * scale + bw / 2.0,
                iy as f64 * scale + bh / 2.0,
                self.spin_deg,
            );
            scene.draw_image(&image, transform);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Spin for SFSymbolImage {
    fn set_spin(&mut self, degrees: f32) {
        self.spin_deg = degrees;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut symbol = SFSymbolImage::new("star.fill").size(32.0);
        assert_eq!(symbol.measure(&mut fonts), (32.0, 32.0));
        symbol.place(&mut fonts, 0.0, 0.0, 32.0, 32.0);
        assert_eq!(symbol.rect(), (0.0, 0.0, 32.0, 32.0));
    }

    #[test]
    fn manual_color_wins_over_theme() {
        let symbol = SFSymbolImage::new("star.fill")
            .color(Color::from_rgb8(0xff, 0x2d, 0x55));
        assert_eq!(symbol.tint(), Color::from_rgb8(0xff, 0x2d, 0x55));
        let mut plain = SFSymbolImage::new("star.fill");
        plain.set_theme(Color::from_rgb8(0x27, 0x27, 0x27), false);
        assert_eq!(plain.tint(), Color::from_rgb8(0x27, 0x27, 0x27));
    }

    #[test]
    fn clamps_negative_size() {
        let symbol = SFSymbolImage::new("star.fill").size(-4.0);
        assert_eq!(symbol.size_value(), 0.0);
    }
}
