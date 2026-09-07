//! Color Opacity — style modifier for SwiftUI `Color.opacity(_:)`.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Style: the SwiftUI `Color.opacity` modifier.
///
/// Shows 5 swatches with decreasing opacity (1.0 → 0.12) to mirror the
/// screenshot card "Color Opacity — The Color opacity modifier".
pub struct ColorOpacity {
    id: WidgetId,
    base: Color,
    opacities: Vec<f32>,
    swatch_size: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ColorOpacity {
    pub fn new(base: Color) -> Self {
        Self {
            id: next_widget_id(),
            base,
            opacities: vec![1.0, 0.75, 0.5, 0.25, 0.12],
            swatch_size: 28.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn base(mut self, color: Color) -> Self {
        self.base = color;
        self
    }

    pub fn opacities(mut self, values: Vec<f32>) -> Self {
        self.opacities = values;
        self
    }

    pub fn swatch_size(mut self, size: f32) -> Self {
        self.swatch_size = size;
        self
    }

    /// Resolve `base` with `opacity` multiplied.
    pub fn color_with_opacity(&self, opacity: f32) -> Color {
        let a = (self.base.a * opacity).clamp(0.0, 1.0);
        Color::new(self.base.r, self.base.g, self.base.b, a)
    }

    pub fn to_view(self) -> View {
        let w = self.opacities.len() as f32 * (self.swatch_size + 6.0) + 16.0;
        View::new(self).with_frame(0.0, 0.0, w, 52.0)
    }
}

impl Default for ColorOpacity {
    fn default() -> Self {
        Self::new(Color::from_rgb(0, 122, 255))
    }
}

impl ViewContent for ColorOpacity {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Directly on window — no extra background card (TontooOS #1d1d1d dark / #ececec light)
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        outer.set_size_request(frame.width.max(140.0) as i32, 48);

        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);

        for &op in &self.opacities {
            let c = self.color_with_opacity(op);
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let s = self.swatch_size as i32;
            sw.set_size_request(s, s);
            sw.add_css_class("cop-swatch");
            let css = format!(
                ".cop-swatch {{ background: {}; border-radius: 4px; min-width: {}px; min-height: {}px; }}",
                c.to_css(),
                s,
                s
            );
            uikit::widget::apply_css(&sw, &css);
            row.append(&sw);
        }

        outer.append(&row);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let w = self.opacities.len() as f32 * (self.swatch_size + 6.0) + 16.0;
        Size::new(w, 48.0)
    }
}

impl Widget for ColorOpacity {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 200.0, 48.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opacity_computes_alpha() {
        let co = ColorOpacity::new(Color::from_rgb(0, 122, 255));
        let c = co.color_with_opacity(0.5);
        assert!((c.a - 0.5).abs() < 0.01);
        assert_eq!(co.opacities.len(), 5);
    }
    #[test]
    fn opacity_builder() {
        let co = ColorOpacity::new(Color::RED).opacities(vec![1.0, 0.5]).swatch_size(20.0);
        assert_eq!(co.swatch_size, 20.0);
        assert_eq!(co.opacities, vec![1.0, 0.5]);
    }
}
