use std::ops::Range;

use vello::Scene;
use vello::peniko::{Brush, Color};

// CoreText is the only text stack: shaping, layout, crisp rendering,
// caret mapping and decorations all come from the `coretext` crate.
// This module keeps the `FontSystem` name (plus `SolidBrush`,
// `RichSpan`, `draw_layout`) so existing elements keep working, but
// every layout is a CoreText `CTFrame` and every draw goes through
// the CoreText crisp pipeline (scale-quantized layout, pixel-snapped
// origin, hinted glyphs).
pub use coretext::{
    AttrSpan, AttributedString, CTFont, CTFontDescriptor, CTFontStyle, CTFontWeight,
    CTFrame, CTFramesetter, CTLine, CTLineBreakMode, CTParagraphStyle, CTTextAlignment,
    CrispOpts, Decoration, FontRegistry, Shadow, StrokeStyle, caret_geometry, caret_to_point,
    column_at_x, decorations, draw_frame, draw_frame_gradient, draw_frame_mapped, draw_line,
    gradient_brush, hit_byte, line_for_caret, link_at, point_to_caret, run_spans,
    selection_rects, word_range,
};

/// Minimal solid-color brush stored in CoreText layouts. Converted to
/// a Peniko brush when drawing into a Vello scene.
pub type SolidBrush = coretext::typeset::CtBrush;

/// Font loading and text layout, backed by CoreText. Wraps a
/// `CTFramesetter` (system fonts, so SF Pro resolves on TontooOS).
pub struct FontSystem {
    setter: CTFramesetter,
    /// Device pixel ratio of the target window. Layout is built in
    /// physical pixels so glyphs stay crisp.
    pub scale: f32,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            setter: CTFramesetter::new(1.0),
            scale: 1.0,
        }
    }

    fn sync(&mut self) {
        self.setter.set_scale(self.scale);
    }

    /// Direct access to the CoreText framesetter (caret mapping,
    /// custom frames, measurement).
    pub fn framesetter(&mut self) -> &mut CTFramesetter {
        self.sync();
        &mut self.setter
    }

    /// Lay out `content` at `size` logical px with regular weight.
    /// `max_width` is in logical px; `None` disables wrapping.
    pub fn layout_text(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
    ) -> CTFrame {
        self.layout_text_weighted(content, size, color, 400.0, max_width)
    }

    /// Lay out `content` with an explicit font weight (400 regular,
    /// 600 semibold, ...). `size` and `max_width` are logical px.
    pub fn layout_text_weighted(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        weight: f32,
        max_width: Option<f32>,
    ) -> CTFrame {
        self.sync();
        let paragraph = CTParagraphStyle::default();
        self.setter
            .create_plain_frame(content, &paragraph, size, color, weight, max_width)
    }

    /// Lay out `content` with `alignment` (applied when wrapping,
    /// like before: unbounded text keeps leading alignment and the
    /// element shifts the draw origin itself).
    pub fn layout_text_aligned(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        weight: f32,
        max_width: Option<f32>,
        alignment: CTTextAlignment,
    ) -> CTFrame {
        self.sync();
        let mut paragraph = CTParagraphStyle::default();
        if max_width.is_some() {
            paragraph = paragraph.alignment(alignment);
        }
        self.setter
            .create_plain_frame(content, &paragraph, size, color, weight, max_width)
    }

    /// Physical width/height of a finished frame. Divide by
    /// `FontSystem::scale` for logical px.
    pub fn layout_size(frame: &CTFrame) -> (f32, f32) {
        let (w, h) = frame.size();
        let scale = frame.scale();
        (w * scale, h * scale)
    }

    /// Lay out `content` with inline `spans` (byte ranges into
    /// `content`) at `size` logical px. Bold pins weight 700,
    /// `monospace` switches the run to the monospace generic family,
    /// decorations take the span color or fall back to the base
    /// `color`. Out-of-bounds ranges are clamped, empty ranges
    /// skipped.
    pub fn layout_rich_text(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
        spans: &[RichSpan],
    ) -> CTFrame {
        self.layout_rich_text_aligned(content, size, color, max_width, spans, CTTextAlignment::Leading)
    }

    /// Rich layout with `alignment` (applied when wrapping).
    pub fn layout_rich_text_aligned(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
        spans: &[RichSpan],
        alignment: CTTextAlignment,
    ) -> CTFrame {
        self.sync();
        let mut paragraph = CTParagraphStyle::default();
        if max_width.is_some() {
            paragraph = paragraph.alignment(alignment);
        }
        let mut string = AttributedString::new(content);
        for span in spans {
            let start = span.range.start.min(content.len());
            let end = span.range.end.min(content.len());
            if start >= end {
                continue;
            }
            let mut attr = AttrSpan::new(start..end);
            attr.bold = span.bold;
            attr.italic = span.italic;
            attr.monospace = span.monospace;
            attr.color = span.color.map(color_to_rgba8);
            attr.underline = span.underline;
            attr.underline_color = span.underline_color.map(color_to_rgba8);
            attr.strikethrough = span.strikethrough;
            attr.strikethrough_color = span.strikethrough_color.map(color_to_rgba8);
            string.push_span(attr);
        }
        self.setter
            .create_frame(&string, &paragraph, size, color, 400.0, max_width)
    }
}

/// Inline span style for `layout_rich_text`: a byte `range` into the
/// content plus style flags. `underline_color`/`strikethrough_color`
/// fall back to `color`, then to the layout base color.
#[derive(Clone, Debug, Default)]
pub struct RichSpan {
    pub range: Range<usize>,
    pub bold: bool,
    pub italic: bool,
    pub monospace: bool,
    pub color: Option<Color>,
    pub underline: bool,
    pub underline_color: Option<Color>,
    pub strikethrough: bool,
    pub strikethrough_color: Option<Color>,
}

fn color_to_rgba8(color: Color) -> [u8; 4] {
    let c = color.to_rgba8();
    [c.r, c.g, c.b, c.a]
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a finished frame into `scene` at logical position (`x`, `y`)
/// through the CoreText crisp pipeline (pixel-snapped origin,
/// hinted glyphs).
pub fn draw_layout(scene: &mut Scene, frame: &CTFrame, x: f32, y: f32, scale: f32) {
    draw_frame(
        scene,
        frame,
        x,
        y,
        CrispOpts {
            scale,
            hint: true,
            subpixel: true,
        },
    );
}

/// Gradient twin of `draw_layout`: every run paints `brush`.
pub fn draw_with_brush(
    scene: &mut Scene,
    frame: &CTFrame,
    x: f32,
    y: f32,
    scale: f32,
    brush: &Brush,
) {
    draw_frame_gradient(
        scene,
        frame,
        x,
        y,
        CrispOpts {
            scale,
            hint: true,
            subpixel: true,
        },
        brush,
    );
}
