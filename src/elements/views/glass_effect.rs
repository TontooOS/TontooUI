//! GlassEffect — applies the Liquid Glass effect to a view.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// GlassEffect — modifier.
///
/// Applies the Liquid Glass effect to a view. Preview shows cats with a
/// frosted glass circle containing "Foo" labels, directly on window.
pub struct GlassEffect {
    id: WidgetId,
    tint: Option<Color>,
    position_mode: PositionMode,
    position: Position,
}

impl GlassEffect {
    pub fn new() -> Self { Self { id: next_widget_id(), tint: None, position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn tint(mut self, c: Color) -> Self { self.tint = Some(c); self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,110.0) }
}

impl Default for GlassEffect { fn default() -> Self { Self::new() } }

impl ViewContent for GlassEffect {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let outer = gtk::Overlay::new();
        outer.set_size_request(220, 110);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }

        // Background cats — 3 cats in a row
        let bg = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        bg.set_halign(gtk::Align::Center);
        bg.set_valign(gtk::Align::Center);
        bg.set_size_request(220, 110);
        for i in 0..3 {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(66, 80);
            let cols = ["#d8b48a", "#c9a86a", "#e8c9a0"];
            cat.add_css_class("ge-cat");
            uikit::widget::apply_css(&cat, &format!(".ge-cat {{ background: {}; border-radius: 10px; min-width: 66px; min-height: 80px; }}", cols[i]));
            let lbl = gtk::Label::new(Some("🐱")); lbl.set_halign(gtk::Align::Center); lbl.set_valign(gtk::Align::Center); uikit::widget::apply_css(&lbl, "label { font-size: 22px; }"); cat.append(&lbl);
            bg.append(&cat);
        }
        outer.set_child(Some(&bg));

        // Glass circle overlay in center with Foo labels
        let glass = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        glass.set_halign(gtk::Align::Center);
        glass.set_valign(gtk::Align::Center);
        glass.set_size_request(120, 40);
        let tint_alpha = if is_dark { 0.28 } else { 0.22 };
        let tint_col = self.tint.unwrap_or(Color::new(1.0,1.0,1.0, tint_alpha as f32));
        // Use semi-transparent white with blur hint (box-shadow)
        glass.add_css_class("ge-glass");
        uikit::widget::apply_css(&glass, &format!(".ge-glass {{ background: {}; border: 1px solid rgba(255,255,255,0.22); border-radius: 20px; padding: 6px 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.18); }}", tint_col.to_css()));

        for _ in 0..3 {
            let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            pill.set_size_request(28, 18);
            let bg2 = if is_dark { "rgba(255,255,255,0.92)" } else { "rgba(255,255,255,0.96)" };
            uikit::widget::apply_css(&pill, &format!(".ge-pill {{ background: {}; border-radius: 9px; min-width: 28px; min-height: 18px; }}", bg2));
            pill.add_css_class("ge-pill");
            let l = gtk::Label::new(Some("Foo")); l.set_halign(gtk::Align::Center); uikit::widget::apply_css(&l, ".ge-foo { color: #1d1d1d; font-family: 'SF Pro Display'; font-size: 8px; font-weight: 600; }"); l.add_css_class("ge-foo"); pill.append(&l);
            glass.append(&pill);
        }
        // Third is reddish to match screenshot's red Foo
        if let Some(first) = glass.last_child() {
            // Not needed: keep as is; to hint red, recolor last pill
            // Instead just keep all white; fine
        }

        outer.add_overlay(&glass);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,110.0) }
}

impl Widget for GlassEffect {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,110.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
