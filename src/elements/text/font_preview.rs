use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::foreground::{ResolvedForeground, TextForeground};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{CTFrame, FontSystem, draw_layout};
use crate::theme::ThemeMode;

/// Family name size in logical px (semibold).
pub const FONT_PREVIEW_TITLE_SIZE: f32 = 17.0;
/// Family name weight.
pub const FONT_PREVIEW_TITLE_WEIGHT: f32 = 600.0;
/// Sample text size in logical px (regular).
pub const FONT_PREVIEW_SAMPLE_SIZE: f32 = 28.0;
/// Sample text weight.
pub const FONT_PREVIEW_SAMPLE_WEIGHT: f32 = 400.0;
/// Vertical gap between the family name and the sample in logical px.
pub const FONT_PREVIEW_GAP: f32 = 4.0;
/// Default sample alphabet plus digits.
pub const FONT_PREVIEW_DEFAULT_SAMPLE: &str = "AaBbCcDdEeFfGg 0123456789";

/// Font preview: the family name rendered in its own family plus a
/// sample line, stacked vertically (Font Book style row).
///
/// Both lines lay out through CoreText in the preview `family` with
/// a system-ui fallback, so unknown families degrade to the system
/// font instead of .notdef boxes. Colors follow the theme palette
/// (`Primary`); display-only, no mouse handling.
pub struct FontPreview {
    family: String,
    sample: String,
    title_size: f32,
    sample_size: f32,
    show_family_label: bool,
    wrap_width: Option<f32>,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    title_layout: Option<CTFrame>,
    sample_layout: Option<CTFrame>,
    layout_scale: f32,
    layout_family: String,
    dirty: bool,
}

