//! `GlassContainer` — keep the content, replace the background with glass.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let field = TextInput::new("Search...").transparent();
//! let glass = GlassContainer::new(field)
//!     .behind_file("examples/assets/glass_bg.jpg")
//!     .size(340.0, 64.0)
//!     .radius(20.0);
//! ```
//!
//! The `behind` element is placed **live** underneath the glass, so it is
//! the actual UI behind it. The frost layer composites from the same
//! pixels wherever they are known (image, file, color). Note: GTK4 offers
//! no API to snapshot an arbitrary live widget into pixels, so a generic
//! widget behind falls back to a neutral dark frost while still showing
//! live outside the glass mask. True backdrop blur over arbitrary UI
//! belongs in the compositor (TontooCompositor), not in the toolkit.

use image::{Rgb, RgbImage};
use uikit::style::{Color, Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use super::material::{ClearGlass, GlassMaterial, render_clear_glass, render_glass};

/// How the container glass renders.
#[derive(Debug, Clone)]
pub enum GlassStyle {
    /// Simple glass: same color as behind, micro lift, light top/bottom
    /// edges, darker left/right edges, full pass-through.
    Clear(ClearGlass),
    /// Frosted glass: blur, refraction, dispersion, grain.
    Frosted(GlassMaterial),
}

impl Default for GlassStyle {
    fn default() -> Self {
        Self::Clear(ClearGlass::default())
    }
}

/// What lives behind the glass. The element is always shown live; the
/// frost composites from its pixels wherever they are known.
pub enum GlassBehind {
    /// Any widget, shown live. Frost falls back to a dark fill because
    /// GTK4 cannot rasterize arbitrary widgets.
    Widget(Box<dyn Widget>),
    /// Decoded image, shown live and used for the frost.
    Image(RgbImage),
    /// Image file, shown live and loaded for the frost.
    File(String),
    /// Flat color, shown live and used for the frost.
    Color(Color),
}

impl GlassBehind {
    #[cfg(test)]
    fn debug_name(&self) -> &'static str {
        match self {
            GlassBehind::Widget(_) => "widget",
            GlassBehind::Image(_) => "image",
            GlassBehind::File(_) => "file",
            GlassBehind::Color(_) => "color",
        }
    }
}

/// The app window background (`#1d1d1d` dark / `#ececec` light).
fn app_bg() -> Color {
    if crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark {
        Color::from_hex("#1d1d1d").unwrap()
    } else {
        Color::from_hex("#ececec").unwrap()
    }
}

