use std::any::Any;
use std::ops::Range;

use parley::{Alignment, AlignmentOptions, Cluster, Layout, PositionedLayoutItem};
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::BUTTON_ACCENT;
use super::super::layout::View;
use super::foreground::{ResolvedForeground, TextForeground};
use super::span::{Span, parse_markdown};
use super::style::TextStyle;
use super::text::{TextAlignment, gradient_brush};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, RichSpan, SolidBrush};
use crate::theme::ThemeMode;

/// Formatted text: inline spans (bold, italic, code, underline,
/// strikethrough, colors, links) either parsed from markdown
/// (`FormattedText::markdown`) or built explicitly
/// (`FormattedText::spans`). `line_limit` truncates with an ellipsis,
/// links fire `on_link` on click.
///
/// Markdown subset, single paragraph: `**bold**`, `*italic*`,
/// `_italic_`, `***bold italic***`, `~~strikethrough~~`, `` `code` ``,
/// `[label](url)`, `\` escapes. Unmatched markers stay literal.
///
/// Built on the additive `FontSystem::layout_rich_text` (the plain
/// `layout_text` API stays unchanged). Decorations paint from the
/// Parley run metrics; links hit-test through `Cluster::from_point`.
pub struct FormattedText {
    spans: Vec<Span>,
    markdown: Option<String>,
    style: TextStyle,
    foreground: TextForeground,
    alignment: TextAlignment,
    wrap_width: Option<f32>,
    line_limit: Option<usize>,
    accent: Color,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    rich: Vec<RichSpan>,
    links: Vec<(Range<usize>, String)>,
    layout: Option<Layout<SolidBrush>>,
    intrinsic: (f32, f32),
    scale: f32,
    pressed_link: Option<String>,
    armed: bool,
    on_link: Option<Box<dyn FnMut(&str)>>,
    dirty: bool,
}

impl FormattedText {
    pub fn markdown(source: impl Into<String>) -> Self {
        Self {
            markdown: Some(source.into()),
            ..Self::empty()
        }
    }

    pub fn spans(spans: Vec<Span>) -> Self {
        Self {
            spans,
            ..Self::empty()
        }
    }

    fn empty() -> Self {
        Self {
            spans: Vec::new(),
            markdown: None,
            style: TextStyle::Body,
            foreground: TextForeground::Primary,
            alignment: TextAlignment::Leading,
            wrap_width: None,
            line_limit: None,
            accent: BUTTON_ACCENT,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            content: String::new(),
            rich: Vec::new(),
            links: Vec::new(),
            layout: None,
            intrinsic: (0.0, 0.0),
            scale: 1.0,
            pressed_link: None,
            armed: false,
            on_link: None,
            dirty: true,
        }
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self.dirty = true;
        self
    }

    pub fn foreground(mut self, foreground: TextForeground) -> Self {
        self.foreground = foreground;
        self.dirty = true;
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self.dirty = true;
        self
    }

    /// Fixed box width in logical px for wrapping.
    pub fn width(mut self, px: f32) -> Self {
        self.wrap_width = Some(px.max(0.0));
        self.dirty = true;
        self
    }

    /// Max visible lines; overflow truncates with "…".
    pub fn line_limit(mut self, lines: usize) -> Self {
        self.line_limit = Some(lines.max(1));
        self.dirty = true;
        self
    }

    /// Link color (default theme blue). Also recolors link underlines.
    pub fn accent(mut self, accent: Color) -> Self {
        self.accent = accent;
        self.dirty = true;
        self
    }

    /// Link click action, fired with the link URL.
    pub fn on_link(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        self.on_link = Some(Box::new(callback));
        self
    }

    pub fn set_source(&mut self, spans: Vec<Span>) {
        self.spans = spans;
        self.markdown = None;
        self.dirty = true;
    }

    pub fn set_markdown(&mut self, source: impl Into<String>) {
        self.markdown = Some(source.into());
        self.dirty = true;
    }

    pub fn set_accent(&mut self, accent: Color) {
        if accent != self.accent {
            self.accent = accent;
            self.dirty = true;
        }
    }

    pub fn set_line_limit(&mut self, lines: Option<usize>) {
        let lines = lines.map(|n| n.max(1));
        if lines != self.line_limit {
            self.line_limit = lines;
            self.dirty = true;
        }
    }

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

    fn active_spans(&self) -> Vec<Span> {
        match &self.markdown {
            Some(source) => parse_markdown(source),
            None => self.spans.clone(),
        }
    }

