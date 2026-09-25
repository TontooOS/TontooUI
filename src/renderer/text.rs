use std::ops::Range;

use parley::{
    Alignment, AlignmentOptions, FontContext, FontStyle, FontWeight, GenericFamily, Layout,
    LayoutContext, LineHeight, PositionedLayoutItem, StyleProperty,
};
use vello::Scene;
use vello::peniko::{Brush, Color, Fill};

/// Minimal solid-color brush used as the `Brush` generic for Parley layouts.
/// Converted to a Peniko brush when drawing into a Vello scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolidBrush {
    pub color: Color,
}

impl Default for SolidBrush {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
        }
    }
}

/// Font loading and text layout. Wraps a Parley `FontContext` (system fonts,
/// so SF Pro resolves on TontooOS) and a `LayoutContext`.
pub struct FontSystem {
    font_cx: FontContext,
    layout_cx: LayoutContext<SolidBrush>,
    /// Device pixel ratio of the target window. Layout is built in physical
    /// pixels so glyphs stay crisp.
    pub scale: f32,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            scale: 1.0,
        }
    }

    /// Lay out `content` at `size` logical px with regular weight. `max_width`
    /// is in logical px; `None` disables wrapping.
    pub fn layout_text(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
    ) -> Layout<SolidBrush> {
        self.layout_text_weighted(content, size, color, 400.0, max_width)
    }

    /// Lay out `content` with an explicit font weight (400 regular, 600
    /// semibold, ...).
    ///
    /// `size` and `max_width` are logical px. The builder receives the
    /// window display scale so Parley quantizes glyph positions to
    /// physical pixel boundaries (crisp text, no blur). Line breaking
    /// still takes physical px, hence `max_width * scale`.
    pub fn layout_text_weighted(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        weight: f32,
        max_width: Option<f32>,
    ) -> Layout<SolidBrush> {
        let mut builder =
            self.layout_cx
                .ranged_builder(&mut self.font_cx, content, self.scale, true);
        builder.push_default(StyleProperty::Brush(SolidBrush { color }));
        builder.push_default(GenericFamily::SystemUi);
        builder.push_default(LineHeight::FontSizeRelative(1.25));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
        let mut layout = builder.build(content);
        layout.break_all_lines(max_width.map(|w| w * self.scale));
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
    }

    /// Physical width/height of a finished layout. Divide by
    /// `FontSystem::scale` for logical px.
    pub fn layout_size(layout: &Layout<SolidBrush>) -> (f32, f32) {
        (layout.width(), layout.height())
    }

    /// Lay out `content` with inline `spans` (byte ranges into `content`)
    /// at `size` logical px. Additive companion to `layout_text_weighted`
    /// (which stays unchanged): bold pins weight 700, `monospace` switches
    /// the run to the monospace generic family, decorations take the
    /// span color or fall back to the base `color`. Out-of-bounds ranges
    /// are clamped, empty ranges skipped.
    pub fn layout_rich_text(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
        spans: &[RichSpan],
    ) -> Layout<SolidBrush> {
        let mut builder =
            self.layout_cx
                .ranged_builder(&mut self.font_cx, content, self.scale, true);
        builder.push_default(StyleProperty::Brush(SolidBrush { color }));
        builder.push_default(GenericFamily::SystemUi);
        builder.push_default(LineHeight::FontSizeRelative(1.25));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(400.0)));
        for span in spans {
            let start = span.range.start.min(content.len());
            let end = span.range.end.min(content.len());
            if start >= end {
                continue;
            }
            let range = start..end;
            if span.bold {
                builder.push(
                    StyleProperty::FontWeight(FontWeight::new(700.0)),
                    range.clone(),
                );
            }
            if span.italic {
                builder.push(StyleProperty::FontStyle(FontStyle::Italic), range.clone());
            }
            if span.monospace {
                builder.push(GenericFamily::Monospace, range.clone());
            }
            if let Some(span_color) = span.color {
                builder.push(
                    StyleProperty::Brush(SolidBrush { color: span_color }),
                    range.clone(),
                );
            }
            if span.underline {
                builder.push(StyleProperty::Underline(true), range.clone());
                builder.push(
                    StyleProperty::UnderlineBrush(Some(SolidBrush {
                        color: span.underline_color.or(span.color).unwrap_or(color),
                    })),
                    range.clone(),
                );
            }
            if span.strikethrough {
                builder.push(StyleProperty::Strikethrough(true), range.clone());
                builder.push(
                    StyleProperty::StrikethroughBrush(Some(SolidBrush {
                        color: span.strikethrough_color.or(span.color).unwrap_or(color),
                    })),
                    range.clone(),
                );
            }
        }
        let mut layout = builder.build(content);
        layout.break_all_lines(max_width.map(|w| w * self.scale));
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
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

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a finished layout into `scene` at logical position (`x`, `y`).
///
/// The origin is snapped to physical pixels first: Parley quantizes
/// glyphs to the pixel grid, and a fractional draw offset would push
/// every glyph off-grid again (blurry text, esp. at fractional window
/// scales like 125%/150%). Hinting is enabled so small glyphs stay
/// sharp like native (DirectWrite/ClearType) text.
pub fn draw_layout(scene: &mut Scene, layout: &Layout<SolidBrush>, x: f32, y: f32, scale: f32) {
    let ox = (x * scale).round();
    let oy = (y * scale).round();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let brush = Brush::Solid(glyph_run.style().brush.color);
                let glyphs = glyph_run.positioned_glyphs().map(|glyph| vello::Glyph {
                    id: glyph.id,
                    x: ox + glyph.x,
                    y: oy + glyph.y,
                });
                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .hint(true)
                    .brush(brush)
                    .draw(Fill::NonZero, glyphs);
            }
        }
    }
}
