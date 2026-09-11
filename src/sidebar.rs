//! Sidebar — macOS-style sidebar with traffic lights, search, and item list.
//!
//! Apple-style behavior: clicking a row moves the blue selection immediately
//! and fires `on_select`. Pair with [`TabView`](crate::TabView) when the
//! selection should also swap the detail content on the right (like
//! SwiftUI's `.sidebarAdaptable` tab view style).
//!
//! With the `coreicon` feature (default), icons are generated at render time
//! via CoreIcon. Specify a SF Symbol name and a color (or gradient) — the
//! icon PNG is created automatically.
//!
//! Without `coreicon`, pass a PNG file path as a string.
//!
//! ```rust,ignore
//! use tontooui::prelude::*;
//!
//! let sidebar = Sidebar::new()
//!     .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
//!     .item("Bluetooth", SidebarIcon::sf("antenna.radiowaves.left.and.right", Color::from_rgb(0, 122, 255)))
//!     .item("Sound", SidebarIcon::sf_gradient("speaker.wave.2.fill",
//!         coreicon::Gradient::linear_two(
//!             coreicon::Color::new(1.0, 0.27, 0.23, 1.0),
//!             coreicon::Color::new(1.0, 0.62, 0.04, 1.0))))
//!     .section("General")
//!     .item("About", SidebarIcon::sf("info.circle", Color::from_rgb(142, 142, 147)))
//!     .selected(0)
//!     .on_select(|i| println!("Selected: {}", i));
//!
//! let view = View::new(sidebar);
//! ```

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Label as GtkLabel};
use gtk::Orientation;

// ═══════════════════════════════════════════════════════════════
// SidebarIcon — color or gradient per item
// ═══════════════════════════════════════════════════════════════

/// Icon source for a sidebar item — SF Symbol with solid color or gradient,
/// or a finished image file used as-is.
#[cfg(feature = "coreicon")]
pub enum SidebarIcon {
    /// SF Symbol with a solid background color and white foreground.
    Sf { symbol: String, color: Color },
    /// SF Symbol with a gradient background and white foreground.
    SfGradient { symbol: String, gradient: coreicon::Gradient },
    /// Finished artwork (e.g. an app icon PNG) used as-is: center-cropped to
    /// a square and high-quality downscaled, no tile, no boldening.
    File { path: String },
}

#[cfg(feature = "coreicon")]
impl SidebarIcon {
    /// Create an icon from an SF Symbol name with a solid background color.
    pub fn sf(symbol: impl Into<String>, color: Color) -> Self {
        Self::Sf { symbol: symbol.into(), color }
    }

    /// Create an icon from an SF Symbol name with a gradient background.
    pub fn sf_gradient(symbol: impl Into<String>, gradient: coreicon::Gradient) -> Self {
        Self::SfGradient { symbol: symbol.into(), gradient }
    }

    /// Create an icon from a finished image file (PNG/JPG, e.g. an app
    /// icon). The file is center-cropped to a square and Lanczos-downscaled
    /// to 3x the display size — no rounded tile, no recolor, no boldening.
    pub fn file(path: impl Into<String>) -> Self {
        Self::File { path: path.into() }
    }

    /// Render the icon and return a PNG path sized for crisp display.
    ///
    /// `display_px` is the on-screen size (e.g. `Sidebar::icon_size`). The
    /// file is rendered at 3x with high-quality Lanczos3 downscaling: GTK
    /// draws the 1024px master with a single fast downscale step, which
    /// turns thin glyph strokes to mush — a 3x supersampled file stays
    /// sharp on 1x displays and covers HiDPI scale factors 2-3.
    fn to_path(&self, display_px: u32) -> Option<String> {
        // Finished artwork: no canvas, no tile — just a square,
        // high-quality 3x file. The key carries size + mtime so edited
        // source files regenerate instead of serving stale cache.
        if let SidebarIcon::File { path } = self {
            let meta = std::fs::metadata(path).ok()?;
            let mtime = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            let stem = std::path::Path::new(path)
                .file_stem()?
                .to_str()?
                .replace(['.', ' ', '-'], "_");
            let px = display_px.clamp(8, 256);
            let key = format!("sbf_{}_{}_{}_{px}_v2", stem, meta.len(), mtime);
            let out_path = std::env::temp_dir().join(format!("{key}.png"));
            if !out_path.exists() {
                let img = image::open(path).ok()?.to_rgba8();
                let (w, h) = img.dimensions();
                let side = w.min(h);
                let cropped = image::imageops::crop_imm(
                    &img,
                    (w - side) / 2,
                    (h - side) / 2,
                    side,
                    side,
                )
                .to_image();
                let out = (px * 3).min(768);
                let small = image::imageops::resize(
                    &cropped,
                    out,
                    out,
                    image::imageops::FilterType::Lanczos3,
                );
                small.save(&out_path).ok()?;
            }
            return Some(out_path.to_str()?.to_string());
        }

        let sf = match self {
            SidebarIcon::Sf { symbol, .. } => coreicon::SFSymbol::from_name(symbol)?,
            SidebarIcon::SfGradient { symbol, .. } => coreicon::SFSymbol::from_name(symbol)?,
            // Handled by the early return above.
            SidebarIcon::File { .. } => return None,
        };

        // Ensure CoreIcon can find its SF Symbol PNGs
        let coreicon_assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .join("CoreIcon/assets/icons");
        if coreicon_assets.exists() {
            unsafe {
                coreicon::generator::ASSETS_DIR = Box::leak(
                    coreicon_assets.to_str()?.to_string().into_boxed_str(),
                );
            }
        }

        let temp = std::env::temp_dir();
        let key = match self {
            SidebarIcon::Sf { symbol, color } => {
                format!("sb_{}_{:02x}{:02x}{:02x}",
                    symbol.replace('.', "_"),
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8)
            }
            SidebarIcon::SfGradient { symbol, gradient } => {
                let h: u32 = gradient.stops.iter().fold(0u32, |acc, s| {
                    acc.wrapping_add((s.color.r * 255.0) as u32)
                        .wrapping_add((s.color.g * 255.0) as u32)
                        .wrapping_add((s.color.b * 255.0) as u32)
                });
                format!("sb_{}_{:08x}", symbol.replace('.', "_"), h)
            }
            // Handled by the early return above.
            SidebarIcon::File { .. } => return None,
        };
        let master_path = temp.join(format!("{key}.png"));
        if !master_path.exists() {
            let white = coreicon::Color::new(1.0, 1.0, 1.0, 1.0);

            let bg = match self {
                SidebarIcon::Sf { color, .. } => {
                    coreicon::generator::Background::color(to_ci(*color))
                }
            SidebarIcon::SfGradient { gradient, .. } => {
                coreicon::generator::Background::gradient(gradient.clone())
            }
            // Handled by the early return above.
            SidebarIcon::File { .. } => return None,
        };

            let canvas = coreicon::generator::IconCanvas::new()
                .background(bg)
                .corner_radius(220.0)
                .layer(
                    coreicon::generator::Layer::new(coreicon::generator::LayerContent::icon(sf))
                        .position(120.0, 120.0)
                        .size(784.0, 784.0)
                        .tint(white),
                );

            canvas.save(&master_path).ok()?;
        }

        // Sized variant for the requested display size (3x supersampled).
        // The master is boldened once (see `bolden_for_small_sizes`) so thin
        // glyph strokes stay solid instead of dissolving into gray; sizes
        // share the boldened master and only pay a fast Lanczos each.
        let px = display_px.clamp(8, 256);
        let bold_path = temp.join(format!("{key}_bold.png"));
        if !bold_path.exists() {
            let master = image::open(&master_path).ok()?;
            let bold = bolden_for_small_sizes(&master.to_rgba8());
            bold.save(&bold_path).ok()?;
        }
        let sized_path = temp.join(format!("{key}_{px}_v2.png"));
        if !sized_path.exists() {
            let bold = image::open(&bold_path).ok()?;
            let out = (px * 3).min(768);
            let small = image::imageops::resize(
                &bold,
                out,
                out,
                image::imageops::FilterType::Lanczos3,
            );
            small.save(&sized_path).ok()?;
        }
        Some(sized_path.to_str()?.to_string())
    }
}