    /// Join spans into plain content plus rich ranges and link targets.
    /// `fallback` colors decorations without an explicit color.
    fn build_parts(
        spans: &[Span],
        accent: Color,
        fallback: Color,
    ) -> (String, Vec<RichSpan>, Vec<(Range<usize>, String)>) {
        let mut content = String::new();
        let mut rich = Vec::new();
        let mut links = Vec::new();
        for span in spans {
            if span.text.is_empty() {
                continue;
            }
            let start = content.len();
            content.push_str(&span.text);
            let range = start..content.len();
            let color = span.color.or_else(|| span.link.as_ref().map(|_| accent));
            let underline = span.underline || span.link.is_some();
            let underline_color = span
                .underline_color
                .or(span.color)
                .or_else(|| span.link.as_ref().map(|_| accent))
                .or(if underline { Some(fallback) } else { None });
            let strikethrough_color = span
                .strikethrough_color
                .or(span.color)
                .or(if span.strikethrough {
                    Some(fallback)
                } else {
                    None
                });
            rich.push(RichSpan {
                range: range.clone(),
                bold: span.bold,
                italic: span.italic,
                monospace: span.code,
                color,
                underline,
                underline_color,
                strikethrough: span.strikethrough,
                strikethrough_color,
            });
            if let Some(url) = &span.link {
                links.push((range, url.clone()));
            }
        }
        (content, rich, links)
    }

    /// Clamp rich/link ranges to `cut` bytes (for truncation).
    fn clamp_parts(
        rich: &[RichSpan],
        links: &[(Range<usize>, String)],
        cut: usize,
    ) -> (Vec<RichSpan>, Vec<(Range<usize>, String)>) {
        let clamped_rich = rich
            .iter()
            .filter(|span| span.range.start < cut)
            .map(|span| RichSpan {
                range: span.range.start..span.range.end.min(cut),
                ..span.clone()
            })
            .collect();
        let clamped_links = links
            .iter()
            .filter(|(range, _)| range.start < cut)
            .map(|(range, url)| (range.start..range.end.min(cut), url.clone()))
            .collect();
        (clamped_rich, clamped_links)
    }

