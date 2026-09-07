//! Menu — SwiftUI-style menu with Liquid Glass.
//! Dark #1d1d1d / Light #ececec, SF Pro, glass/transparent.
//! Opens on click (Menu) or secondary gesture (ContextMenu).
//! Supports: items, dividers, sections, nested submenus, custom preview.

use std::sync::Arc;
use uikit::style::{Rect, Size};
use uikit::view::ViewContent;
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self};

use crate::elements::resolve_scheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuRole { Default, Destructive }

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub role: MenuRole,
    pub on_activate: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), description: None, icon: None, role: MenuRole::Default, on_activate: None }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self { self.description = Some(d.into()); self }
    pub fn icon(mut self, name: impl Into<String>) -> Self { self.icon = Some(name.into()); self }
    pub fn destructive(mut self) -> Self { self.role = MenuRole::Destructive; self }
    pub fn on_activate(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_activate = Some(Arc::new(f)); self }
}

#[derive(Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Divider,
    Section { title: Option<String>, items: Vec<MenuEntry> },
    Submenu { title: String, items: Vec<MenuEntry> },
}

// ─────────────────────────────────────────────────────────────────────────────
// helpers — glass CSS + icon loading

fn glass_bg(is_dark: bool) -> (&'static str, &'static str) {
    if is_dark {
        ("rgba(38,38,40,0.84)", "rgba(255,255,255,0.12)")
    } else {
        ("rgba(248,248,250,0.92)", "rgba(0,0,0,0.08)")
    }
}

fn ensure_popover_transparent() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.load(Ordering::Relaxed) { return; }
    if let Some(display) = gtk::gdk::Display::default() {
        let provider = gtk::CssProvider::new();
        provider.load_from_string("popover, popover contents, popover > contents, popover arrow, popover background, popover > arrow, popover > background { background: transparent; background-color: rgba(0,0,0,0); border: none; box-shadow: none; outline: none; padding: 0; margin: 0; } window.popover, window.background.popover { background: transparent; background-color: transparent; }");
        gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_USER as u32);
        DONE.store(true, Ordering::Relaxed);
    }
}

#[cfg(feature = "coreicon")]
fn icon_image(name: &str, is_dark: bool) -> Option<gtk::Image> {
    let color = if is_dark { coreicon::Color::new(0.85,0.85,0.87,1.0) } else { coreicon::Color::new(0.18,0.18,0.19,1.0) };
    let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent()?.join("CoreIcon/assets/icons");
    if !assets.exists() { return None; }
    let src = assets.join(name).with_extension("png");
    if !src.exists() { return None; }
    let key = format!("menu_{}_{:02x}{:02x}{:02x}.png", name.replace('.', "_"), (color.r*255.0) as u8,(color.g*255.0) as u8,(color.b*255.0) as u8);
    let out = std::env::temp_dir().join(key);
    let path = if out.exists() { out } else {
        let img = image::open(&src).ok()?.to_rgba8();
        let (r,g,b) = ((color.r*255.0) as u8,(color.g*255.0) as u8,(color.b*255.0) as u8);
        let mut rgba = img;
        for p in rgba.pixels_mut() { if p[3]>0 { p[0]=r; p[1]=g; p[2]=b; } }
        let _ = rgba.save(&out);
        out
    };
    let img = gtk::Image::from_file(&path);
    img.set_pixel_size(14);
    Some(img)
}