#[cfg(feature = "coreicon")]
fn to_ci(c: Color) -> coreicon::Color {
    coreicon::Color::new(c.r, c.g, c.b, c.a)
}

/// Thicken glyph strokes by ~1px so they survive minification to sidebar
/// size as solid lines instead of dissolving into gray.
///
/// Two passes over the 1024px master (cached on disk, so this runs once per
/// icon): alpha dilation grows glyphs on transparent ground (dark/light tab
/// glyphs, propagating the peak-alpha color), then bright-stroke dilation
/// grows white glyphs on opaque tiles. Flat areas are untouched.
#[cfg(feature = "coreicon")]
pub(crate) fn bolden_for_small_sizes(master: &image::RgbaImage) -> image::RgbaImage {
    let (w, h) = master.dimensions();
    let src = master.as_raw();

    let at = |buf: &[u8], x: i64, y: i64| -> Option<[u8; 4]> {
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            return None;
        }
        let i = ((y as u32 * w + x as u32) * 4) as usize;
        Some([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]])
    };

    // Pass 1: alpha dilation with color propagation.
    let mut mid = src.clone();
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let own_a = src[i + 3];
            let mut best = [src[i], src[i + 1], src[i + 2], own_a];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if let Some(p) = at(src, x as i64 + dx, y as i64 + dy) {
                        if p[3] > best[3] {
                            best = p;
                        }
                    }
                }
            }
            if best[3] > own_a {
                mid[i..i + 4].copy_from_slice(&best);
            }
        }
    }

    // Pass 2: bright-stroke dilation on opaque ground.
    let mut out = mid.clone();
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if mid[i + 3] <= 128 {
                continue;
            }
            let mut best = [mid[i], mid[i + 1], mid[i + 2]];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(p) = at(&mid, x as i64 + dx, y as i64 + dy) {
                        if p[3] > 128 {
                            for c in 0..3 {
                                if p[c] > best[c] {
                                    best[c] = p[c];
                                }
                            }
                        }
                    }
                }
            }
            out[i] = best[0];
            out[i + 1] = best[1];
            out[i + 2] = best[2];
        }
    }

    image::ImageBuffer::from_raw(w, h, out).expect("icon buffer keeps dimensions")
}

// ═══════════════════════════════════════════════════════════════
// SidebarItem
// ═══════════════════════════════════════════════════════════════

pub struct SidebarItem {
    label: String,
    #[cfg(feature = "coreicon")]
    icon: SidebarIcon,
    #[cfg(not(feature = "coreicon"))]
    icon_path: String,
}

#[cfg(feature = "coreicon")]
impl SidebarItem {
    pub fn new(label: impl Into<String>, icon: SidebarIcon) -> Self {
        Self { label: label.into(), icon }
    }
}

#[cfg(not(feature = "coreicon"))]
impl SidebarItem {
    pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self { label: label.into(), icon_path: icon.into() }
    }
}

// ═══════════════════════════════════════════════════════════════
// Sidebar entries — items with optional section headers between them
// ═══════════════════════════════════════════════════════════════

enum SidebarEntry {
    Section(String),
    Item(SidebarItem),
}

// ═══════════════════════════════════════════════════════════════
// Sidebar
// ═══════════════════════════════════════════════════════════════

