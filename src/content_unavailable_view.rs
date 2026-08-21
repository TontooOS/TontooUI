//! ContentUnavailableView — iOS SwiftUI-style empty state.
//!
//! A centered magnifier icon, bold title and thin hint message, recreating the
//! iOS SwiftUI "No Results for ..." empty state without the search field. The
//! title renders the configured query statically. Theme-aware (dark / light)
//! with no manual toggle — it follows the given or detected scheme.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let view = ContentUnavailableView::new()
//!     .query("foo")
//!     .title("No Results for")
//!     .message("Check the spelling or try a new search.");
//! ```

use uikit::app::ColorScheme;
use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Label as GtkLabel};
use gtk::Orientation;

const TITLE_LIGHT: &str = "#18181b";
const TITLE_DARK: &str = "#ececec";
const MUTE_LIGHT: &str = "#a1a1aa";
const MUTE_DARK: &str = "#71717a";

/// iOS SwiftUI-style empty state with icon, title and hint message.
pub struct ContentUnavailableView {
    id: WidgetId,
    query: String,
    title_prefix: String,
    message: String,
    width: f32,
    height: f32,
    color_scheme: Option<ColorScheme>,
    position_mode: PositionMode,
    position: Position,
}

impl ContentUnavailableView {
    /// Create a new empty state with default English strings.
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            query: String::new(),
            title_prefix: "No Results for".into(),
            message: "Check the spelling or try a new search.".into(),
            width: 320.0,
            height: 240.0,
            color_scheme: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Set the query text shown quoted in the title.
    pub fn query(mut self, text: impl Into<String>) -> Self { self.query = text.into(); self }
    /// Set the title prefix before the quoted query (default: "No Results for").
    pub fn title(mut self, prefix: impl Into<String>) -> Self { self.title_prefix = prefix.into(); self }
    /// Set the hint message below the title.
    pub fn message(mut self, text: impl Into<String>) -> Self { self.message = text.into(); self }
    /// Set the element size.
    pub fn frame(mut self, w: f32, h: f32) -> Self { self.width = w; self.height = h; self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn height(mut self, h: f32) -> Self { self.height = h; self }
    /// Force a color scheme (defaults to detecting the system scheme).
    pub fn color_scheme(mut self, c: ColorScheme) -> Self { self.color_scheme = Some(c); self }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl Default for ContentUnavailableView {
    fn default() -> Self {
        Self::new()
    }
}

/// Load the SF Symbol asset from CoreIcon and recolor it to `color` using its
/// alpha channel as a mask (the shipped assets are black glyphs on
/// transparent). The result is cached in the temp dir per (symbol, color).
#[cfg(feature = "coreicon")]
fn sf_icon_path(symbol: &str, color: coreicon::Color) -> Option<String> {
    let coreicon_assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .join("CoreIcon/assets/icons");
    if !coreicon_assets.exists() {
        return None;
    }
    let src = coreicon_assets.join(symbol).with_extension("png");
    if !src.exists() {
        return None;
    }

    let key = format!(
        "cuv_{}_{:02x}{:02x}{:02x}.png",
        symbol.replace('.', "_"),
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );
    let out = std::env::temp_dir().join(key);
    if out.exists() {
        return Some(out.to_str()?.to_string());
    }

    let img = image::open(&src).ok()?;
    let mut rgba = img.to_rgba8();
    let (r, g, b) = (
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
    );
    for p in rgba.pixels_mut() {
        if p[3] > 0 {
            p[0] = r;
            p[1] = g;
            p[2] = b;
        }
    }
    rgba.save(&out).ok()?;
    Some(out.to_str()?.to_string())
}

/// Build the title text: `No Results for "foo"` (a space when empty, matching
/// the reference SwiftUI fallback behavior).
fn title_text(prefix: &str, query: &str) -> String {
    let shown = if query.trim().is_empty() { " " } else { query };
    format!("{} \"{}\"", prefix, shown)
}

impl ViewContent for ContentUnavailableView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let scheme = self.color_scheme.unwrap_or_else(ColorScheme::detect_system);
        let dark = scheme == ColorScheme::Dark;

        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };

        let container = gtk::Box::new(Orientation::Vertical, 0);
        container.set_width_request(w.max(1.0) as i32);
        container.set_height_request(h.max(1.0) as i32);

        // ── Centered empty state ──
        let center = gtk::Box::new(Orientation::Vertical, 0);
        center.set_valign(gtk::Align::Center);
        center.set_halign(gtk::Align::Center);
        center.set_hexpand(true);
        center.set_vexpand(true);

        #[cfg(feature = "coreicon")]
        {
            let icon_color = if dark {
                coreicon::Color::new(0.44, 0.44, 0.47, 1.0) // zinc-500
            } else {
                coreicon::Color::new(0.63, 0.63, 0.67, 1.0) // zinc-400
            };
            if let Some(p) = sf_icon_path("magnifyingglass", icon_color) {
                let icon = gtk::Image::from_file(&p);
                icon.set_pixel_size(64);
                center.append(&icon);
            }
        }

        let title = GtkLabel::new(Some(&title_text(&self.title_prefix, &self.query)));
        title.set_wrap(true);
        title.set_justify(gtk::Justification::Center);
        title.set_max_width_chars((w.max(1.0) / 10.0) as i32);
        let tcss = format!(
            ".cuv-title {{
                font-family: 'SF Pro Display';
                font-size: 22px;
                font-weight: 700;
                letter-spacing: -0.5px;
                color: {c};
            }}",
            c = if dark { TITLE_DARK } else { TITLE_LIGHT },
        );
        uikit::widget::apply_css(&title, &tcss);
        center.append(&title);

        let msg = GtkLabel::new(Some(&self.message));
        msg.set_wrap(true);
        msg.set_justify(gtk::Justification::Center);
        let mcss = format!(
            ".cuv-msg {{
                font-family: 'SF Pro Display';
                font-size: 15px;
                color: {c};
                margin-top: 8px;
            }}",
            c = if dark { MUTE_DARK } else { MUTE_LIGHT },
        );
        uikit::widget::apply_css(&msg, &mcss);
        center.append(&msg);

        container.append(&center);

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool { false }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for ContentUnavailableView {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, self.width, self.height))
    }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuv_builder() {
        let v = ContentUnavailableView::new().query("foo").message("bar").width(400.0);
        assert_eq!(v.query, "foo");
        assert_eq!(v.message, "bar");
        assert_eq!(v.width, 400.0);
        assert_eq!(v.title_prefix, "No Results for");
    }

    #[test]
    fn cuv_title_text() {
        assert_eq!(title_text("No Results for", "foo"), "No Results for \"foo\"");
        assert_eq!(title_text("No Results for", "  "), "No Results for \" \"");
    }
}