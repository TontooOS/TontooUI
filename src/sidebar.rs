//! Sidebar — macOS-style sidebar with traffic lights, search, and item list.
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
//!     .selected(0)
//!     .on_select(|i| println!("Selected: {}", i));
//!
//! let view = View::new(sidebar);
//! ```

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

/// Icon source for a sidebar item — SF Symbol with solid color or gradient.
#[cfg(feature = "coreicon")]
pub enum SidebarIcon {
    /// SF Symbol with a solid background color and white foreground.
    Sf { symbol: String, color: Color },
    /// SF Symbol with a gradient background and white foreground.
    SfGradient { symbol: String, gradient: coreicon::Gradient },
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

    fn to_path(&self, selected: bool) -> Option<String> {
        let sf = match self {
            SidebarIcon::Sf { symbol, .. } => coreicon::SFSymbol::from_name(symbol)?,
            SidebarIcon::SfGradient { symbol, .. } => coreicon::SFSymbol::from_name(symbol)?,
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
                format!("sb_{}_{:02x}{:02x}{:02x}{}.png",
                    symbol.replace('.', "_"),
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    if selected { "_sel" } else { "" })
            }
            SidebarIcon::SfGradient { symbol, gradient } => {
                let h: u32 = gradient.stops.iter().fold(0u32, |acc, s| {
                    acc.wrapping_add((s.color.r * 255.0) as u32)
                        .wrapping_add((s.color.g * 255.0) as u32)
                        .wrapping_add((s.color.b * 255.0) as u32)
                });
                format!("sb_{}_{:08x}{}.png", symbol.replace('.', "_"), h,
                    if selected { "_sel" } else { "" })
            }
        };
        let path = temp.join(&key);
        if path.exists() {
            return Some(path.to_str()?.to_string());
        }

        let white = coreicon::Color::new(1.0, 1.0, 1.0, 1.0);

        let bg = match self {
            SidebarIcon::Sf { color, .. } => {
                coreicon::generator::Background::color(to_ci(*color))
            }
            SidebarIcon::SfGradient { gradient, .. } => {
                coreicon::generator::Background::gradient(gradient.clone())
            }
        };

        let mut canvas = coreicon::generator::IconCanvas::new()
            .background(bg)
            .corner_radius(220.0)
            .padding(120.0)
            .layer(
                coreicon::generator::Layer::new(coreicon::generator::LayerContent::icon(sf))
                    .position(0.0, 0.0)
                    .size(1024.0, 1024.0)
                    .tint(white),
            );

        if selected {
            let checkmark_sf = coreicon::SFSymbol::from_name("checkmark")?;
            // Blue circle badge (bottom-right)
            canvas = canvas.layer(
                coreicon::generator::Layer::new(coreicon::generator::LayerContent::circle(320.0))
                    .position(640.0, 640.0)
                    .tint(coreicon::Color::new(0.2, 0.2, 0.22, 1.0)),
            );
            // White checkmark inside the circle
            canvas = canvas.layer(
                coreicon::generator::Layer::new(coreicon::generator::LayerContent::icon(checkmark_sf))
                    .position(700.0, 680.0)
                    .size(200.0, 200.0)
                    .tint(coreicon::Color::new(1.0, 1.0, 1.0, 1.0)),
            );
        }

        canvas.save(&path).ok()?;
        Some(path.to_str()?.to_string())
    }
}