/// macOS-style sidebar with traffic lights, search bar, and selectable item list.
///
/// Clicking a row moves the Apple-blue selection immediately and fires
/// `on_select`. The search field filters rows live; section headers hide
/// when none of their items match.
pub struct Sidebar {
    id: WidgetId,
    entries: Vec<SidebarEntry>,
    selected: usize,
    search_placeholder: String,
    show_search: bool,
    background_color: Option<Color>,
    #[cfg(feature = "coreicon")]
    background_gradient: Option<coreicon::Gradient>,
    border_color: Option<Color>,
    glow_color: Option<Color>,
    selected_color: Color,
    font_size: f32,
    bold: bool,
    text_color: Option<Color>,
    icon_size: f32,
    side_margins: f32,
    icon_spacing: f32,
    row_spacing: f32,
    row_height: f32,
    selection_overhang: f32,
    show_traffic_lights: bool,
    width: f32,
    height: f32,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            entries: Vec::new(),
            selected: 0,
            search_placeholder: "Search".into(),
            show_search: true,
            background_color: None,
            #[cfg(feature = "coreicon")]
            background_gradient: None,
            border_color: None,
            glow_color: None,
            selected_color: Color::from_rgb(10, 132, 255),
            font_size: 13.5,
            bold: true,
            text_color: None,
            icon_size: 24.0,
            side_margins: 12.0,
            icon_spacing: 8.0,
            row_spacing: 2.0,
            row_height: 30.0,
            selection_overhang: 2.0,
            show_traffic_lights: true,
            width: 220.0,
            height: 0.0,
            on_select: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Add a sidebar item with a `SidebarIcon` (SF Symbol + color/gradient).
    #[cfg(feature = "coreicon")]
    pub fn item(mut self, label: impl Into<String>, icon: SidebarIcon) -> Self {
        self.entries.push(SidebarEntry::Item(SidebarItem::new(label, icon)));
        self
    }

    /// Add a sidebar item with a PNG icon path.
    #[cfg(not(feature = "coreicon"))]
    pub fn item(mut self, label: impl Into<String>, icon: impl Into<String>) -> Self {
        self.entries.push(SidebarEntry::Item(SidebarItem::new(label, icon)));
        self
    }

    /// Insert a section header above the following items (like SwiftUI's
    /// `TabSection("Foo")` inside a `.sidebarAdaptable` tab view).
    pub fn section(mut self, title: impl Into<String>) -> Self {
        self.entries.push(SidebarEntry::Section(title.into()));
        self
    }

    pub fn selected(mut self, index: usize) -> Self { self.selected = index; self }
    /// Hide the traffic lights at the top of the sidebar. Useful together
    /// with [`App::force_window_bar`](uikit::app::App::force_window_bar):
    /// when the system decoration bar is on anyway, the sidebar drops its
    /// own lights so exactly one set stays visible. Also disables the
    /// automatic decoration-bar hiding (see `hides_window_bar`).
    pub fn without_traffic_lights(mut self) -> Self { self.show_traffic_lights = false; self }
    pub fn search_placeholder(mut self, text: impl Into<String>) -> Self { self.search_placeholder = text.into(); self }
    pub fn no_search(mut self) -> Self { self.show_search = false; self }
    /// Sidebar fill. Defaults to the Apple sidebar color for the current
    /// scheme (dark `#2C2C2E` / light `#F5F5F7`).
    pub fn background_color(mut self, c: Color) -> Self { self.background_color = Some(c); self }
    /// Set the sidebar background to a gradient (overrides `background_color`).
    #[cfg(feature = "coreicon")]
    pub fn background_gradient(mut self, g: coreicon::Gradient) -> Self { self.background_gradient = Some(g); self }
    pub fn border_color(mut self, c: Color) -> Self { self.border_color = Some(c); self }
    pub fn glow_color(mut self, c: Color) -> Self { self.glow_color = Some(c); self }
    pub fn selected_color(mut self, c: Color) -> Self { self.selected_color = c; self }
    /// Label font size in px (default `13.5`).
    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    /// Bold labels (default `true`).
    pub fn bold(mut self, bold: bool) -> Self { self.bold = bold; self }
    /// Label text color. Default (`None`) follows the scheme: near-white in
    /// dark mode, near-black in light mode. Selected rows stay white.
    pub fn text_color(mut self, c: Color) -> Self { self.text_color = Some(c); self }
    /// Icon size in px (default `24`).
    pub fn icon_size(mut self, size: f32) -> Self { self.icon_size = size; self }
    /// Row inset from the left/right sidebar edges in px (default `12`).
    pub fn side_margins(mut self, m: f32) -> Self { self.side_margins = m; self }
    /// Gap between icon and label in px (default `8`).
    pub fn icon_spacing(mut self, s: f32) -> Self { self.icon_spacing = s; self }
    /// Vertical gap between rows (and headers) in px (default `2`).
    pub fn row_spacing(mut self, s: f32) -> Self { self.row_spacing = s; self }
    /// Row height in px (default `30`).
    pub fn row_height(mut self, h: f32) -> Self { self.row_height = h; self }
    /// How far the selected row's blue pill extends past the normal row
    /// inset on the left, in px (default `2`). The icon stays aligned with
    /// the other rows via an inner spacer — only the pill grows. Clamped to
    /// `side_margins`.
    pub fn selection_overhang(mut self, o: f32) -> Self { self.selection_overhang = o; self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn height(mut self, h: f32) -> Self { self.height = h; self }

    pub fn on_select(mut self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    pub fn selected_index(&self) -> usize { self.selected }

    /// Number of selectable items (section headers are not counted).
    pub fn item_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, SidebarEntry::Item(_))).count()
    }

    pub fn to_view(self) -> View {
        let w = self.width;
        View::new(self).with_frame(0.0, 0.0, w, 100.0)
    }
}

