use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Circle};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::LABEL_DOT;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{CTFrame, FontSystem, draw_layout};
use crate::theme::{ThemeMode, desaturate};

/// Label text style: title, body or a colored status (dot plus
/// colored text, like the green "Downloaded" reference row).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LabelStyle {
    /// Semibold theme text.
    Title,
    /// Regular dim text.
    Body,
    /// Colored dot plus colored text.
    Status(Color),
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self::Body
    }
}

/// Styled label: text in a title, body or status style. The status
/// style paints a colored dot plus colored text. Display-only (no
/// mouse handling).
pub struct StyledLabel {
    text: String,
    style: LabelStyle,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
    layout: Option<CTFrame>,
    layout_scale: f32,
    dirty: bool,
}

impl StyledLabel {
    pub fn new(text: impl Into<String>, style: LabelStyle) -> Self {
        Self {
            text: text.into(),
            style,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
            layout: None,
            layout_scale: 0.0,
            dirty: true,
        }
    }

    /// Semibold title label.
    pub fn title(text: impl Into<String>) -> Self {
        Self::new(text, LabelStyle::Title)
    }

    /// Regular dim body label.
    pub fn body(text: impl Into<String>) -> Self {
        Self::new(text, LabelStyle::Body)
    }

    /// Colored status label (dot plus text in `color`).
    pub fn status(text: impl Into<String>, color: Color) -> Self {
        Self::new(text, LabelStyle::Status(color))
    }

    /// Live theme for title/body colors (status keeps its color).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark == self.dark {
            return;
        }
        self.dark = dark;
        self.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        if focused != self.focused {
            self.focused = focused;
            self.dirty = true;
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        if text != self.text {
            self.text = text;
            self.dirty = true;
        }
    }

    pub fn set_style(&mut self, style: LabelStyle) {
        if style != self.style {
            self.style = style;
            self.dirty = true;
        }
    }

    pub fn text_value(&self) -> &str {
        &self.text
    }

    pub fn style_value(&self) -> LabelStyle {
        self.style
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn colors(&self) -> (Color, f32, f32) {
        // (text color, size, weight).
        match self.style {
            LabelStyle::Title => (
                if self.dark {
                    Color::WHITE
                } else {
                    Color::from_rgb8(0x27, 0x27, 0x27)
                },
                17.0,
                600.0,
            ),
            LabelStyle::Body => (
                if self.dark {
                    Color::from_rgb8(0x9a, 0x9a, 0x9e)
                } else {
                    Color::from_rgb8(0x6e, 0x6e, 0x72)
                },
                15.0,
                400.0,
            ),
            LabelStyle::Status(color) => (color, 15.0, 600.0),
        }
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty
            && self.layout.is_some()
            && self.layout_scale == fonts.scale
        {
            return;
        }
        let (color, size, weight) = self.colors();
        let color = if self.focused {
            color
        } else {
            desaturate(color)
        };
        self.layout = Some(fonts.layout_text_weighted(&self.text, size, color, weight, None));
        self.layout_scale = fonts.scale;
        self.dirty = false;
    }
}

impl View for StyledLabel {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        let (tw, th) = FontSystem::layout_size(layout);
        let dot = match self.style {
            LabelStyle::Status(_) => LABEL_DOT + 8.0,
            _ => 0.0,
        };
        (tw / fonts.scale + dot, th / fonts.scale)
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
        _images: &mut ImageLoader<'_>,
    ) {
        self.ensure_layout(fonts);
        let scale = fonts.scale as f64;
        let mut tx = self.x;
        if let LabelStyle::Status(color) = self.style {
            let dot = LABEL_DOT;
            let cy = self.y + self.placed_h / 2.0;
            let c = if self.focused {
                color
            } else {
                desaturate(color)
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(c),
                None,
                &Circle::new(
                    ((tx + dot / 2.0) as f64 * scale, cy as f64 * scale),
                    dot as f64 * scale / 2.0,
                ),
            );
            tx += dot + 8.0;
        }
        if let Some(layout) = self.layout.as_ref() {
            let (_, th) = FontSystem::layout_size(layout);
            draw_layout(
                scene,
                layout,
                tx,
                self.y + (self.placed_h - th / fonts.scale) / 2.0,
                fonts.scale,
            );
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
    fn presets_hold_styles() {
        assert_eq!(StyledLabel::title("T").style_value(), LabelStyle::Title);
        let green = Color::from_rgb8(0x34, 0xc7, 0x59);
        assert_eq!(
            StyledLabel::status("Downloaded", green).style_value(),
            LabelStyle::Status(green)
        );
    }

    #[test]
    fn status_reserves_dot_width() {
        let mut fonts = FontSystem::new();
        let green = Color::from_rgb8(0x34, 0xc7, 0x59);
        let mut status = StyledLabel::status("Downloaded", green);
        let mut body = StyledLabel::body("Downloaded");
        assert!(
            status.measure(&mut fonts).0
                > body.measure(&mut fonts).0 + LABEL_DOT
        );
    }

    #[test]
    fn set_text_updates_content() {
        let mut label = StyledLabel::title("Old");
        label.set_text("New");
        assert_eq!(label.text_value(), "New");
    }
}
