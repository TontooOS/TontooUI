use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::images::{ImageFit, UrlImage};
use super::super::layout::View;
use super::basic::BasicLink;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Gap between image and link in logical px.
pub const LINK_IMAGE_GAP: f32 = 20.0;

/// Link with image: an async URL image on top (spinner while
/// loading, `Error {code}` on failure, like `UrlImage`) plus a
/// `BasicLink` below (like the reference rows: blue logo, download
/// icon plus "Download App"). The image stays colorful as downloaded
/// unless `.tint()` pins a fixed color (RGB replaced, alpha kept),
/// e.g. a blue glyph.
pub struct LinkWithImage {
    image: UrlImage,
    link: BasicLink,
    gap: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl LinkWithImage {
    pub fn new(
        image_url: impl Into<String>,
        label: impl Into<String>,
        link_url: impl Into<String>,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            image: UrlImage::new(image_url, width, height),
            link: BasicLink::new(label, link_url),
            gap: LINK_IMAGE_GAP,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Fixed image tint (e.g. brand blue). Without it the image
    /// stays normal/colorful as downloaded.
    pub fn tint(mut self, color: Color) -> Self {
        self.image.set_tint(Some(color));
        self
    }

    /// Fixed image tint, or `None` back to colorful.
    pub fn set_tint(&mut self, tint: Option<Color>) {
        self.image.set_tint(tint);
    }

    pub fn tint_value(&self) -> Option<Color> {
        self.image.tint_value()
    }

    /// Gap between image and link in logical px.
    pub fn spacing(mut self, px: f32) -> Self {
        self.gap = px.max(0.0);
        self
    }

    /// Custom open handler for the link (tests, in-app routing).
    pub fn opener(mut self, opener: impl FnMut(&str) + 'static) -> Self {
        self.link.set_opener(Some(Box::new(opener)));
        self
    }

    /// Image for state updates (theme, retry, ...).
    pub fn image_mut(&mut self) -> &mut UrlImage {
        &mut self.image
    }

    /// Link for state updates (color, opener, ...).
    pub fn link_mut(&mut self) -> &mut BasicLink {
        &mut self.link
    }

    pub fn set_image_fit(&mut self, fit: ImageFit) {
        self.image.set_fit(fit);
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl View for LinkWithImage {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (iw, ih) = self.image.measure(fonts);
        let (lw, lh) = self.link.measure(fonts);
        (iw.max(lw), ih + self.gap + lh)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        let (iw, ih) = self.image.measure(fonts);
        let (lw, lh) = self.link.measure(fonts);
        self.image.place(fonts, x + (w - iw) / 2.0, y, iw, ih);
        self.link
            .place(fonts, x + (w - lw) / 2.0, y + ih + self.gap, lw, lh);
        let _ = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.image.draw(scene, fonts, images);
        self.link.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.image.mouse_down(x, y);
        self.link.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.image.mouse_up(x, y);
        self.link.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.link.set_hover(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn card() -> LinkWithImage {
        LinkWithImage::new(
            "https://example.com/logo.png",
            "Download App",
            "https://example.com/download",
            200.0,
            160.0,
        )
    }

    #[test]
    fn stacks_image_over_link() {
        let mut card = card();
        let mut fonts = FontSystem::new();
        let (w, h) = card.measure(&mut fonts);
        assert_eq!(w, 200.0);
        assert!(h > 160.0 + LINK_IMAGE_GAP);
        card.place(&mut fonts, 0.0, 0.0, w, h);
        let (_, _, _, lh) = card.link_mut().rect();
        assert!(lh > 0.0);
    }

    #[test]
    fn tint_pins_fixed_color() {
        let blue = Color::from_rgb8(0x0a, 0x84, 0xff);
        let card = card().tint(blue);
        assert_eq!(card.tint_value(), Some(blue));
        let mut plain = card;
        plain.set_tint(None);
        assert_eq!(plain.tint_value(), None);
    }

    #[test]
    fn link_click_reports_through_opener() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let out = seen.clone();
        let mut card = card().opener(move |url| {
            out.borrow_mut().push(url.to_string());
        });
        let mut fonts = FontSystem::new();
        let (w, h) = card.measure(&mut fonts);
        card.place(&mut fonts, 0.0, 0.0, w, h);
        // Reach the link rect for a centered click (no browser
        // launches: the recording opener stands in).
        let (lx, ly, lw, lh) = card.link_mut().rect();
        card.mouse_down((lx + lw / 2.0) as f64, (ly + lh / 2.0) as f64);
        card.mouse_up((lx + lw / 2.0) as f64, (ly + lh / 2.0) as f64);
        assert_eq!(
            *seen.borrow(),
            vec!["https://example.com/download".to_string()]
        );
    }
}
