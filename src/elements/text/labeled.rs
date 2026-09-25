use std::any::Any;

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::Color;

use super::super::layout::View;
use super::foreground::{ResolvedForeground, TextForeground};
use super::style::TextStyle;
use super::text::{BasicText, TextAlignment};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{ThemeMode, desaturate};

/// Gap between icon and text in logical px.
pub const LABELED_GAP: f32 = 6.0;

/// Text with an SF Symbol (SwiftUI `Label`): an icon from CoreIcon
/// (`COREICON_ASSETS_DIR` override or the system resources on
/// TontooOS) beside a `BasicText`. A missing icon draws the text
/// alone.
///
/// The text part is a real `BasicText`, so style, foreground
/// (including gradients), alignment and wrapping behave identically.
/// The icon defaults to the text box size and inherits the resolved
/// text color unless `icon_color` overrides it.
pub struct LabeledText {
    icon: String,
    label: BasicText,
    tracked_size: f32,
    icon_size: Option<f32>,
    gap: f32,
    icon_color: Option<Color>,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl LabeledText {
    pub fn new(text: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: BasicText::new(text),
            tracked_size: TextStyle::Body.size(),
            icon_size: None,
            gap: LABELED_GAP,
            icon_color: None,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            // BasicText defaults match (Body, Primary, Leading).
        }
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.tracked_size = style.size();
        self.label.set_style(style);
        self
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.tracked_size = style.size();
        self.label.set_style(style);
    }

    pub fn foreground(mut self, foreground: TextForeground) -> Self {
        self.label.set_foreground(foreground);
        self
    }

    /// Shortcut for a fixed text color (icon inherits it too, like
    /// the blue handset row).
    pub fn foreground_color(mut self, color: Color) -> Self {
        self.label.set_foreground(TextForeground::Color(color));
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.label.set_alignment(alignment);
        self
    }

    /// Fixed text box width in logical px for wrapping (icon and gap
    /// add on top).
    pub fn width(mut self, px: f32) -> Self {
        self.label.set_width(Some(px));
        self
    }

    /// Icon box in logical px (aspect kept). Default: the text size.
    pub fn icon_size(mut self, px: f32) -> Self {
        self.icon_size = Some(px.max(0.0));
        self
    }

    /// Gap between icon and text in logical px.
    pub fn gap(mut self, px: f32) -> Self {
        self.gap = px.max(0.0);
        self
    }

    /// Fixed icon color. `None` (default) inherits the resolved text
    /// color.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.label.set_text(text);
    }

    pub fn set_foreground(&mut self, foreground: TextForeground) {
        self.label.set_foreground(foreground);
    }

    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        self.label.set_alignment(alignment);
    }

    pub fn set_width(&mut self, px: Option<f32>) {
        self.label.set_width(px);
    }

    pub fn set_icon(&mut self, icon: impl Into<String>) {
        self.icon = icon.into();
    }

    pub fn set_icon_color(&mut self, color: Option<Color>) {
        self.icon_color = color;
    }

    /// Live theme for text and inherited icon color.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark != self.dark {
            self.dark = dark;
            self.label.set_theme(mode);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if focused != self.focused {
            self.focused = focused;
            self.label.set_focused(focused);
        }
    }

    fn mode(&self) -> ThemeMode {
        if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    fn resolved_icon_size(&self) -> f32 {
        // Style accessor would need a getter; the label owns it.
        // Approximation stays in sync through `icon_size()`.
        self.icon_size.unwrap_or_else(|| self.label_text_size())
    }

    fn label_text_size(&self) -> f32 {
        self.tracked_size
    }

    /// First-line height in logical px (matches the layout line
    /// height factor 1.25). The icon centers on it.
    fn first_line_height(&self) -> f32 {
        self.label_text_size() * 1.25
    }

    fn icon_tint(&self) -> Color {
        let base = match self.icon_color {
            Some(color) => color,
            None => match self.label.resolve_foreground(self.mode(), self.focused) {
                ResolvedForeground::Solid(color) => color,
                ResolvedForeground::Gradient(colors) => colors
                    .first()
                    .copied()
                    .unwrap_or(Color::WHITE),
            },
        };
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }
}

impl View for LabeledText {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let box_px = self.resolved_icon_size();
        let (tw, th) = self.label.measure(fonts);
        (box_px + self.gap + tw, th.max(box_px))
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        let box_px = self.resolved_icon_size();
        let (total_w, total_h) = self.measure(fonts);
        self.x = x;
        self.y = y;
        self.width = w.max(total_w);
        self.height = total_h;
        self.label.place(
            fonts,
            x + box_px + self.gap,
            y,
            (w - box_px - self.gap).max(0.0),
            h,
        );
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LabeledText {
    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        let scale = fonts.scale;
        let box_px = self.resolved_icon_size();
        let line_h = self.first_line_height();
        let icon_y = self.y + (line_h - box_px) / 2.0;
        let tint = self.icon_tint();
        let target = (box_px * scale * 2.0).ceil().max(1.0) as u32;
        if let Some((image, iw, ih)) = images.get(&self.icon.clone(), tint, target) {
            let s = (box_px / iw as f32).min(box_px / ih as f32);
            let scale64 = scale as f64;
            let ix = self.x + (box_px - iw as f32 * s) / 2.0;
            let iy = icon_y + (box_px - ih as f32 * s) / 2.0;
            let transform = Affine::translate((ix as f64 * scale64, iy as f64 * scale64))
                * Affine::scale(s as f64 * scale64);
            scene.draw_image(&image, transform);
        }
        self.label.draw(scene, fonts, images);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeMode;

    #[test]
    fn measure_adds_icon_and_gap() {
        let mut fonts = FontSystem::new();
        let mut labeled = LabeledText::new("Starred", "star");
        let mut plain = BasicText::new("Starred");
        let (lw, lh) = labeled.measure(&mut fonts);
        let (tw, th) = plain.measure(&mut fonts);
        // Default icon box is the Body size, gap is LABELED_GAP.
        assert_eq!(lw, TextStyle::Body.size() + LABELED_GAP + tw);
        assert_eq!(lh, th.max(TextStyle::Body.size()));
    }

    #[test]
    fn icon_size_override_grows_measure() {
        let mut fonts = FontSystem::new();
        let (small, _) = LabeledText::new("Hi", "star").measure(&mut fonts);
        let (big, _) = LabeledText::new("Hi", "star").icon_size(32.0).measure(&mut fonts);
        assert!(big > small);
    }

    #[test]
    fn theme_and_focus_do_not_crash() {
        let mut fonts = FontSystem::new();
        let mut labeled = LabeledText::new("Custom Label", "heart")
            .icon_color(Color::from_rgb8(0xff, 0x3b, 0x30));
        labeled.set_theme(ThemeMode::Light);
        labeled.set_focused(false);
        labeled.place(&mut fonts, 0.0, 0.0, 400.0, 100.0);
        let (w, h) = labeled.measure(&mut fonts);
        assert!(w > 0.0 && h > 0.0);
    }
}
