use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::images::SFSymbolImage;
use super::super::layout::{Align, View, HStack};
use super::super::text::{BasicText, TextStyle};
use super::{LABEL_GAP, LABEL_ICON_GRAY, LABEL_ICON_SIZE};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::ThemeMode;

/// Basic label: SF icon plus title (like the reference rows: star,
/// heart, bookmark, mail). Display-only (no mouse handling).
pub struct BasicLabel {
    icon: String,
    icon_color: Option<Color>,
    title: String,
    dark: bool,
    focused: bool,
    row: HStack,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl BasicLabel {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        let mut label = Self {
            icon: icon.into(),
            icon_color: None,
            title: title.into(),
            dark: true,
            focused: true,
            row: HStack::new(),
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        };
        label.rebuild();
        label
    }

    /// Optional icon tint. Without it the glyph follows the theme
    /// text color.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self.rebuild();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.set_title(title);
        self
    }

    /// Live theme for icon (unless tinted) and title.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark == self.dark {
            return;
        }
        self.dark = dark;
        self.rebuild();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.apply_state();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if title != self.title {
            self.title = title;
            self.rebuild();
        }
    }

    pub fn set_icon(&mut self, name: impl Into<String>) {
        let name = name.into();
        if name != self.icon {
            self.icon = name;
            self.rebuild();
        }
    }

    pub fn set_icon_color(&mut self, color: Option<Color>) {
        if color != self.icon_color {
            self.icon_color = color;
            self.rebuild();
        }
    }

    pub fn icon_value(&self) -> &str {
        &self.icon
    }

    pub fn title_value(&self) -> &str {
        &self.title
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn rebuild(&mut self) {
        let mut symbol =
            SFSymbolImage::new(self.icon.clone()).size(LABEL_ICON_SIZE);
        if let Some(color) = self.icon_color {
            symbol = symbol.color(color);
        } else {
            symbol = symbol.color(if self.dark {
                LABEL_ICON_GRAY
            } else {
                Color::from_rgb8(0x27, 0x27, 0x27)
            });
        }
        self.row = HStack::new()
            .align(Align::Center)
            .spacing(LABEL_GAP)
            .child(symbol)
            .child(
                BasicText::new(self.title.clone())
                    .style(TextStyle::Title2)
                    .foreground_color(if self.dark {
                        Color::WHITE
                    } else {
                        Color::from_rgb8(0x27, 0x27, 0x27)
                    }),
            );
        self.apply_state();
    }

    fn apply_state(&mut self) {
        for index in 0..self.row.len() {
            if let Some(symbol) = self.row.child_mut::<SFSymbolImage>(index) {
                symbol.set_focused(self.focused);
            } else if let Some(text) = self.row.child_mut::<BasicText>(index) {
                text.set_focused(self.focused);
            }
        }
    }
}

impl View for BasicLabel {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.row.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.row.place(fonts, x, y, w, h);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.row.draw(scene, fonts, images);
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
    fn keeps_parts() {
        let label = BasicLabel::new("star", "Star");
        assert_eq!(label.icon_value(), "star");
        assert_eq!(label.title_value(), "Star");
    }

    #[test]
    fn intrinsic_size_covers_icon_and_text() {
        let mut fonts = FontSystem::new();
        let mut label = BasicLabel::new("star", "Star");
        let (w, h) = label.measure(&mut fonts);
        assert!(w > LABEL_ICON_SIZE + LABEL_GAP);
        assert!(h >= LABEL_ICON_SIZE);
    }

    #[test]
    fn theme_rebuilds_on_change_only() {
        let mut label = BasicLabel::new("star", "Star");
        label.set_theme(ThemeMode::Dark);
        label.set_theme(ThemeMode::Light);
        assert_eq!(label.title_value(), "Star");
    }
}