impl Default for Sidebar { fn default() -> Self { Self::new() } }

/// Move a color toward white by `t` (0.0 keeps, 1.0 white), with alpha `a`.
fn lighten_toward_white(c: Color, t: f32, a: f32) -> Color {
    Color::new(c.r + (1.0 - c.r) * t, c.g + (1.0 - c.g) * t, c.b + (1.0 - c.b) * t, a)
}

/// Move a color toward black by `t` (0.0 keeps, 1.0 black), with alpha `a`.
fn darken_toward_black(c: Color, t: f32, a: f32) -> Color {
    Color::new(c.r * (1.0 - t), c.g * (1.0 - t), c.b * (1.0 - t), a)
}

/// Automatic border + glow for a background: plain white/black for solid
/// fills, and a subtle whisper of the gradient's own hue for gradients
/// (strongly lightened/darkened, low alpha — never a saturated frame).
/// Pure helper so the math stays unit-testable without GTK.
fn auto_frame_colors(base: Color, is_gradient: bool) -> (Color, Color) {
    let lum = (base.r + base.g + base.b) / 3.0;
    if is_gradient {
        if lum < 0.5 {
            // Dark gradient: near-white tinted with the hue.
            (lighten_toward_white(base, 0.85, 0.18), lighten_toward_white(base, 0.70, 0.08))
        } else {
            // Light gradient: near-black tinted with the hue.
            (darken_toward_black(base, 0.10, 0.18), darken_toward_black(base, 0.05, 0.08))
        }
    } else if lum < 0.5 {
        (Color::new(1.0, 1.0, 1.0, 0.15), Color::new(1.0, 1.0, 1.0, 0.05))
    } else {
        (Color::new(0.0, 0.0, 0.0, 0.15), Color::new(0.0, 0.0, 0.0, 0.05))
    }
}

