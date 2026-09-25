use std::any::Any;

use parley::{Alignment, AlignmentOptions, Layout, PositionedLayoutItem};
use vello::Scene;
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use super::super::layout::View;
use super::foreground::{ResolvedForeground, TextForeground};
use super::style::TextStyle;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::ThemeMode;

/// Horizontal text alignment inside the placed rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlignment {
    /// Lines start at the leading edge (default).
    #[default]
    Leading,
    /// Lines are centered (each line individually when wrapped).
    Center,
    /// Lines end at the trailing edge.
    Trailing,
}

/// Basic text: a label in a `TextStyle` size with a `TextForeground`
/// color or gradient. Single line by default; `width` fixes the box
/// so longer content wraps (each line follows `alignment`).
///
/// Built on the unchanged `FontSystem` API (`layout_text_weighted`
/// plus `draw_layout` for solid colors). Only gradients take a custom
/// draw loop that paints the same glyph runs with a horizontal
/// gradient brush across the text bounds.
pub struct BasicText {
    content: String,
    style: TextStyle,
    foreground: TextForeground,
    alignment: TextAlignment,
    wrap_width: Option<f32>,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    layout: Option<Layout<SolidBrush>>,
    layout_scale: f32,
    dirty: bool,
}

impl BasicText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::Body,
            foreground: TextForeground::Primary,
            alignment: TextAlignment::Leading,
            wrap_width: None,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            layout: None,
            layout_scale: 0.0,
            dirty: true,
        }
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self.dirty = true;
        self
    }

    pub fn set_style(&mut self, style: TextStyle) {
        if self.style != style {
            self.style = style;
            self.dirty = true;
        }
    }

    pub fn foreground(mut self, foreground: TextForeground) -> Self {
        self.foreground = foreground;
        self.dirty = true;
        self
    }

    /// Shortcut for `foreground(TextForeground::Color(color))`.
    pub fn foreground_color(mut self, color: Color) -> Self {
        self.foreground = TextForeground::Color(color);
        self.dirty = true;
        self
    }

    /// Shortcut for `foreground(TextForeground::Gradient(colors))`.
    pub fn foreground_gradient(mut self, colors: Vec<Color>) -> Self {
        self.foreground = TextForeground::Gradient(colors);
        self.dirty = true;
        self
    }

    pub fn set_foreground(&mut self, foreground: TextForeground) {
        if self.foreground != foreground {
            self.foreground = foreground;
            self.dirty = true;
        }
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self.dirty = true;
        self
    }

    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        if self.alignment != alignment {
            self.alignment = alignment;
            self.dirty = true;
        }
    }

    /// Fixed box width in logical px: longer content wraps. `None`
    /// (default) measures the intrinsic single-line size.
    pub fn width(mut self, px: f32) -> Self {
        self.wrap_width = Some(px.max(0.0));
        self.dirty = true;
        self
    }

    pub fn set_width(&mut self, px: Option<f32>) {
        let px = px.map(|w| w.max(0.0));
        if self.wrap_width != px {
            self.wrap_width = px;
            self.dirty = true;
        }
    }

    pub fn set_text(&mut self, content: impl Into<String>) {
        let content = content.into();
        if content != self.content {
            self.content = content;
            self.dirty = true;
        }
    }

    /// Live theme: picks the semantic foreground colors. Fixed
    /// `Color`/`Gradient` foregrounds stay as set.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark != self.dark {
            self.dark = dark;
            self.dirty = true;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if focused != self.focused {
            self.focused = focused;
            self.dirty = true;
        }
    }

    fn mode(&self) -> ThemeMode {
        if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    /// Resolved base foreground (shared with `LabeledText` icon tint).
    pub(crate) fn resolve_foreground(
        &self,
        mode: ThemeMode,
        focused: bool,
    ) -> ResolvedForeground {
        self.foreground.resolve(mode, focused)
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty && self.layout.is_some() && self.layout_scale == fonts.scale {
            return;
        }
        // Bake the solid color into the layout; gradients paint white
        // and override the brush while drawing.
        let bake = match self.foreground.resolve(self.mode(), self.focused) {
            ResolvedForeground::Solid(color) => color,
            ResolvedForeground::Gradient(_) => Color::WHITE,
        };
        let mut layout = fonts.layout_text_weighted(
            &self.content,
            self.style.size(),
            bake,
            self.style.weight(),
            self.wrap_width,
        );
        if self.wrap_width.is_some() {
            layout.align(
                match self.alignment {
                    TextAlignment::Leading => Alignment::Start,
                    TextAlignment::Center => Alignment::Center,
                    TextAlignment::Trailing => Alignment::End,
                },
                AlignmentOptions::default(),
            );
        }
        self.layout = Some(layout);
        self.layout_scale = fonts.scale;
        self.dirty = false;
    }

    /// Block width the text occupies: fixed wrap width or intrinsic.
    fn block_width(&mut self, fonts: &mut FontSystem) -> f32 {
        if let Some(wrap) = self.wrap_width {
            wrap
        } else {
            self.ensure_layout(fonts);
            let layout = self.layout.as_ref().expect("layout built");
            FontSystem::layout_size(layout).0 / fonts.scale
        }
    }

    /// Draw origin x for the placed rect and block width.
    fn origin_x(&self, block: f32) -> f32 {
        match self.alignment {
            TextAlignment::Leading => self.x,
            TextAlignment::Center => self.x + (self.width - block) / 2.0,
            TextAlignment::Trailing => self.x + self.width - block,
        }
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        self.ensure_layout(fonts);
        let scale = fonts.scale;
        let block = self.block_width(fonts);
        let ox = self.origin_x(block).max(self.x);
        let layout = self.layout.as_ref().expect("layout built");
        match self.foreground.resolve(self.mode(), self.focused) {
            ResolvedForeground::Solid(_) => {
                draw_layout(scene, layout, ox, self.y, scale);
            }
            ResolvedForeground::Gradient(colors) => {
                let (tw, th) = FontSystem::layout_size(layout);
                let brush = gradient_brush(&colors, ox, self.y, tw / scale, th / scale, scale);
                draw_with_brush(scene, layout, ox, self.y, scale, &brush);
            }
        }
    }
}