fn build_entry_widget(entry: &MenuEntry, is_dark: bool, popover_weak: &glib::WeakRef<gtk::Popover>) -> gtk::Widget {
    match entry {
        MenuEntry::Divider => {
            let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
            let col = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.08)" };
            uikit::widget::apply_css(&sep, &format!("separator {{ background: {}; min-height: 1px; margin: 6px 0; }}", col));
            sep.upcast()
        }
        MenuEntry::Section { title, items } => {
            let v = gtk::Box::new(gtk::Orientation::Vertical, 0);
            if let Some(t) = title {
                let lbl = gtk::Label::new(Some(t));
                lbl.set_halign(gtk::Align::Start);
                let col = if is_dark { "rgba(235,235,245,0.55)" } else { "rgba(60,60,67,0.60)" };
                uikit::widget::apply_css(&lbl, &format!(".msec {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; margin: 6px 8px 4px 8px; }}", col));
                lbl.add_css_class("msec");
                v.append(&lbl);
            }
            for e in items {
                v.append(&build_entry_widget(e, is_dark, popover_weak));
            }
            v.upcast()
        }
        MenuEntry::Submenu { title, items } => {
            let btn = gtk::Button::new();
            btn.set_has_frame(false);
            btn.add_css_class("mitem");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_hexpand(true);
            let lbl = gtk::Label::new(Some(title.as_str()));
            lbl.set_halign(gtk::Align::Start);
            lbl.set_hexpand(true);
            let tcol = if is_dark { "#ececec" } else { "#1d1d1f" };
            uikit::widget::apply_css(&lbl, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 13px; }}", tcol));
            row.append(&lbl);
            let arrow = gtk::Label::new(Some("›"));
            arrow.set_halign(gtk::Align::End);
            let acol = if is_dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.5)" };
            uikit::widget::apply_css(&arrow, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 14px; }}", acol));
            row.append(&arrow);
            btn.set_child(Some(&row));
            let hcol = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.06)" };
            uikit::widget::apply_css(&btn, &format!(".mitem {{ padding: 6px 8px; border-radius: 6px; }} .mitem:hover {{ background: {}; }}", hcol));

            // submenu popover — eigenes Window, komplett transparent (kein weißer Rand)
            ensure_popover_transparent();
            let sub_pop = gtk::Popover::new();
            sub_pop.set_has_arrow(false);
            sub_pop.set_autohide(true);
            let (bg, border) = glass_bg(is_dark);
            let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
            // need to set parent before styling? style the box
            uikit::widget::apply_css(&content, &format!(".sub {{ background: {bg}; border: 1px solid {border}; border-radius: 12px; padding: 6px; }} .sub button {{ font-family: 'SF Pro Display'; }}"));
            content.add_css_class("sub");
            for se in items {
                // use weak to close both
                let w = popover_weak.clone();
                let widget = build_entry_widget(se, is_dark, &w);
                content.append(&widget);
            }
            sub_pop.set_child(Some(&content));
            // show submenu on click/hover
            let sub_pop_clone = sub_pop.clone();
            let pop_weak2 = popover_weak.clone();
            btn.connect_clicked(move |b| {
                sub_pop_clone.set_parent(b);
                sub_pop_clone.popup();
                let _ = &pop_weak2;
            });
            // also hover to popup after delay? keep simple click.

            // komplett transparent — nur innerer Box (sub) hat Liquid Glass, kein extra weißer Rand/Frame
            uikit::widget::apply_css(&sub_pop, "popover, popover contents, popover > contents, popover arrow { background: transparent; background-color: transparent; border: none; box-shadow: none; outline: none; padding: 0; margin: 0; }");
            btn.upcast()
        }
        MenuEntry::Item(item) => {
            let btn = gtk::Button::new();
            btn.set_has_frame(false);
            btn.add_css_class("mitem");
            let hcol = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.06)" };
            uikit::widget::apply_css(&btn, &format!(".mitem {{ padding: 6px 8px; border-radius: 6px; min-height: 22px; }} .mitem:hover {{ background: {}; }}", hcol));

            let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
            outer.set_hexpand(true);
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            row.set_hexpand(true);

            #[cfg(feature = "coreicon")]
            {
                if let Some(ref icon_name) = item.icon {
                    if let Some(img) = icon_image(icon_name, is_dark) {
                        row.append(&img);
                    }
                }
            }
            #[cfg(not(feature = "coreicon"))]
            { let _ = &item.icon; }

            let lbl = gtk::Label::new(Some(item.label.as_str()));
            lbl.set_halign(gtk::Align::Start);
            lbl.set_hexpand(true);
            let col = if item.role == MenuRole::Destructive { "#ff3b30" } else if is_dark { "#ececec" } else { "#1d1d1f" };
            uikit::widget::apply_css(&lbl, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 13px; }}", col));
            row.append(&lbl);
            // placeholder for checkmark/icon on right? keep empty
            outer.append(&row);
            if let Some(ref desc) = item.description {
                let d = gtk::Label::new(Some(desc.as_str()));
                d.set_halign(gtk::Align::Start);
                let dcol = if is_dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.55)" };
                uikit::widget::apply_css(&d, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}", dcol));
                outer.append(&d);
            }
            btn.set_child(Some(&outer));
            if let Some(cb) = item.on_activate.clone() {
                let pw = popover_weak.clone();
                btn.connect_clicked(move |_| {
                    cb();
                    if let Some(p) = pw.upgrade() { p.popdown(); }
                });
            } else {
                let pw = popover_weak.clone();
                btn.connect_clicked(move |_| {
                    if let Some(p) = pw.upgrade() { p.popdown(); }
                });
            }
            btn.upcast()
        }
    }
}