impl ViewContent for Sidebar {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };

        let container = gtk::Box::new(Orientation::Vertical, 0);
        container.set_width_request(w as i32);
        if h > 0.0 { container.set_height_request(h as i32); }
        container.set_hexpand(false);
        container.set_halign(gtk::Align::Start);
        container.set_vexpand(true);
        container.set_valign(gtk::Align::Fill);

        let dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;

        // Apple sidebar fill when the app did not set an explicit color:
        // dark `#2C2C2E` / light `#F5F5F7`.
        let bg = self.background_color.unwrap_or(if dark {
            Color::from_rgb(44, 44, 46)
        } else {
            Color::from_rgb(245, 245, 247)
        });
        let bg_hex = format!("#{:02x}{:02x}{:02x}",
            (bg.r * 255.0) as u8, (bg.g * 255.0) as u8, (bg.b * 255.0) as u8);

        // Border and glow adapt to the background automatically. For a gradient the
        // frame takes the gradient's own hue (average of the stop colors) and
        // is lightened or darkened for contrast; for a solid fill it is plain
        // white on dark fills and plain black on light fills. Explicit
        // `border_color` / `glow_color` calls override this.
        #[cfg(feature = "coreicon")]
        let gradient_avg: Option<Color> = self.background_gradient.as_ref().map(|g| {
            let n = g.stops.len().max(1) as f32;
            let (mut r, mut gr, mut b) = (0.0f32, 0.0f32, 0.0f32);
            for s in &g.stops {
                r += s.color.r;
                gr += s.color.g;
                b += s.color.b;
            }
            Color::new(r / n, gr / n, b / n, 1.0)
        });
        #[cfg(not(feature = "coreicon"))]
        let gradient_avg: Option<Color> = None;

        let base = gradient_avg.unwrap_or(bg);

        let (auto_border, auto_glow) = auto_frame_colors(base, gradient_avg.is_some());
        let border = self.border_color.unwrap_or(auto_border);
        let glow = self.glow_color.unwrap_or(auto_glow);

        let border_rgba = format!("rgba({:.0},{:.0},{:.0},{:.2})",
            border.r * 255.0, border.g * 255.0, border.b * 255.0, border.a);
        let glow_rgba = format!("rgba({:.0},{:.0},{:.0},{:.2})",
            glow.r * 255.0, glow.g * 255.0, glow.b * 255.0, glow.a);

        #[cfg(feature = "coreicon")]
        let bg_css = match &self.background_gradient {
            Some(g) => {
                let stops = g.stops.iter().map(|s| {
                    format!("rgba({:.0},{:.0},{:.0},{:.2}) {:.0}%",
                        s.color.r * 255.0, s.color.g * 255.0, s.color.b * 255.0, s.color.a,
                        s.position * 100.0)
                }).collect::<Vec<_>>().join(", ");
                if matches!(g.direction, coreicon::GradientDirection::CenterRadial) {
                    format!("radial-gradient(circle at center, {})", stops)
                } else {
                    let direction = match g.direction {
                        coreicon::GradientDirection::TopToBottom => "to bottom",
                        coreicon::GradientDirection::BottomToTop => "to top",
                        coreicon::GradientDirection::LeftToRight => "to right",
                        coreicon::GradientDirection::RightToLeft => "to left",
                        coreicon::GradientDirection::TopLeadingToBottomTrailing => "to bottom right",
                        coreicon::GradientDirection::TopTrailingToBottomLeading => "to bottom left",
                        coreicon::GradientDirection::CenterRadial => "to bottom",
                    };
                    format!("linear-gradient({}, {})", direction, stops)
                }
            }
            None => String::new(),
        };
        #[cfg(not(feature = "coreicon"))]
        let bg_css = String::new();

        // A custom gradient or an explicit border/glow keeps the legacy framed
        // card look. By default the sidebar is Apple-plain: square corners, no
        // margin, no glow — just a 1px separator on the trailing edge.
        let framed = !bg_css.is_empty() || self.border_color.is_some() || self.glow_color.is_some();
        let css = if framed {
            if bg_css.is_empty() {
                format!(".sidebar {{ background-color: {bg_hex}; border-radius: 12px; border: 2px solid {border_rgba}; box-shadow: 0 0 20px {glow_rgba}; overflow: hidden; margin: 6px; }}")
            } else {
                format!(".sidebar {{ background-color: {bg_hex}; background-image: {bg_css}; border-radius: 12px; border: 2px solid {border_rgba}; box-shadow: 0 0 20px {glow_rgba}; overflow: hidden; margin: 6px; }}")
            }
        } else {
            let sep = if dark { "rgba(255,255,255,0.10)" } else { "rgba(0,0,0,0.12)" };
            format!(".sidebar {{ background-color: {bg_hex}; border-radius: 0px; \
                border-width: 0px 1px 0px 0px; border-style: solid; border-color: {sep}; \
                box-shadow: none; margin: 0px; }}")
        };
        uikit::widget::apply_css(&container, &css);
        container.add_css_class("sidebar");

        // Traffic lights are drawn only when no system decoration bar is
        // shown: with the bar on, the bar already has lights, so a second
        // set in the sidebar would duplicate them. The App writes the
        // effective bar visibility before rendering, so in-app this stays
        // consistent with the auto-hide (see `hides_window_bar`).
        if self.show_traffic_lights && !uikit::app::is_window_bar_visible() {
            let tl = uikit::widgets::TrafficLights::new().at(16.0, 18.0).size(21.0);
            // Explicit margins: the sidebar stacks children in a plain box,
            // so `at()` alone would not move the lights away from the roof.
            let tl_widget = tl.to_gtk();
            tl_widget.set_margin_top(18);
            tl_widget.set_margin_start(16);
            tl_widget.set_margin_end(16);
            tl_widget.set_margin_bottom(4);
            container.append(&tl_widget);
        }

        // Row + label colors for the Apple selection model. Every row carries
        // all three states as CSS classes so selection/hover are pure class
        // toggles — no provider is re-registered on click.
        let sel = self.selected_color;
        let sel_hex = format!("rgba({:.0},{:.0},{:.0},{:.2})",
            sel.r * 255.0, sel.g * 255.0, sel.b * 255.0, sel.a);
        let hover_hex = if dark { "rgba(255,255,255,0.07)" } else { "rgba(0,0,0,0.05)" };
        let label_hex = match self.text_color {
            Some(c) => format!("rgba({:.0},{:.0},{:.0},{:.2})",
                c.r * 255.0, c.g * 255.0, c.b * 255.0, c.a),
            None => if dark { "rgba(235,235,245,0.9)".to_string() } else { "#1d1d1d".to_string() },
        };
        let section_hex = if dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.6)" };

        if self.show_search {
            let search_outer = gtk::Box::new(Orientation::Vertical, 0);
            search_outer.set_margin_top(10);
            search_outer.set_margin_bottom(4);
            search_outer.set_margin_start(10);
            search_outer.set_margin_end(10);

            let entry = gtk::SearchEntry::new();
            entry.set_placeholder_text(Some(&self.search_placeholder));
            // Round, borderless search field (14px radius, no border in any
            // state) matching the current sidebar fill.
            let accent_hex = "#0c85ee";
            let (field_bg, field_fg) = if dark {
                ("#3a3a3c", "#ececec")
            } else {
                ("#ffffff", "#1d1d1d")
            };
            let ecss = format!(
                ".sb-search {{
                    background-color: {field_bg};
                    color: {field_fg};
                    border-radius: 14px;
                    border: none;
                    box-shadow: none;
                    padding: 6px 12px;
                    min-height: 0px;
                    font-family: 'SF Pro Display';
                    font-size: 13px;
                    caret-color: {accent};
                }}",
                accent = accent_hex,
            );
            uikit::widget::apply_css(&entry, &ecss);
            entry.add_css_class("sb-search");
            search_outer.append(&entry);
            container.append(&search_outer);
        }

        let items_box = gtk::Box::new(Orientation::Vertical, self.row_spacing as i32);
        items_box.set_margin_top(4);
        items_box.add_css_class("sb-list");

        let selected = Rc::new(RefCell::new(self.selected));
        let cb = self.on_select.clone();

        // Row widgets for live selection + search filtering. The position in
        // `rows` is the item index (`on_select` value); section headers live
        // in `headers` with the item range they group.
        let mut rows: Vec<gtk::Box> = Vec::new();
        let mut row_labels: Vec<GtkLabel> = Vec::new();
        let mut row_texts: Vec<String> = Vec::new();
        // Leading spacers (one per row) that absorb `selection_overhang` so
        // the icon stays aligned while the selected pill grows left.
        let mut spacers: Vec<gtk::Box> = Vec::new();
        let base_margin = self.side_margins.max(0.0) as i32;
        let overhang = self.selection_overhang.max(0.0).min(self.side_margins.max(0.0)) as i32;
        let mut headers: Vec<(GtkLabel, usize, usize)> = Vec::new();
        // Widgets in entry order (headers + rows interleaved).
        let mut ordered: Vec<gtk::Widget> = Vec::new();
        let mut item_idx = 0usize;

        for entry_item in &self.entries {
            match entry_item {
                SidebarEntry::Section(title) => {
                    let header = GtkLabel::new(Some(title));
                    header.set_halign(gtk::Align::Start);
                    header.set_margin_top(10);
                    header.set_margin_bottom(2);
                    header.set_margin_start(14);
                    header.set_margin_end(14);
                    let hcss = format!(".sb-section {{ font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; color: {section_hex}; }}");
                    uikit::widget::apply_css(&header, &hcss);
                    header.add_css_class("sb-section");
                    headers.push((header.clone(), item_idx, item_idx));
                    ordered.push(header.upcast());
                }
                SidebarEntry::Item(item) => {
                    let row = gtk::Box::new(Orientation::Horizontal, self.icon_spacing as i32);
                    row.set_height_request(self.row_height as i32);
                    row.set_margin_start(self.side_margins as i32);
                    row.set_margin_end(self.side_margins as i32);
                    row.set_valign(gtk::Align::Center);
                    row.set_focusable(true);
                    row.add_css_class("sb-row");
                    let rcss = format!(
                        ".sb-row {{ background-color: transparent; background-image: none; border-radius: 7px; }} \
                         .sb-row.sb-hover {{ background-color: {hover_hex}; }} \
                         .sb-row.sb-sel {{ background-color: {sel_hex}; background-image: none; }}");
                    uikit::widget::apply_css(&row, &rcss);

                    // Absorbs the selection overhang (width set on select).
                    let spacer = gtk::Box::new(Orientation::Vertical, 0);
                    spacer.set_size_request(0, 1);
                    row.append(&spacer);

                    let icon_px = self.icon_size as i32;
                    #[cfg(feature = "coreicon")]
                    match item.icon.to_path(icon_px.max(1) as u32) {
                        Some(path) => {
                            let img = gtk::Image::from_file(&path);
                            img.set_pixel_size(icon_px);
                            row.append(&img);
                        }
                        None => {
                            let placeholder = gtk::Box::new(Orientation::Vertical, 0);
                            placeholder.set_size_request(icon_px, icon_px);
                            row.append(&placeholder);
                        }
                    }
                    #[cfg(not(feature = "coreicon"))]
                    {
                        if std::path::Path::new(&item.icon_path).exists() {
                            let icon = gtk::Image::from_file(&item.icon_path);
                            icon.set_pixel_size(icon_px);
                            row.append(&icon);
                        } else {
                            let placeholder = gtk::Box::new(Orientation::Vertical, 0);
                            placeholder.set_size_request(icon_px, icon_px);
                            row.append(&placeholder);
                        }
                    }

                    let label = GtkLabel::new(Some(&item.label));
                    label.set_halign(gtk::Align::Start);
                    label.set_hexpand(true);
                    // Truncate long labels with an ellipsis instead of letting the
                    // text overflow the row and get hard-clipped mid-letter.
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    let weight = if self.bold { "700" } else { "400" };
                    let lcss = format!(
                        ".sb-lbl {{ font-family: 'SF Pro Display'; font-size: {}px; font-weight: {}; color: {label_hex}; }} \
                         .sb-lbl.sb-lbl-sel {{ color: white; }}",
                        self.font_size, weight);
                    uikit::widget::apply_css(&label, &lcss);
                    label.add_css_class("sb-lbl");
                    row.append(&label);

                    // Hover highlight (pure class toggle, no repaint).
                    let row_hover = row.clone();
                    let motion = gtk::EventControllerMotion::new();
                    motion.connect_enter(move |_, _, _| {
                        row_hover.add_css_class("sb-hover");
                    });
                    let row_unhover = row.clone();
                    motion.connect_leave(move |_| {
                        row_unhover.remove_css_class("sb-hover");
                    });
                    row.add_controller(motion);

                    rows.push(row.clone());
                    row_labels.push(label);
                    row_texts.push(item.label.clone());
                    spacers.push(spacer);
                    ordered.push(row.clone().upcast());
                    if let Some((_, _, end)) = headers.last_mut() {
                        *end = item_idx + 1;
                    }
                    item_idx += 1;
                }
            }
        }

        // Apply the initial selection.
        let initial = (*selected.borrow()).min(rows.len().saturating_sub(1));
        *selected.borrow_mut() = initial;
        for (k, row) in rows.iter().enumerate() {
            if k == initial {
                row.add_css_class("sb-sel");
                row_labels[k].add_css_class("sb-lbl-sel");
                row.set_margin_start(base_margin - overhang);
                spacers[k].set_size_request(overhang, 1);
            }
        }

        // Click moves the selection immediately (Apple behavior) and then
        // fires the callback. `released` + button 1 = a real left-click.
        let rows_cb: Rc<Vec<gtk::Box>> = Rc::new(rows);
        let labels_cb: Rc<Vec<GtkLabel>> = Rc::new(row_labels);
        let spacers_cb: Rc<Vec<gtk::Box>> = Rc::new(spacers);
        for (i, row) in rows_cb.iter().enumerate() {
            if let Some(ref h) = cb {
                let h = h.clone();
                let rows_inner = rows_cb.clone();
                let labels_inner = labels_cb.clone();
                let spacers_inner = spacers_cb.clone();
                let selected_inner = selected.clone();
                let g = gtk::GestureClick::new();
                g.set_button(1);
                g.connect_released(move |_, _, _, _| {
                    *selected_inner.borrow_mut() = i;
                    for (k, r) in rows_inner.iter().enumerate() {
                        if k == i {
                            r.add_css_class("sb-sel");
                            labels_inner[k].add_css_class("sb-lbl-sel");
                            r.set_margin_start(base_margin - overhang);
                            spacers_inner[k].set_size_request(overhang, 1);
                        } else {
                            r.remove_css_class("sb-sel");
                            labels_inner[k].remove_css_class("sb-lbl-sel");
                            r.set_margin_start(base_margin);
                            spacers_inner[k].set_size_request(0, 1);
                        }
                    }
                    h(i);
                });
                row.add_controller(g);
            }
        }

        // Live search filter: hide non-matching rows, hide section headers
        // whose items are all hidden.
        let rows_filter = rows_cb.clone();
        let texts_filter: Rc<Vec<String>> = Rc::new(row_texts);
        let headers_filter: Rc<Vec<(GtkLabel, usize, usize)>> = Rc::new(headers);
        // The search entry is the last child appended so far (after traffic
        // lights): find it by walking down to the SearchEntry.
        {
            let mut entry_opt: Option<gtk::SearchEntry> = None;
            let mut child = container.first_child();
            while let Some(w) = child {
                let next = w.next_sibling();
                let mut inner = w.first_child();
                while let Some(v) = inner {
                    let vnext = v.next_sibling();
                    if let Ok(e) = v.clone().downcast::<gtk::SearchEntry>() {
                        entry_opt = Some(e);
                        break;
                    }
                    // One more level (entry sits in search_outer box).
                    let mut deep = v.first_child();
                    while let Some(d) = deep {
                        let dnext = d.next_sibling();
                        if let Ok(e) = d.clone().downcast::<gtk::SearchEntry>() {
                            entry_opt = Some(e);
                            break;
                        }
                        deep = dnext;
                    }
                    if entry_opt.is_some() { break; }
                    inner = vnext;
                }
                if entry_opt.is_some() { break; }
                child = next;
            }
            if let Some(entry) = entry_opt {
                entry.connect_search_changed(move |e| {
                    let q = e.text().to_string().to_lowercase();
                    for (k, row) in rows_filter.iter().enumerate() {
                        let vis = q.is_empty()
                            || texts_filter.get(k).map(|t| t.to_lowercase().contains(&q)).unwrap_or(true);
                        row.set_visible(vis);
                    }
                    for (header, start, end) in headers_filter.iter() {
                        if q.is_empty() {
                            header.set_visible(true);
                            continue;
                        }
                        let mut any = false;
                        for k in *start..(*end).min(rows_filter.len()) {
                            if rows_filter[k].is_visible() {
                                any = true;
                                break;
                            }
                        }
                        header.set_visible(any);
                    }
                });
            }
        }
        for widget in ordered {
            items_box.append(&widget);
        }

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_child(Some(&items_box));
        scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
        scroll.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
        uikit::smooth_scroll::apply_smooth_scrolling(&scroll);
        // Keep the whole list area transparent so the sidebar
        // `background_gradient` stays visible behind the items: the
        // app-level CSS paints every scrolledwindow/viewport with the
        // solid window background (#1d1d1d dark / #ececec light), which
        // would otherwise cover the gradient exactly in the item area
        // (top area outside the scroll keeps showing the gradient).
        // Each layer gets its OWN provider matching ITSELF — ancestor
        // descendant selectors alone do not reliably reach the internal
        // viewport, so the provider is attached directly to the scroll,
        // the viewport child and the list box. Rows stay untouched so
        // the `.sb-sel` highlight keeps working.
        scroll.add_css_class("sb-scroll");
        uikit::widget::apply_css(
            &scroll,
            ".sb-scroll { background-color: transparent; background-image: none; \
                border-color: transparent; box-shadow: none; }",
        );
        uikit::widget::apply_css(
            &items_box,
            ".sb-list { background-color: transparent; background-image: none; \
                border-color: transparent; box-shadow: none; }",
        );
        // The Box child is wrapped in an internal GtkViewport; style that
        // widget directly instead of relying on cascade selectors.
        let mut child = scroll.first_child();
        while let Some(w) = child {
            let next = w.next_sibling();
            if let Ok(viewport) = w.clone().downcast::<gtk::Viewport>() {
                viewport.add_css_class("sb-viewport");
                uikit::widget::apply_css(
                    &viewport,
                    ".sb-viewport { background-color: transparent; background-image: none; \
                        border-color: transparent; box-shadow: none; }",
                );
            }
            child = next;
        }
        // Backup: same transparency via the container so late-created
        // internal nodes are covered as well.
        let sb_provider = gtk::CssProvider::new();
        sb_provider.load_from_string(
            ".sidebar scrolledwindow, .sidebar viewport, .sidebar .sb-list { \
                background-color: transparent; background-image: none; \
                border-color: transparent; box-shadow: none; }",
        );
        container.style_context().add_provider(
            &sb_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER as u32,
        );
        container.append(&scroll);

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool { false }
    fn size_that_fits(&self, available: Size) -> Size {
        let h = if self.height > 0.0 { self.height } else { available.height };
        Size::new(self.width, h)
    }
}

