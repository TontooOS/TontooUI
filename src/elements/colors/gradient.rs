//! Color Gradient — modifier `Color.gradient`.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Modifier: the SwiftUI `Color.gradient` modifier.
///
/// Renders a 3×3-ish grid mirroring "Color Gradient — The Color gradient modifier".
/// Internally it is a linear gradient; the preview splits the gradient into
/// 9 discrete swatches for the card aesthetic.
pub struct ColorGradient {
    id: WidgetId,
    colors: Vec<Color>,
    swatch_size: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ColorGradient {
    pub fn new(colors: Vec<Color>) -> Self {
        Self {
            id: next_widget_id(),
            colors,
            swatch_size: 22.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn linear(colors: Vec<Color>) -> Self {
        Self::new(colors)
    }

    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    pub fn swatch_size(mut self, s: f32) -> Self {
        self.swatch_size = s;
        self
    }

    /// Build a CSS linear-gradient string from `colors`.
    pub fn css_gradient(&self) -> String {
        if self.colors.is_empty() {
            return "transparent".into();
        }
        if self.colors.len() == 1 {
            return self.colors[0].to_css();
        }
        let stops = self
            .colors
            .iter()
            .map(|c| c.to_css())
            .collect::<Vec<_>>()
            .join(", ");
        format!("linear-gradient(to right, {})", stops)
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 110.0, 72.0)
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self::new(vec![
            Color::from_rgb(10, 132, 255),
            Color::from_rgb(0, 60, 120),
            Color::from_rgb(0, 30, 80),
        ])
    }
}

impl ViewContent for ColorGradient {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Directly on window — no extra background card
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, 72);
        }

        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        // If we have a gradient with 2+ colors, show both a large gradient bar
        // plus a 3x3 discrete grid to match screenshot density.
        // Fallback to simple grid when no gradient.
        let cols = 4;
        let rows = 2;
        let n = cols * rows;
        // Interpolate colors to n steps
        let steps = if self.colors.is_empty() {
            vec![Color::from_rgb(10, 132, 255); n]
        } else {
            interpolate_colors(&self.colors, n)
        };

        for r in 0..rows {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            row.set_halign(gtk::Align::Center);
            for c in 0..cols {
                let idx = r * cols + c;
                let col = steps[idx];
                let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
                let s = self.swatch_size as i32;
                sw.set_size_request(s, s);
                sw.add_css_class("cg-swatch");
                uikit::widget::apply_css(
                    &sw,
                    &format!(
                        ".cg-swatch {{ background: {}; border-radius: 3px; min-width: {}px; min-height: {}px; }}",
                        col.to_css(), s, s
                    ),
                );
                row.append(&sw);
            }
            container.append(&row);
        }

        // Large gradient bar below grid
        if self.colors.len() >= 2 {
            let bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            bar.set_size_request(96, 10);
            bar.add_css_class("cg-bar");
            let grad = self.css_gradient();
            uikit::widget::apply_css(
                &bar,
                &format!(".cg-bar {{ background: {}; border-radius: 5px; min-height: 10px; }}", grad),
            );
            // wrap bar in centered box
            let bar_wrap = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            bar_wrap.set_halign(gtk::Align::Center);
            bar_wrap.append(&bar);
            container.append(&bar_wrap);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(110.0, 72.0)
    }
}

impl Widget for ColorGradient {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 110.0, 72.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

fn interpolate_colors(colors: &[Color], steps: usize) -> Vec<Color> {
    if steps == 0 { return vec![]; }
    if colors.len() == 1 { return vec![colors[0]; steps]; }
    let mut out = Vec::with_capacity(steps);
    for i in 0..steps {
        let t = if steps == 1 { 0.0 } else { i as f32 / (steps - 1) as f32 };
        let scaled = t * (colors.len() - 1) as f32;
        let idx = scaled.floor() as usize;
        let frac = scaled - idx as f32;
        if idx + 1 >= colors.len() {
            out.push(*colors.last().unwrap());
        } else {
            let a = colors[idx];
            let b = colors[idx + 1];
            out.push(Color::new(
                a.r + (b.r - a.r) * frac,
                a.g + (b.g - a.g) * frac,
                a.b + (b.b - a.b) * frac,
                a.a + (b.a - a.a) * frac,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gradient_css() {
        let g = ColorGradient::new(vec![Color::RED, Color::BLUE]);
        assert!(g.css_gradient().contains("linear-gradient"));
        assert_eq!(ColorGradient::new(vec![Color::RED]).css_gradient(), Color::RED.to_css());
    }
    #[test]
    fn interpolate() {
        let v = interpolate_colors(&[Color::RED, Color::BLUE], 3);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], Color::RED);
        assert_eq!(v[2], Color::BLUE);
    }
}
