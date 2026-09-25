use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::super::text::{BasicText, TextAlignment, TextStyle};
use super::{LINK_BLUE, open_url};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Styled link look: filled dark pill or blue outline pill (like the
/// reference rows).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkStyle {
    /// Dark fill with colored label.
    #[default]
    Filled,
    /// Transparent fill with colored border and label.
    Border,
}

/// Pill corner radius in logical px.
pub const LINK_PILL_RADIUS: f32 = 16.0;
/// Pill padding (x, y) in logical px.
pub const LINK_PILL_PAD_X: f32 = 28.0;
/// Pill padding (x, y) in logical px.
pub const LINK_PILL_PAD_Y: f32 = 14.0;
/// Border width in logical px.
pub const LINK_PILL_BORDER: f32 = 2.5;
/// Filled pill background (both modes).
pub const LINK_PILL_BG: Color = Color::from_rgb8(0x1e, 0x20, 0x24);

fn with_alpha(color: Color, alpha: f32) -> Color {
    let c = color.to_rgba8();
    Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha.clamp(0.0, 1.0)).round() as u8)
}

/// Styled link: pill button opening a URL (filled dark or bordered,
/// like the reference "Styled Link" and "Border Link" rows).
/// Clicking opens the URL through the opener (or the system
/// handler); hover brightens the pill slightly.
pub struct StyledLink {
    label: String,
    url: String,
    style: LinkStyle,
    color: Color,
    opener: Option<Box<dyn FnMut(&str)>>,
    text: BasicText,
    hovered: bool,
    pressed: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl StyledLink {
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        let label = label.into();
        let mut link = Self {
            text: BasicText::new(label.clone())
                .style(TextStyle::Headline)
                .alignment(TextAlignment::Center),
            label,
            url: url.into(),
            style: LinkStyle::Filled,
            color: LINK_BLUE,
            opener: None,
            hovered: false,
            pressed: false,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        };
        link.apply_colors();
        link
    }

    pub fn style(mut self, style: LinkStyle) -> Self {
        self.style = style;
        self
    }

    /// Link color instead of blue (label and border).
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.apply_colors();
        self
    }

    /// Custom open handler (tests, in-app routing). Without it,
    /// clicks use the system handler.
    pub fn opener(mut self, opener: impl FnMut(&str) + 'static) -> Self {
        self.opener = Some(Box::new(opener));
        self
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        let label = label.into();
        if label != self.label {
            self.label = label.clone();
            self.text.set_text(label);
        }
    }

    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }

    pub fn set_style(&mut self, style: LinkStyle) {
        self.style = style;
    }

    pub fn set_color(&mut self, color: Color) {
        if color != self.color {
            self.color = color;
            self.apply_colors();
        }
    }

    pub fn label_value(&self) -> &str {
        &self.label
    }

    pub fn url_value(&self) -> &str {
        &self.url
    }

    pub fn style_value(&self) -> LinkStyle {
        self.style
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.apply_colors();
    }

    /// Open the URL now (same path as a click).
    pub fn open(&mut self) {
        if let Some(opener) = self.opener.as_mut() {
            opener(&self.url.clone());
        } else {
            open_url(&self.url);
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn label_color(&self) -> Color {
        let mut color = self.color;
        if self.pressed {
            color = with_alpha(color, 0.6);
        }
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn apply_colors(&mut self) {
        self.text.set_foreground(crate::elements::TextForeground::Color(
            self.label_color(),
        ));
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.placed_w && y >= self.y && y <= self.y + self.placed_h
    }
}

impl View for StyledLink {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (tw, th) = self.text.measure(fonts);
        (tw + LINK_PILL_PAD_X * 2.0, th + LINK_PILL_PAD_Y * 2.0)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        let (tw, th) = self.text.measure(fonts);
        self.text.place(fonts, x + (w - tw) / 2.0, y + (h - th) / 2.0, tw, th);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let pill = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            px(LINK_PILL_RADIUS),
        );
        let color = if self.focused {
            self.color
        } else {
            desaturate(self.color)
        };
        match self.style {
            LinkStyle::Filled => {
                let mut bg = LINK_PILL_BG;
                if self.hovered {
                    bg = Color::from_rgb8(0x2c, 0x2e, 0x33);
                }
                if !self.focused {
                    bg = desaturate(bg);
                }
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(bg),
                    None,
                    &pill,
                );
            }
            LinkStyle::Border => {
                if self.hovered {
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(with_alpha(color, 0.12)),
                        None,
                        &pill,
                    );
                }
                scene.stroke(
                    &Stroke::new(px(LINK_PILL_BORDER)),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &pill,
                );
            }
        }
        self.text.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        if self.hit(x as f32, y as f32) {
            self.pressed = true;
            self.apply_colors();
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        let was_pressed = self.pressed;
        self.pressed = false;
        self.apply_colors();
        if was_pressed && self.hit(x as f32, y as f32) {
            self.open();
        }
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.hit(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn styles_hold() {
        let filled = StyledLink::new("Styled Link", "https://example.com");
        assert_eq!(filled.style_value(), LinkStyle::Filled);
        let border = StyledLink::new("Border Link", "https://example.com")
            .style(LinkStyle::Border);
        assert_eq!(border.style_value(), LinkStyle::Border);
    }

    #[test]
    fn click_reports_url_through_opener() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let out = seen.clone();
        let mut link = StyledLink::new("Styled Link", "https://example.com")
            .style(LinkStyle::Border)
            .opener(move |url| {
                out.borrow_mut().push(url.to_string());
            });
        let mut fonts = FontSystem::new();
        let (w, h) = link.measure(&mut fonts);
        link.place(&mut fonts, 0.0, 0.0, w, h);
        link.mouse_down((w / 2.0) as f64, (h / 2.0) as f64);
        link.mouse_up((w / 2.0) as f64, (h / 2.0) as f64);
        assert_eq!(*seen.borrow(), vec!["https://example.com".to_string()]);
        link.mouse_down((w / 2.0) as f64, (h / 2.0) as f64);
        link.mouse_up((w + 50.0) as f64, (h / 2.0) as f64);
        assert_eq!(seen.borrow().len(), 1);
    }

    #[test]
    fn custom_color_wins() {
        let pink = Color::from_rgb8(0xff, 0x2d, 0x55);
        let mut link = StyledLink::new("x", "https://example.com").color(pink);
        assert_eq!(link.color, pink);
        link.set_color(LINK_BLUE);
        assert_eq!(link.color, LINK_BLUE);
    }
}