fn build_menu_box(entries: &[MenuEntry], is_dark: bool, popover_weak: &glib::WeakRef<gtk::Popover>) -> gtk::Widget {
    let v = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (bg, border) = glass_bg(is_dark);
    v.add_css_class("tmenu");
    uikit::widget::apply_css(&v, &format!(".tmenu {{ background: {bg}; border: 1px solid {border}; border-radius: 12px; padding: 6px; min-width: 180px; }}"));
    for e in entries {
        v.append(&build_entry_widget(e, is_dark, popover_weak));
    }
    // popover chrome
    // parent popover styling handled by caller
    v.upcast()
}

// ─────────────────────────────────────────────────────────────────────────────

pub struct Menu {
    id: WidgetId,
    label: String,
    entries: Vec<MenuEntry>,
    position_mode: PositionMode,
    position: Position,
}

impl Menu {
    pub fn new(label: impl Into<String>) -> Self {
        Self { id: next_widget_id(), label: label.into(), entries: Vec::new(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.entries.push(MenuEntry::Item(MenuItem::new(label))); self
    }
    pub fn item_with_desc(mut self, label: impl Into<String>, desc: impl Into<String>) -> Self {
        self.entries.push(MenuEntry::Item(MenuItem::new(label).description(desc))); self
    }
    pub fn item_with_icon(mut self, label: impl Into<String>, icon: impl Into<String>) -> Self {
        self.entries.push(MenuEntry::Item(MenuItem::new(label).icon(icon))); self
    }
    pub fn item_full(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item)); self
    }
    pub fn divider(mut self) -> Self { self.entries.push(MenuEntry::Divider); self }
    pub fn section(mut self, title: impl Into<String>, f: impl FnOnce(Vec<MenuEntry>) -> Vec<MenuEntry>) -> Self {
        let inner = f(Vec::new());
        self.entries.push(MenuEntry::Section { title: Some(title.into()), items: inner }); self
    }
    pub fn section_items(mut self, title: Option<String>, items: Vec<MenuEntry>) -> Self {
        self.entries.push(MenuEntry::Section { title, items }); self
    }
    pub fn submenu(mut self, title: impl Into<String>, items: Vec<MenuEntry>) -> Self {
        self.entries.push(MenuEntry::Submenu { title: title.into(), items }); self
    }
    pub fn submenu_with(mut self, title: impl Into<String>, f: impl FnOnce(Vec<MenuEntry>) -> Vec<MenuEntry>) -> Self {
        let inner = f(Vec::new());
        self.entries.push(MenuEntry::Submenu { title: title.into(), items: inner }); self
    }
    /// Low-level: push raw entry
    pub fn entry(mut self, e: MenuEntry) -> Self { self.entries.push(e); self }
}

