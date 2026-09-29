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

/// Drop shadow layers: (y offset, blur, alpha). The largest reach
/// (offset + 1.25 x blur, since vello blurs with std_dev = blur / 2
/// and fades out at ~2.5 x std_dev) must stay inside the 24 px margin
/// or the shadow clips with a hard edge at the window border.
const SHADOWS: [(f32, f32, u8); 3] = [(2.0, 4.0, 38), (4.0, 12.0, 31), (6.0, 12.0, 20)];

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

/// Rounded body shape in physical px. Content must be clipped to this
/// (see `window.rs`) or square views spill over the rounded corners and
/// the frame strokes no longer line up with the visible edge.
pub fn body_shape(width: u32, height: u32, scale: f32) -> RoundedRect {
    let (rect, radius) = body_rect(width, height, scale);
    RoundedRect::from_rect(rect, radius)
}

/// Request a transparency-capable composite alpha mode for `surface` and
/// reconfigure it. vello configures `Auto`, which wgpu resolves to `Opaque`
/// first: on Wayland the 24 px margin then renders solid black with square
/// outer corners instead of a soft round shadow over the desktop. Prefers
/// `PreMultiplied`, falls back to `PostMultiplied`; when neither is
/// advertised the surface keeps its current mode. Persists across resizes
/// because vello reconfigures from the stored `config`.
pub fn ensure_transparent_alpha(
    context: &vello::util::RenderContext,
    surface: &mut vello::util::RenderSurface,
) {
    let adapter = context.devices[surface.dev_id].adapter();
    let caps = surface.surface.get_capabilities(adapter);
    let mode = [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
    ]
    .into_iter()
    .find(|mode| caps.alpha_modes.contains(mode));
    if let Some(mode) = mode {
        surface.config.alpha_mode = mode;
        let device = &context.devices[surface.dev_id].device;
        surface.surface.configure(device, &surface.config);
    }
}

/// Draw behind content: layered drop shadows plus the rounded body.
/// `width`/`height` are physical pixels. `None` skips shadows and body for
/// fully transparent windows (glass demos); the frame lines still draw.
pub fn draw_behind(
    scene: &mut Scene,
    width: u32,
    height: u32,
    scale: f32,
    body: Option<Color>,
) {
    let Some(body) = body else {
        return;
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadows_fit_margin() {
        // vello blurs with std_dev = blur / 2 and fades out at ~2.5 x
        // std_dev: a larger reach clips with a hard edge at the window
        // border and reads as a square black bar.
        for (dy, blur, _) in SHADOWS {
            let reach = dy + 2.5 * (blur / 2.0);
            assert!(
                reach <= MARGIN,
                "shadow reach {reach} exceeds margin {MARGIN}"
            );
        }
    }

    #[test]
    fn body_shape_matches_margin() {
        use vello::kurbo::Shape;
        let bb = body_shape(800, 600, 1.0).bounding_box();
        assert_eq!((bb.x0, bb.y0), (MARGIN as f64, MARGIN as f64));
        assert_eq!((bb.x1, bb.y1), (800.0 - MARGIN as f64, 600.0 - MARGIN as f64));
    }
}
