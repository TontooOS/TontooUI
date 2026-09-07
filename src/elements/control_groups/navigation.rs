//! NavigationControlGroupStyle — the navigation control group style.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use super::control_group::{ControlGroup, ControlGroupStyle};

/// NavigationControlGroupStyle — `style`.
///
/// The navigation control group style. Directly on window, SF Pro.
pub struct NavigationControlGroupStyle {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl NavigationControlGroupStyle {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0, 0.0, 180.0, 64.0) }
}

impl Default for NavigationControlGroupStyle { fn default() -> Self { Self::new() } }

impl ViewContent for NavigationControlGroupStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let cg = ControlGroup::new().style(ControlGroupStyle::Navigation);
        cg.render(frame)
    }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(180.0, 64.0) }
}

impl Widget for NavigationControlGroupStyle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,180.0,64.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
