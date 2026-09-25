use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::{LABEL_ICON_GRAY, LABEL_ICON_SIZE};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::ThemeMode;

/// Icon-only label: a bare SF Symbol with no text. Thin wrapper over
/// `SFSymbolImage`, so glyphs always match the icon rows.
pub struct IconLabel {
    inner: SFSymbolImage,
    icon: String,
    color: Option<Color>,
    dark: bool,
    focused: bool,
}

impl IconLabel {
    pub fn new(icon: impl Into<String>) -> Self {
        let icon = icon.into();
        let mut label = Self {
            inner: SFSymbolImage::new(icon.clone()).size(LABEL_ICON_SIZE),
            icon,
            color: None,
            dark: true,
            focused: true,
        };
        label.apply_theme();
        label
    }

    /// Glyph box in logical px (clamped to >= 0).
    pub fn size(mut self, px: f32) -> Self {
        self.inner = SFSymbolImage::new(self.icon.clone()).size(px);
        if let Some(color) = self.color {
            self.inner = self.inner.color(color);
        }
        self.apply_theme();
        self.inner.set_focused(self.focused);
        self
    }

    /// Optional glyph tint. Without it the glyph follows the theme
    /// text color.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self.inner.set_color(Some(color));
        self
    }

    /// Live theme for the glyph (unless tinted).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.dark = mode == ThemeMode::Dark;
        self.apply_theme();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.inner.set_focused(focused);
    }

    pub fn set_icon(&mut self, name: impl Into<String>) {
        let name = name.into();
        if name != self.icon {
            self.icon = name.clone();
            self.inner.set_symbol(name);
        }
    }

    pub fn icon_value(&self) -> &str {
        &self.icon
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        self.inner.rect()
    }

    fn apply_theme(&mut self) {
        if self.color.is_none() {
            let dark = self.dark;
            self.inner.set_theme(self.text_color(dark), dark);
        }
    }

    fn text_color(&self, dark: bool) -> Color {
        if dark {
            LABEL_ICON_GRAY
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        }
    }
}

impl View for IconLabel {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.inner.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.inner.place(fonts, x, y, w, h);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.inner.draw(scene, fonts, images);
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
    fn keeps_icon() {
        let label = IconLabel::new("star");
        assert_eq!(label.icon_value(), "star");
    }

    #[test]
    fn intrinsic_size_matches_box() {
        let mut fonts = FontSystem::new();
        let mut label = IconLabel::new("star").size(32.0);
        assert_eq!(label.measure(&mut fonts), (32.0, 32.0));
    }
}
