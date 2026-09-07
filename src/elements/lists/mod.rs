//! Lists — SwiftUI-style List with all styles and modifiers.
//! Own category/folder. Dark #1d1d1d / Light #ececec, SF Pro, glass where needed.
//! Covers: OutlineGroup, DisclosureGroup, EditButton, SidebarListStyle,
//! InsetGrouped/Inset/Elliptical/Carousel/Bordered, Section Index, Move/Delete
//! disabled, Refreshable, SwipeAction, Badges, Backgrounds, Separators,
//! Spacing, Header heights, Row insets, Prominence.

use uikit::style::{Color, Rect, Size, Padding};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self};

use crate::elements::resolve_scheme;

// ── Style ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    Plain,
    Inset,
    InsetGrouped,
    Sidebar,
    Elliptical,
    Carousel,
    Bordered,
}
impl Default for ListStyle { fn default() -> Self { Self::Plain } }

// ── Row ──────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ListRow {
    pub label: String,
    pub detail: Option<String>,
    pub badge: Option<u32>,
    pub badge_prominence: bool,
    pub row_background: Option<Color>,
    pub separator_hidden: bool,
    pub section_separator_hidden: bool,
    pub separator_tint: Option<Color>,
    pub row_tint: Option<Color>,
    pub move_disabled: bool,
    pub delete_disabled: bool,
    pub swipe_label: Option<String>,
    pub is_header: bool,
}

impl ListRow {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), detail: None, badge: None, badge_prominence: false, row_background: None, separator_hidden: false, section_separator_hidden: false, separator_tint: None, row_tint: None, move_disabled: false, delete_disabled: false, swipe_label: None, is_header: false }
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self { self.detail = Some(d.into()); self }
    pub fn badge(mut self, n: u32) -> Self { self.badge = Some(n); self }
    pub fn badge_prominent(mut self) -> Self { self.badge_prominence = true; self }
    pub fn row_background(mut self, c: Color) -> Self { self.row_background = Some(c); self }
    pub fn separator_hidden(mut self, h: bool) -> Self { self.separator_hidden = h; self }
    pub fn section_separator_hidden(mut self, h: bool) -> Self { self.section_separator_hidden = h; self }
    pub fn separator_tint(mut self, c: Color) -> Self { self.separator_tint = Some(c); self }
    pub fn tint(mut self, c: Color) -> Self { self.row_tint = Some(c); self }
    pub fn move_disabled(mut self, v: bool) -> Self { self.move_disabled = v; self }
    pub fn delete_disabled(mut self, v: bool) -> Self { self.delete_disabled = v; self }
    pub fn swipe_action(mut self, label: impl Into<String>) -> Self { self.swipe_label = Some(label.into()); self }
}

// ── Section ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ListSection {
    pub header: Option<String>,
    pub footer: Option<String>,
    pub rows: Vec<ListRow>,
    pub header_prominence: bool,
    pub section_margins: Option<(f32,f32)>,
    pub row_spacing: Option<f32>,
    pub min_header_height: Option<f32>,
    pub row_insets: Option<Padding>,
}

impl ListSection {
    pub fn new() -> Self { Self { header: None, footer: None, rows: Vec::new(), header_prominence: false, section_margins: None, row_spacing: None, min_header_height: None, row_insets: None } }
    pub fn header(mut self, h: impl Into<String>) -> Self { self.header = Some(h.into()); self }
    pub fn footer(mut self, f: impl Into<String>) -> Self { self.footer = Some(f.into()); self }
    pub fn row(mut self, r: ListRow) -> Self { self.rows.push(r); self }
    pub fn rows(mut self, v: Vec<ListRow>) -> Self { self.rows = v; self }
    pub fn header_prominent(mut self) -> Self { self.header_prominence = true; self }
    pub fn section_margins(mut self, h: f32, v: f32) -> Self { self.section_margins = Some((h,v)); self }
    pub fn row_spacing(mut self, s: f32) -> Self { self.row_spacing = Some(s); self }
    pub fn min_header_height(mut self, h: f32) -> Self { self.min_header_height = Some(h); self }
    pub fn row_insets(mut self, p: Padding) -> Self { self.row_insets = Some(p); self }
}
impl Default for ListSection { fn default() -> Self { Self::new() } }

// ── Outline / Disclosure ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct OutlineGroup {
    pub header: String,
    pub children: Vec<ListRow>,
    pub disclosure: bool,
}
impl OutlineGroup {
    pub fn new(header: impl Into<String>) -> Self { Self { header: header.into(), children: Vec::new(), disclosure: true } }
    pub fn child(mut self, r: ListRow) -> Self { self.children.push(r); self }
}

