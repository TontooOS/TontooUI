//! TabView — Creates Tabs with title, image, systemImage and custom Label.
//!
//! Functional container in the Apple `.sidebarAdaptable` style: a title bar
//! with traffic lights, sidebar toggle and title on top, a sidebar with the
//! tabs and section headers on the left, and the selected tab's detail
//! content on the right.
//!
//! ```rust,ignore
//! use tontooui::prelude::*;
//! use tontooui::{Tab, TabSection, TabView};
//!
//! let view = TabView::new()
//!     .title("ExploreSwiftUISandbox")
//!     .tab(Tab::new(Text::new("0").font_size(28.0)))
//!     .section(TabSection::new().header("Foo").tab(
//!         Tab::new(Text::new("1").font_size(28.0))
//!             .title("1")
//!             .system_image("1.circle"),
//!     ))
//!     .tab(Tab::new(Text::new("2").font_size(28.0))
//!         .title("2")
//!         .image("cats24x24"))
//!     .tab(Tab::new(Text::new("3").font_size(28.0))
//!         .title("3")
//!         .system_image("3.circle"))
//!     .selected(0)
//!     .on_select(|i| println!("Tab selected: {}", i))
//!     .to_view();
//! ```
//!
//! Without tabs the element keeps its legacy `180x80` preview rendering, so
//! the `tab_views` example gallery is unaffected.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::tab::Tab;
use super::tab_section::TabSection;
use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// TabView — initializer — Creates Tabs with title, image, systemImage and custom Label.
/// With tabs: Apple `.sidebarAdaptable` container (title bar + sidebar +
/// detail). Without tabs: legacy `180x80` preview directly on window
/// (#1d1d1d dark / #ececec light), SF Pro.
pub struct TabView {
    id: WidgetId,
    title: String,
    tabs: Vec<Tab>,
    sections: Vec<TabSection>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    sidebar_width: f32,
    show_toggle: bool,
    position_mode: PositionMode,
    position: Position,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            title: String::new(),
            tabs: Vec::new(),
            sections: Vec::new(),
            selected: 0,
            on_select: None,
            sidebar_width: 220.0,
            show_toggle: true,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Window title shown bold in the title bar (SwiftUI window title).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Append a top-level tab (shown above all sections).
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Append a section with its tabs (SwiftUI's `TabSection("Foo") { ... }`).
    /// Empty sections (no tabs) are ignored at render time.
    pub fn section(mut self, section: TabSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Initially selected tab (flat index over top-level tabs first, then
    /// section tabs in order). Defaults to `0`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Callback fired with the flat tab index whenever the user clicks a row.
    pub fn on_select(mut self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    /// Sidebar width in px (default `220`).
    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }

    /// Show or hide the sidebar toggle button in the title bar.
    pub fn show_toggle(mut self, show: bool) -> Self {
        self.show_toggle = show;
        self
    }

    /// Number of selectable tabs (top-level plus all section tabs).
    pub fn tab_count(&self) -> usize {
        self.tabs.len() + self.sections.iter().map(|s| s.tab_count()).sum::<usize>()
    }

    /// Whether the view holds any tabs (otherwise the legacy preview renders).
    pub fn is_empty(&self) -> bool {
        self.tab_count() == 0
    }

    pub fn to_view(self) -> View {
        if self.is_empty() {
            View::new(self).with_frame(0.0, 0.0, 180.0, 80.0)
        } else {
            View::new(self).with_frame(0.0, 0.0, 780.0, 520.0)
        }
    }
}

impl Default for TabView { fn default() -> Self { Self::new() } }

// ── Icon helpers ─────────────────────────────────────────────────────

/// Monochrome sidebar glyph for an SF Symbol name (transparent background,
/// white/black tint — like Apple's sidebar tabs, not Settings-style color
/// tiles). The file is 3x supersampled with Lanczos3 (see
/// `SidebarIcon::to_path`): GTK's single-step downscale of the 1024px master
/// turns thin glyph strokes to mush. Cached in the temp dir per scheme.
#[cfg(feature = "coreicon")]
fn glyph_icon_path(symbol: &str, dark: bool, display_px: u32) -> Option<String> {
    let sf = coreicon::SFSymbol::from_name(symbol)?;

    let coreicon_assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .join("CoreIcon/assets/icons");
    if coreicon_assets.exists() {
        unsafe {
            coreicon::generator::ASSETS_DIR =
                Box::leak(coreicon_assets.to_str()?.to_string().into_boxed_str());
        }
    }

    let temp = std::env::temp_dir();
    let scheme = if dark { "d" } else { "l" };
    let master_path = temp.join(format!(
        "tb_{}_{}.png",
        symbol.replace('.', "_"),
        scheme,
    ));
    if !master_path.exists() {
        let tint = if dark {
            coreicon::Color::new(0.92, 0.92, 0.96, 1.0)
        } else {
            coreicon::Color::new(0.11, 0.11, 0.12, 1.0)
        };
        let canvas = coreicon::generator::IconCanvas::new()
            .background(coreicon::generator::Background::color(coreicon::Color::new(
                0.0, 0.0, 0.0, 0.0,
            )))
            .corner_radius(0.0)
            .layer(
                coreicon::generator::Layer::new(coreicon::generator::LayerContent::icon(sf))
                    .position(120.0, 120.0)
                    .size(784.0, 784.0)
                    .tint(tint),
            );
        canvas.save(&master_path).ok()?;
    }

    let px = display_px.clamp(8, 256);
    let bold_path = temp.join(format!(
        "tb_{}_{}_bold.png",
        symbol.replace('.', "_"),
        scheme,
    ));
    if !bold_path.exists() {
        let master = image::open(&master_path).ok()?;
        let bold = crate::sidebar::bolden_for_small_sizes(&master.to_rgba8());
        bold.save(&bold_path).ok()?;
    }
    let sized_path = temp.join(format!(
        "tb_{}_{}_{}_v2.png",
        symbol.replace('.', "_"),
        scheme,
        px,
    ));
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

// ── Rendering ────────────────────────────────────────────────────────

/// Legacy preview shell (no tabs): title, pill bar, hint.
fn render_preview(frame: Rect) -> gtk::Widget {
    let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
    let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
    let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
    let (bg, border) = if is_dark { ("#2c2c2e", "1px solid rgba(255,255,255,0.10)") } else { ("#ffffff", "1px solid rgba(0,0,0,0.08)") };

    // Directly on window — no extra card
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
    outer.set_halign(gtk::Align::Center);
    outer.set_valign(gtk::Align::Center);
    if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 80); }

    let title = gtk::Label::new(Some("TabView"));
    title.set_halign(gtk::Align::Center);
    title.add_css_class("tv-title");
    uikit::widget::apply_css(&title, &format!(".tv-title {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }}", fg));
    outer.append(&title);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_halign(gtk::Align::Center);
    let pill = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    pill.set_halign(gtk::Align::Center);
    pill.set_size_request(140, 26);
    pill.add_css_class("tv-pill");
    uikit::widget::apply_css(&pill, &format!(".tv-pill {{ background: {}; border: {}; border-radius: 999px; padding: 4px 10px; }}", bg, border));
    for icon in ["◉", "▭", "⬡"] {
        let lbl = gtk::Label::new(Some(icon));
        lbl.add_css_class("tv-icon");
        uikit::widget::apply_css(&lbl, &format!(".tv-icon {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", fg_dim));
        pill.append(&lbl);
    }
    let dot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    dot.set_size_request(14, 14);
    dot.add_css_class("tv-dot");
    uikit::widget::apply_css(&dot, ".tv-dot { background: #0A84FF; border-radius: 7px; min-width: 14px; min-height: 14px; }");
    let d = gtk::Label::new(Some("1"));
    d.set_halign(gtk::Align::Center); d.set_valign(gtk::Align::Center);
    uikit::widget::apply_css(&d, ".tv-dot-lbl { color: white; font-family: 'SF Pro Display'; font-size: 7px; }");
    d.add_css_class("tv-dot-lbl");
    dot.append(&d);
    pill.append(&dot);
    row.append(&pill);
    outer.append(&row);

    let hint = gtk::Label::new(Some("Creates Tabs with title, image, system"));
    hint.set_halign(gtk::Align::Center);
    hint.set_wrap(true);
    hint.set_max_width_chars(28);
    hint.add_css_class("tv-hint");
    uikit::widget::apply_css(&hint, &format!(".tv-hint {{ color: {}; font-family: 'SF Pro Display'; font-size: 7px; }}", fg_dim));
    outer.append(&hint);

    outer.upcast()
}

impl ViewContent for TabView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        if self.is_empty() {
            return render_preview(frame);
        }
        let dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;

        // Flatten tabs: top-level first, then section tabs in order.
        // `flat` holds (label, system_image, image_path, content).
        let mut flat: Vec<(&str, Option<&str>, Option<&str>, Option<Rc<dyn Widget>>)> = Vec::new();
        for t in &self.tabs {
            flat.push((t.label_text(), t.system_image.as_deref(), t.image_path.as_deref(), t.content.clone()));
        }
        // Display order with section headers: (header, flat_index).
        enum Entry { Header(String), Tab(usize) }
        let mut order: Vec<Entry> = Vec::new();
        for i in 0..self.tabs.len() {
            order.push(Entry::Tab(i));
        }
        for sec in &self.sections {
            if sec.tab_count() == 0 {
                continue;
            }
            if let Some(h) = sec.header_text() {
                order.push(Entry::Header(h.to_string()));
            }
            for t in sec.tabs() {
                let idx = flat.len();
                flat.push((t.label_text(), t.system_image.as_deref(), t.image_path.as_deref(), t.content.clone()));
                order.push(Entry::Tab(idx));
            }
        }

        let sidebar_bg = if dark { "#2C2C2E" } else { "#F5F5F7" };
        let sep_color = if dark { "rgba(255,255,255,0.10)" } else { "rgba(0,0,0,0.12)" };
        let label_hex = if dark { "rgba(235,235,245,0.9)" } else { "#1d1d1d" };
        let section_hex = if dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.6)" };
        let title_hex = if dark { "#FFFFFF" } else { "#1d1d1d" };
        let hover_hex = if dark { "rgba(255,255,255,0.07)" } else { "rgba(0,0,0,0.05)" };
        let sel_hex = "rgba(10,132,255,1.0)";

