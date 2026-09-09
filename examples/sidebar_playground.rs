//! Sidebar playground — tweak the sidebar live and find your look.
//!
//! Left: the sidebar. Right: controls for bold, text size, icon size, text
//! color, edge margins, gaps and row height. Every change rebuilds only the
//! sidebar, so the controls keep focus and drag state.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example sidebar_playground
//! ```

use gtk::prelude::*;
use std::cell::RefCell;
use std::sync::{LazyLock, Mutex};
use tontooui::prelude::*;
#[cfg(feature = "coreicon")]
use tontooui::SidebarIcon;
use uikit::style::VAlignment;
use uikit::widget::{Widget as UIKitWidget, WidgetId, next_widget_id};

// ── Live settings (shared with the Send+Sync control callbacks) ──

#[derive(Clone)]
struct Settings {
    bold: bool,
    font_size: f32,
    icon_size: f32,
    /// Hex color like `#ffffff`. Empty = automatic scheme color.
    text_hex: String,
    /// Row inset from the left/right sidebar edges.
    side_margin: f32,
    /// Gap between icon and label.
    icon_gap: f32,
    /// Vertical gap between rows.
    row_gap: f32,
    row_height: f32,
    /// How far the selected blue pill extends past the row inset.
    selection_overhang: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bold: true,
            font_size: 13.5,
            icon_size: 24.0,
            text_hex: String::new(),
            side_margin: 12.0,
            icon_gap: 8.0,
            row_gap: 2.0,
            row_height: 30.0,
            selection_overhang: 2.0,
        }
    }
}

static SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| Mutex::new(Settings::default()));

// The sidebar holder lives on the GTK main thread; callbacks only swap its
// child, so sliders keep dragging and the text field keeps focus.
thread_local! {
    static SLOT: RefCell<Option<gtk::Box>> = RefCell::new(None);
}

/// Layout slot for the live sidebar: renders the holder box stored in `SLOT`,
/// so `rebuild` can swap the sidebar without touching the window layout.
struct SidebarSlot {
    id: WidgetId,
    box_: gtk::Box,
}

impl UIKitWidget for SidebarSlot {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.box_.clone().upcast()
    }
    fn expand_vertically(&self) -> bool {
        true
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    let t = hex.trim();
    if t.is_empty() {
        return None;
    }
    let with_hash = if t.starts_with('#') {
        t.to_string()
    } else {
        format!("#{t}")
    };
    Color::from_hex(&with_hash)
}

#[cfg(feature = "coreicon")]
fn build_sidebar(s: &Settings) -> Sidebar {
    let mut sb = Sidebar::new()
        .item(
            "Wi-Fi",
            SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)),
        )
        .item(
            "Bluetooth",
            SidebarIcon::sf(
                "antenna.radiowaves.left.and.right",
                Color::from_rgb(0, 122, 255),
            ),
        )
        .item(
            "Network",
            SidebarIcon::sf("globe", Color::from_rgb(0, 122, 255)),
        )
        .item(
            "Battery",
            SidebarIcon::sf("battery.100", Color::from_rgb(52, 199, 89)),
        )
        .section("System")
        .item(
            "General",
            SidebarIcon::sf("gearshape", Color::from_rgb(142, 142, 147)),
        )
        .item(
            "Sound",
            SidebarIcon::sf_gradient(
                "speaker.wave.2.fill",
                coreicon::Gradient::linear_two(
                    coreicon::Color::new(1.0, 0.27, 0.23, 1.0),
                    coreicon::Color::new(1.0, 0.62, 0.04, 1.0),
                ),
            ),
        )
        .selected(0)
        .width(260.0)
        .font_size(s.font_size)
        .bold(s.bold)
        .icon_size(s.icon_size)
        .side_margins(s.side_margin)
        .icon_spacing(s.icon_gap)
        .row_spacing(s.row_gap)
        .row_height(s.row_height)
        .selection_overhang(s.selection_overhang)
        .on_select(|i| println!("Sidebar selected: {i}"));
    if let Some(c) = parse_hex(&s.text_hex) {
        sb = sb.text_color(c);
    }
    sb
}