// Helper to build Vec<MenuEntry> fluently for sections/submenus
pub trait MenuVecExt {
    fn push_item(self, label: impl Into<String>) -> Self;
    fn push_item_desc(self, label: impl Into<String>, desc: impl Into<String>) -> Self;
    fn push_item_icon(self, label: impl Into<String>, icon: impl Into<String>) -> Self;
}
impl MenuVecExt for Vec<MenuEntry> {
    fn push_item(mut self, label: impl Into<String>) -> Self { self.push(MenuEntry::Item(MenuItem::new(label))); self }
    fn push_item_desc(mut self, label: impl Into<String>, desc: impl Into<String>) -> Self { self.push(MenuEntry::Item(MenuItem::new(label).description(desc))); self }
    fn push_item_icon(mut self, label: impl Into<String>, icon: impl Into<String>) -> Self { self.push(MenuEntry::Item(MenuItem::new(label).icon(icon))); self }
}

impl Widget for Menu {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        ensure_popover_transparent();
        let btn = gtk::Button::with_label(&self.label);
        btn.set_has_frame(true);
        // trigger button style — glass pill, SF Pro
        let bbg = if is_dark { "rgba(58,58,60,0.9)" } else { "rgba(255,255,255,0.9)" };
        let bfg = if is_dark { "#ececec" } else { "#1d1d1f" };
        let border = if is_dark { "rgba(255,255,255,0.12)" } else { "rgba(0,0,0,0.08)" };
        uikit::widget::apply_css(&btn, &format!("button {{ background: {bbg}; color: {bfg}; border: 1px solid {border}; border-radius: 8px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 500; padding: 6px 14px; }} button:hover {{ background: {}; }}", if is_dark { "rgba(72,72,74,0.95)" } else { "rgba(242,242,245,0.98)" }));
        let pop = gtk::Popover::new();
        pop.set_has_arrow(false);
        pop.set_autohide(true);
        // need weak for entry callbacks to close popover
        let pop_weak = pop.downgrade();
        let content = build_menu_box(&self.entries, is_dark, &pop_weak);
        pop.set_child(Some(&content));
        pop.set_parent(&btn);
        // komplett transparent — nur tmenu hat Liquid Glass, kein extra weißer Rand/Frame hinter dem Menü
        uikit::widget::apply_css(&pop, "popover, popover contents, popover > contents, popover arrow, popover background { background: transparent; background-color: transparent; border: none; box-shadow: none; outline: none; padding: 0; margin: 0; }");
        let pop_clone = pop.clone();
        btn.connect_clicked(move |_| { pop_clone.popup(); });
        btn.upcast()
    }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

impl ViewContent for Menu {
    fn render(&self, _frame: Rect) -> gtk::Widget { self.to_gtk() }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(120.0, 32.0) }
}

// ─────────────────────────────────────────────────────────────────────────────
// ContextMenu — wraps a child, shows menu on secondary gesture (right-click)

pub struct ContextMenu {
    id: WidgetId,
    child: Box<dyn Widget>,
    entries: Vec<MenuEntry>,
    preview: Option<Box<dyn Widget>>,
    position_mode: PositionMode,
    position: Position,
}

impl ContextMenu {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self { id: next_widget_id(), child: Box::new(child), entries: Vec::new(), preview: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn item(mut self, label: impl Into<String>) -> Self { self.entries.push(MenuEntry::Item(MenuItem::new(label))); self }
    pub fn item_with_desc(mut self, label: impl Into<String>, desc: impl Into<String>) -> Self { self.entries.push(MenuEntry::Item(MenuItem::new(label).description(desc))); self }
    pub fn entry(mut self, e: MenuEntry) -> Self { self.entries.push(e); self }
    pub fn divider(mut self) -> Self { self.entries.push(MenuEntry::Divider); self }
    pub fn preview(mut self, w: impl Widget + 'static) -> Self { self.preview = Some(Box::new(w)); self }
    pub fn entries(mut self, v: Vec<MenuEntry>) -> Self { self.entries = v; self }
}

impl Widget for ContextMenu {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        ensure_popover_transparent();
        let child_gtk = self.child.to_gtk();
        // container that holds child + gesture
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&child_gtk);

