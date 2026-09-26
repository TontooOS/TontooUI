use std::any::Any;
use std::sync::mpsc::{self, Receiver};

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};

use super::super::animation::Spin;
use super::super::layout::View;
use super::super::progress::Spinner;
use super::{IMAGE_RADIUS, IMAGE_TEXT_SIZE, ImageFit, fit_rect, spin_transform};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Load state of the remote image.
#[derive(Clone, Debug, PartialEq, Eq)]
enum UrlState {
    Loading,
    Loaded,
    Failed(Option<u16>),
}

/// Raster image from an `https://` (or `http://`) URL. The download
/// runs on a background thread (NetworkKit blocking client) so the UI
/// never stalls: the frame shows a `Spinner` while loading, the image
/// once decoded, and `Error {code}` text when the server answers with
/// an HTTP error status. Display-only (no mouse handling).
pub struct UrlImage {
    url: String,
    width: f32,
    height: f32,
    fit: ImageFit,
    radius: f32,
    spinner: Spinner,
    text_color: Color,
    tint: Option<Color>,
    dark: bool,
    focused: bool,
    state: UrlState,
    bytes: Vec<u8>,
    rx: Option<Receiver<Result<Vec<u8>, Option<u16>>>>,
    spin_deg: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl UrlImage {
    pub fn new(url: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            url: url.into(),
            width: width.max(0.0),
            height: height.max(0.0),
            fit: ImageFit::Cover,
            radius: IMAGE_RADIUS,
            spinner: Spinner::new(),
            text_color: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            tint: None,
            dark: true,
            focused: true,
            state: UrlState::Loading,
            bytes: Vec::new(),
            rx: None,
            spin_deg: 0.0,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// How the raster fills the frame: cover (crop, default) or fit
    /// (letterbox).
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Corner radius in logical px (clamped to >= 0).
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = px.max(0.0);
        self
    }

    /// Spinner spoke color while loading.
    pub fn spinner_color(mut self, color: Color) -> Self {
        self.spinner = Spinner::new().color(color);
        self
    }

    /// Fixed tint for the raster (RGB replaced, alpha kept), e.g. a
    /// blue logo. Without it the photo stays colorful as downloaded.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    /// Fixed tint, or `None` back to the colorful original.
    pub fn set_tint(&mut self, tint: Option<Color>) {
        self.tint = tint;
    }

    pub fn tint_value(&self) -> Option<Color> {
        self.tint
    }

    /// Restart the download (e.g. after a failure or an URL change).
    pub fn retry(&mut self) {
        self.state = UrlState::Loading;
        self.bytes.clear();
        self.rx = None;
    }

    /// Live theme for the spinner caption gray and the error text.
    pub fn set_theme(&mut self, dark: bool) {
        self.dark = dark;
        self.spinner.set_dark(dark);
        self.text_color = if dark {
            Color::from_rgb8(0x9a, 0x9a, 0x9e)
        } else {
            Color::from_rgb8(0x6e, 0x6e, 0x72)
        };
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_fit(&mut self, fit: ImageFit) {
        self.fit = fit;
    }

    pub fn set_radius(&mut self, px: f32) {
        self.radius = px.max(0.0);
    }

    /// Current load state.
    pub fn state_value(&self) -> &'static str {
        match &self.state {
            UrlState::Loading => "loading",
            UrlState::Loaded => "loaded",
            UrlState::Failed(_) => "error",
        }
    }

    /// Error caption (`Error {code}` with the HTTP status, plain
    /// `Error` for transport failures). `None` unless failed.
    pub fn error_text(&self) -> Option<String> {
        match &self.state {
            UrlState::Failed(Some(code)) => Some(format!("Error {code}")),
            UrlState::Failed(None) => Some("Error".to_string()),
            _ => None,
        }
    }

    pub fn url_value(&self) -> &str {
        &self.url
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn ensure_started(&mut self) {
        if self.rx.is_some() || self.state != UrlState::Loading {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let url = self.url.clone();
        std::thread::spawn(move || {
            let result = match networkkit::http::HttpRequest::get(&url).send() {
                Ok(resp) if resp.is_success() => Ok(resp.body.clone()),
                Ok(resp) => Err(Some(resp.status)),
                Err(_) => Err(None),
            };
            let _ = tx.send(result);
        });
    }

    fn poll(&mut self) {
        let done = match &mut self.rx {
            Some(rx) => match rx.try_recv() {
                Ok(Ok(bytes)) => {
                    self.bytes = bytes;
                    self.state = UrlState::Loaded;
                    true
                }
                Ok(Err(code)) => {
                    self.state = UrlState::Failed(code);
                    true
                }
                Err(mpsc::TryRecvError::Empty) => false,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state = UrlState::Failed(None);
                    true
                }
            },
            None => false,
        };
        if done {
            self.rx = None;
        }
    }

    fn eff_text(&self) -> Color {
        if self.focused {
            self.text_color
        } else {
            desaturate(self.text_color)
        }
    }
}

impl View for UrlImage {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.width, self.height)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
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
        self.ensure_started();
        self.poll();
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        match &self.state {
            UrlState::Loading => {
                self.spinner.place(fonts, self.x, self.y, self.placed_w, self.placed_h);
                self.spinner.draw(scene, fonts, images);
            }
            UrlState::Loaded => {
                let radius = self
                    .radius
                    .min(self.placed_w / 2.0)
                    .min(self.placed_h / 2.0)
                    .max(0.0);
                let frame = RoundedRect::new(
                    px(self.x),
                    px(self.y),
                    px(self.x + self.placed_w),
                    px(self.y + self.placed_h),
                    px(radius),
                );
                // Crisp supersampling: upload at ~2x the display size.
                let target = (self.placed_w.max(self.placed_h) * fonts.scale * 2.0)
                    .ceil()
                    .max(1.0) as u32;
                let key = match self.tint {
                    Some(tint) => {
                        let c = tint.to_rgba8();
                        format!(
                            "url:{}#{:02x}{:02x}{:02x}{:02x}",
                            self.url, c.r, c.g, c.b, c.a
                        )
                    }
                    None => format!("url:{}", self.url),
                };
                let bytes = self.bytes.clone();
                let loaded = match self.tint {
                    Some(tint) => images.raster_tinted(&key, &bytes, tint, target),
                    None => images.raster(&key, &bytes, target),
                };
                match loaded {
                    Some((image, iw, ih)) => {
                        let (dw, _dh, dx, dy) = fit_rect(
                            iw as f32,
                            ih as f32,
                            self.placed_w,
                            self.placed_h,
                            self.fit,
                        );
                        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &frame);
                        let s = (dw / iw as f32) as f64 * scale;
                        let base = Affine::translate((
                            (self.x + dx) as f64 * scale,
                            (self.y + dy) as f64 * scale,
                        )) * Affine::scale(s);
                        let transform = spin_transform(
                            base,
                            (self.x + self.placed_w / 2.0) as f64 * scale,
                            (self.y + self.placed_h / 2.0) as f64 * scale,
                            self.spin_deg,
                        );
                        scene.draw_image(&image, transform);
                        scene.pop_layer();
                    }
                    None => {
                        // Decoded nothing: treat undecodable bodies as an
                        // error so the frame never stays blank silently.
                        self.state = UrlState::Failed(None);
                    }
                }
            }
            UrlState::Failed(_) => {
                let text = self.error_text().unwrap_or_else(|| "Error".to_string());
                let layout =
                    fonts.layout_text(&text, IMAGE_TEXT_SIZE, self.eff_text(), None);
                let (tw, th) = FontSystem::layout_size(&layout);
                draw_layout(
                    scene,
                    &layout,
                    self.x + (self.placed_w - tw / fonts.scale) / 2.0,
                    self.y + (self.placed_h - th / fonts.scale) / 2.0,
                    fonts.scale,
                );
            }
        }
        let _ = self.dark;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Spin for UrlImage {
    fn set_spin(&mut self, degrees: f32) {
        self.spin_deg = degrees;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn starts_loading() {
        let image = UrlImage::new("https://example.com/a.png", 200.0, 120.0);
        assert_eq!(image.state_value(), "loading");
        assert_eq!(image.error_text(), None);
    }

    #[test]
    fn error_text_carries_http_code() {
        let mut image = UrlImage::new("https://example.com/a.png", 200.0, 120.0);
        image.state = UrlState::Failed(Some(404));
        assert_eq!(image.error_text(), Some("Error 404".to_string()));
        image.state = UrlState::Failed(None);
        assert_eq!(image.error_text(), Some("Error".to_string()));
    }

    #[test]
    fn retry_restarts_loading() {
        let mut image = UrlImage::new("https://example.com/a.png", 200.0, 120.0);
        image.state = UrlState::Failed(Some(500));
        image.retry();
        assert_eq!(image.state_value(), "loading");
        assert_eq!(image.error_text(), None);
    }

    #[test]
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut image = UrlImage::new("https://example.com/a.png", 200.0, 120.0);
        assert_eq!(image.measure(&mut fonts), (200.0, 120.0));
        image.place(&mut fonts, 0.0, 0.0, 200.0, 120.0);
        assert_eq!(image.rect(), (0.0, 0.0, 200.0, 120.0));
    }
}
