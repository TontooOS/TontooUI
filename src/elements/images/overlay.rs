use std::any::Any;
use std::path::PathBuf;

use vello::Scene;
use vello::kurbo::{Affine, Point, RoundedRect};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use super::super::animation::Spin;
use super::super::layout::View;
use super::{
    IMAGE_BADGE_SIZE, IMAGE_PLACEHOLDER_DARK, IMAGE_PLACEHOLDER_LIGHT, IMAGE_RADIUS,
    IMAGE_TEXT_SIZE, ImageFit, fit_rect, resolve_resource_path, spin_transform,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Raster source of the overlay card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlaySource {
    File(PathBuf),
    Resource(String),
}

/// Photo card: raster image (file or app resource) with a bottom
/// gradient, a caption and an optional SF Symbol badge in the top
/// right corner. Display-only (no mouse handling); a missing file
/// draws a theme placeholder box with the caption still on top.
pub struct ImageOverlay {
    source: OverlaySource,
    width: f32,
    height: f32,
    radius: f32,
    caption: Option<String>,
    badge: Option<String>,
    badge_color: Color,
    dark: bool,
    focused: bool,
    spin_deg: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl ImageOverlay {
    fn with_source(source: OverlaySource, width: f32, height: f32) -> Self {
        Self {
            source,
            width: width.max(0.0),
            height: height.max(0.0),
            radius: IMAGE_RADIUS,
            caption: None,
            badge: None,
            badge_color: Color::WHITE,
            dark: true,
            focused: true,
            spin_deg: 0.0,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Card from a file path on disk.
    pub fn file(path: impl Into<PathBuf>, width: f32, height: f32) -> Self {
        Self::with_source(OverlaySource::File(path.into()), width, height)
    }

    /// Card from an app resource name (`$APP_RESOURCES_DIR`, then
    /// `assets/`, with and without `.png`).
    pub fn resource(name: impl Into<String>, width: f32, height: f32) -> Self {
        Self::with_source(OverlaySource::Resource(name.into()), width, height)
    }

    /// Caption line over the bottom gradient (`None` hides it).
    pub fn caption(mut self, text: impl Into<String>) -> Self {
        self.caption = Some(text.into());
        self
    }

    /// SF Symbol badge in the top right corner (`None` hides it).
    pub fn badge(mut self, symbol: impl Into<String>) -> Self {
        self.badge = Some(symbol.into());
        self
    }

    /// Badge glyph color (default white).
    pub fn badge_color(mut self, color: Color) -> Self {
        self.badge_color = color;
        self
    }

    /// Corner radius in logical px (clamped to >= 0).
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = px.max(0.0);
        self
    }

    /// Live theme for the missing-file placeholder.
    pub fn set_theme(&mut self, dark: bool) {
        self.dark = dark;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_caption(&mut self, text: Option<String>) {
        self.caption = text;
    }

    pub fn set_badge(&mut self, symbol: Option<String>) {
        self.badge = symbol;
    }

    pub fn set_badge_color(&mut self, color: Color) {
        self.badge_color = color;
    }

    pub fn set_radius(&mut self, px: f32) {
        self.radius = px.max(0.0);
    }

    pub fn source_value(&self) -> &OverlaySource {
        &self.source
    }

    pub fn caption_value(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    pub fn badge_value(&self) -> Option<&str> {
        self.badge.as_deref()
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn raster_path(&self) -> Option<PathBuf> {
        match &self.source {
            OverlaySource::File(path) => Some(path.clone()),
            OverlaySource::Resource(name) => Some(resolve_resource_path(name)),
        }
    }
}

impl View for ImageOverlay {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.width, self.height)
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
        let px = |v: f32| v as f64 * scale;
        let radius = self.radius.min(self.placed_w / 2.0).min(self.placed_h / 2.0).max(0.0);
        let frame = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            px(radius),
        );
        // Crisp supersampling: upload at ~2x the display size.
        let target = (self.placed_w.max(self.placed_h) * fonts.scale * 2.0)
            .ceil()
            .max(1.0) as u32;
        let loaded = self.raster_path().and_then(|path| {
            images
                .raster_file(&path, target)
                .map(|(image, iw, ih)| (image, iw as f32, ih as f32))
        });
        let has_image = loaded.is_some();
        match loaded {
            Some((image, iw, ih)) => {
                let (dw, _dh, dx, dy) =
                    fit_rect(iw, ih, self.placed_w, self.placed_h, ImageFit::Cover);
                scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &frame);
                let s = (dw / iw) as f64 * scale;
                let base = Affine::translate((
                    (self.x + dx) as f64 * scale,
                    (self.y + dy) as f64 * scale,
                )) * Affine::scale(s);
                // Only the photo spins; gradient, caption and badge
                // stay put.
                let transform = spin_transform(
                    base,
                    (self.x + self.placed_w / 2.0) as f64 * scale,
                    (self.y + self.placed_h / 2.0) as f64 * scale,
                    self.spin_deg,
                );
                scene.draw_image(&image, transform);
                // Bottom gradient so the caption stays readable.
                let gradient = Gradient::new_linear(
                    Point::new(px(self.x), px(self.y + self.placed_h * 0.45)),
                    Point::new(px(self.x), px(self.y + self.placed_h)),
                )
                .with_stops([
                    ColorStop {
                        offset: 0.0,
                        color: Color::TRANSPARENT.into(),
                    },
                    ColorStop {
                        offset: 1.0,
                        color: Color::from_rgba8(0, 0, 0, 170).into(),
                    },
                ]);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Gradient(gradient),
                    None,
                    &frame,
                );
                scene.pop_layer();
            }
            None => {
                let fill = if self.dark {
                    IMAGE_PLACEHOLDER_DARK
                } else {
                    IMAGE_PLACEHOLDER_LIGHT
                };
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fill), None, &frame);
            }
        }
        if let Some(badge) = self.badge.clone() {
            let target = (IMAGE_BADGE_SIZE * fonts.scale * 2.0).ceil().max(1.0) as u32;
            let tint = if self.focused {
                self.badge_color
            } else {
                desaturate(self.badge_color)
            };
            if let Some((image, iw, ih)) = images.get(&badge, tint, target) {
                let s = (IMAGE_BADGE_SIZE / iw as f32)
                    .min(IMAGE_BADGE_SIZE / ih as f32);
                let ix = self.x + self.placed_w - 12.0 - IMAGE_BADGE_SIZE;
                let iy = self.y + 12.0;
                let transform = Affine::translate((ix as f64 * scale, iy as f64 * scale))
                    * Affine::scale(s as f64 * scale);
                scene.draw_image(&image, transform);
            }
        }
        if let Some(caption) = self.caption.clone() {
            // White over the photo gradient; theme text over the
            // flat placeholder when the file is missing.
            let tint = if has_image {
                Color::WHITE
            } else if self.dark {
                Color::from_rgb8(0xd8, 0xd9, 0xd9)
            } else {
                Color::from_rgb8(0x27, 0x27, 0x27)
            };
            let layout = fonts.layout_text(
                &caption,
                IMAGE_TEXT_SIZE,
                tint,
                Some(self.placed_w - 24.0),
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + 12.0,
                self.y + self.placed_h - 12.0 - th / fonts.scale,
                fonts.scale,
            );
            let _ = tw;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Spin for ImageOverlay {
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
        let mut card = ImageOverlay::resource("photo", 240.0, 140.0)
            .caption("Sunset")
            .badge("star.fill");
        assert_eq!(card.measure(&mut fonts), (240.0, 140.0));
        card.place(&mut fonts, 0.0, 0.0, 240.0, 140.0);
        assert_eq!(card.rect(), (0.0, 0.0, 240.0, 140.0));
        assert_eq!(card.caption_value(), Some("Sunset"));
        assert_eq!(card.badge_value(), Some("star.fill"));
    }

    #[test]
    fn file_source_keeps_path() {
        let card = ImageOverlay::file("/tmp/photo.png", 240.0, 140.0);
        assert_eq!(
            card.source_value(),
            &OverlaySource::File(PathBuf::from("/tmp/photo.png"))
        );
        assert_eq!(card.caption_value(), None);
    }

    #[test]
    fn clamps_negative_values() {
        let card = ImageOverlay::resource("photo", -10.0, -5.0).radius(-4.0);
        let mut fonts = FontSystem::new();
        let mut card = card;
        assert_eq!(card.measure(&mut fonts), (0.0, 0.0));
    }
}
