//! MenuControlGroupStyle — presents content as a menu when the user interacts.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use super::control_group::{ControlGroup, ControlGroupStyle};

/// MenuControlGroupStyle — `style`.
///
/// A control group style that presents its content as a menu when the user
/// interacts. Directly on window, no extra card.
pub struct MenuControlGroupStyle {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl MenuControlGroupStyle {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0, 0.0, 190.0, 54.0) }
}

impl Default for MenuControlGroupStyle { fn default() -> Self { Self::new() } }

impl ViewContent for MenuControlGroupStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let cg = ControlGroup::new().style(ControlGroupStyle::Menu);
        cg.render(frame)
    }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(190.0, 54.0) }
}

impl Widget for MenuControlGroupStyle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,190.0,54.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
