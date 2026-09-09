//! TabSection — A container that you can use to add hierarchy within a tab view.
//!
//! Functional usage inside a [`TabView`](super::tab_view::TabView) (Apple
//! `.sidebarAdaptable` style):
//!
//! ```rust,ignore
//! use tontooui::{Tab, TabSection};
//!
//! let section = TabSection::new()
//!     .header("Foo")
//!     .tab(Tab::new(Text::new("1")).title("1").system_image("1.circle"));
//! ```
//!
//! Without tabs the element keeps its legacy `180x80` preview rendering.

use super::tab::Tab;
use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// TabSection — initializer — A container that you can use to add hierarchy within a tab view.
/// Directly on window (#1d1d1d dark / #ececec light), no extra card, SF Pro.
pub struct TabSection {
    id: WidgetId,
    header: Option<String>,
    tabs: Vec<Tab>,
    position_mode: PositionMode,
    position: Position,
}

impl TabSection {
    pub fn new() -> Self {
        Self { id: next_widget_id(), header: None, tabs: Vec::new(), position_mode: PositionMode::Auto, position: Position::new() }
    }

    /// Section header shown above the tabs in the sidebar (SwiftUI's
    /// `TabSection("Foo")`).
    pub fn header(mut self, title: impl Into<String>) -> Self {
        self.header = Some(title.into());
        self
    }

    /// Append a tab to this section.
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Section header text, if set.
    pub fn header_text(&self) -> Option<&str> {
        self.header.as_deref()
    }

    /// Tabs in this section.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Number of tabs in this section.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn to_view(self) -> View {
        if self.tabs.is_empty() {
            View::new(self).with_frame(0.0, 0.0, 180.0, 80.0)
        } else {
            let h = 44.0 + self.tabs.len() as f32 * 30.0;
            View::new(self).with_frame(0.0, 0.0, 220.0, h)
        }
    }
}

impl Default for TabSection { fn default() -> Self { Self::new() } }

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

    let title = gtk::Label::new(Some("TabSection"));
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

    let hint = gtk::Label::new(Some("A container that you can use to add hi"));
    hint.set_halign(gtk::Align::Center);
    hint.set_wrap(true);
    hint.set_max_width_chars(28);
    hint.add_css_class("tv-hint");
    uikit::widget::apply_css(&hint, &format!(".tv-hint {{ color: {}; font-family: 'SF Pro Display'; font-size: 7px; }}", fg_dim));
    outer.append(&hint);

    outer.upcast()
}

impl ViewContent for TabSection {
    fn render(&self, frame: Rect) -> gtk::Widget {
        if self.tabs.is_empty() {
            return render_preview(frame);
        }
        // Standalone section with tabs: static (non-interactive) sidebar
        // fragment — header plus rows, first tab highlighted.
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "rgba(235,235,245,0.9)" } else { "#1d1d1d" };
        let dim = if is_dark { "rgba(235,235,245,0.5)" } else { "rgba(60,60,67,0.6)" };

        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, -1);
        }
        if let Some(ref h) = self.header {
            let hl = gtk::Label::new(Some(h.as_str()));
            hl.set_halign(gtk::Align::Start);
            hl.set_margin_top(8);
            hl.set_margin_bottom(2);
            hl.set_margin_start(14);
            hl.set_margin_end(14);
            uikit::widget::apply_css(&hl, &format!(".ts-header {{ color: {dim}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; }}"));
            hl.add_css_class("ts-header");
            outer.append(&hl);
        }
        for (i, tab) in self.tabs.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_height_request(28);
            row.set_margin_start(8);
            row.set_margin_end(8);
            row.set_valign(gtk::Align::Center);
            let bg = if i == 0 { "rgba(10,132,255,1.0)" } else { "transparent" };
            uikit::widget::apply_css(&row, &format!(".ts-row {{ background-color: {bg}; border-radius: 7px; }}"));
            row.add_css_class("ts-row");
            let lbl = gtk::Label::new(Some(tab.label_text()));
            lbl.set_halign(gtk::Align::Start);
            lbl.set_hexpand(true);
            let lc = if i == 0 { "white" } else { fg };
            uikit::widget::apply_css(&lbl, &format!(".ts-lbl {{ color: {lc}; font-family: 'SF Pro Display'; font-size: 13px; }}"));
            lbl.add_css_class("ts-lbl");
            row.append(&lbl);
            outer.append(&row);
        }
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size {
        if self.tabs.is_empty() {
            Size::new(180.0, 80.0)
        } else {
            Size::new(220.0, 44.0 + self.tabs.len() as f32 * 30.0)
        }
    }
}

impl Widget for TabSection {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        if self.tabs.is_empty() {
            self.render(Rect::new(0.0,0.0,180.0,80.0))
        } else {
            self.render(Rect::new(0.0,0.0,220.0,44.0 + self.tabs.len() as f32 * 30.0))
        }
    }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uikit::widgets::Text;

    #[test]
    fn tab_section_exists() { let _ = TabSection::new(); }

    #[test]
    fn tab_section_builder() {
        let s = TabSection::new()
            .header("Foo")
            .tab(Tab::new(Text::new("1")).title("1").system_image("1.circle"));
        assert_eq!(s.header_text(), Some("Foo"));
        assert_eq!(s.tab_count(), 1);
        assert_eq!(s.tabs()[0].label_text(), "1");
    }

    #[test]
    fn tab_section_empty_by_default() {
        let s = TabSection::new();
        assert_eq!(s.header_text(), None);
        assert_eq!(s.tab_count(), 0);
    }
}