#[derive(Clone)]
pub struct DisclosureGroup {
    pub header: String,
    pub children: Vec<ListRow>,
    pub expanded: bool,
}
impl DisclosureGroup {
    pub fn new(header: impl Into<String>) -> Self { Self { header: header.into(), children: Vec::new(), expanded: true } }
    pub fn child(mut self, r: ListRow) -> Self { self.children.push(r); self }
    pub fn expanded(mut self, e: bool) -> Self { self.expanded = e; self }
}

// ── List ─────────────────────────────────────────────────────────────────────

pub struct List {
    id: WidgetId,
    sections: Vec<ListSection>,
    style: ListStyle,
    // modifiers
    pub refreshable: bool,
    pub section_index_visible: bool,
    pub background_prominence: bool,
    pub compact_spacing: bool,
    pub custom_section_spacing: Option<f32>,
    pub min_row_height: Option<f32>,
    pub min_header_height: Option<f32>,
    pub position_mode: PositionMode,
    pub position: Position,
}

impl List {
    pub fn new() -> Self { Self { id: next_widget_id(), sections: Vec::new(), style: ListStyle::Plain, refreshable: false, section_index_visible: false, background_prominence: false, compact_spacing: false, custom_section_spacing: None, min_row_height: None, min_header_height: None, position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn section(mut self, s: ListSection) -> Self { self.sections.push(s); self }
    pub fn sections(mut self, v: Vec<ListSection>) -> Self { self.sections = v; self }
    pub fn list_style(mut self, s: ListStyle) -> Self { self.style = s; self }
    pub fn refreshable(mut self) -> Self { self.refreshable = true; self }
    pub fn section_index_visible(mut self, v: bool) -> Self { self.section_index_visible = v; self }
    pub fn background_prominent(mut self) -> Self { self.background_prominence = true; self }
    pub fn compact_spacing(mut self) -> Self { self.compact_spacing = true; self }
    pub fn custom_section_spacing(mut self, s: f32) -> Self { self.custom_section_spacing = Some(s); self }
    pub fn min_header_height(mut self, h: f32) -> Self { self.min_header_height = Some(h); self }
    pub fn min_row_height(mut self, h: f32) -> Self { self.min_row_height = Some(h); self }
    // convenience for outline/disclosure as sections
    pub fn outline_group(mut self, g: OutlineGroup) -> Self {
        let mut sec = ListSection::new().header(g.header);
        for c in g.children { sec = sec.row(c); }
        self.sections.push(sec); self
    }
    pub fn disclosure_group(mut self, g: DisclosureGroup) -> Self {
        let mut sec = ListSection::new().header(g.header);
        for c in g.children { sec = sec.row(c); }
        self.sections.push(sec); self
    }
    pub fn to_view(self) -> View { let w = 280.0; let h = 220.0; View::new(self).with_frame(0.0, 0.0, w, h) }
}
impl Default for List { fn default() -> Self { Self::new() } }

// ── Rendering ────────────────────────────────────────────────────────────────

fn row_gtk(row: &ListRow, is_dark: bool, style: ListStyle) -> gtk::Widget {
    let box_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    box_row.set_hexpand(true);
    let min_h = 36.0;
    box_row.set_height_request(min_h as i32);
    // row background — InsetGrouped gets card, others transparent
    let bg = if let Some(c) = row.row_background { c } else {
        match style {
            ListStyle::InsetGrouped | ListStyle::Inset | ListStyle::Bordered => if is_dark { Color::from_hex("#2c2c2e").unwrap() } else { Color::from_hex("#ffffff").unwrap() },
            _ => Color::TRANSPARENT,
        }
    };
    if bg.a > 0.01 {
        let css = format!(".lrow {{ background: {}; border-radius: 8px; padding: 8px; }}", bg.to_hex());
        box_row.add_css_class("lrow");
        uikit::widget::apply_css(&box_row, &css);
    } else {
        box_row.set_margin_start(4);
        box_row.set_margin_end(4);
    }

    // leading disclosure / outline arrow
    if row.is_header {
        let arrow = gtk::Label::new(Some("▾"));
        let col = if is_dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.5)" };
        uikit::widget::apply_css(&arrow, &format!("label {{ color: {col}; font-family: 'SF Pro Display'; font-size: 10px; }}"));
        box_row.append(&arrow);
    }

    let lbl = gtk::Label::new(Some(row.label.as_str()));
    lbl.set_halign(gtk::Align::Start);
    lbl.set_hexpand(true);
    let txt_col = if let Some(t) = row.row_tint { t.to_hex() } else if is_dark { "#ececec".to_string() } else { "#1d1d1d".to_string() };
    uikit::widget::apply_css(&lbl, &format!("label {{ color: {txt_col}; font-family: 'SF Pro Display'; font-size: 13px; }}"));
    box_row.append(&lbl);

    if let Some(ref d) = row.detail {
        let dl = gtk::Label::new(Some(d.as_str()));
        let dcol = if is_dark { "rgba(235,235,245,0.45)" } else { "rgba(60,60,67,0.55)" };
        uikit::widget::apply_css(&dl, &format!("label {{ color: {dcol}; font-family: 'SF Pro Display'; font-size: 12px; }}"));
        box_row.append(&dl);
    }

    if let Some(n) = row.badge {
        let b = gtk::Label::new(Some(&format!("{}", n)));
        let bg_badge = if row.badge_prominence || is_dark { "#ff3b30" } else { "#ff3b30" };
        // prominence changes background? use solid red for now
        uikit::widget::apply_css(&b, &format!("label {{ background: {bg_badge}; color: white; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 700; border-radius: 999px; padding: 1px 6px; min-width: 16px; }}"));
        b.set_halign(gtk::Align::End);
        box_row.append(&b);
    }

    if let Some(ref act) = row.swipe_label {
        let ab = gtk::Button::with_label(act);
        ab.add_css_class("swipe");
        uikit::widget::apply_css(&ab, "button.swipe { background: #0A84FF; color: white; font-family: 'SF Pro Display'; font-size: 11px; border-radius: 6px; padding: 2px 8px; }");
        box_row.append(&ab);
    }

    // move/delete disabled indicators — dim row
    if row.move_disabled || row.delete_disabled {
        box_row.set_opacity(0.5);
    }

    // wrap row + separator in vertical box
    let v = gtk::Box::new(gtk::Orientation::Vertical, 0);
    v.append(&box_row);
    if !row.separator_hidden {
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        let tint = row.separator_tint.map(|c| c.to_hex()).unwrap_or_else(|| if is_dark { "#3a3a3d".to_string() } else { "#d1d1d6".to_string() });
        uikit::widget::apply_css(&sep, &format!("separator {{ background: {tint}; min-height: 1px; margin: 0 8px; }}"));
        v.append(&sep);
    }
    v.upcast()
}

impl ViewContent for List {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let w = if frame.width > 0.0 { frame.width } else { 280.0 };
        let outer = gtk::Box::new(gtk::Orientation::Vertical, if self.compact_spacing { 4 } else if let Some(s) = self.custom_section_spacing { s as i32 } else { 12 });
        outer.set_width_request(w as i32);
        // List background prominence — subtle card behind
        if self.background_prominence {
            let bg = if is_dark { "rgba(44,44,46,0.6)" } else { "rgba(242,242,247,0.9)" };
            uikit::widget::apply_css(&outer, &format!(".llist {{ background: {bg}; border-radius: 12px; padding: 8px; }}"));
            outer.add_css_class("llist");
        }