        // ── Title bar: traffic lights + toggle + bold title ──
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.set_height_request(48);
        bar.set_valign(gtk::Align::Center);
        bar.set_margin_start(16);
        bar.set_margin_end(16);

        let lights = uikit::widgets::TrafficLights::new().size(12.0);
        let lights_w = lights.to_gtk();
        lights_w.set_valign(gtk::Align::Center);
        bar.append(&lights_w);

        if self.show_toggle {
            let toggle = gtk::Button::new();
            toggle.set_valign(gtk::Align::Center);
            uikit::widget::apply_css(&toggle, ".tb-toggle { background: transparent; background-image: none; border: none; box-shadow: none; padding: 4px; }");
            toggle.add_css_class("tb-toggle");
            #[cfg(feature = "coreicon")]
            let toggle_icon = glyph_icon_path("sidebar.left", dark, 16);
            #[cfg(not(feature = "coreicon"))]
            let toggle_icon: Option<String> = None;
            match toggle_icon {
                Some(path) => {
                    let img = gtk::Image::from_file(&path);
                    img.set_pixel_size(16);
                    toggle.set_child(Some(&img));
                }
                None => {
                    toggle.set_label("◫");
                }
            }
            bar.append(&toggle);
        }

        let title_lbl = gtk::Label::new(Some(self.title.as_str()));
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk::Align::Center);
        title_lbl.set_valign(gtk::Align::Center);
        title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
        uikit::widget::apply_css(
            &title_lbl,
            &format!(".tb-title {{ color: {title_hex}; font-family: 'SF Pro Display'; font-size: 14px; font-weight: 700; }}"),
        );
        title_lbl.add_css_class("tb-title");
        bar.append(&title_lbl);

        // Balances lights + toggle so the title stays centered.
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_size_request(72, 1);
        bar.append(&spacer);

        // ── Sidebar ──
        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_width_request(self.sidebar_width as i32);
        sidebar.set_vexpand(true);
        sidebar.set_valign(gtk::Align::Fill);
        uikit::widget::apply_css(
            &sidebar,
            &format!(".tb-side {{ background-color: {sidebar_bg}; border-radius: 0px; border: none; box-shadow: none; }}"),
        );
        sidebar.add_css_class("tb-side");

        let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        list.set_margin_top(8);
        sidebar.append(&list);

        let selected = Rc::new(RefCell::new(self.selected.min(flat.len().saturating_sub(1))));
        let cb = self.on_select.clone();

        let mut rows: Vec<gtk::Box> = Vec::new();
        let mut row_labels: Vec<gtk::Label> = Vec::new();
        let mut row_order: Vec<gtk::Widget> = Vec::new();
        let mut flat_of_row: Vec<usize> = Vec::new();

        for entry in &order {
            match entry {
                Entry::Header(h) => {
                    let hl = gtk::Label::new(Some(h.as_str()));
                    hl.set_halign(gtk::Align::Start);
                    hl.set_margin_top(10);
                    hl.set_margin_bottom(2);
                    hl.set_margin_start(14);
                    hl.set_margin_end(14);
                    uikit::widget::apply_css(&hl, &format!(".tb-section {{ color: {section_hex}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; }}"));
                    hl.add_css_class("tb-section");
                    row_order.push(hl.upcast());
                }
                Entry::Tab(fi) => {
                    let tab_ref = &flat[*fi];
                    let (label_text, sys, img_path): (&str, Option<&str>, Option<&str>) =
                        (tab_ref.0, tab_ref.1, tab_ref.2);
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                    row.set_height_request(30);
                    row.set_margin_start(6);
                    row.set_margin_end(6);
                    row.set_valign(gtk::Align::Center);
                    row.set_focusable(true);
                    row.add_css_class("tb-row");
                    uikit::widget::apply_css(
                        &row,
                        &format!(
                            ".tb-row {{ background-color: transparent; background-image: none; border-radius: 7px; }} \
                             .tb-row.tb-hover {{ background-color: {hover_hex}; }} \
                             .tb-row.tb-sel {{ background-color: {sel_hex}; background-image: none; }}"
                        ),
                    );

                    let mut icon_done = false;
                    #[cfg(feature = "coreicon")]
                    if let Some(sym) = sys {
                        if let Some(path) = glyph_icon_path(sym, dark, 22) {
                            let img = gtk::Image::from_file(&path);
                            img.set_pixel_size(22);
                            row.append(&img);
                            icon_done = true;
                        }
                    }
                    #[cfg(not(feature = "coreicon"))]
                    let _ = sys;
                    if !icon_done {
                        if let Some(p) = img_path {
                            if std::path::Path::new(p).exists() {
                                let img = gtk::Image::from_file(p);
                                img.set_pixel_size(22);
                                row.append(&img);
                            } else {
                                let ph = gtk::Box::new(gtk::Orientation::Vertical, 0);
                                ph.set_size_request(22, 22);
                                row.append(&ph);
                            }
                        } else {
                            let ph = gtk::Box::new(gtk::Orientation::Vertical, 0);
                            ph.set_size_request(22, 22);
                            row.append(&ph);
                        }
                    }

                    let lbl = gtk::Label::new(Some(label_text));
                    lbl.set_halign(gtk::Align::Start);
                    lbl.set_hexpand(true);
                    lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    uikit::widget::apply_css(
                        &lbl,
                        &format!(
                            ".tb-lbl {{ color: {label_hex}; font-family: 'SF Pro Display'; font-size: 13px; }} \
                             .tb-lbl.tb-lbl-sel {{ color: white; }}"
                        ),
                    );
                    lbl.add_css_class("tb-lbl");
                    row.append(&lbl);

                    let row_hover = row.clone();
                    let motion = gtk::EventControllerMotion::new();
                    motion.connect_enter(move |_, _, _| {
                        row_hover.add_css_class("tb-hover");
                    });
                    let row_unhover = row.clone();
                    motion.connect_leave(move |_| {
                        row_unhover.remove_css_class("tb-hover");
                    });
                    row.add_controller(motion);

                    flat_of_row.push(*fi);
                    rows.push(row.clone());
                    row_labels.push(lbl);
                    row_order.push(row.upcast());
                }
            }
        }
        for w in row_order {
            list.append(&w);
        }

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_child(Some(&list));
        scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
        scroll.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
        scroll.add_css_class("tb-scroll");
        uikit::widget::apply_css(
            &scroll,
            ".tb-scroll { background-color: transparent; background-image: none; border: none; box-shadow: none; }",
        );
        uikit::widget::apply_css(
            &list,
            ".tb-list { background-color: transparent; background-image: none; border: none; box-shadow: none; }",
        );
        list.add_css_class("tb-list");
        sidebar.append(&scroll);

        // ── Detail area ──
        let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
        detail.set_hexpand(true);
        detail.set_vexpand(true);
        detail.set_halign(gtk::Align::Fill);
        detail.set_valign(gtk::Align::Fill);

        let show_detail = |detail: &gtk::Box, fi: usize| {
            if let Some(child) = detail.first_child() {
                detail.remove(&child);
            }
            if let Some(content) = flat.get(fi).and_then(|f| f.3.clone()) {
                let w = content.to_gtk();
                w.set_hexpand(true);
                w.set_vexpand(true);
                w.set_halign(gtk::Align::Center);
                w.set_valign(gtk::Align::Center);
                detail.append(&w);
            } else {
                let empty = gtk::Box::new(gtk::Orientation::Vertical, 0);
                empty.set_hexpand(true);
                empty.set_vexpand(true);
                detail.append(&empty);
            }
        };
        show_detail(&detail, *selected.borrow());

        // ── Selection wiring ──
        let rows_rc: Rc<Vec<gtk::Box>> = Rc::new(rows);
        let labels_rc: Rc<Vec<gtk::Label>> = Rc::new(row_labels);
        let flat_of_row_rc: Rc<Vec<usize>> = Rc::new(flat_of_row);
        let initial = *selected.borrow();
        for (k, row) in rows_rc.iter().enumerate() {
            if flat_of_row_rc[k] == initial {
                row.add_css_class("tb-sel");
                labels_rc[k].add_css_class("tb-lbl-sel");
            }
        }
        for (k, row) in rows_rc.iter().enumerate() {
            let rows_inner = rows_rc.clone();
            let labels_inner = labels_rc.clone();
            let flat_inner = flat_of_row_rc.clone();
            let selected_inner = selected.clone();
            let detail_inner = detail.clone();
            let flat_contents: Vec<Option<Rc<dyn Widget>>> =
                flat.iter().map(|f| f.3.clone()).collect();
            let cb_inner = cb.clone();
            let g = gtk::GestureClick::new();
            g.set_button(1);
            g.connect_released(move |_, _, _, _| {
                let fi = flat_inner[k];
                *selected_inner.borrow_mut() = fi;
                for (j, r) in rows_inner.iter().enumerate() {
                    if flat_inner[j] == fi {
                        r.add_css_class("tb-sel");
                        labels_inner[j].add_css_class("tb-lbl-sel");
                    } else {
                        r.remove_css_class("tb-sel");
                        labels_inner[j].remove_css_class("tb-lbl-sel");
                    }
                }
                if let Some(child) = detail_inner.first_child() {
                    detail_inner.remove(&child);
                }
                if let Some(content) = flat_contents.get(fi).and_then(|c| c.clone()) {
                    let w = content.to_gtk();
                    w.set_hexpand(true);
                    w.set_vexpand(true);
                    w.set_halign(gtk::Align::Center);
                    w.set_valign(gtk::Align::Center);
                    detail_inner.append(&w);
                }
                if let Some(ref h) = cb_inner {
                    h(fi);
                }
            });
            row.add_controller(g);
        }

        // ── Assemble: bar / separator / body ──
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_hexpand(true);
        outer.set_vexpand(true);
        outer.append(&bar);

        let hsep = gtk::Separator::new(gtk::Orientation::Horizontal);
        uikit::widget::apply_css(&hsep, &format!("separator {{ background: {sep_color}; min-height: 1px; }}"));
        outer.append(&hsep);

        let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        body.set_hexpand(true);
        body.set_vexpand(true);
        body.append(&sidebar);

        let vsep = gtk::Separator::new(gtk::Orientation::Vertical);
        uikit::widget::apply_css(&vsep, &format!("separator {{ background: {sep_color}; min-width: 1px; }}"));
        body.append(&vsep);

        body.append(&detail);
        outer.append(&body);

        // Sidebar toggle collapses the left column.
        if self.show_toggle {
            let mut toggle_btn: Option<gtk::Button> = None;
            let mut child = bar.first_child();
            while let Some(w) = child {
                let next = w.next_sibling();
                if let Ok(b) = w.clone().downcast::<gtk::Button>() {
                    toggle_btn = Some(b);
                    break;
                }
                child = next;
            }
            if let Some(btn) = toggle_btn {
                let side = sidebar.clone();
                let sep = vsep.clone();
                btn.connect_clicked(move |_| {
                    let vis = !side.is_visible();
                    side.set_visible(vis);
                    sep.set_visible(vis);
                });
            }
        }

        outer.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        if self.is_empty() {
            Size::new(180.0, 80.0)
        } else {
            Size::new(available.width.max(780.0), available.height.max(520.0))
        }
    }
}

