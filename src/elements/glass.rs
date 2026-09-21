use std::any::Any;

use vello::Scene;
use super::layout::View;
use vello::kurbo::{Affine, Point, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

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

/// Liquid glass container: frosted body, liquid bevel rim with specular
/// top light and depth shade, chromatic edge split and a soft shadow.
/// Optional content draws on top.
///
/// True backdrop blur and refraction need the compositor (it owns the
/// desktop pixels behind a transparent window); this kit does everything
/// downstream of that: tint, bevel, rim light, chroma and shadow.
pub struct GlassContainer {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    tint: Color,
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

    pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T> {
        self.child.as_mut()?.as_any_mut().downcast_mut::<T>()
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
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

        // Soft shadow under the glass.
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 64),
            radius,
            12.0 * scale,
        );

        // Frosted body.
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.tint),
            None,
            &body,
        );

        // Liquid bevel: specular top melting into depth shade at the bottom.
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
                color: GLASS_SPECULAR.into(),
            },
            ColorStop {
                offset: 0.35,
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
