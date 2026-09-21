use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use super::window::WINDOW_CORNER_RADIUS;

/// Margin between screen edge and window body in logical px.
pub const MARGIN: f32 = 24.0;

/// Thin 1px edge around the body.
pub const EDGE: Color = Color::from_rgba8(255, 255, 255, 36);
/// Inner top highlight, fading down.
pub const INNER_TOP: Color = Color::from_rgba8(255, 255, 255, 71);
/// Outer 1px outline ring.
pub const OUTER: Color = Color::from_rgba8(0, 0, 0, 140);

/// Drop shadow layers: (y offset, blur, alpha). Shrunk proportionally from
/// the UIKit spec so the largest reach (offset + blur) fits the 24 px margin
/// instead of clipping at the window edge.
const SHADOWS: [(f32, f32, u8); 3] = [(2.0, 4.0, 38), (4.0, 12.0, 31), (7.0, 16.0, 20)];

/// Logical content rect inside the frame: (x, y, width, height).
pub fn content_rect(width: f32, height: f32) -> (f32, f32, f32, f32) {
    (MARGIN, MARGIN, width - MARGIN * 2.0, height - MARGIN * 2.0)
}

fn body_rect(width: u32, height: u32, scale: f32) -> (Rect, f64) {
    let s = scale as f64;
    let radius = WINDOW_CORNER_RADIUS as f64 * s;
    let rect = Rect::new(
        MARGIN as f64 * s,
        MARGIN as f64 * s,
        width as f64 - MARGIN as f64 * s,
        height as f64 - MARGIN as f64 * s,
    );
    (rect, radius)
}

/// Draw behind content: layered drop shadows plus the rounded body.
/// `width`/`height` are physical pixels.
pub fn draw_behind(scene: &mut Scene, width: u32, height: u32, scale: f32, body: Color) {
    let s = scale as f64;
    let (body_rect, radius) = body_rect(width, height, scale);

    for (dy, blur, alpha) in SHADOWS {
        let rect = Rect::new(
            body_rect.x0,
            body_rect.y0 + dy as f64 * s,
            body_rect.x1,
            body_rect.y1 + dy as f64 * s,
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, alpha),
            radius,
            blur as f64 * s / 2.0,
        );
    }

    let shape = RoundedRect::new(
        body_rect.x0,
        body_rect.y0,
        body_rect.x1,
        body_rect.y1,
        radius,
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(body),
        None,
        &shape,
    );
}

/// Draw above content: inner top highlight, 1px edge and outer 1px outline.
/// Runs after the view so bars and fields can never cover the frame.
pub fn draw_frame(scene: &mut Scene, width: u32, height: u32, scale: f32) {
    let s = scale as f64;
    let (body_rect, radius) = body_rect(width, height, scale);

    // Inner top highlight: full inner ring with a top-to-transparent gradient.
    // Inset 2 px so it never overlaps the 1 px edge stroke; overlap would
    // make the top edge brighter than the rest.
    let inset = 2.0 * s;
    let inner = RoundedRect::new(
        body_rect.x0 + inset - 0.5 * s,
        body_rect.y0 + inset - 0.5 * s,
        body_rect.x1 - inset + 0.5 * s,
        body_rect.y1 - inset + 0.5 * s,
        (radius - inset + 0.5 * s).max(0.0),
    );
    let gradient = Gradient::new_linear(
        Point::new(body_rect.x0, body_rect.y0),
        Point::new(body_rect.x0, body_rect.y1),
    )
    .with_stops([
        ColorStop {
            offset: 0.0,
            color: INNER_TOP.into(),
        },
        ColorStop {
            offset: 0.35,
            color: Color::TRANSPARENT.into(),
        },
    ]);
    scene.stroke(
        &Stroke::new(1.0 * s),
        Affine::IDENTITY,
        &Brush::Gradient(gradient),
        None,
        &inner,
    );

    // 1px edge centered on the body outline.
    let shape = RoundedRect::new(
        body_rect.x0,
        body_rect.y0,
        body_rect.x1,
        body_rect.y1,
        radius,
    );
    scene.stroke(
        &Stroke::new(1.0 * s),
        Affine::IDENTITY,
        &Brush::Solid(EDGE),
        None,
        &shape,
    );

    // Outer 1px outline ring.
    let outer = RoundedRect::new(
        body_rect.x0 - 1.0 * s,
        body_rect.y0 - 1.0 * s,
        body_rect.x1 + 1.0 * s,
        body_rect.y1 + 1.0 * s,
        radius + 1.0 * s,
    );
    scene.stroke(
        &Stroke::new(1.0 * s),
        Affine::IDENTITY,
        &Brush::Solid(OUTER),
        None,
        &outer,
    );
}
