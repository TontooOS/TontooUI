//! Color Variants — HierarchicalShapeStyle / shape style variants.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Hierarchical level — maps to `HierarchicalShapeStyle` numbered styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchicalVariant {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
}

impl HierarchicalVariant {
    pub fn all() -> [Self; 4] {
        [Self::Primary, Self::Secondary, Self::Tertiary, Self::Quaternary]
    }

    /// Opacity multiplier for a given level.
    pub fn opacity(self) -> f32 {
        match self {
            Self::Primary => 1.0,
            Self::Secondary => 0.6,
            Self::Tertiary => 0.3,
            Self::Quaternary => 0.15,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Tertiary => "Tertiary",
            Self::Quaternary => "Quaternary",
        }
    }
}

/// Modifier: "Color Variants — HierarchicalShapeStyle, a shape style that maps
/// to one of the numbered content styles."
///
/// Preview shows 3 base colors (blue/red/yellow) each rendered at 4 hierarchical
/// levels (4 columns) — 12 swatches total, as in the screenshot.
pub struct ColorVariants {
    id: WidgetId,
    bases: Vec<Color>,
    position_mode: PositionMode,
    position: Position,
}

impl ColorVariants {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            bases: vec![
                Color::from_rgb(10, 132, 255),
                Color::from_rgb(255, 59, 48),
                Color::from_rgb(255, 204, 0),
            ],
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn bases(mut self, colors: Vec<Color>) -> Self {
        self.bases = colors;
        self
    }

    /// Resolve a base color at a hierarchical level.
    pub fn color_for(&self, base: Color, level: HierarchicalVariant) -> Color {
        let op = level.opacity();
        Color::new(base.r, base.g, base.b, base.a * op)
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 110.0, 80.0)
    }
}

impl Default for ColorVariants {
    fn default() -> Self { Self::new() }
}

impl ViewContent for ColorVariants {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, 80);
        }

        // Directly on window — no extra background card
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        for &base in &self.bases {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            row.set_halign(gtk::Align::Center);
            for level in HierarchicalVariant::all() {
                let c = self.color_for(base, level);
                let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
                sw.set_size_request(20, 20);
                sw.add_css_class("cv-swatch");
                uikit::widget::apply_css(
                    &sw,
                    &format!(
                        ".cv-swatch {{ background: {}; border-radius: 3px; min-width: 20px; min-height: 20px; }}",
                        c.to_css()
                    ),
                );
                row.append(&sw);
            }
            container.append(&row);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(110.0, 80.0)
    }
}

impl Widget for ColorVariants {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 110.0, 80.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variant_opacity() {
        assert_eq!(HierarchicalVariant::Primary.opacity(), 1.0);
        assert!(HierarchicalVariant::Quaternary.opacity() < 0.2);
    }
    #[test]
    fn color_for_applies_alpha() {
        let cv = ColorVariants::new();
        let c = cv.color_for(Color::from_rgb(10, 132, 255), HierarchicalVariant::Secondary);
        assert!((c.a - 0.6).abs() < 0.01);
    }
}
