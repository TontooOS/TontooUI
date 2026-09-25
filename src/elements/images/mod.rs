pub mod app;
pub mod overlay;
pub mod symbol;
pub mod url;

pub use app::AppImage;
pub use overlay::{ImageOverlay, OverlaySource};
pub use symbol::SFSymbolImage;
pub use url::UrlImage;

use std::path::PathBuf;

use vello::kurbo::Affine;
use vello::peniko::Color;

/// Box size for `SFSymbolImage` in logical px.
pub const IMAGE_SYMBOL_SIZE: f32 = 24.0;
/// Corner radius for raster frames in logical px.
pub const IMAGE_RADIUS: f32 = 12.0;
/// Caption and error text size in logical px.
pub const IMAGE_TEXT_SIZE: f32 = 13.0;
/// Badge symbol box in logical px.
pub const IMAGE_BADGE_SIZE: f32 = 20.0;
/// Placeholder fill for missing files (dark mode).
pub const IMAGE_PLACEHOLDER_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
/// Placeholder fill for missing files (light mode).
pub const IMAGE_PLACEHOLDER_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);

/// How a raster fills its frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// Scale to cover the frame, cropping the overflow (default).
    #[default]
    Cover,
    /// Scale to fit inside the frame, letterboxing the rest.
    Fit,
}

/// Drawn size and centered offset for a natural (`iw`, `ih`) image in
/// a (`w`, `h`) frame under `fit`. Returns `(draw_w, draw_h, dx, dy)`
/// in logical px relative to the frame origin.
pub(crate) fn fit_rect(iw: f32, ih: f32, w: f32, h: f32, fit: ImageFit) -> (f32, f32, f32, f32) {
    if iw <= 0.0 || ih <= 0.0 || w <= 0.0 || h <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let scale = match fit {
        ImageFit::Cover => (w / iw).max(h / ih),
        ImageFit::Fit => (w / iw).min(h / ih),
    };
    let (dw, dh) = (iw * scale, ih * scale);
    (dw, dh, (w - dw) / 2.0, (h - dh) / 2.0)
}

/// Resolve an app resource `name` to a file path. Priority: first
/// existing candidate from `resolve_resource_candidates`; falls back
/// to the override dir join so missing-file messages stay familiar.
pub(crate) fn resolve_resource_path(name: &str) -> PathBuf {
    for candidate in resolve_resource_candidates(name) {
        if candidate.exists() {
            return candidate;
        }
    }
    if let Ok(env) = std::env::var("APP_RESOURCES_DIR") {
        if !env.is_empty() {
            return PathBuf::from(env).join(name);
        }
    }
    PathBuf::from("assets").join(name)
}

/// Wrap a `draw_image` transform with a rotation around the physical
/// (`cx`, `cy`) center. Zero degrees return the base untouched, so
/// idle elements pay nothing.
pub(crate) fn spin_transform(base: Affine, cx: f64, cy: f64, degrees: f32) -> Affine {
    if degrees == 0.0 {
        base
    } else {
        Affine::translate((cx, cy))
            * Affine::rotate(degrees.to_radians() as f64)
            * Affine::translate((-cx, -cy))
            * base
    }
}

/// Candidate paths for an app resource `name`, in priority order.
pub(crate) fn resolve_resource_candidates(name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(env) = std::env::var("APP_RESOURCES_DIR") {
        if !env.is_empty() {
            let base = PathBuf::from(env);
            out.push(base.join(name));
            out.push(base.join(format!("{name}.png")));
        }
    }
    out.push(PathBuf::from("assets").join(name));
    out.push(PathBuf::from("assets").join(format!("{name}.png")));
    out.push(PathBuf::from(name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_fills_frame() {
        // Wide image in a square frame: scaled to the frame height,
        // overflowing horizontally and centered.
        let (dw, dh, dx, _dy) = fit_rect(200.0, 100.0, 100.0, 100.0, ImageFit::Cover);
        assert_eq!((dw, dh), (200.0, 100.0));
        assert_eq!(dx, -50.0);
    }

    #[test]
    fn fit_letterboxes() {
        // Wide image in a square frame: scaled to the frame width,
        // centered vertically.
        let (dw, dh, _dx, dy) = fit_rect(200.0, 100.0, 100.0, 100.0, ImageFit::Fit);
        assert_eq!((dw, dh), (100.0, 50.0));
        assert_eq!(dy, 25.0);
    }

    #[test]
    fn degenerate_inputs_stay_empty() {
        assert_eq!(fit_rect(0.0, 100.0, 100.0, 100.0, ImageFit::Cover), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(fit_rect(100.0, 100.0, 0.0, 100.0, ImageFit::Fit), (0.0, 0.0, 0.0, 0.0));
    }
}
