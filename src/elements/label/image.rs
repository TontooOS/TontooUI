use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::images::UrlImage;
use super::super::layout::View;
use super::super::text::{BasicText, TextStyle};
use super::{LABEL_GAP, LABEL_IMAGE_SIZE};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::ThemeMode;

/// Label with image: async URL image plus title (like the reference
/// rows: picture icon plus "Custom Title"). The image loads with the
/// usual spinner and error states; the row itself is display-only
/// (no mouse handling).
pub struct ImageLabel {
    url: String,
    title: String,
    image: UrlImage,
    text: BasicText,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl ImageLabel {
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        let url = url.into();
        let title = title.into();
        Self {
            image: UrlImage::new(url.clone(), LABEL_IMAGE_SIZE, LABEL_IMAGE_SIZE),
            text: BasicText::new(title.clone()).style(TextStyle::Title2),
            url,
            title,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Fixed image tint (RGB replaced, alpha kept). Without it the
    /// image stays normal/colorful as downloaded.
    pub fn tint(mut self, color: Color) -> Self {
        self.image.set_tint(Some(color));
        self
    }

    /// Fixed image tint, or `None` back to colorful.
    pub fn set_tint(&mut self, tint: Option<Color>) {
        self.image.set_tint(tint);
    }

    pub fn tint_value(&self) -> Option<Color> {
        self.image.tint_value()
    }

    /// Swap the image URL (restarts the download).
    pub fn set_image_url(&mut self, url: impl Into<String>) {
        let url = url.into();
        if url != self.url {
            self.url = url.clone();
            let tint = self.image.tint_value();
            let mut image = UrlImage::new(url, LABEL_IMAGE_SIZE, LABEL_IMAGE_SIZE);
            image.set_tint(tint);
            image.set_theme(self.dark);
            image.set_focused(self.focused);
            self.image = image;
        }
    }

    /// Retry the download after a failure.
    pub fn retry(&mut self) {
        self.image.retry();
    }

    /// Live theme for image states and title (in place: downloads
    /// keep running).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark == self.dark {
            return;
        }
        self.dark = dark;
        self.image.set_theme(dark);
        self.text.set_theme(mode);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.image.set_focused(focused);
        self.text.set_focused(focused);
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if title != self.title {
            self.title = title.clone();
            self.text.set_text(title);
        }
    }

    pub fn url_value(&self) -> &str {
        &self.url
    }

    pub fn title_value(&self) -> &str {
        &self.title
    }

    /// Image for state updates (retry, spinner color, ...).
    pub fn image_mut(&mut self) -> &mut UrlImage {
        &mut self.image
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl View for ImageLabel {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (iw, ih) = self.image.measure(fonts);
        let (tw, th) = self.text.measure(fonts);
        (iw + LABEL_GAP + tw, ih.max(th))
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        let (iw, ih) = self.image.measure(fonts);
        let (_, th) = self.text.measure(fonts);
        self.image.place(fonts, x, y + (h - ih) / 2.0, iw, ih);
        self.text.place(
            fonts,
            x + iw + LABEL_GAP,
            y + (h - th) / 2.0,
            (w - iw - LABEL_GAP).max(0.0),
            th,
        );
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.image.draw(scene, fonts, images);
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
    fn keeps_parts() {
        let label = ImageLabel::new("https://example.com/a.png", "Custom Title");
        assert_eq!(label.url_value(), "https://example.com/a.png");
        assert_eq!(label.title_value(), "Custom Title");
        assert_eq!(label.tint_value(), None);
    }

    #[test]
    fn tint_pins_fixed_color() {
        let blue = Color::from_rgb8(0x0a, 0x84, 0xff);
        let label = ImageLabel::new("https://example.com/a.png", "T").tint(blue);
        assert_eq!(label.tint_value(), Some(blue));
    }

    #[test]
    fn intrinsic_size_covers_image_and_text() {
        let mut fonts = FontSystem::new();
        let mut label = ImageLabel::new("https://example.com/a.png", "Custom Title");
        let (w, h) = label.measure(&mut fonts);
        assert!(w > LABEL_IMAGE_SIZE + LABEL_GAP);
        assert!(h >= LABEL_IMAGE_SIZE);
    }

    #[test]
    fn theme_keeps_download_state() {
        // In-place theme: no rebuild, so loading/error states and
        // tints survive per-frame theme calls.
        let mut label = ImageLabel::new("https://example.com/a.png", "T")
            .tint(Color::from_rgb8(0x0a, 0x84, 0xff));
        label.set_theme(ThemeMode::Dark);
        label.set_theme(ThemeMode::Light);
        assert_eq!(
            label.tint_value(),
            Some(Color::from_rgb8(0x0a, 0x84, 0xff))
        );
        assert_eq!(label.image_mut().state_value(), "loading");
    }
}
