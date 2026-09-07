//! CompactMenuControlGroupStyle — compact menu variant.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use super::control_group::{ControlGroup, ControlGroupStyle};

/// CompactMenuControlGroupStyle — `style`.
///
/// A control group style that presents its content as a compact menu when
/// the user interacts. Directly on window, SF Pro.
pub struct CompactMenuControlGroupStyle {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl CompactMenuControlGroupStyle {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0, 0.0, 160.0, 52.0) }
}

impl Default for CompactMenuControlGroupStyle { fn default() -> Self { Self::new() } }

impl ViewContent for CompactMenuControlGroupStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let cg = ControlGroup::new().style(ControlGroupStyle::CompactMenu);
        cg.render(frame)
    }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(160.0, 52.0) }
}

impl Widget for CompactMenuControlGroupStyle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,160.0,52.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