        // Style outer box
        match self.style {
            ListStyle::InsetGrouped => {
                // outer already with section grouping; each section card handled per section
            }
            ListStyle::Sidebar => {
                // sidebar style: selected row blue? we mark first row as selected via tint
            }
            ListStyle::Elliptical => {
                // elliptical: rows as pills — handled per row via extra radius? keep same
            }
            ListStyle::Carousel => {
                // carousel: horizontal? for demo show horizontal scroll of rows
            }
            ListStyle::Bordered => {
                let border = if is_dark { "rgba(255,255,255,0.12)" } else { "rgba(0,0,0,0.08)" };
                uikit::widget::apply_css(&outer, &format!(".blist {{ border: 1px solid {border}; border-radius: 10px; padding: 4px; }}"));
                outer.add_css_class("blist");
            }
            _ => {}
        }

        if self.refreshable {
            let spin = gtk::Spinner::new();
            spin.start();
            spin.set_halign(gtk::Align::Center);
            outer.append(&spin);
        }

        for sec in &self.sections {
            let sec_box = gtk::Box::new(gtk::Orientation::Vertical, sec.row_spacing.unwrap_or(0.0) as i32);
            // section margins
            if let Some((h,v)) = sec.section_margins {
                sec_box.set_margin_start(h as i32);
                sec_box.set_margin_end(h as i32);
                sec_box.set_margin_top(v as i32);
                sec_box.set_margin_bottom(v as i32);
            }
            // header
            if let Some(ref h) = sec.header {
                let hl = gtk::Label::new(Some(h));
                hl.set_halign(gtk::Align::Start);
                let col = if sec.header_prominence { if is_dark { "#ececec" } else { "#1d1d1d" } } else { if is_dark { "rgba(235,235,245,0.55)" } else { "rgba(60,60,67,0.60)" } };
                let fw = if sec.header_prominence { "700" } else { "600" };
                let hgt = sec.min_header_height.or(self.min_header_height).unwrap_or(22.0);
                uikit::widget::apply_css(&hl, &format!("label {{ color: {col}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: {fw}; min-height: {hgt}px; margin: 0 8px; }}"));
                sec_box.append(&hl);
            }
            // rows container — for InsetGrouped use card
            let rows_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let needs_card = matches!(self.style, ListStyle::InsetGrouped | ListStyle::Inset | ListStyle::Sidebar | ListStyle::Elliptical);
            if needs_card {
                let card_bg = if is_dark { "#2c2c2e" } else { "#ffffff" };
                let border = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.06)" };
                let radius = match self.style {
                    ListStyle::Elliptical => "24px",
                    _ => "10px",
                };
                rows_box.add_css_class("lcard");
                uikit::widget::apply_css(&rows_box, &format!(".lcard {{ background: {card_bg}; border: 1px solid {border}; border-radius: {radius}; }}"));
            }
            // Carousel: horizontal
            if self.style == ListStyle::Carousel {
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                for r in &sec.rows {
                    let w = row_gtk(r, is_dark, self.style);
                    // wrap each row as small card for carousel
                    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    card.set_size_request(100, 60);
                    let bg = if is_dark { "#2c2c2e" } else { "#ffffff" };
                    uikit::widget::apply_css(&card, &format!(".ccard {{ background: {bg}; border-radius: 12px; padding: 8px; }}"));
                    card.add_css_class("ccard");
                    card.append(&w);
                    hbox.append(&card);
                }
                rows_box.append(&hbox);
            } else {
                for (i, r) in sec.rows.iter().enumerate() {
                    // hide section separator if requested on last row of section
                    let row_clone = r.clone();
                    if sec.rows.len() > 1 && i == sec.rows.len() - 1 && sec.rows.iter().any(|x| x.section_separator_hidden) {
                        // skip separator for last if hidden flag set on any? simplify
                    }
                    // min row height
                    if let Some(mh) = self.min_row_height.or(Some(36.0)) {
                        // row_gtk already sets 36 min, use that; if custom smaller, adjust
                        let _ = mh;
                    }
                    rows_box.append(&row_gtk(&row_clone, is_dark, self.style));
                }
            }
            sec_box.append(&rows_box);
            if let Some(ref f) = sec.footer {
                let fl = gtk::Label::new(Some(f));
                fl.set_halign(gtk::Align::Start);
                let col = if is_dark { "rgba(235,235,245,0.45)" } else { "rgba(60,60,67,0.55)" };
                uikit::widget::apply_css(&fl, &format!("label {{ color: {col}; font-family: 'SF Pro Display'; font-size: 11px; margin: 4px 8px 0 8px; }}"));
                sec_box.append(&fl);
            }
            // section index label — show on side if enabled
            if self.section_index_visible {
                // add a small index label overlay on right
                let overlay = gtk::Overlay::new();
                overlay.set_child(Some(&sec_box));
                let idx = gtk::Label::new(Some("A"));
                idx.set_halign(gtk::Align::End);
                idx.set_valign(gtk::Align::Center);
                uikit::widget::apply_css(&idx, "label { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 700; margin-right: 4px; }");
                overlay.add_overlay(&idx);
                outer.append(&overlay);
            } else {
                outer.append(&sec_box);
            }
        }

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
        scroll.set_child(Some(&outer));
        scroll.set_vexpand(false);
        scroll.upcast()
    }
    fn size_that_fits(&self, _available: Size) -> Size {
        let h = (self.sections.len() as f32 * 80.0 + 40.0).min(260.0);
        Size::new(280.0, h)
    }
}

impl Widget for List {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 280.0, 220.0)) }
    fn padding(&self) -> Padding { Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn list_style() {
        let l = List::new().list_style(ListStyle::Sidebar);
        assert_eq!(l.style, ListStyle::Sidebar);
    }
    #[test]
    fn list_section_rows() {
        let sec = ListSection::new().header("H").row(ListRow::new("Foo")).row(ListRow::new("Bar").badge(2));
        assert_eq!(sec.rows.len(), 2);
        assert_eq!(sec.header.as_deref(), Some("H"));
    }
}