    fn build_layout(
        &self,
        fonts: &mut FontSystem,
        content: &str,
        rich: &[RichSpan],
        bake: Color,
    ) -> Layout<SolidBrush> {
        let mut layout = fonts.layout_rich_text(
            content,
            self.style.size(),
            bake,
            self.wrap_width,
            rich,
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
        layout
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty && self.layout.is_some() && self.scale == fonts.scale {
            return;
        }
        let spans = self.active_spans();
        let (base_solid, gradient) = match self.foreground.resolve(self.mode(), self.focused) {
            ResolvedForeground::Solid(color) => (color, None),
            ResolvedForeground::Gradient(colors) => (Color::WHITE, Some(colors)),
        };
        let fallback = gradient.as_ref().and_then(|c| c.first().copied()).unwrap_or(base_solid);
        let (mut content, mut rich, mut links) =
            Self::build_parts(&spans, self.accent_for_draw(), fallback);
        let bake = if gradient.is_some() {
            Color::WHITE
        } else {
            base_solid
        };
        // Truncation: longest char-prefix plus "…" fitting the limit.
        if let Some(limit) = self.line_limit {
            let layout = self.build_layout(fonts, &content, &rich, bake);
            if layout.lines().count() > limit {
                let bounds: Vec<usize> = content
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain(std::iter::once(content.len()))
                    .collect();
                let mut lo = 0;
                let mut hi = bounds.len() - 1;
                let mut best = 0;
                while lo <= hi {
                    let mid = (lo + hi) / 2;
                    let cut = bounds[mid];
                    let (rich_cut, _) = Self::clamp_parts(&rich, &links, cut);
                    let candidate = format!("{}…", content[..cut].trim_end());
                    let probe = self.build_layout(fonts, &candidate, &rich_cut, bake);
                    if probe.lines().count() <= limit {
                        best = mid;
                        lo = mid + 1;
                    } else if mid == 0 {
                        break;
                    } else {
                        hi = mid - 1;
                    }
                    if lo > hi {
                        break;
                    }
                }
                let cut = bounds[best];
                content = format!("{}…", content[..cut].trim_end());
                let (rich_cut, links_cut) = Self::clamp_parts(&rich, &links, content.len());
                rich = rich_cut;
                links = links_cut;
            }
        }
        let layout = self.build_layout(fonts, &content, &rich, bake);
        let (tw, th) = FontSystem::layout_size(&layout);
        self.intrinsic = (tw / fonts.scale, th / fonts.scale);
        self.scale = fonts.scale;
        self.content = content;
        self.rich = rich;
        self.links = links;
        self.layout = Some(layout);
        self.dirty = false;
    }

    fn accent_for_draw(&self) -> Color {
        if self.focused {
            self.accent
        } else {
            crate::theme::desaturate(self.accent)
        }
    }

    /// Byte ranges with an explicit text color (links and `color`
    /// spans). Everything else paints the base foreground.
    fn explicit_ranges(&self) -> Vec<Range<usize>> {
        self.rich
            .iter()
            .filter(|span| span.color.is_some())
            .map(|span| span.range.clone())
            .collect()
    }

    fn origin_x(&self, block: f32) -> f32 {
        match self.alignment {
            TextAlignment::Leading => self.x,
            TextAlignment::Center => self.x + (self.width - block) / 2.0,
            TextAlignment::Trailing => self.x + self.width - block,
        }
    }

    /// Link URL at logical position `(x, y)`, or `None`.
    pub fn link_at(&self, x: f32, y: f32) -> Option<String> {
        let layout = self.layout.as_ref()?;
        let block = self.wrap_width.unwrap_or(self.intrinsic.0);
        let ox = self.origin_x(block).max(self.x);
        let lx = (x - ox) * self.scale;
        let ly = (y - self.y) * self.scale;
        // Exact hit: no clamping to nearby clusters, so padding
        // around the text never counts as a link.
        let (cluster, _) = Cluster::from_point_exact(layout, lx, ly)?;
        let at = cluster.text_range().start;
        self.links
            .iter()
            .find(|(range, _)| range.contains(&at))
            .map(|(_, url)| url.clone())
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if let Some(url) = self.link_at(x as f32, y as f32) {
            self.pressed_link = Some(url);
            self.armed = true;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn finish_press(&mut self, x: f64, y: f64) {
        let armed = self.armed;
        let pressed = self.pressed_link.clone();
        self.armed = false;
        self.pressed_link = None;
        if armed {
            if let (Some(url), Some(hit)) = (pressed, self.link_at(x as f32, y as f32)) {
                if url == hit {
                    if let Some(callback) = self.on_link.as_mut() {
                        callback(&url);
                    }
                }
            }
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
        let block = self.wrap_width.unwrap_or(self.intrinsic.0);
        let ox = self.origin_x(block).max(self.x);
        let oy = self.y;
        // Snap to physical pixels like `draw_layout`: Parley
        // quantizes glyphs to the pixel grid, a fractional offset
        // would push them off-grid (blurry text).
        let pox = (ox * scale).round();
        let poy = (oy * scale).round();
        let layout = self.layout.as_ref().expect("layout built");
        let limit = self.line_limit.unwrap_or(usize::MAX);
        let explicit = self.explicit_ranges();
        let gradient_brush = match self.foreground.resolve(self.mode(), self.focused) {
            ResolvedForeground::Gradient(colors) => {
                Some(gradient_brush(&colors, ox, oy, block, self.intrinsic.1, scale))
            }
            ResolvedForeground::Solid(_) => None,
        };
        for line in layout.lines().take(limit) {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    let run = glyph_run.run();
                    // Explicit-color runs keep their baked color;
                    // default runs paint the base (or gradient) brush.
                    let is_explicit = explicit.iter().any(|range| {
                        range.contains(&run.text_range().start)
                    });
                    let brush = match (&gradient_brush, is_explicit) {
                        (Some(g), false) => g,
                        _ => &Brush::Solid(glyph_run.style().brush.color),
                    };
                    let glyphs =
                        glyph_run.positioned_glyphs().map(|glyph| vello::Glyph {
                            id: glyph.id,
                            x: pox + glyph.x,
                            y: poy + glyph.y,
                        });
                    scene
                        .draw_glyphs(run.font())
                        .font_size(run.font_size())
                        .hint(true)
                        .brush(brush)
                        .draw(Fill::NonZero, glyphs);
                    self.draw_decorations(scene, &glyph_run, pox, poy);
                }
            }
        }
    }

    fn draw_decorations(
        &self,
        scene: &mut Scene,
        glyph_run: &parley::GlyphRun<'_, SolidBrush>,
        px: f32,
        py: f32,
    ) {
        let metrics = glyph_run.run().metrics();
        let x0 = (px + glyph_run.offset()) as f64;
        let x1 = x0 + glyph_run.advance() as f64;
        if x1 <= x0 {
            return;
        }
        let base = (py + glyph_run.baseline()) as f64;
        let style = glyph_run.style();
        if let Some(underline) = &style.underline {
            let size = underline
                .size
                .unwrap_or(metrics.underline_size)
                .max(1.0) as f64;
            let top = base + underline.offset.unwrap_or(metrics.underline_offset) as f64;
            let rect = Rect::new(x0, top, x1, top + size);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(underline.brush.color),
                None,
                &rect,
            );
        }
        if let Some(strike) = &style.strikethrough {
            let size = strike
                .size
                .unwrap_or(metrics.strikethrough_size)
                .max(1.0) as f64;
            let top = base + strike.offset.unwrap_or(metrics.strikethrough_offset) as f64;
            let rect = Rect::new(x0, top, x1, top + size);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(strike.brush.color),
                None,
                &rect,
            );
        }
    }
}

