use std::any::Any;

use vello::Scene;
use super::layout::View;
use vello::kurbo::{Affine, Point, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use crate::renderer::backdrop::{fill_frosted_glass, fill_lens_glass};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Frost tint for dark mode glass (white glow over the backdrop).
pub const GLASS_TINT_DARK: Color = Color::from_rgba8(255, 255, 255, 26);
/// Frost tint for light mode glass (darkening over the backdrop).
pub const GLASS_TINT_LIGHT: Color = Color::from_rgba8(0, 0, 0, 20);
/// Specular top light running into the bevel.
pub const GLASS_SPECULAR: Color = Color::from_rgba8(255, 255, 255, 115);
/// Depth shade pooling at the bottom of the bevel.
pub const GLASS_DEPTH: Color = Color::from_rgba8(0, 0, 0, 46);
/// Chromatic rim split, red side.
pub const GLASS_CHROMA_RED: Color = Color::from_rgba8(255, 90, 120, 30);
/// Chromatic rim split, cyan side.
pub const GLASS_CHROMA_CYAN: Color = Color::from_rgba8(90, 200, 255, 30);
/// Width of the frosted edge band in logical px for small glass.
/// Large glass (see `GLASS_LARGE_MIN_SIDE`) uses `GLASS_EDGE_WIDTH_LARGE`.
pub const GLASS_EDGE_WIDTH: f32 = 2.0;
/// Edge band width in logical px once the glass counts as large.
pub const GLASS_EDGE_WIDTH_LARGE: f32 = 3.0;
/// Minimum smaller side in logical px from which a glass counts as large
/// (gets the 3 px edge band instead of 2 px).
pub const GLASS_LARGE_MIN_SIDE: f32 = 200.0;

/// Edge band width in logical px for a glass body with the given smaller
/// side: 2 px normally, 3 px once large. Shared by the container and the
/// small knobs so every glass rim stays hairline.
pub fn glass_edge_width(min_side: f32) -> f32 {
    if min_side >= GLASS_LARGE_MIN_SIDE {
        GLASS_EDGE_WIDTH_LARGE
    } else {
        GLASS_EDGE_WIDTH
    }
}
/// Lens zoom of the clear center: below 1.0 the backdrop behind the glass
/// shrinks (minify), above 1.0 it grows. Default minifies slightly.
pub const GLASS_ZOOM: f64 = 0.80;
/// Blur veil alpha for the frosted glass type: light frost over the whole
/// body on top of the lens, so the edge stays strongest.
pub const GLASS_FROST_VEIL: f32 = 0.35;

/// Glass finish: how the backdrop shows through the body.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassType {
    /// Clear minified center, blur only on the narrow edge band.
    #[default]
    Lens,
    /// Same lens plus a light blur veil over the whole body: frost
    /// everywhere, strongest at the edge, light inside.
    Frosted,
}

/// Liquid glass container: clear minified lens center, frosted edge band,
/// liquid bevel rim with specular top light and depth shade, chromatic edge
/// split and a soft shadow. Optional content draws on top.
///
/// When the shell runs a backdrop pass (`App::wants_backdrop`), the center
/// samples the sharp in-app capture slightly minified so content behind
/// the glass shows through shrunk, while only a narrow rim band samples
/// the blurred capture. Desktop pixels behind a transparent window still
/// need the compositor; without a backdrop the body is only the frost tint.
pub struct GlassContainer {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    tint: Color,
    specular: Color,
    grain: bool,
    focused: bool,
    glass_type: GlassType,
    child: Option<Box<dyn View>>,
}

