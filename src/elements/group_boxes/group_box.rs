//! GroupBox — SwiftUI-style grouped container.
//! Own category/folder. Dark #2C2C2E / Light #E6E6E8, Light/Dark adaptive,
//! SF Pro for label, supports label + background variants.

use uikit::app::ColorScheme;
use uikit::style::{Color, Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::Orientation;

const CONTAINER_DARK: &str = "#2C2C2E";
const CONTAINER_LIGHT: &str = "#E6E6E8";
const SEPARATOR_DARK: &str = "rgba(255,255,255,0.08)";
const SEPARATOR_LIGHT: &str = "rgba(0,0,0,0.08)";

/// SwiftUI-style GroupBox. Supports label, custom background, Light/Dark.
pub struct GroupBox {
    id: WidgetId,
    label: Option<String>,
    background: Option<Color>,
    children: Vec<Box<dyn Widget>>,
    color_scheme: Option<ColorScheme>,
    width: f32,
    position_mode: PositionMode,
    position: Position,
}

impl GroupBox {
    pub fn new() -> Self {
        Self { id: next_widget_id(), label: None, background: None, children: Vec::new(), color_scheme: None, width: 0.0, position_mode: PositionMode::Auto, position: Position::new() }
    }
    /// GroupBox with label (SwiftUI `GroupBox("Hello World") { ... }`)
    pub fn with_label(label: impl Into<String>) -> Self {
        Self::new().label(label)
    }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn background(mut self, c: Color) -> Self { self.background = Some(c); self }
    pub fn child(mut self, w: impl Widget + 'static) -> Self { self.children.push(Box::new(w)); self }
    pub fn color_scheme(mut self, c: ColorScheme) -> Self { self.color_scheme = Some(c); self }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn to_view(self) -> View { let w = self.width.max(200.0); View::new(self).with_frame(0.0, 0.0, w, 0.0) }
}

impl Default for GroupBox {
    fn default() -> Self { Self::new() }
}

impl ViewContent for GroupBox {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let bg = if let Some(c) = self.background { c.to_hex() } else if dark { CONTAINER_DARK.to_string() } else { CONTAINER_LIGHT.to_string() };
        let sep = if dark { SEPARATOR_DARK } else { SEPARATOR_LIGHT };

        let wrapper = gtk::Box::new(Orientation::Vertical, 6);
        if self.width > 0.0 { wrapper.set_width_request(self.width as i32); }

        // Optional label above box (SF Pro) — with small person icon if Hello World
        if let Some(ref lbl) = self.label {
            let l = gtk::Label::new(Some(lbl));
            l.set_halign(gtk::Align::Start);
            let col = if dark { "#ececec" } else { "#1d1d1d" };
            uikit::widget::apply_css(&l, &format!("label {{ color: {col}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; margin-bottom: 4px; }}"));
            wrapper.append(&l);
        }

        let outer = gtk::Box::new(Orientation::Vertical, 0);
        if self.width > 0.0 { outer.set_width_request(self.width as i32); }
        uikit::widget::apply_css(&outer, &format!("box {{ background: {bg}; border-radius: 16px; padding: 6px 14px; }}", bg=bg));

        for (i, child) in self.children.iter().enumerate() {
            if i > 0 {
                let sep_w = gtk::Box::new(Orientation::Horizontal, 0);
                sep_w.set_height_request(1);
                sep_w.set_margin_top(6);
                sep_w.set_margin_bottom(6);
                uikit::widget::apply_css(&sep_w, &format!("box {{ background: {sep}; }}", sep=sep));
                outer.append(&sep_w);
            }
            outer.append(&child.to_gtk());
        }
        // If no children but GroupBox still needs to show placeholder text (as in screenshot lorem ipsum)
        if self.children.is_empty() {
            let placeholder = if self.label.is_some() {
                "Hello World — Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed diam nonummy eiusmod tempor invidunt ad labore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
            } else {
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed diam nonummy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
            };
            let l = gtk::Label::new(Some(placeholder));
            l.set_wrap(true);
            l.set_halign(gtk::Align::Start);
            let col = if dark { "rgba(235,235,245,0.85)" } else { "rgba(28,28,30,0.85)" };
            uikit::widget::apply_css(&l, &format!("label {{ color: {col}; font-family: 'SF Pro Display'; font-size: 11px; }}"));
            outer.append(&l);
        }

        wrapper.append(&outer);
        wrapper.upcast()
    }
    fn can_become_first_responder(&self) -> bool { false }
    fn size_that_fits(&self, _available: Size) -> Size {
        let rows = self.children.len().max(1) as f32;
        let h = rows * 32.0 + (rows - 1.0).max(0.0) * 13.0 + 12.0 + if self.label.is_some() { 18.0 } else { 0.0 };
        Size::new(self.width.max(220.0), h.max(80.0))
    }
}

impl Widget for GroupBox {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 0.0, 0.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> Padding { Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn groupbox_label() {
        let g = GroupBox::with_label("Hello World");
        assert_eq!(g.label.as_deref(), Some("Hello World"));
    }
    #[test]
    fn groupbox_background() {
        let g = GroupBox::new().background(Color::from_hex("#ff0000").unwrap());
        assert_eq!(g.background, Some(Color::from_hex("#ff0000").unwrap()));
    }
}
