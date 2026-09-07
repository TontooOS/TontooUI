//! SearchTabRole — On Liquid Glass, this tab is placed in its own group. In left-to-right (LTR) layout...

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// SearchTabRole — initializer — On Liquid Glass, this tab is placed in its own group. In left-to-right (LTR) layout...
/// Directly on window (#1d1d1d dark / #ececec light), no extra card, SF Pro.
pub struct SearchTabRole {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl SearchTabRole {
    pub fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 80.0)
    }
}

impl Default for SearchTabRole { fn default() -> Self { Self::new() } }

impl ViewContent for SearchTabRole {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let (bg, border) = if is_dark { ("#2c2c2e", "1px solid rgba(255,255,255,0.10)") } else { ("#ffffff", "1px solid rgba(0,0,0,0.08)") };

        // Directly on window — no extra card
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 80); }

        let title = gtk::Label::new(Some("SearchTabRole"));
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

        let hint = gtk::Label::new(Some("On Liquid Glass, this tab is placed in"));
        hint.set_halign(gtk::Align::Center);
        hint.set_wrap(true);
        hint.set_max_width_chars(28);
        hint.add_css_class("tv-hint");
        uikit::widget::apply_css(&hint, &format!(".tv-hint {{ color: {}; font-family: 'SF Pro Display'; font-size: 7px; }}", fg_dim));
        outer.append(&hint);

        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 80.0) }
}

impl Widget for SearchTabRole {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,180.0,80.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_tab_role_exists() { let _ = SearchTabRole::new(); }
}
