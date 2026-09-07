//! NavigationSubtitle — modifier that configures the view's subtitle for navigation.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// NavigationSubtitle — mirrors `SwiftUI.View/navigationSubtitle(_:)`.
///
/// Configures the view's subtitle for purposes of navigation, using a
/// localized string key. The screenshot shows a navigation bar with title
/// `Foo` and subtitle `Bar` on a black status-bar area (signal/wifi/battery).
/// Rendered directly on the window background (#1d1d1d dark / #ececec light),
/// no extra card, SF Pro.
pub struct NavigationSubtitle {
    id: WidgetId,
    title: String,
    subtitle: String,
    position_mode: PositionMode,
    position: Position,
}

impl NavigationSubtitle {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            title: title.into(),
            subtitle: subtitle.into(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn subtitle(subtitle: impl Into<String>) -> Self {
        Self::new("Foo", subtitle)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn subtitle_text(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 260.0, 64.0)
    }
}

impl Default for NavigationSubtitle {
    fn default() -> Self { Self::new("Foo", "Bar") }
}

impl ViewContent for NavigationSubtitle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let title_color = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let subtitle_color = if is_dark { "rgba(255,255,255,0.62)" } else { "rgba(60,60,67,0.6)" };
        let status_color = if is_dark { "rgba(255,255,255,0.85)" } else { "rgba(29,29,29,0.85)" };

        // Outer — directly on window, no card
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 64); }

        // Simulated status bar: signal/wifi/battery icons on top right (like screenshot)
        let status = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        status.set_halign(gtk::Align::End);
        status.set_hexpand(true);
        // spacer to push icons right
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        status.append(&spacer);
        for icon in ["▂", "◉", "▭"] {
            let lbl = gtk::Label::new(Some(icon));
            lbl.add_css_class("nav-status");
            uikit::widget::apply_css(&lbl, &format!(".nav-status {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", status_color));
            status.append(&lbl);
        }
        // Only show status row if we want to mimic screenshot; keep subtle
        // outer.append(&status);

        // Title "Foo"
        let title = gtk::Label::new(Some(&self.title));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("nav-title");
        uikit::widget::apply_css(&title, &format!(".nav-title {{ color: {}; font-family: 'SF Pro Display'; font-size: 15px; font-weight: 600; }}", title_color));
        outer.append(&title);

        // Subtitle "Bar" slightly smaller, dim
        let subtitle = gtk::Label::new(Some(&self.subtitle));
        subtitle.set_halign(gtk::Align::Start);
        subtitle.add_css_class("nav-subtitle");
        uikit::widget::apply_css(&subtitle, &format!(".nav-subtitle {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}", subtitle_color));
        outer.append(&subtitle);

        // Optional divider line below like navigation bar separator
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.set_margin_top(4);
        let sep_color = if is_dark { "rgba(255,255,255,0.10)" } else { "rgba(0,0,0,0.08)" };
        uikit::widget::apply_css(&sep, &format!("separator {{ background: {}; min-height: 1px; }}", sep_color));
        outer.append(&sep);

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(260.0, 64.0) }
}

impl Widget for NavigationSubtitle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,260.0,64.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nav_subtitle_default() {
        let n = NavigationSubtitle::default();
        assert_eq!(n.title, "Foo");
        assert_eq!(n.subtitle, "Bar");
    }
    #[test]
    fn nav_subtitle_custom() {
        let n = NavigationSubtitle::new("MyTitle", "MySubtitle");
        assert_eq!(n.subtitle, "MySubtitle");
    }
}
