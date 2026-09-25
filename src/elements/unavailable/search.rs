use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::content::ContentUnavailable;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::ThemeMode;

/// Search empty state: a large SF Symbol on top, a semibold title
/// and a gray message, with no refresh button (like the reference
/// rows: magnifier, "No Results", "Check the spelling or try a new
/// search."). Thin wrapper over the `ContentUnavailable` composition,
/// so icon, texts and layout always match the refresh variant.
pub struct SearchEmpty {
    inner: ContentUnavailable,
}

impl SearchEmpty {
    pub fn new(
        icon: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ContentUnavailable::new(icon, title, message).refresh(false),
        }
    }

    /// SF Symbol name (CoreIcon lookup, like everywhere else).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.inner.set_icon(name);
        self
    }

    /// Optional icon tint. Without it the glyph stays
    /// `UNAVAILABLE_ICON_GRAY`.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.inner.set_icon_color(Some(color));
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.inner.set_title(title);
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.inner.set_message(message);
        self
    }

    /// Live theme for the texts (no button, so no accent needed).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.inner
            .set_theme(mode, Color::from_rgb8(0x00, 0x7a, 0xff));
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.inner.set_focused(focused);
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.inner.set_title(title);
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.inner.set_message(message);
    }

    pub fn icon_value(&self) -> &str {
        self.inner.icon_value()
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        self.inner.rect()
    }
}

impl View for SearchEmpty {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.inner.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.inner.place(fonts, x, y, w, h);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.inner.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.inner.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.inner.mouse_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> SearchEmpty {
        SearchEmpty::new(
            "magnifyingglass",
            "No Results",
            "Check the spelling or try a new search.",
        )
    }

    #[test]
    fn builders_set_parts() {
        let view = view()
            .icon("star.fill")
            .icon_color(Color::from_rgb8(0xff, 0x9f, 0x0a))
            .title("T")
            .message("M");
        assert_eq!(view.icon_value(), "star.fill");
    }

    #[test]
    fn clicks_do_nothing() {
        let mut view = view();
        let mut fonts = FontSystem::new();
        let (w, h) = view.measure(&mut fonts);
        view.place(&mut fonts, 0.0, 0.0, w.max(400.0), h);
        view.mouse_down(200.0, 100.0);
        view.mouse_up(200.0, 100.0);
        let (x, y, pw, ph) = view.rect();
        assert!(pw > 0.0 && ph > 0.0);
        assert_eq!((x, y), (0.0, 0.0));
    }

    #[test]
    fn intrinsic_width_matches_sibling() {
        // Same composition as the refresh variant: same width.
        let mut search = view();
        let mut plain = ContentUnavailable::new("tray", "No Data", "Msg").refresh(false);
        let mut fonts = FontSystem::new();
        assert_eq!(
            search.measure(&mut fonts).0,
            plain.measure(&mut fonts).0
        );
    }
}