impl GlassContainer {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 180.0,
            radius: 24.0,
            tint: GLASS_TINT_DARK,
            specular: GLASS_SPECULAR,
            grain: false,
            focused: true,
            glass_type: GlassType::Lens,
            child: None,
        }
    }

    pub fn bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.x = x;
        self.y = y;
        self.width = width.max(0.0);
        self.height = height.max(0.0);
        self
    }

    pub fn radius(mut self, px: f32) -> Self {
        self.radius = px.max(0.0);
        self
    }

    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Glass finish: `GlassType::Lens` (default, clear center) or
    /// `GlassType::Frosted` (light blur everywhere, strongest at the edge).
    pub fn glass_type(mut self, glass_type: GlassType) -> Self {
        self.glass_type = glass_type;
        self
    }

    pub fn content(mut self, child: impl View + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x;
        self.y = y;
        self.width = width.max(0.0);
        self.height = height.max(0.0);
    }

    pub fn set_tint(&mut self, tint: Color) {
        self.tint = tint;
    }

    pub fn set_glass_type(&mut self, glass_type: GlassType) {
        self.glass_type = glass_type;
    }

    /// Inactive windows desaturate the frost like the rest of the palette.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Live glass stage from the system setting. Less glass is mostly
    /// opaque mode color with a brighter rim and a grainy black outer
    /// edge; balanced glass sits in the middle; much glass keeps the dark
    /// look unchanged and goes lighter in light mode.
    pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount) {
        let dark = mode == ThemeMode::Dark;
        let (tint, specular, grain) = match (amount, dark) {
            (GlassAmount::Less, true) => (
                Color::from_rgba8(10, 10, 12, 150),
                Color::from_rgba8(255, 255, 255, 160),
                true,
            ),
            (GlassAmount::Less, false) => (
                Color::from_rgba8(255, 255, 255, 150),
                Color::from_rgba8(255, 255, 255, 200),
                true,
            ),
            (GlassAmount::Much, false) => (
                Color::from_rgba8(255, 255, 255, 14),
                GLASS_SPECULAR,
                false,
            ),
            _ => (
                if dark { GLASS_TINT_DARK } else { GLASS_TINT_LIGHT },
                GLASS_SPECULAR,
                false,
            ),
        };
        self.tint = tint;
        self.specular = specular;
        self.grain = grain;
    }

    pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T> {
        self.child.as_mut()?.as_any_mut().downcast_mut::<T>()
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        // Capture pass: skip the whole container (body + children) so the
        // blur sees only what sits behind the glass.
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let radius = self.radius as f64 * scale;
        let rect = vello::kurbo::Rect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + self.height),
        );
        let body = RoundedRect::from_rect(rect, radius);

        // Soft shadow under the glass: tight and subtle so the outer
        // edge never reads as a thick dark rim.
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 48),
            radius,
            8.0 * scale,
        );

        // Backdrop finish: clear lens center with a blurred rim band, or
        // the same lens plus a light blur veil everywhere for Frosted.
        // Tiny bodies fall back to a full blur fill (see backdrop).
        let edge_logical = glass_edge_width(self.width.min(self.height));
        let band = edge_logical as f64 * scale;
        match self.glass_type {
            GlassType::Lens => {
                fill_lens_glass(scene, images, &rect, radius, GLASS_ZOOM, band);
            }
            GlassType::Frosted => {
                fill_frosted_glass(
                    scene,
                    images,
                    &rect,
                    radius,
                    GLASS_ZOOM,
                    band,
                    GLASS_FROST_VEIL,
                );
            }
        }

        // Frosted body (gray when the window is inactive).
        let tint = if self.focused {
            self.tint
        } else {
            desaturate(self.tint)
        };
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(tint),
            None,
            &body,
        );

        // Liquid bevel: specular top light and depth shade at the bottom,
        // transparent along the sides (long transparent mid stops so the
        // vertical gradient leaves the flanks clean).
        let bevel = RoundedRect::new(
            rect.x0 + 1.0 * scale,
            rect.y0 + 1.0 * scale,
            rect.x1 - 1.0 * scale,
            rect.y1 - 1.0 * scale,
            (radius - 1.0 * scale).max(0.0),
        );
        let bevel_brush = Gradient::new_linear(
            Point::new(rect.x0, rect.y0),
            Point::new(rect.x0, rect.y1),
        )
        .with_stops([
            ColorStop {
                offset: 0.0,
                color: self.specular.into(),
            },
            ColorStop {
                offset: 0.22,
                color: Color::TRANSPARENT.into(),
            },
            ColorStop {
                offset: 0.78,
                color: Color::TRANSPARENT.into(),
            },
            ColorStop {
                offset: 1.0,
                color: GLASS_DEPTH.into(),
            },
        ]);
        scene.stroke(
            &Stroke::new(2.0 * scale),
            Affine::IDENTITY,
            &Brush::Gradient(bevel_brush),
            None,
            &bevel,
        );

        // Chromatic rim split: red outside, cyan inside the crisp edge.
        let red = RoundedRect::new(
            rect.x0 - 0.75 * scale,
            rect.y0 - 0.75 * scale,
            rect.x1 + 0.75 * scale,
            rect.y1 + 0.75 * scale,
            radius + 0.75 * scale,
        );
        scene.stroke(
            &Stroke::new(1.0 * scale),
            Affine::IDENTITY,
            &Brush::Solid(GLASS_CHROMA_RED),
            None,
            &red,
        );
        let cyan = RoundedRect::new(
            rect.x0 + 0.75 * scale,
            rect.y0 + 0.75 * scale,
            rect.x1 - 0.75 * scale,
            rect.y1 - 0.75 * scale,
            (radius - 0.75 * scale).max(0.0),
        );
        scene.stroke(
            &Stroke::new(1.0 * scale),
            Affine::IDENTITY,
            &Brush::Solid(GLASS_CHROMA_CYAN),
            None,
            &cyan,
        );

        // Grainy black outer edge (less glass): scattered speckles outside
        // the crisp rim.
        if self.grain {
            let outer = RoundedRect::new(
                rect.x0 - 1.5 * scale,
                rect.y0 - 1.5 * scale,
                rect.x1 + 1.5 * scale,
                rect.y1 + 1.5 * scale,
                radius + 1.5 * scale,
            );
            scene.stroke(
                &Stroke::new(2.0 * scale).with_dashes(0.0, [1.5 * scale, 2.5 * scale]),
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(0, 0, 0, 110)),
                None,
                &outer,
            );
        }

        // Optional content on top (placed by `place`).
        if let Some(child) = self.child.as_mut() {
            child.draw(scene, fonts, images);
        }
    }
}

impl Default for GlassContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl View for GlassContainer {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.width, self.height)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.set_bounds(x, y, w, h);
        if let Some(child) = self.child.as_mut() {
            child.place(fonts, x, y, w, h);
        }
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
