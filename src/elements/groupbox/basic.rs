use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Fill};

use super::super::layout::View;
use super::{GROUP_BG_DARK, GROUP_BG_LIGHT, GROUP_PAD, GROUP_RADIUS};
use super::super::text::{BasicText, TextAlignment};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{ThemeMode, desaturate};

/// Basic group box: a rounded fill slightly lighter than the app
/// background with simple centered text (like the reference rows).
/// Display-only (no mouse handling). The fill follows the placed
/// rect, so stacks can stretch it full-bleed like dividers; more
/// variants plug in beside it.
pub struct BasicGroupBox {
    content: String,
    text: BasicText,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl BasicGroupBox {
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        Self {
            text: BasicText::new(content.clone()).alignment(TextAlignment::Center),
            content,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Live theme for the fill (always a step lighter than the app
    /// background; on white that means a light gray shade).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.dark = mode == ThemeMode::Dark;
        self.text.set_theme(mode);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.text.set_focused(focused);
    }

    pub fn set_text(&mut self, content: impl Into<String>) {
        let content = content.into();
        if content != self.content {
            self.content = content.clone();
            self.text.set_text(content);
        }
    }

    pub fn text_value(&self) -> &str {
        &self.content
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn fill(&self) -> vello::peniko::Color {
        let base = if self.dark {
            GROUP_BG_DARK
        } else {
            GROUP_BG_LIGHT
        };
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }
}

impl View for BasicGroupBox {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (tw, th) = self.text.measure(fonts);
        (tw + GROUP_PAD * 2.0, th + GROUP_PAD * 2.0)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.text.place(
            fonts,
            x + GROUP_PAD,
            y + GROUP_PAD,
            (w - GROUP_PAD * 2.0).max(0.0),
            (h - GROUP_PAD * 2.0).max(0.0),
        );
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
        let px = |v: f32| v as f64 * scale;
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            GROUP_RADIUS as f64 * scale,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.fill()),
            None,
            &body,
        );
        self.text.draw(scene, fonts, images);
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
    fn measure_adds_padding() {
        let mut fonts = FontSystem::new();
        let mut plain = BasicText::new("This is content inside a GroupBox.");
        let (tw, th) = plain.measure(&mut fonts);
        let mut group = BasicGroupBox::new("This is content inside a GroupBox.");
        let (gw, gh) = group.measure(&mut fonts);
        assert_eq!(gw, tw + GROUP_PAD * 2.0);
        assert_eq!(gh, th + GROUP_PAD * 2.0);
    }

    #[test]
    fn theme_switches_fill() {
        let mut group = BasicGroupBox::new("x");
        group.set_theme(ThemeMode::Dark);
        assert_eq!(group.fill(), GROUP_BG_DARK);
        group.set_theme(ThemeMode::Light);
        assert_eq!(group.fill(), GROUP_BG_LIGHT);
    }

    #[test]
    fn set_text_updates_content() {
        let mut group = BasicGroupBox::new("a");
        group.set_text("b");
        assert_eq!(group.text_value(), "b");
    }
}