#[cfg(feature = "coreicon")]
fn to_ci(c: Color) -> coreicon::Color {
    coreicon::Color::new(c.r, c.g, c.b, c.a)
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
// Sidebar
// ═══════════════════════════════════════════════════════════════

/// macOS-style sidebar with traffic lights, search bar, and selectable item list.
pub struct Sidebar {
    id: WidgetId,
    items: Vec<SidebarItem>,
    selected: usize,
    search_placeholder: String,
    show_search: bool,
    background_color: Color,
    border_color: Color,
    glow_color: Color,
    selected_color: Color,
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
            items: Vec::new(),
            selected: 0,
            search_placeholder: "Search".into(),
            show_search: true,
            background_color: Color::new(0.15, 0.15, 0.15, 1.0),
            border_color: Color::new(1.0, 1.0, 1.0, 0.15),
            glow_color: Color::new(1.0, 1.0, 1.0, 0.05),
            selected_color: Color::new(1.0, 1.0, 1.0, 0.1),
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
        self.items.push(SidebarItem::new(label, icon));
        self
    }

    /// Add a sidebar item with a PNG icon path.
    #[cfg(not(feature = "coreicon"))]
    pub fn item(mut self, label: impl Into<String>, icon: impl Into<String>) -> Self {
        self.items.push(SidebarItem::new(label, icon));
        self
    }

    pub fn selected(mut self, index: usize) -> Self { self.selected = index; self }
    pub fn search_placeholder(mut self, text: impl Into<String>) -> Self { self.search_placeholder = text.into(); self }
    pub fn no_search(mut self) -> Self { self.show_search = false; self }
    pub fn background_color(mut self, c: Color) -> Self { self.background_color = c; self }
    pub fn border_color(mut self, c: Color) -> Self { self.border_color = c; self }
    pub fn glow_color(mut self, c: Color) -> Self { self.glow_color = c; self }
    pub fn selected_color(mut self, c: Color) -> Self { self.selected_color = c; self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn height(mut self, h: f32) -> Self { self.height = h; self }

    pub fn on_select(mut self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    pub fn selected_index(&self) -> usize { self.selected }

    pub fn to_view(self) -> View {
        let w = self.width;
        View::new(self).with_frame(0.0, 0.0, w, 100.0)
    }
}

impl Default for Sidebar { fn default() -> Self { Self::new() } }

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

        let bg = self.background_color;
        let border = self.border_color;
        let glow = self.glow_color;
        let bg_hex = format!("#{:02x}{:02x}{:02x}",
            (bg.r * 255.0) as u8, (bg.g * 255.0) as u8, (bg.b * 255.0) as u8);
        let border_rgba = format!("rgba({:.0},{:.0},{:.0},{:.2})",
            border.r * 255.0, border.g * 255.0, border.b * 255.0, border.a);
        let glow_rgba = format!("rgba({:.0},{:.0},{:.0},{:.2})",
            glow.r * 255.0, glow.g * 255.0, glow.b * 255.0, glow.a);

        let css = format!(".sidebar {{ background-color: {bg_hex}; border-radius: 15px; border: 2px solid {border_rgba}; box-shadow: 0 0 20px {glow_rgba}; overflow: hidden; margin: 6px; }}");
        uikit::widget::apply_css(&container, &css);
        container.add_css_class("sidebar");

        let tl = uikit::widgets::TrafficLights::new().at(16.0, 12.0).size(14.0);
        container.append(&tl.to_gtk());

        if self.show_search {
            let entry = gtk::SearchEntry::new();
            entry.set_placeholder_text(Some(&self.search_placeholder));
            let ecss = ".sb-search {{ background-color: rgba(255,255,255,0.08); border-radius: 8px; border: none; padding: 6px 10px; margin: 0 14px 8px 14px; font-family: 'SF Pro Display'; font-size: 13px; color: rgba(235,235,245,0.6); }}";
            uikit::widget::apply_css(&entry, ecss);
            entry.add_css_class("sb-search");
            container.append(&entry);
        }

        let items_box = gtk::Box::new(Orientation::Vertical, 0);
        items_box.set_margin_top(4);

        let sel = self.selected;
        let cb = self.on_select.clone();
        let sel_color = self.selected_color;

        for (i, item) in self.items.iter().enumerate() {
            let row = gtk::Box::new(Orientation::Horizontal, 10);
            row.set_height_request(32);
            row.set_margin_start(14);
            row.set_margin_end(14);
            row.set_valign(gtk::Align::Center);

            #[cfg(feature = "coreicon")]
            match item.icon.to_path(i == sel) {
                Some(path) => {
                    let img = gtk::Image::from_file(&path);
                    img.set_pixel_size(22);
                    row.append(&img);
                }
                None => {
                    let placeholder = gtk::Box::new(Orientation::Vertical, 0);
                    placeholder.set_size_request(22, 22);
                    row.append(&placeholder);
                }
            }
            #[cfg(not(feature = "coreicon"))]
            {
                let icon = gtk::Image::from_file(&item.icon_path);
                icon.set_pixel_size(22);
                row.append(&icon);
            }

            let label = GtkLabel::new(Some(&item.label));
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            let lcss = ".sb-lbl {{ font-family: 'SF Pro Display'; font-size: 13px; color: rgba(235,235,245,0.8); }}";
            uikit::widget::apply_css(&label, lcss);
            label.add_css_class("sb-lbl");
            row.append(&label);

            if i == sel {
                let sc = sel_color;
                let sel_hex = format!("rgba({:.0},{:.0},{:.0},{:.2})",
                    sc.r * 255.0, sc.g * 255.0, sc.b * 255.0, sc.a);
                let scss = format!(".sb-sel {{ background-color: {sel_hex}; border-radius: 8px; }}");
                uikit::widget::apply_css(&row, &scss);
                row.add_css_class("sb-sel");
            }

            if let Some(ref h) = cb {
                let h = h.clone();
                let g = gtk::GestureClick::new();
                g.connect_pressed(move |_, _, _, _| { h(i); });
                row.add_controller(g);
            }
            items_box.append(&row);
        }

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_child(Some(&items_box));
        scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
        scroll.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_sf() {
        let icon = SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255));
        assert!(icon.to_path(false).is_some());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_selected() {
        let icon = SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255));
        assert!(icon.to_path(true).is_some());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_icon_gradient() {
        let g = coreicon::Gradient::linear_two(
            to_ci(Color::from_rgb(255, 69, 58)), to_ci(Color::from_rgb(255, 159, 10)));
        let icon = SidebarIcon::sf_gradient("speaker.wave.2.fill", g);
        assert!(icon.to_path(false).is_some());
    }

    #[cfg(feature = "coreicon")]
    #[test]
    fn sidebar_builder() {
        let sb = Sidebar::new()
            .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
            .selected(0)
            .width(220.0);
        assert_eq!(sb.items.len(), 1);
        assert_eq!(sb.selected, 0);
    }

    #[cfg(not(feature = "coreicon"))]
    #[test]
    fn sidebar_builder() {
        let sb = Sidebar::new()
            .item("Wi-Fi", "wifi.circle.fill.png")
            .selected(0)
            .width(220.0);
        assert_eq!(sb.items.len(), 1);
        assert_eq!(sb.selected, 0);
    }

    #[test]
    fn sidebar_no_search() {
        let sb = Sidebar::new().no_search();
        assert!(!sb.show_search);
    }
}