        let pop = gtk::Popover::new();
        pop.set_has_arrow(false);
        pop.set_autohide(true);
        pop.set_parent(&container);
        let pop_weak = pop.downgrade();
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let (bg, border) = glass_bg(is_dark);
        content.add_css_class("ctx");
        uikit::widget::apply_css(&content, &format!(".ctx {{ background: {bg}; border: 1px solid {border}; border-radius: 12px; padding: 6px; min-width: 180px; }}"));
        if let Some(preview) = &self.preview {
            let pw = preview.to_gtk();
            // preview pill — Custom Preview
            let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            pill.set_halign(gtk::Align::Center);
            let lbl = gtk::Label::new(Some("Custom Preview"));
            lbl.set_halign(gtk::Align::Center);
            let pill_bg = if is_dark { "rgba(58,58,60,0.9)" } else { "rgba(255,255,255,0.95)" };
            let pill_fg = if is_dark { "#ececec" } else { "#1d1d1f" };
            uikit::widget::apply_css(&lbl, &format!("label {{ background: {pill_bg}; color: {pill_fg}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; border-radius: 999px; padding: 4px 10px; border: 1px solid {border}; }}", pill_bg=pill_bg, pill_fg=pill_fg, border=border));
            // If preview widget provided, use it instead of pill? screenshot shows pill + menu; we show both
            // Try to use provided preview widget wrapped
            let wrap = gtk::Box::new(gtk::Orientation::Vertical, 6);
            wrap.append(&pw);
            wrap.append(&lbl);
            content.append(&wrap);
            let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
            let col = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.08)" };
            uikit::widget::apply_css(&sep, &format!("separator {{ background: {col}; min-height: 1px; margin: 6px 0; }}"));
            content.append(&sep);
        }
        for e in &self.entries {
            content.append(&build_entry_widget(e, is_dark, &pop_weak));
        }
        pop.set_child(Some(&content));
        uikit::widget::apply_css(&pop, "popover, popover contents, popover > contents, popover arrow, popover background { background: transparent; background-color: transparent; border: none; box-shadow: none; outline: none; padding: 0; margin: 0; }");

        let gesture = gtk::GestureClick::new();
        gesture.set_button(3); // right-click secondary
        let pop_clone = pop.clone();
        gesture.connect_pressed(move |_, _, x, y| {
            let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
            pop_clone.set_pointing_to(Some(&rect));
            pop_clone.popup();
        });
        container.add_controller(gesture);

        // Also allow long-press secondary via button 1 with 500ms? Simplify: also add button 1 secondary via 2nd gesture for touch
        // For demo convenience also open on left click if right not available (e.g. WSL no right?)
        let gesture2 = gtk::GestureClick::new();
        gesture2.set_button(1);
        // require 2 clicks? Instead add a separate click on child to also popup for demo ease when user left-clicks
        // We'll not, to avoid conflict with Menu.

        container.upcast()
    }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

impl ViewContent for ContextMenu {
    fn render(&self, _frame: Rect) -> gtk::Widget { self.to_gtk() }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(120.0, 32.0) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use uikit::widgets::Text;
    #[test]
    fn menu_builder() {
        let m = Menu::new("Test").item("A").divider().item_with_desc("B", "desc");
        assert_eq!(m.entries.len(), 3);
    }
    #[test]
    fn context_builder() {
        let c = ContextMenu::new(Text::new("Hi")).item("A").divider();
        assert_eq!(c.entries.len(), 2);
    }
}
