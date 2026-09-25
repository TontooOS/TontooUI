use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Line, Stroke};
use vello::peniko::{Brush, Color};

use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::text::{BasicText, TextStyle};
use super::{LINK_BLUE, LINK_ICON_GAP, LINK_ICON_SIZE};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Open `url` in its default handler (detached process, never
/// blocking the UI): browsers for `http`/`https`, the mail app for
/// `mailto` (`start` on Windows, `open` on macOS, `xdg-open`
/// elsewhere on Unix). Anything else reports false without side
/// effects.
pub fn open_url(url: &str) -> bool {
    if !is_openable(url) {
        return false;
    }
    #[cfg(target_os = "windows")]
    let spawn = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let spawn = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let spawn = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    let spawn: std::io::Result<std::process::Child> =
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "no opener"));
    spawn.is_ok()
}

/// True for URLs the system can hand to an app: web pages and mail
/// links. Pure check without side effects (launching stays in
/// `open_url`).
pub fn is_openable(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let c = color.to_rgba8();
    Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha.clamp(0.0, 1.0)).round() as u8)
}

/// Basic link: blue label with an optional leading SF icon (like the
/// reference rows: "Visit Apple", code icon plus "Swift.org").
/// Clicking opens the URL in the default browser; hover underlines.
/// Display otherwise (no drag, no focus ring).
pub struct BasicLink {
    label: String,
    url: String,
    color: Option<Color>,
    opener: Option<Box<dyn FnMut(&str)>>,
    text: BasicText,
    symbol: Option<SFSymbolImage>,
    hovered: bool,
    pressed: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl BasicLink {
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        let label = label.into();
        let mut link = Self {
            text: BasicText::new(label.clone()).style(TextStyle::Body),
            symbol: None,
            label,
            url: url.into(),
            color: None,
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

    /// Leading SF icon box (CoreIcon lookup, link blue).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.symbol = Some(
            SFSymbolImage::new(name.into())
                .size(LINK_ICON_SIZE)
                .color(self.color.unwrap_or(LINK_BLUE)),
        );
        self
    }

    /// Custom link color instead of blue (icon follows).
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        if let Some(symbol) = self.symbol.as_mut() {
            symbol.set_color(Some(color));
        }
        self.apply_colors();
        self
    }

    /// Custom link color, or `None` back to blue.
    pub fn set_color(&mut self, color: Option<Color>) {
        if color != self.color {
            self.color = color;
            if let Some(symbol) = self.symbol.as_mut() {
                symbol.set_color(color.or(Some(LINK_BLUE)));
            }
            self.apply_colors();
        }
    }

    pub fn color_value(&self) -> Option<Color> {
        self.color
    }

    /// Custom open handler (tests, in-app routing). Without it,
    /// clicks use the system handler.
    pub fn opener(mut self, opener: impl FnMut(&str) + 'static) -> Self {
        self.opener = Some(Box::new(opener));
        self
    }

    /// Custom open handler, or `None` back to the system handler.
    pub fn set_opener(&mut self, opener: Option<Box<dyn FnMut(&str)>>) {
        self.opener = opener;
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

    pub fn label_value(&self) -> &str {
        &self.label
    }

    pub fn url_value(&self) -> &str {
        &self.url
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.apply_colors();
        self.text.set_focused(focused);
        if let Some(symbol) = self.symbol.as_mut() {
            symbol.set_focused(focused);
        }
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

    fn apply_colors(&mut self) {
        let base = self.color.unwrap_or(LINK_BLUE);
        let base = if self.pressed {
            with_alpha(base, 140.0 / 255.0)
        } else {
            base
        };
        let color = if self.focused {
            base
        } else {
            desaturate(base)
        };
        self.text.set_foreground(crate::elements::TextForeground::Color(color));
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.placed_w && y >= self.y && y <= self.y + self.placed_h
    }
}

impl View for BasicLink {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (tw, th) = self.text.measure(fonts);
        let (iw, ih) = match self.symbol.as_mut() {
            Some(symbol) => symbol.measure(fonts),
            None => (0.0, 0.0),
        };
        let gap = if self.symbol.is_some() {
            LINK_ICON_GAP
        } else {
            0.0
        };
        (iw + gap + tw, th.max(ih))
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        let (tw, th) = self.text.measure(fonts);
        let mut cx = x;
        if let Some(symbol) = self.symbol.as_mut() {
            let (iw, ih) = symbol.measure(fonts);
            symbol.place(fonts, cx, y + (h - ih) / 2.0, iw, ih);
            cx += iw + LINK_ICON_GAP;
        }
        self.text.place(fonts, cx, y + (h - th) / 2.0, tw, th);
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
        if let Some(symbol) = self.symbol.as_mut() {
            symbol.draw(scene, fonts, images);
        }
        self.text.draw(scene, fonts, images);
        // Hover underline across the label.
        if self.hovered {
            let (tx, _, tw, _) = self.text.rect();
            let ty = self.y + self.placed_h - 2.0;
            let c = if self.focused {
                LINK_BLUE
            } else {
                desaturate(LINK_BLUE)
            };
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(c),
                None,
                &Line::new(
                    (tx as f64 * scale, ty as f64 * scale),
                    ((tx + tw) as f64 * scale, ty as f64 * scale),
                ),
            );
        }
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
    fn rejects_non_openable_urls() {
        assert!(!is_openable("file:///etc/passwd"));
        assert!(!is_openable("javascript:alert(1)"));
        assert!(!is_openable(""));
        assert!(!is_openable("notaurl"));
        assert!(is_openable("https://apple.com"));
        assert!(is_openable("http://example.com"));
        assert!(is_openable("mailto:hello@example.com"));
        // Launching itself stays out of unit tests; only the scheme
        // gate is covered (never call `open_url` here).
        assert!(!open_url("file:///etc/passwd"));
    }

    #[test]
    fn click_reports_url_through_opener() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let out = seen.clone();
        let mut link = BasicLink::new("Visit Apple", "https://apple.com").opener(move |url| {
            out.borrow_mut().push(url.to_string());
        });
        let mut fonts = FontSystem::new();
        let (w, h) = link.measure(&mut fonts);
        link.place(&mut fonts, 0.0, 0.0, w, h);
        link.mouse_down((w / 2.0) as f64, (h / 2.0) as f64);
        link.mouse_up((w / 2.0) as f64, (h / 2.0) as f64);
        assert_eq!(*seen.borrow(), vec!["https://apple.com".to_string()]);
        // Outside release opens nothing.
        link.mouse_down((w / 2.0) as f64, (h / 2.0) as f64);
        link.mouse_up((w + 50.0) as f64, (h / 2.0) as f64);
        assert_eq!(seen.borrow().len(), 1);
    }

    #[test]
    fn icon_extends_measure() {
        let mut fonts = FontSystem::new();
        let mut plain = BasicLink::new("Swift.org", "https://swift.org");
        let mut iconic = BasicLink::new("Swift.org", "https://swift.org")
            .icon("chevron.left.forwardslash.chevron.right");
        assert!(iconic.measure(&mut fonts).0 > plain.measure(&mut fonts).0);
        assert_eq!(plain.url_value(), "https://swift.org");
    }

    #[test]
    fn hover_tracks_hit() {
        let mut link = BasicLink::new("Visit Apple", "https://apple.com");
        let mut fonts = FontSystem::new();
        let (w, h) = link.measure(&mut fonts);
        link.place(&mut fonts, 10.0, 20.0, w, h);
        link.set_hover(10.0 + w / 2.0, 20.0 + h / 2.0);
        link.set_hover(500.0, 500.0);
    }
}
