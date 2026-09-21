use parley::{
    Alignment, AlignmentOptions, FontContext, GenericFamily, Layout, LayoutContext,
    LineHeight, PositionedLayoutItem, StyleProperty,
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

    /// Lay out `content` at `size` logical px. `max_width` is in logical px;
    /// `None` disables wrapping.
    pub fn layout_text(
        &mut self,
        content: &str,
        size: f32,
        color: Color,
        max_width: Option<f32>,
    ) -> Layout<SolidBrush> {
        let px = size * self.scale;
        let mut builder =
            self.layout_cx
                .ranged_builder(&mut self.font_cx, content, 1.0, true);
        builder.push_default(StyleProperty::Brush(SolidBrush { color }));
        builder.push_default(GenericFamily::SystemUi);
        builder.push_default(LineHeight::FontSizeRelative(1.25));
        builder.push_default(StyleProperty::FontSize(px));
        let mut layout = builder.build(content);
        layout.break_all_lines(max_width.map(|w| w * self.scale));
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
    }

    /// Logical width/height of a finished layout.
    pub fn layout_size(layout: &Layout<SolidBrush>) -> (f32, f32) {
        (layout.width(), layout.height())
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a finished layout into `scene` at logical position (`x`, `y`).
pub fn draw_layout(scene: &mut Scene, layout: &Layout<SolidBrush>, x: f32, y: f32, scale: f32) {
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let brush = Brush::Solid(glyph_run.style().brush.color);
                let glyphs = glyph_run.positioned_glyphs().map(|glyph| vello::Glyph {
                    id: glyph.id,
                    x: x * scale + glyph.x,
                    y: y * scale + glyph.y,
                });
                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .brush(brush)
                    .draw(Fill::NonZero, glyphs);
            }
        }
    }
}