/// Evenly spread gradient stops across `colors` (offsets 0.0 to 1.0).
fn linear_stops(colors: &[Color]) -> Vec<ColorStop> {
    let n = colors.len();
    colors
        .iter()
        .enumerate()
        .map(|(index, color)| ColorStop {
            offset: if n <= 1 {
                0.0
            } else {
                index as f32 / (n - 1) as f32
            },
            color: (*color).into(),
        })
        .collect()
}

/// Horizontal gradient brush spanning the logical text block.
pub(crate) fn gradient_brush(
    colors: &[Color],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    scale: f32,
) -> Brush {
    let s = scale as f64;
    let stops = linear_stops(colors);
    Brush::Gradient(
        Gradient::new_linear(
            (x as f64 * s, (y + height / 2.0) as f64 * s),
            ((x + width) as f64 * s, (y + height / 2.0) as f64 * s),
        )
        .with_stops(stops.as_slice()),
    )
}

/// Same glyph loop as `draw_layout`, but every run paints `brush`
/// instead of its baked solid color. Origin snapped to physical
/// pixels and hinting enabled, same as `draw_layout`.
pub(crate) fn draw_with_brush(
    scene: &mut Scene,
    layout: &Layout<SolidBrush>,
    x: f32,
    y: f32,
    scale: f32,
    brush: &Brush,
) {
    let ox = (x * scale).round();
    let oy = (y * scale).round();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
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

impl View for BasicText {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        let (tw, th) = FontSystem::layout_size(layout);
        match self.wrap_width {
            Some(wrap) => (wrap, th / fonts.scale),
            None => (tw / fonts.scale, th / fonts.scale),
        }
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        let _ = h;
        self.ensure_layout(fonts);
        // Unbounded text keeps its intrinsic width; wrapped text fills
        // the fixed box. The draw origin aligns inside this rect.
        let (tw, th) = FontSystem::layout_size(self.layout.as_ref().expect("layout built"));
        let scale = fonts.scale;
        match self.wrap_width {
            Some(wrap) => {
                self.x = x;
                self.y = y;
                self.width = w.max(wrap);
                self.height = th / scale;
            }
            None => {
                self.x = x;
                self.y = y;
                self.width = w.max(tw / scale);
                self.height = th / scale;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeMode;

    #[test]
    fn styles_match_swiftui_scale() {
        let cases = [
            (TextStyle::LargeTitle, 34.0, 400.0),
            (TextStyle::Title, 28.0, 400.0),
            (TextStyle::Title2, 22.0, 400.0),
            (TextStyle::Title3, 20.0, 400.0),
            (TextStyle::Headline, 17.0, 600.0),
            (TextStyle::Subheadline, 15.0, 400.0),
            (TextStyle::Body, 17.0, 400.0),
            (TextStyle::Callout, 16.0, 400.0),
            (TextStyle::Footnote, 13.0, 400.0),
            (TextStyle::Caption, 12.0, 400.0),
            (TextStyle::Caption2, 11.0, 400.0),
        ];
        for (style, size, weight) in cases {
            assert_eq!(style.size(), size, "{style:?}");
            assert_eq!(style.weight(), weight, "{style:?}");
        }
    }

    #[test]
    fn single_line_measures_intrinsic() {
        let mut fonts = FontSystem::new();
        let mut text = BasicText::new("Hello, SwiftUI!");
        let (w, h) = text.measure(&mut fonts);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn fixed_width_wraps_taller() {
        let content = "Multi-line text that spans across multiple lines to show how text wrapping works.";
        let mut fonts = FontSystem::new();
        let (_, single_h) = BasicText::new(content).measure(&mut fonts);
        let (wrap_w, wrap_h) = BasicText::new(content).width(120.0).measure(&mut fonts);
        assert_eq!(wrap_w, 120.0);
        assert!(wrap_h > single_h);
    }

    #[test]
    fn gradient_stops_spread_evenly() {
        let stops = linear_stops(&[
            Color::from_rgb8(255, 0, 0),
            Color::from_rgb8(0, 255, 0),
            Color::from_rgb8(0, 0, 255),
        ]);
        assert_eq!(stops.len(), 3);
        assert_eq!(stops[0].offset, 0.0);
        assert_eq!(stops[1].offset, 0.5);
        assert_eq!(stops[2].offset, 1.0);
    }

    #[test]
    fn foreground_resolves_theme_and_focus() {
        let dark_primary = TextForeground::Primary.resolve(ThemeMode::Dark, true);
        let light_primary = TextForeground::Primary.resolve(ThemeMode::Light, true);
        assert_ne!(dark_primary, light_primary);
        // Unfocused windows desaturate to gray.
        let gray = TextForeground::Color(Color::from_rgb8(0xff, 0x00, 0x00))
            .resolve(ThemeMode::Dark, false);
        match gray {
            ResolvedForeground::Solid(color) => {
                let c = color.to_rgba8();
                assert_eq!(c.r, c.g);
                assert_eq!(c.g, c.b);
            }
            ResolvedForeground::Gradient(_) => panic!("expected solid"),
        }
        // Empty gradient falls back to primary, single is solid.
        assert_eq!(
            TextForeground::Gradient(vec![]).resolve(ThemeMode::Dark, true),
            TextForeground::Primary.resolve(ThemeMode::Dark, true)
        );
        assert!(matches!(
            TextForeground::Gradient(vec![Color::WHITE]).resolve(ThemeMode::Dark, true),
            ResolvedForeground::Solid(_)
        ));
    }

    #[test]
    fn alignment_places_draw_origin() {
        let mut fonts = FontSystem::new();
        let mut text = BasicText::new("Hi").alignment(TextAlignment::Center);
        text.place(&mut fonts, 0.0, 0.0, 200.0, 50.0);
        let block = text.block_width(&mut fonts);
        assert!(block < 200.0);
        assert!(text.origin_x(block) > 0.0);
    }
}
