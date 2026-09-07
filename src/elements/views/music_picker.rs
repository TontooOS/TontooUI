//! MusicPicker — presents a music picker to select items from the Apple Music catalog.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;
use std::sync::Arc;

/// MusicPicker — modifier (`presents a music picker...`).
///
/// Directly on window (#1d1d1d dark / #ececec light), no extra card, SF Pro.
/// Preview shows a navigation bar with `Bar` + search `Your Library` and list
/// `Library / Playlists`.
pub struct MusicPicker {
    id: WidgetId,
    presented: bool,
    on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl MusicPicker {
    pub fn new() -> Self {
        Self { id: next_widget_id(), presented: false, on_select: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn presented(mut self, shown: bool) -> Self { self.presented = shown; self }
    pub fn on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self { self.on_select = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,120.0) }
}

impl Default for MusicPicker { fn default() -> Self { Self::new() } }

impl ViewContent for MusicPicker {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let (bg, border) = if is_dark { ("#2c2c2e", "1px solid rgba(255,255,255,0.10)") } else { ("#ffffff", "1px solid rgba(0,0,0,0.08)") };

        let outer = gtk::Box::new(gtk::Orientation::Vertical, 8);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 120); }

        // Top bar: X | Bar | ✓
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.set_halign(gtk::Align::Center);
        bar.set_size_request(200, 24);
        bar.add_css_class("mp-bar");
        uikit::widget::apply_css(&bar, &format!(".mp-bar {{ background: {}; border: {}; border-radius: 8px; padding: 4px 8px; }}", bg, border));
        let x = gtk::Label::new(Some("✕")); uikit::widget::apply_css(&x, &format!(".mp-x {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", fg_dim)); bar.append(&x);
        let title = gtk::Label::new(Some("Bar")); title.set_hexpand(true); title.set_halign(gtk::Align::Center); uikit::widget::apply_css(&title, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }}", fg)); bar.append(&title);
        let check = gtk::Label::new(Some("✓")); check.add_css_class("mp-check"); uikit::widget::apply_css(&check, ".mp-check { color: #0A84FF; font-size: 11px; }"); bar.append(&check);
        outer.append(&bar);

        // Search field "Your Library"
        let search = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        search.set_halign(gtk::Align::Center);
        search.set_size_request(200, 26);
        search.add_css_class("mp-search");
        uikit::widget::apply_css(&search, &format!(".mp-search {{ background: {}; border: {}; border-radius: 999px; padding: 4px 10px; }}", bg, border));
        let icon = gtk::Label::new(Some("⌕")); uikit::widget::apply_css(&icon, &format!("label {{ color: {}; font-size: 10px; }}", fg_dim)); search.append(&icon);
        let txt = gtk::Label::new(Some("Your Library")); uikit::widget::apply_css(&txt, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", fg)); search.append(&txt);
        let mic = gtk::Label::new(Some("🎙")); uikit::widget::apply_css(&mic, &format!("label {{ color: {}; font-size: 10px; }}", fg_dim)); search.append(&mic);
        outer.append(&search);

        // Library section — directly on window, no card
        for txt in ["Library", "Playlists"] {
            let lbl = gtk::Label::new(Some(txt));
            lbl.set_halign(gtk::Align::Start);
            lbl.set_margin_start(10);
            uikit::widget::apply_css(&lbl, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; font-weight: 600; }}", fg_dim));
            outer.append(&lbl);
        }

        if let Some(cb) = &self.on_select {
            let h = cb.clone();
            let gesture = gtk::GestureClick::new();
            gesture.connect_pressed(move |_,_,_,_| h("selected".into()));
            outer.add_controller(gesture);
        }

        // Apply core modifier backing (so ViewModifiers store reflects state)
        let _view_mod = uikit::view::View::empty().musicPicker(self.presented);

        outer.upcast()
    }
    fn size_that_fits(&self, _a: Size) -> Size { Size::new(220.0,120.0) }
}

impl Widget for MusicPicker {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,120.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests { use super::*; #[test] fn music_picker_default(){ let m = MusicPicker::new(); assert!(!m.presented); } }
