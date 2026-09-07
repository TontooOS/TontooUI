//! SheetBackgroundInteraction — Controls whether people can interact with the view behind a presentation.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Interaction mode for the view behind a sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetBackgroundInteractionKind {
    #[default]
    Disabled,
    Enabled,
}

/// SheetBackgroundInteraction — modifier — Controls whether people can
/// interact with the view behind a presentation.
pub struct SheetBackgroundInteraction {
    id: WidgetId,
    interaction: SheetBackgroundInteractionKind,
    position_mode: PositionMode,
    position: Position,
}

impl SheetBackgroundInteraction {
    pub fn new() -> Self {
        Self { id: next_widget_id(), interaction: SheetBackgroundInteractionKind::Enabled, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn interaction(mut self, v: SheetBackgroundInteractionKind) -> Self { self.interaction = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SheetBackgroundInteraction { fn default() -> Self { Self::new() } }

impl ViewContent for SheetBackgroundInteraction {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shbi-phone");
        uikit::widget::apply_css(&phone, ".shbi-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let behind = gtk::Label::new(Some("You can still tap on me with the sheet open"));
        behind.set_halign(gtk::Align::Center);
        behind.set_valign(gtk::Align::Start);
        behind.set_wrap(true);
        behind.set_max_width_chars(24);
        behind.add_css_class("shbi-behind");
        uikit::widget::apply_css(&behind, ".shbi-behind { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 7px; margin-top: 10px; }");
        overlay.add_overlay(&behind);
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(160, 56);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::End);
        sheet.add_css_class("shbi-sheet");
        uikit::widget::apply_css(&sheet, ".shbi-sheet { background: #2c2c2e; border-radius: 12px 12px 12px 12px; }");
        let lbl = gtk::Label::new(Some("Bar"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.set_vexpand(true);
        lbl.add_css_class("shbi-bar");
        uikit::widget::apply_css(&lbl, ".shbi-bar { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; }");
        sheet.append(&lbl);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SheetBackgroundInteraction {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 180.0, 110.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interaction_exists() { let _ = SheetBackgroundInteraction::new(); }
}
