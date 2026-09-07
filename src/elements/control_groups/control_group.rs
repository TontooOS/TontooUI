//! ControlGroup — creates a new ControlGroup with the specified children.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Style for a ControlGroup — mirrors `SwiftUI.ControlGroupStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlGroupStyle {
    Automatic,
    Palette,
    Navigation,
    Menu,
    CompactMenu,
}

impl Default for ControlGroupStyle {
    fn default() -> Self { Self::Automatic }
}

impl ControlGroupStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Palette => "palette",
            Self::Navigation => "navigation",
            Self::Menu => "menu",
            Self::CompactMenu => "compactMenu",
        }
    }
}

/// ControlGroup — initializer.
///
/// Creates a new ControlGroup with the specified children. Renders directly
/// on the window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
pub struct ControlGroup {
    id: WidgetId,
    children: Vec<String>,
    style: ControlGroupStyle,
    position_mode: PositionMode,
    position: Position,
}

impl ControlGroup {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            children: vec!["Increase".into(), "Decrease".into()],
            style: ControlGroupStyle::Automatic,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn child(mut self, label: impl Into<String>) -> Self {
        self.children.push(label.into());
        self
    }

    pub fn children(mut self, labels: Vec<String>) -> Self {
        self.children = labels;
        self
    }

    pub fn style(mut self, style: ControlGroupStyle) -> Self {
        self.style = style;
        self
    }

    pub fn control_group_style(mut self, style: ControlGroupStyle) -> Self {
        self.style = style;
        self
    }

    pub fn to_view(self) -> View {
        // size depends on style
        let (w, h) = match self.style {
            ControlGroupStyle::Automatic => (220.0, 36.0),
            ControlGroupStyle::Palette => (180.0, 74.0),
            ControlGroupStyle::Navigation => (180.0, 64.0),
            ControlGroupStyle::Menu => (190.0, 54.0),
            ControlGroupStyle::CompactMenu => (160.0, 52.0),
        };
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl Default for ControlGroup {
    fn default() -> Self { Self::new() }
}

fn is_dark() -> bool {
    crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark
}

fn control_bg() -> (&'static str, &'static str) {
    // (bg, border)
    if is_dark() {
        ("#2c2c2e", "1px solid rgba(255,255,255,0.12)")
    } else {
        ("#ffffff", "1px solid rgba(0,0,0,0.10)")
    }
}

fn control_fg() -> &'static str {
    if is_dark() { "rgba(255,255,255,0.92)" } else { "rgba(29,29,29,0.92)" }
}

fn control_fg_dim() -> &'static str {
    if is_dark() { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.60)" }
}

impl ViewContent for ControlGroup {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        let w = if frame.width > 0.0 { frame.width as i32 } else { 220 };
        let h = if frame.height > 0.0 { frame.height as i32 } else { 36 };
        outer.set_size_request(w, h);