impl View for FormattedText {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.ensure_layout(fonts);
        match self.wrap_width {
            Some(wrap) => (wrap, self.intrinsic.1),
            None => self.intrinsic,
        }
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, _h: f32) {
        self.ensure_layout(fonts);
        self.x = x;
        self.y = y;
        self.width = w.max(self.wrap_width.unwrap_or(self.intrinsic.0));
        self.height = self.intrinsic.1;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeMode;

    fn placed(source: &str, width: f32) -> (FormattedText, FontSystem) {
        let mut fonts = FontSystem::new();
        let mut text = FormattedText::markdown(source).width(width);
        text.place(&mut fonts, 0.0, 0.0, width, 600.0);
        (text, fonts)
    }

    #[test]
    fn markdown_bold_italic_layout() {
        let (mut text, mut fonts) = placed("**Bold** and *italic* together", 400.0);
        text.ensure_layout(&mut fonts);
        assert_eq!(text.content, "Bold and italic together");
        assert!(text.rich.iter().any(|span| span.bold));
        assert!(text.rich.iter().any(|span| span.italic));
    }

    #[test]
    fn link_range_recorded() {
        let (mut text, mut fonts) = placed("tap [here](https://x.test) now", 400.0);
        text.ensure_layout(&mut fonts);
        assert_eq!(text.links.len(), 1);
        assert_eq!(text.links[0].1, "https://x.test");
        let (range, _) = &text.links[0];
        assert_eq!(&text.content[range.clone()], "here");
    }

    #[test]
    fn link_hit_and_press() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let fired: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let moved = fired.clone();
        let mut fonts = FontSystem::new();
        let mut text = FormattedText::markdown("[Link](https://x.test)")
            .width(400.0)
            .on_link(move |url| moved.borrow_mut().push(url.to_string()));
        text.place(&mut fonts, 10.0, 20.0, 400.0, 600.0);
        // Middle of the laid-out link text hits.
        let (tw, _) = text.intrinsic;
        let url = text.link_at(10.0 + tw / 2.0, 20.0 + 8.0);
        assert_eq!(url.as_deref(), Some("https://x.test"));
        assert_eq!(text.link_at(390.0, 500.0), None);
        if let Some(found) = url {
            let (tw, _) = text.intrinsic;
            text.mouse_down((10.0 + tw / 2.0) as f64, 28.0);
            text.mouse_up((10.0 + tw / 2.0) as f64, 28.0);
            assert_eq!(fired.borrow().as_slice(), [found]);
        }
    }

    #[test]
    fn line_limit_truncates_with_ellipsis() {
        let content = "Word ".repeat(60);
        let mut fonts = FontSystem::new();
        let mut text = FormattedText::markdown(content).width(200.0).line_limit(2);
        text.place(&mut fonts, 0.0, 0.0, 200.0, 600.0);
        text.ensure_layout(&mut fonts);
        assert!(text.content.ends_with('…'));
        assert!(text.content.len() < "Word ".repeat(60).len());
        let layout = text.layout.as_ref().expect("layout built");
        assert!(layout.lines().count() <= 2);
    }

    #[test]
    fn explicit_spans_keep_colors() {
        let spans = vec![
            Span::new("Underlined ").underline_color(Color::from_rgb8(0xff, 0x00, 0x00)),
            Span::new("Strike").strikethrough_color(Color::from_rgb8(0x00, 0xff, 0x00)),
            Span::new("Mono").code(),
        ];
        let mut fonts = FontSystem::new();
        let mut text = FormattedText::spans(spans);
        text.set_theme(ThemeMode::Light);
        text.place(&mut fonts, 0.0, 0.0, 600.0, 100.0);
        text.ensure_layout(&mut fonts);
        assert!(text.rich[0].underline);
        assert_eq!(
            text.rich[0].underline_color,
            Some(Color::from_rgb8(0xff, 0x00, 0x00))
        );
        assert!(text.rich[1].strikethrough);
        assert!(text.rich[2].monospace);
    }
}