impl FontPreview {
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            sample: FONT_PREVIEW_DEFAULT_SAMPLE.to_string(),
            title_size: FONT_PREVIEW_TITLE_SIZE,
            sample_size: FONT_PREVIEW_SAMPLE_SIZE,
            show_family_label: true,
            wrap_width: None,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            title_layout: None,
            sample_layout: None,
            layout_scale: 0.0,
            layout_family: String::new(),
            dirty: true,
        }
    }

    pub fn sample(mut self, sample: impl Into<String>) -> Self {
        self.set_sample(sample);
        self
    }

    pub fn title_size(mut self, size: f32) -> Self {
        self.set_title_size(size);
        self
    }

    pub fn sample_size(mut self, size: f32) -> Self {
        self.set_sample_size(size);
        self
    }

    pub fn show_family_label(mut self, show: bool) -> Self {
        self.set_show_family_label(show);
        self
    }

    /// Fixed box width in logical px: longer content wraps. `None`
    /// (default) measures the intrinsic single-line size.
    pub fn width(mut self, px: f32) -> Self {
        self.set_width(Some(px));
        self
    }

    pub fn set_family(&mut self, family: impl Into<String>) {
        let family = family.into();
        if family != self.family {
            self.family = family;
            self.dirty = true;
        }
    }

    pub fn set_sample(&mut self, sample: impl Into<String>) {
        let sample = sample.into();
        if sample != self.sample {
            self.sample = sample;
            self.dirty = true;
        }
    }

    pub fn set_title_size(&mut self, size: f32) {
        let size = size.max(1.0);
        if size != self.title_size {
            self.title_size = size;
            self.dirty = true;
        }
    }

    pub fn set_sample_size(&mut self, size: f32) {
        let size = size.max(1.0);
        if size != self.sample_size {
            self.sample_size = size;
            self.dirty = true;
        }
    }

    pub fn set_show_family_label(&mut self, show: bool) {
        if show != self.show_family_label {
            self.show_family_label = show;
            self.dirty = true;
        }
    }

    pub fn set_width(&mut self, px: Option<f32>) {
        let px = px.map(|w| w.max(0.0));
        if px != self.wrap_width {
            self.wrap_width = px;
            self.dirty = true;
        }
    }

    pub fn family_value(&self) -> &str {
        &self.family
    }

    pub fn sample_value(&self) -> &str {
        &self.sample
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Live theme: picks the semantic foreground colors.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark != self.dark {
            self.dark = dark;
            self.dirty = true;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if focused != self.focused {
            self.focused = focused;
            self.dirty = true;
        }
    }

    fn mode(&self) -> ThemeMode {
        if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    fn base_color(&self) -> Color {
        match TextForeground::Primary.resolve(self.mode(), self.focused) {
            ResolvedForeground::Solid(color) => color,
            ResolvedForeground::Gradient(stops) => stops.first().copied().unwrap_or(Color::WHITE),
        }
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty
            && self.sample_layout.is_some()
            && self.layout_scale == fonts.scale
            && self.layout_family == self.family
        {
            return;
        }
        let color = self.base_color();
        if self.show_family_label {
            let title = fonts.layout_text_in_family(
                &self.family,
                &self.family,
                self.title_size,
                color,
                FONT_PREVIEW_TITLE_WEIGHT,
                self.wrap_width,
            );
            self.title_layout = Some(title);
        } else {
            self.title_layout = None;
        }
        let sample = fonts.layout_text_in_family(
            &self.sample,
            &self.family,
            self.sample_size,
            color,
            FONT_PREVIEW_SAMPLE_WEIGHT,
            self.wrap_width,
        );
        self.sample_layout = Some(sample);
        self.layout_scale = fonts.scale;
        self.layout_family = self.family.clone();
        self.dirty = false;
    }

    fn sizes(&mut self, fonts: &mut FontSystem) -> ((f32, f32), (f32, f32)) {
        self.ensure_layout(fonts);
        let title = self
            .title_layout
            .as_ref()
            .map(|frame| {
                let (w, h) = FontSystem::layout_size(frame);
                (w / fonts.scale, h / fonts.scale)
            })
            .unwrap_or((0.0, 0.0));
        let sample = self
            .sample_layout
            .as_ref()
            .map(|frame| {
                let (w, h) = FontSystem::layout_size(frame);
                (w / fonts.scale, h / fonts.scale)
            })
            .unwrap_or((0.0, 0.0));
        (title, sample)
    }
}

impl View for FontPreview {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (title, sample) = self.sizes(fonts);
        match self.wrap_width {
            Some(wrap) => {
                let mut height = sample.1;
                if self.show_family_label {
                    height += title.1 + FONT_PREVIEW_GAP;
                }
                (wrap, height)
            }
            None => {
                let width = title.0.max(sample.0);
                let mut height = sample.1;
                if self.show_family_label {
                    height += title.1 + FONT_PREVIEW_GAP;
                }
                (width, height)
            }
        }
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        let _ = h;
        let (title, sample) = self.sizes(fonts);
        self.x = x;
        self.y = y;
        match self.wrap_width {
            Some(wrap) => {
                self.width = w.max(wrap);
                let mut height = sample.1;
                if self.show_family_label {
                    height += title.1 + FONT_PREVIEW_GAP;
                }
                self.height = height;
            }
            None => {
                let width = title.0.max(sample.0);
                let mut height = sample.1;
                if self.show_family_label {
                    height += title.1 + FONT_PREVIEW_GAP;
                }
                self.x = x;
                self.y = y;
                self.width = w.max(width);
                self.height = height;
            }
        }
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        self.ensure_layout(fonts);
        let scale = fonts.scale;
        let mut dy = self.y;
        if let Some(title) = self.title_layout.as_ref() {
            draw_layout(scene, title, self.x, dy, scale);
            let (_, th) = FontSystem::layout_size(title);
            dy += th / scale + FONT_PREVIEW_GAP;
        }
        if let Some(sample) = self.sample_layout.as_ref() {
            draw_layout(scene, sample, self.x, dy, scale);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeMode;

    #[test]
    fn keeps_family_and_sample() {
        let preview = FontPreview::new("SF Pro");
        assert_eq!(preview.family_value(), "SF Pro");
        assert_eq!(preview.sample_value(), FONT_PREVIEW_DEFAULT_SAMPLE);
    }

    #[test]
    fn intrinsic_size_is_positive() {
        let mut fonts = FontSystem::new();
        let mut preview = FontPreview::new("SF Pro");
        let (w, h) = preview.measure(&mut fonts);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn label_hidden_is_shorter() {
        let mut fonts = FontSystem::new();
        let (_, full_h) = FontPreview::new("SF Pro").measure(&mut fonts);
        let (_, bare_h) = FontPreview::new("SF Pro")
            .show_family_label(false)
            .measure(&mut fonts);
        assert!(bare_h < full_h);
    }

    #[test]
    fn fixed_width_wraps() {
        let mut fonts = FontSystem::new();
        let (wrap_w, _) = FontPreview::new("SF Pro").width(120.0).measure(&mut fonts);
        assert_eq!(wrap_w, 120.0);
    }

    #[test]
    fn family_change_rebuilds() {
        let mut fonts = FontSystem::new();
        let mut preview = FontPreview::new("SF Pro");
        let _ = preview.measure(&mut fonts);
        preview.set_family("Inter");
        assert_eq!(preview.family_value(), "Inter");
        let (w, h) = preview.measure(&mut fonts);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn scale_change_rebuilds() {
        let mut fonts = FontSystem::new();
        let mut preview = FontPreview::new("SF Pro");
        let _ = preview.measure(&mut fonts);
        fonts.scale = 2.0;
        let (w, h) = preview.measure(&mut fonts);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn theme_and_focus_update() {
        let mut preview = FontPreview::new("SF Pro");
        preview.set_theme(ThemeMode::Light);
        preview.set_focused(false);
        assert_eq!(preview.sample_value(), FONT_PREVIEW_DEFAULT_SAMPLE);
    }
}