impl Widget for TabView {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        if self.is_empty() {
            self.render(Rect::new(0.0,0.0,180.0,80.0))
        } else {
            self.render(Rect::new(0.0,0.0,780.0,520.0))
        }
    }
    fn is_interactive(&self) -> bool { true }
    fn expand_vertically(&self) -> bool { true }
    fn fill_width(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
    /// A functional tab view draws its own title bar (traffic lights,
    /// toggle, title), so it hides the system decoration bar like
    /// `Sidebar` does. The legacy preview (no tabs) hides nothing.
    fn hides_window_bar(&self) -> bool { !self.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uikit::widgets::Text;

    #[test]
    fn tab_view_exists() { let _ = TabView::new(); }

    #[test]
    fn tab_view_empty_by_default() {
        let v = TabView::new();
        assert!(v.is_empty());
        assert_eq!(v.tab_count(), 0);
    }

    #[test]
    fn tab_view_builder() {
        let v = TabView::new()
            .title("ExploreSwiftUISandbox")
            .tab(Tab::new(Text::new("0")))
            .section(
                TabSection::new()
                    .header("Foo")
                    .tab(Tab::new(Text::new("1")).title("1").system_image("1.circle")),
            )
            .tab(Tab::new(Text::new("2")).title("2").image("cats24x24"))
            .selected(0);
        assert!(!v.is_empty());
        assert_eq!(v.tab_count(), 3);
        assert_eq!(v.title, "ExploreSwiftUISandbox");
    }

    #[test]
    fn tab_view_empty_sections_ignored() {
        let v = TabView::new()
            .tab(Tab::new(Text::new("0")))
            .section(TabSection::new().header("Empty"));
        assert_eq!(v.tab_count(), 1);
    }

    #[test]
    fn tab_view_hides_window_bar_only_when_functional() {
        use uikit::widget::Widget;
        assert!(!TabView::new().hides_window_bar());
        let v = TabView::new().tab(Tab::new(Text::new("0")));
        assert!(v.hides_window_bar());
    }
}