impl Widget for Sidebar {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        let w = self.render(Rect::new(0.0, 0.0, self.width, self.height));
        w.set_vexpand(true);
        w.set_valign(gtk::Align::Fill);
        w
    }
    fn is_interactive(&self) -> bool { true }
    fn expand_vertically(&self) -> bool { true }
    fn fill_width(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
    /// A sidebar with traffic lights hides the system decoration bar (which
    /// would duplicate them); without its own lights it hides nothing.
    /// The App hides the bar automatically via
    /// `hides_window_bar_recursive`, unless forced on with
    /// [`App::force_window_bar`](uikit::app::App::force_window_bar) — then
    /// the sidebar drops its own lights instead.
    fn hides_window_bar(&self) -> bool { self.show_traffic_lights }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_sf() {
        let icon = SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255));
        assert!(icon.to_path(24).is_some());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_no_selection_badge() {
        // Apple shows no badge on the selected icon: the path is stable and
        // the selection is purely the blue row highlight.
        let icon = SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255));
        let a = icon.to_path(24);
        let b = icon.to_path(24);
        assert!(a.is_some());
        assert_eq!(a, b);
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_sized_supersampled() {
        // The returned file is 3x the display size (crisp on 1x, covers
        // HiDPI 2-3x) instead of the 1024px master GTK would downscale.
        let icon = SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255));
        let path = icon.to_path(24).expect("icon path");
        assert!(path.ends_with("_24_v2.png"));
        let img = image::open(&path).expect("sized icon readable");
        assert_eq!((img.width(), img.height()), (72, 72));
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_file() {
        // Finished artwork is used as-is: square-cropped, 3x downscaled.
        let demo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("CoreIcon/examples/app_icon_demo/images_dark.png");
        let icon = SidebarIcon::file(demo.to_str().unwrap());
        let path = icon.to_path(28).expect("file icon path");
        assert!(path.contains("_28_v2"));
        let img = image::open(&path).expect("file icon readable");
        assert_eq!((img.width(), img.height()), (84, 84));
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_file_missing() {
        let icon = SidebarIcon::file("/tmp/does-not-exist-tontooui.png");
        assert!(icon.to_path(28).is_none());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_gradient() {
        let g = coreicon::Gradient::linear_two(
            to_ci(Color::from_rgb(255, 69, 58)), to_ci(Color::from_rgb(255, 159, 10)));
        let icon = SidebarIcon::sf_gradient("speaker.wave.2.fill", g);
        assert!(icon.to_path(24).is_some());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_builder() {
        let sb = Sidebar::new()
            .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
            .selected(0)
            .width(220.0);
        assert_eq!(sb.item_count(), 1);
        assert_eq!(sb.selected, 0);
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_background_gradient() {
        let g = coreicon::Gradient::linear_two(
            to_ci(Color::from_rgb(30, 30, 34)),
            to_ci(Color::from_rgb(0, 122, 255)),
        );
        let sb = Sidebar::new().background_gradient(g.clone());
        assert!(sb.background_gradient.is_some());
        assert_eq!(sb.background_gradient.as_ref().unwrap().stops.len(), 2);
    }

    #[cfg(not(feature = "coreicon"))]
    #[test]
    fn sidebar_builder() {
        let sb = Sidebar::new()
            .item("Wi-Fi", "wifi.circle.fill.png")
            .selected(0)
            .width(220.0);
        assert_eq!(sb.item_count(), 1);
        assert_eq!(sb.selected, 0);
    }

    #[test]
    fn sidebar_sections_do_not_count_as_items() {
        let sb = Sidebar::new().section("Foo").no_search();
        assert_eq!(sb.item_count(), 0);
        assert_eq!(sb.entries.len(), 1);
    }

    #[test]
    fn sidebar_default_selection_is_apple_blue() {
        let sb = Sidebar::new();
        assert_eq!(sb.selected_color, Color::from_rgb(10, 132, 255));
    }

    #[test]
    fn sidebar_default_background_follows_scheme() {
        // No explicit color: render falls back to the Apple sidebar fill.
        let sb = Sidebar::new();
        assert!(sb.background_color.is_none());
    }

    #[test]
    fn sidebar_styling_defaults() {
        let sb = Sidebar::new();
        assert_eq!(sb.font_size, 13.5);
        assert!(sb.bold);
        assert!(sb.text_color.is_none());
        assert_eq!(sb.icon_size, 24.0);
        assert_eq!(sb.side_margins, 12.0);
        assert_eq!(sb.icon_spacing, 8.0);
        assert_eq!(sb.row_spacing, 2.0);
        assert_eq!(sb.row_height, 30.0);
        assert_eq!(sb.selection_overhang, 2.0);
    }

    #[test]
    fn sidebar_styling_builders() {
        let sb = Sidebar::new()
            .font_size(15.0)
            .bold(true)
            .text_color(Color::from_rgb(255, 255, 255))
            .icon_size(28.0)
            .side_margins(10.0)
            .icon_spacing(8.0)
            .row_spacing(4.0)
            .row_height(36.0)
            .selection_overhang(8.0);
        assert_eq!(sb.font_size, 15.0);
        assert!(sb.bold);
        assert_eq!(sb.text_color, Some(Color::from_rgb(255, 255, 255)));
        assert_eq!(sb.icon_size, 28.0);
        assert_eq!(sb.side_margins, 10.0);
        assert_eq!(sb.icon_spacing, 8.0);
        assert_eq!(sb.row_spacing, 4.0);
        assert_eq!(sb.row_height, 36.0);
        assert_eq!(sb.selection_overhang, 8.0);
    }

    #[test]
    fn sidebar_no_search() {
        let sb = Sidebar::new().no_search();
        assert!(!sb.show_search);
    }

    #[test]
    fn sidebar_traffic_lights_toggle() {
        let sb = Sidebar::new();
        assert!(sb.show_traffic_lights);
        assert!(sb.hides_window_bar());
        let bare = Sidebar::new().without_traffic_lights();
        assert!(!bare.show_traffic_lights);
        assert!(!bare.hides_window_bar());
    }

    #[test]
    fn auto_frame_gradient_stays_subtle() {
        // Demo gradient (dark green): frame must be near-white, low alpha —
        // never a saturated green border.
        let avg = Color::new(0.08, 0.34, 0.16, 1.0);
        let (border, glow) = auto_frame_colors(avg, true);
        assert_eq!(border.a, 0.18);
        assert_eq!(glow.a, 0.08);
        assert!(border.r > 0.8 && border.g > 0.8 && border.b > 0.8);
        // Solid dark fill keeps the plain-white subtle frame.
        let (b2, g2) = auto_frame_colors(Color::new(0.15, 0.15, 0.15, 1.0), false);
        assert_eq!((b2.r, b2.g, b2.b, b2.a), (1.0, 1.0, 1.0, 0.15));
        assert_eq!((g2.r, g2.g, g2.b, g2.a), (1.0, 1.0, 1.0, 0.05));
    }
}