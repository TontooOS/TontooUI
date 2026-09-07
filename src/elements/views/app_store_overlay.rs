//! AppStoreOverlay — presents a StoreKit overlay when a given condition is true.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// AppStoreOverlay — modifier.
pub struct AppStoreOverlay {
    id: WidgetId,
    app_id: String,
    presented: bool,
    position_mode: PositionMode,
    position: Position,
}

impl AppStoreOverlay {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self { id: next_widget_id(), app_id: app_id.into(), presented: true, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn presented(mut self, v: bool) -> Self { self.presented = v; self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,70.0) }
}

impl Default for AppStoreOverlay { fn default() -> Self { Self::new("com.example.app") } }

impl ViewContent for AppStoreOverlay {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let (bg, border) = if is_dark { ("#2c2c2e", "1px solid rgba(255,255,255,0.10)") } else { ("#ffffff", "1px solid rgba(0,0,0,0.08)") };

        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 70); }

        // App Store card preview — directly on window (no outer card)
        let card = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        card.set_halign(gtk::Align::Center);
        card.set_size_request(200, 44);
        card.add_css_class("aso-card");
        uikit::widget::apply_css(&card, &format!(".aso-card {{ background: {}; border: {}; border-radius: 12px; padding: 8px; }}", bg, border));

        let icon = gtk::Box::new(gtk::Orientation::Vertical, 0);
        icon.set_size_request(36, 36);
        uikit::widget::apply_css(&icon, ".aso-icon { background: #0A84FF; border-radius: 8px; min-width: 36px; min-height: 36px; }");
        icon.add_css_class("aso-icon");
        let a = gtk::Label::new(Some("A")); uikit::widget::apply_css(&a, ".aso-a { color: white; font-family: 'SF Pro Display'; font-size: 16px; font-weight: 700; }"); a.set_halign(gtk::Align::Center); a.set_valign(gtk::Align::Center);
        icon.append(&a);
        card.append(&icon);

        let txt = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let t1 = gtk::Label::new(Some("App Store")); t1.set_halign(gtk::Align::Start); uikit::widget::apply_css(&t1, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }}", fg)); txt.append(&t1);
        let t2 = gtk::Label::new(Some("Developer Preview")); t2.set_halign(gtk::Align::Start); uikit::widget::apply_css(&t2, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", fg_dim)); txt.append(&t2);
        card.append(&txt);
        outer.append(&card);

        let _view_mod = uikit::view::View::empty().appStoreOverlay(self.presented, self.app_id.clone());
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,70.0) }
}

impl Widget for AppStoreOverlay {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,70.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