        match self.style {
            ControlGroupStyle::Automatic => {
                // Single pill with Increase | Decrease
                let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                pill.set_halign(gtk::Align::Center);
                pill.set_valign(gtk::Align::Center);
                pill.set_size_request(190, 28);
                let (bg, border) = control_bg();
                pill.add_css_class("cg-pill");
                uikit::widget::apply_css(&pill, &format!(".cg-pill {{ background: {}; border: {}; border-radius: 999px; padding: 2px; }}", bg, border));

                for (i, child) in self.children.iter().enumerate() {
                    let lbl = gtk::Label::new(Some(child));
                    lbl.set_halign(gtk::Align::Center);
                    lbl.set_hexpand(true);
                    lbl.add_css_class("cg-lbl");
                    uikit::widget::apply_css(&lbl, &format!(".cg-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; padding: 4px 12px; }}", control_fg()));
                    pill.append(&lbl);
                    if i + 1 < self.children.len() {
                        let sep = gtk::Separator::new(gtk::Orientation::Vertical);
                        let sep_color = if is_dark() { "rgba(255,255,255,0.12)" } else { "rgba(0,0,0,0.10)" };
                        uikit::widget::apply_css(&sep, &format!("separator {{ background: {}; min-width: 1px; margin: 4px 0; }}", sep_color));
                        pill.append(&sep);
                    }
                }
                outer.append(&pill);
            }
            ControlGroupStyle::Palette => {
                // Vertical palette: - Decrease / + Increase with search below
                let palette = gtk::Box::new(gtk::Orientation::Vertical, 6);
                palette.set_halign(gtk::Align::Center);
                let (bg, border) = control_bg();
                // Palette container directly on window — use frosted look but no extra outer card
                let box_v = gtk::Box::new(gtk::Orientation::Vertical, 4);
                box_v.set_halign(gtk::Align::Center);
                box_v.set_size_request(140, 44);
                box_v.add_css_class("cg-palette");
                uikit::widget::apply_css(&box_v, &format!(".cg-palette {{ background: {}; border: {}; border-radius: 12px; padding: 4px 8px; }}", bg, border));

                for (i, child) in self.children.iter().enumerate() {
                    // prefix +/- as in screenshot
                    let prefix = if child.to_lowercase().contains("increase") { "+" } else { "−" };
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                    row.set_halign(gtk::Align::Start);
                    let p = gtk::Label::new(Some(prefix));
                    p.add_css_class("cg-prefix");
                    uikit::widget::apply_css(&p, &format!(".cg-prefix {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", control_fg_dim()));
                    row.append(&p);
                    let lbl = gtk::Label::new(Some(child));
                    lbl.add_css_class("cg-palette-lbl");
                    uikit::widget::apply_css(&lbl, &format!(".cg-palette-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", control_fg()));
                    row.append(&lbl);
                    // divider between rows
                    if i + 1 < self.children.len() {
                        // no divider, just rows
                    }
                    box_v.append(&row);
                }
                palette.append(&box_v);

                // Search row "Foo" with magnifier
                let search = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                search.set_halign(gtk::Align::Center);
                let icon = gtk::Label::new(Some("⌕"));
                icon.add_css_class("cg-search-icon");
                uikit::widget::apply_css(&icon, &format!(".cg-search-icon {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", control_fg_dim()));
                search.append(&icon);
                let q = gtk::Label::new(Some("Foo"));
                q.add_css_class("cg-search-lbl");
                uikit::widget::apply_css(&q, &format!(".cg-search-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", control_fg_dim()));
                search.append(&q);
                palette.append(&search);

                outer.append(&palette);
            }
            ControlGroupStyle::Navigation => {
                // 3 rows of + Increase - Decrease
                let col = gtk::Box::new(gtk::Orientation::Vertical, 4);
                col.set_halign(gtk::Align::Center);
                for _ in 0..3 {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                    row.set_halign(gtk::Align::Center);
                    for child in &self.children {
                        let prefix = if child.to_lowercase().contains("increase") { "+" } else { "−" };
                        let lbl = gtk::Label::new(Some(&format!("{} {}", prefix, child)));
                        lbl.add_css_class("cg-nav-lbl");
                        uikit::widget::apply_css(&lbl, &format!(".cg-nav-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", control_fg_dim()));
                        row.append(&lbl);
                    }
                    col.append(&row);
                }
                outer.append(&col);
            }
            ControlGroupStyle::Menu => {
                // Horizontal pill with + Increase / - Decrease
                let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                pill.set_halign(gtk::Align::Center);
                pill.set_size_request(150, 28);
                let (bg, border) = control_bg();
                pill.add_css_class("cg-menu-pill");
                uikit::widget::apply_css(&pill, &format!(".cg-menu-pill {{ background: {}; border: {}; border-radius: 999px; padding: 2px 8px; }}", bg, border));
                for child in &self.children {
                    let lbl = gtk::Label::new(Some(child));
                    lbl.set_hexpand(true);
                    lbl.set_halign(gtk::Align::Center);
                    lbl.add_css_class("cg-menu-lbl");
                    uikit::widget::apply_css(&lbl, &format!(".cg-menu-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", control_fg()));
                    pill.append(&lbl);
                }
                outer.append(&pill);
                // search below
                let search = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                search.set_halign(gtk::Align::Center);
                let icon = gtk::Label::new(Some("⌕"));
                uikit::widget::apply_css(&icon, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", control_fg_dim()));
                search.append(&icon);
                let q = gtk::Label::new(Some("Foo"));
                uikit::widget::apply_css(&q, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", control_fg_dim()));
                search.append(&q);
                outer.append(&search);
            }
            ControlGroupStyle::CompactMenu => {
                // Compact pill with + / - icons only + search
                let pill = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                pill.set_halign(gtk::Align::Center);
                pill.set_size_request(100, 28);
                let (bg, border) = control_bg();
                pill.add_css_class("cg-compact");
                uikit::widget::apply_css(&pill, &format!(".cg-compact {{ background: {}; border: {}; border-radius: 999px; padding: 4px 16px; }}", bg, border));
                for prefix in ["+", "−"] {
                    let lbl = gtk::Label::new(Some(prefix));
                    lbl.add_css_class("cg-compact-lbl");
                    uikit::widget::apply_css(&lbl, &format!(".cg-compact-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; }}", control_fg()));
                    pill.append(&lbl);
                }
                outer.append(&pill);
                let search = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                search.set_halign(gtk::Align::Center);
                let icon = gtk::Label::new(Some("⌕"));
                uikit::widget::apply_css(&icon, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", control_fg_dim()));
                search.append(&icon);
                let q = gtk::Label::new(Some("Foo"));
                uikit::widget::apply_css(&q, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", control_fg_dim()));
                search.append(&q);
                outer.append(&search);
            }
        }

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        match self.style {
            ControlGroupStyle::Automatic => Size::new(220.0, 36.0),
            ControlGroupStyle::Palette => Size::new(180.0, 74.0),
            ControlGroupStyle::Navigation => Size::new(180.0, 64.0),
            ControlGroupStyle::Menu => Size::new(190.0, 54.0),
            ControlGroupStyle::CompactMenu => Size::new(160.0, 52.0),
        }
    }
}

impl Widget for ControlGroup {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,36.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn control_group_default_children() {
        let cg = ControlGroup::new();
        assert_eq!(cg.children.len(), 2);
        assert_eq!(cg.style, ControlGroupStyle::Automatic);
    }
    #[test]
    fn control_group_style_palette() {
        let cg = ControlGroup::new().style(ControlGroupStyle::Palette);
        assert_eq!(cg.style, ControlGroupStyle::Palette);
    }
}