#[cfg(not(feature = "coreicon"))]
fn build_sidebar(s: &Settings) -> Sidebar {
    let mut sb = Sidebar::new()
        .item("Wi-Fi", "wifi.circle.fill.png")
        .item("Bluetooth", "antenna.radiowaves.left.and.right.png")
        .item("Network", "globe.png")
        .section("System")
        .item("General", "gearshape.png")
        .selected(0)
        .width(260.0)
        .font_size(s.font_size)
        .bold(s.bold)
        .icon_size(s.icon_size)
        .side_margins(s.side_margin)
        .icon_spacing(s.icon_gap)
        .row_spacing(s.row_gap)
        .row_height(s.row_height)
        .selection_overhang(s.selection_overhang)
        .on_select(|i| println!("Sidebar selected: {i}"));
    if let Some(c) = parse_hex(&s.text_hex) {
        sb = sb.text_color(c);
    }
    sb
}

/// Rebuild the sidebar from the current settings and swap it into the holder.
fn rebuild() {
    let settings = SETTINGS.lock().unwrap().clone();
    let widget = UIKitWidget::to_gtk(&build_sidebar(&settings));
    SLOT.with(|slot| {
        if let Some(holder) = slot.borrow().as_ref() {
            while let Some(child) = holder.first_child() {
                holder.remove(&child);
            }
            holder.append(&widget);
        }
    });
}

fn tweak(f: impl FnOnce(&mut Settings)) {
    {
        let mut s = SETTINGS.lock().unwrap();
        f(&mut s);
    }
    rebuild();
}

fn main() {
    // The sidebar holder is a GTK widget created below, before `app.run()`
    // initializes GTK itself — so initialize explicitly first (same pattern
    // as `ColorScheme::detect_system`, which calls `let _ = gtk::init()`).
    let _ = gtk::init();

    let mut app = App::new("Sidebar Playground", 1060, 800);
    app.no_window_bar();

    // Live sidebar holder: fixed 280px, filled by `rebuild`.
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    holder.set_size_request(280, -1);
    holder.set_vexpand(true);
    SLOT.with(|slot| *slot.borrow_mut() = Some(holder.clone()));
    let slot = SidebarSlot {
        id: next_widget_id(),
        box_: holder,
    };

    let dim = Color::from_hex("#98989d").unwrap();
    let controls = VStack::new()
        .spacing(8.0)
        .child(Text::new("Sidebar Playground").font_size(20.0).bold())
        .child(
            Text::new("Every change rebuilds only the sidebar.")
                .font_size(11.0)
                .color(dim),
        )
        .child(Text::new("Text").font_size(14.0).bold())
        .child(Toggle::new("Bold").value(true).on_change(|v| tweak(|s| s.bold = v)))
        .child(
            Slider::new(8.0, 24.0)
                .value(13.5)
                .step(0.5)
                .label("Text size")
                .on_change(|v| tweak(|s| s.font_size = v)),
        )
        .child(
            TextInput::new("#ffffff, empty = auto")
                .on_change(|t| tweak(|s| s.text_hex = t)),
        )
        .child(
            Text::new("Text color as hex, e.g. #ffffff. Empty = automatic.")
                .font_size(11.0)
                .color(dim),
        )
        .child(Text::new("Icons").font_size(14.0).bold())
        .child(
            Slider::new(12.0, 44.0)
                .value(24.0)
                .step(1.0)
                .label("Icon size")
                .on_change(|v| tweak(|s| s.icon_size = v)),
        )
        .child(Text::new("Spacing").font_size(14.0).bold())
        .child(
            Slider::new(0.0, 24.0)
                .value(12.0)
                .step(1.0)
                .label("Left edge space")
                .on_change(|v| tweak(|s| s.side_margin = v)),
        )
        .child(
            Slider::new(0.0, 24.0)
                .value(8.0)
                .step(1.0)
                .label("Icon-text gap")
                .on_change(|v| tweak(|s| s.icon_gap = v)),
        )
        .child(
            Slider::new(0.0, 16.0)
                .value(2.0)
                .step(1.0)
                .label("Row spacing")
                .on_change(|v| tweak(|s| s.row_gap = v)),
        )
        .child(
            Slider::new(20.0, 60.0)
                .value(30.0)
                .step(1.0)
                .label("Row height")
                .on_change(|v| tweak(|s| s.row_height = v)),
        )
        .child(
            Slider::new(0.0, 16.0)
                .value(2.0)
                .step(1.0)
                .label("Selection overhang")
                .on_change(|v| tweak(|s| s.selection_overhang = v)),
        );

    // Sidebar traffic lights draw the window controls, so the panel sits in
    // a plain stack next to them.
    let root = HStack::new()
        .spacing(16.0)
        .alignment(VAlignment::Top)
        .child(slot)
        .child(controls);

    // Fill the holder before the window shows (swapping children works
    // before the box is attached, so no timer is needed).
    rebuild();

    app.set_root(root);
    app.run();
}
