//! Divider — SwiftUI-style separator line.
//! Light/Dark adaptive: Dark #3a3a3d / Light #d1d1d6, thickness, orientation.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use crate::elements::resolve_scheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation { Horizontal, Vertical }

impl Default for DividerOrientation {
    fn default() -> Self { Self::Horizontal }
}

pub struct Divider {
    id: WidgetId,
    orientation: DividerOrientation,
    thickness: f32,
    color: Option<Color>,
    length: Option<f32>,
    position_mode: PositionMode,
    position: Position,
}

impl Divider {
    pub fn new() -> Self {
        Self { id: next_widget_id(), orientation: DividerOrientation::Horizontal, thickness: 1.0, color: None, length: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn horizontal() -> Self { Self::new() }
    pub fn vertical() -> Self { Self { orientation: DividerOrientation::Vertical, ..Self::new() } }
    pub fn thickness(mut self, t: f32) -> Self { self.thickness = t.max(0.5); self }
    pub fn color(mut self, c: Color) -> Self { self.color = Some(c); self }
    pub fn length(mut self, l: f32) -> Self { self.length = Some(l); self }
    pub fn frame(mut self, w: f32, h: f32) -> Self {
        // for horizontal, w is length; for vertical, h is length
        match self.orientation {
            DividerOrientation::Horizontal => self.length = Some(w),
            DividerOrientation::Vertical => self.length = Some(h),
        }
        self
    }
}

impl Default for Divider {
    fn default() -> Self { Self::new() }
}

fn resolved_color(explicit: Option<Color>, is_dark: bool) -> Color {
    if let Some(c) = explicit { return c; }
    if is_dark { Color::from_hex("#3a3a3d").unwrap() } else { Color::from_hex("#d1d1d6").unwrap() }
}

impl ViewContent for Divider {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let col = resolved_color(self.color, is_dark);
        let hex = col.to_hex();
        let sep = gtk::Separator::new(match self.orientation {
            DividerOrientation::Horizontal => gtk::Orientation::Horizontal,
            DividerOrientation::Vertical => gtk::Orientation::Vertical,
        });
        let css = match self.orientation {
            DividerOrientation::Horizontal => format!("separator {{ min-height: {}px; background-color: {}; }}", self.thickness, hex),
            DividerOrientation::Vertical => format!("separator {{ min-width: {}px; background-color: {}; }}", self.thickness, hex),
        };
        uikit::widget::apply_css(&sep, &css);
        // size via frame if provided
        if self.orientation == DividerOrientation::Horizontal {
            if let Some(l) = self.length { sep.set_width_request(l as i32); }
            else if frame.width > 0.0 { sep.set_width_request(frame.width as i32); }
        } else {
            if let Some(l) = self.length { sep.set_height_request(l as i32); }
            else if frame.height > 0.0 { sep.set_height_request(frame.height as i32); }
        }
        sep.upcast()
    }
    fn size_that_fits(&self, available: Size) -> Size {
        match self.orientation {
            DividerOrientation::Horizontal => Size::new(self.length.unwrap_or(available.width), self.thickness),
            DividerOrientation::Vertical => Size::new(self.thickness, self.length.unwrap_or(available.height)),
        }
    }
}

impl Widget for Divider {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let col = resolved_color(self.color, is_dark);
        let hex = col.to_hex();
        let sep = gtk::Separator::new(match self.orientation {
            DividerOrientation::Horizontal => gtk::Orientation::Horizontal,
            DividerOrientation::Vertical => gtk::Orientation::Vertical,
        });
        let css = match self.orientation {
            DividerOrientation::Horizontal => format!("separator {{ min-height: {}px; background-color: {}; }}", self.thickness, hex),
            DividerOrientation::Vertical => format!("separator {{ min-width: {}px; background-color: {}; }}", self.thickness, hex),
        };
        uikit::widget::apply_css(&sep, &css);
        if let Some(l) = self.length {
            match self.orientation {
                DividerOrientation::Horizontal => sep.set_width_request(l as i32),
                DividerOrientation::Vertical => sep.set_height_request(l as i32),
            }
        }
        sep.upcast()
    }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
    fn is_interactive(&self) -> bool { false }
}

impl Divider {
    pub fn to_view(self) -> View {
        let (w, h) = match self.orientation {
            DividerOrientation::Horizontal => (self.length.unwrap_or(200.0), self.thickness),
            DividerOrientation::Vertical => (self.thickness, self.length.unwrap_or(200.0)),
        };
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn divider_horizontal() {
        let d = Divider::horizontal();
        assert_eq!(d.orientation, DividerOrientation::Horizontal);
        assert_eq!(d.thickness, 1.0);
    }
    #[test]
    fn divider_vertical_thickness() {
        let d = Divider::vertical().thickness(2.0);
        assert_eq!(d.thickness, 2.0);
    }
}
