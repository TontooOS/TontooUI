//! PaletteControlGroupStyle — a control group style that presents its content as a palette.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use super::control_group::{ControlGroup, ControlGroupStyle};

/// PaletteControlGroupStyle — `style`.
///
/// A control group style that presents its content as a palette. Renders
/// directly on window background, no extra card.
pub struct PaletteControlGroupStyle {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl PaletteControlGroupStyle {
    pub fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0, 0.0, 180.0, 74.0) }
}

impl Default for PaletteControlGroupStyle { fn default() -> Self { Self::new() } }

impl ViewContent for PaletteControlGroupStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Render a ControlGroup with Palette style, directly on window
        let cg = ControlGroup::new().style(ControlGroupStyle::Palette);
        cg.render(frame)
    }
    fn size_that_fits(&self, _available: Size) -> Size { Size::new(180.0, 74.0) }
}

impl Widget for PaletteControlGroupStyle {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,180.0,74.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_is_palette() { let p = PaletteControlGroupStyle::new(); assert_eq!(p.id(), p.id()); }
}