fn flat(c: Color, w: u32, h: u32) -> RgbImage {
    RgbImage::from_pixel(
        w.max(1),
        h.max(1),
        Rgb([(c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8]),
    )
}

/// A container that keeps its content but replaces the background with
/// live-composited liquid glass. Content should paint transparently where
/// the glass must show through (e.g. `TextInput::transparent()`).
pub struct GlassContainer {
    id: WidgetId,
    content: Box<dyn Widget>,
    style: GlassStyle,
    behind: Option<GlassBehind>,
    width: f32,
    height: f32,
    radius: f32,
    padding: f32,
    position_mode: PositionMode,
    position: Position,
}

impl GlassContainer {
    /// Wrap any content in glass. Defaults: simple [`Clear`] glass,
    /// `320x64`, capsule radius, lying directly on the app background
    /// (`#1d1d1d` dark / `#ececec` light). Put something else behind it
    /// with `behind_*`, switch to frosted glass with `material`.
    pub fn new(content: impl Widget + 'static) -> Self {
        Self {
            id: next_widget_id(),
            content: Box::new(content),
            style: GlassStyle::default(),
            behind: None,
            width: 320.0,
            height: 64.0,
            radius: 32.0,
            padding: 8.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Use the simple clear glass.
    pub fn clear(mut self, c: ClearGlass) -> Self {
        self.style = GlassStyle::Clear(c);
        self
    }

    /// Use frosted glass with a fully custom material.
    pub fn material(mut self, m: GlassMaterial) -> Self {
        self.style = GlassStyle::Frosted(m);
        self
    }

    fn frosted_material(&mut self) -> &mut GlassMaterial {
        if !matches!(self.style, GlassStyle::Frosted(_)) {
            self.style = GlassStyle::Frosted(GlassMaterial::default());
        }
        match &mut self.style {
            GlassStyle::Frosted(m) => m,
            GlassStyle::Clear(_) => unreachable!(),
        }
    }

    /// Tint the glass (`Color` + alpha `0..=100`). Switches to frosted.
    pub fn tint(mut self, c: Color, alpha: f32) -> Self {
        let m = self.frosted_material();
        m.tint_r = (c.r * 255.0).round();
        m.tint_g = (c.g * 255.0).round();
        m.tint_b = (c.b * 255.0).round();
        m.tint_a = alpha;
        self
    }

    /// Backdrop blur sigma (`0..=25`). Switches to frosted.
    pub fn sigma(mut self, s: f32) -> Self {
        self.frosted_material().sigma = s;
        self
    }

    /// Lens refraction strength (`0..=100`). Switches to frosted.
    pub fn refraction(mut self, r: f32) -> Self {
        self.frosted_material().refraction = r;
        self
    }

    /// Use an explicit behind element, shown live under the glass.
    pub fn behind(mut self, b: GlassBehind) -> Self {
        self.behind = Some(b);
        self
    }

    /// Put any widget live behind the glass. Note: its frost falls back
    /// to a dark fill (GTK4 cannot rasterize arbitrary widgets).
    pub fn behind_widget(mut self, w: impl Widget + 'static) -> Self {
        self.behind = Some(GlassBehind::Widget(Box::new(w)));
        self
    }

    /// Put a decoded image live behind the glass and frost from it.
    pub fn behind_image(mut self, img: RgbImage) -> Self {
        self.behind = Some(GlassBehind::Image(img));
        self
    }

    /// Put an image file live behind the glass and frost from it.
    pub fn behind_file(mut self, path: impl Into<String>) -> Self {
        self.behind = Some(GlassBehind::File(path.into()));
        self
    }

    /// Put a flat color live behind the glass and frost from it.
    pub fn behind_color(mut self, c: Color) -> Self {
        self.behind = Some(GlassBehind::Color(c));
        self
    }

    /// Container size in pixels.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Corner radius in pixels (capsule when `>= height / 2`).
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }

    /// Inset of the content inside the glass.
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }

    /// Frost pixels for the behind element (dark fill when unknown).
    /// With no behind element the app background is used, so the glass
    /// simply lies on the app.
    fn frost_source(&self, w: u32, h: u32) -> RgbImage {
        let fill = || RgbImage::from_pixel(w.max(1), h.max(1), Rgb([20, 26, 34]));
        match &self.behind {
            Some(GlassBehind::Image(img)) => img.clone(),
            Some(GlassBehind::File(path)) => image::open(path).map(|i| i.to_rgb8()).unwrap_or_else(|_| fill()),
            Some(GlassBehind::Color(c)) => flat(*c, w, h),
            Some(GlassBehind::Widget(_)) => fill(),
            None => flat(app_bg(), w, h),
        }
    }

    /// The live widget shown under the glass.
    fn behind_live(&self, _w: u32, _h: u32) -> Option<gtk::Widget> {
        match &self.behind {
            Some(GlassBehind::Widget(content)) => {
                let live = content.to_gtk();
                live.set_hexpand(true);
                live.set_vexpand(true);
                live.set_halign(gtk::Align::Fill);
                live.set_valign(gtk::Align::Fill);
                Some(live)
            }
            Some(GlassBehind::Image(img)) => {
                let bytes = gtk::glib::Bytes::from_owned(img.clone().into_raw());
                let tex = gtk::gdk::MemoryTexture::new(
                    img.width() as i32,
                    img.height() as i32,
                    gtk::gdk::MemoryFormat::R8g8b8,
                    &bytes,
                    (img.width() * 3) as usize,
                );
                let pic = gtk::Picture::for_paintable(&tex);
                pic.set_content_fit(gtk::ContentFit::Cover);
                pic.set_can_shrink(true);
                pic.set_hexpand(true);
                pic.set_vexpand(true);
                pic.set_halign(gtk::Align::Fill);
                pic.set_valign(gtk::Align::Fill);
                Some(pic.upcast())
            }
            Some(GlassBehind::File(path)) => {
                let pic = gtk::Picture::for_filename(path);
                pic.set_content_fit(gtk::ContentFit::Cover);
                pic.set_can_shrink(true);
                pic.set_hexpand(true);
                pic.set_vexpand(true);
                pic.set_halign(gtk::Align::Fill);
                pic.set_valign(gtk::Align::Fill);
                Some(pic.upcast())
            }
            Some(GlassBehind::Color(c)) => {
                let fill = gtk::Box::new(gtk::Orientation::Vertical, 0);
                fill.set_hexpand(true);
                fill.set_vexpand(true);
                fill.set_halign(gtk::Align::Fill);
                fill.set_valign(gtk::Align::Fill);
                let css = format!(
                    ".gc-behind {{ background: rgb({},{},{}); }}",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8
                );
                fill.add_css_class("gc-behind");
                uikit::widget::apply_css(&fill, &css);
                Some(fill.upcast())
            }
            // No behind element: nothing to show, the window itself plus
            // the transparent glass corners shine through.
            None => None,
        }
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        let (w, h) = (self.width, self.height);
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl Default for GlassContainer {
    fn default() -> Self {
        Self::new(uikit::widgets::Text::new(""))
    }
}

impl ViewContent for GlassContainer {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let w = self.width.max(1.0) as u32;
        let h = self.height.max(1.0) as u32;
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(w as i32, h as i32);

        // Live behind element (the actual UI under the glass).
        if let Some(live) = self.behind_live(w, h) {
            overlay.set_child(Some(&live));
        }

        // Glass layer composited once at render time (RGBA: transparent
        // outside the mask, the real window shines through).
        let bg = self.frost_source(w, h);
        let img = match &self.style {
            GlassStyle::Clear(c) => render_clear_glass(c, &bg, w, h, self.radius),
            GlassStyle::Frosted(m) => render_glass(m, &bg, w, h, self.radius).0,
        };
        let bytes = gtk::glib::Bytes::from_owned(img.into_raw());
        let tex = gtk::gdk::MemoryTexture::new(
            w as i32,
            h as i32,
            gtk::gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (w * 4) as usize,
        );
        let pic = gtk::Picture::for_paintable(&tex);
        pic.set_can_shrink(false);
        pic.set_hexpand(true);
        pic.set_vexpand(true);
        pic.set_halign(gtk::Align::Fill);
        pic.set_valign(gtk::Align::Fill);
        overlay.add_overlay(&pic);

        // Content floats above, inset by padding. It must paint
        // transparently where the glass should show through.
        let inner = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        inner.set_halign(gtk::Align::Fill);
        inner.set_valign(gtk::Align::Fill);
        inner.set_hexpand(true);
        inner.set_vexpand(true);
        let pad = self.padding.max(0.0) as i32;
        inner.set_margin_start(pad);
        inner.set_margin_end(pad);
        inner.set_margin_top(pad / 2);
        inner.set_margin_bottom(pad / 2);
        let child = self.content.to_gtk();
        child.set_hexpand(true);
        child.set_valign(gtk::Align::Center);
        inner.append(&child);
        overlay.add_overlay(&inner);

        overlay.upcast()
    }

    fn size_that_fits(&self, _: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for GlassContainer {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }
    fn position(&self) -> Position {
        self.position
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, self.width, self.height))
    }
    fn is_interactive(&self) -> bool {
        true
    }
    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_builder() {
        let c = GlassContainer::new(uikit::widgets::Text::new("hi"))
            .size(340.0, 64.0)
            .radius(20.0)
            .sigma(4.0)
            .padding(10.0);
        assert_eq!((c.width, c.height, c.radius, c.padding), (340.0, 64.0, 20.0, 10.0));
        match &c.style {
            GlassStyle::Frosted(m) => assert_eq!(m.sigma, 4.0),
            GlassStyle::Clear(_) => panic!("sigma() must switch to frosted"),
        }
    }

    #[test]
    fn container_defaults_to_clear() {
        let c = GlassContainer::new(uikit::widgets::Text::new("hi"));
        assert!(matches!(c.style, GlassStyle::Clear(_)));
    }

    #[test]
    fn container_tint_sugar() {
        let c = GlassContainer::new(uikit::widgets::Text::new("hi"))
            .tint(Color::from_rgb(168, 213, 255), 17.0);
        match &c.style {
            GlassStyle::Frosted(m) => {
                assert_eq!((m.tint_r, m.tint_g, m.tint_b), (168.0, 213.0, 255.0));
                assert_eq!(m.tint_a, 17.0);
            }
            GlassStyle::Clear(_) => panic!("tint() must switch to frosted"),
        }
    }

    #[test]
    fn missing_file_behind_falls_back() {
        let c = GlassContainer::new(uikit::widgets::Text::new("hi"))
            .behind_file("/does/not/exist.png")
            .size(64.0, 32.0);
        let bg = c.frost_source(64, 32);
        assert_eq!((bg.width(), bg.height()), (64, 32));
    }

    #[test]
    fn behind_variants_resolve() {
        let img = RgbImage::from_pixel(80, 40, Rgb([1, 2, 3]));
        let c = GlassContainer::new(uikit::widgets::Text::new("hi")).behind_image(img);
        assert_eq!(c.behind.as_ref().map(|b| b.debug_name()), Some("image"));
        let c = GlassContainer::new(uikit::widgets::Text::new("hi"))
            .behind_widget(uikit::widgets::Text::new("live"));
        assert_eq!(c.behind.as_ref().map(|b| b.debug_name()), Some("widget"));
    }
}
