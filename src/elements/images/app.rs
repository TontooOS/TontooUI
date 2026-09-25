use std::any::Any;
use std::path::PathBuf;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Fill};

use super::super::layout::View;
use super::{
    IMAGE_PLACEHOLDER_DARK, IMAGE_PLACEHOLDER_LIGHT, IMAGE_RADIUS, ImageFit,
    fit_rect, resolve_resource_path,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Raster image from the app resources: `$APP_RESOURCES_DIR` override
/// (with and without `.png`), then the relative `assets/` dir (dev /
/// `cargo run`). Decoded without tinting, cached per path, drawn
/// cover-fit by default. Display-only (no mouse handling); a missing
/// file draws a theme placeholder box.
pub struct AppImage {
    name: String,
    width: f32,
    height: f32,
    fit: ImageFit,
    radius: f32,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl AppImage {
    pub fn new(name: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            name: name.into(),
            width: width.max(0.0),
            height: height.max(0.0),
            fit: ImageFit::Cover,
            radius: IMAGE_RADIUS,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// How the raster fills the frame: cover (crop, default) or fit
    /// (letterbox).
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
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

    pub fn set_fit(&mut self, fit: ImageFit) {
        self.fit = fit;
    }

    pub fn set_radius(&mut self, px: f32) {
        self.radius = px.max(0.0);
    }

    /// Resolved file path for the resource name.
    pub fn path(&self) -> PathBuf {
        resolve_resource_path(&self.name)
    }

    pub fn name_value(&self) -> &str {
        &self.name
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl View for AppImage {
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
        let path = self.path();
        match images.raster_file(&path, target) {
            Some((image, iw, ih)) => {
                let (dw, _dh, dx, dy) = fit_rect(
                    iw as f32,
                    ih as f32,
                    self.placed_w,
                    self.placed_h,
                    self.fit,
                );
                scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &frame);
                let s = (dw / iw as f32) as f64 * scale;
                let transform = Affine::translate((
                    (self.x + dx) as f64 * scale,
                    (self.y + dy) as f64 * scale,
                )) * Affine::scale(s);
                scene.draw_image(&image, transform);
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
        let _ = self.focused;
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
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut image = AppImage::new("logo", 200.0, 120.0);
        assert_eq!(image.measure(&mut fonts), (200.0, 120.0));
        image.place(&mut fonts, 0.0, 0.0, 200.0, 120.0);
        assert_eq!(image.rect(), (0.0, 0.0, 200.0, 120.0));
    }

    #[test]
    fn resolves_png_extension() {
        let image = AppImage::new("logo", 200.0, 120.0);
        let path = image.path();
        let name = path.to_string_lossy().into_owned();
        assert!(name.ends_with("logo") || name.ends_with("logo.png"), "{name}");
    }

    #[test]
    fn clamps_negative_values() {
        let mut image = AppImage::new("logo", -10.0, -5.0).radius(-4.0);
        let mut fonts = FontSystem::new();
        assert_eq!(image.measure(&mut fonts), (0.0, 0.0));
        image.set_radius(-2.0);
        image.place(&mut fonts, 0.0, 0.0, 100.0, 100.0);
    }
}
