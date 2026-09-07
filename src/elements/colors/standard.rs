//! Standard Colors — SwiftUI standard colors.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// SwiftUI standard colors — `Color.red`, `Color.blue`, `Color.green`, etc.
///
/// Mirrors `SwiftUI.Color` static members. Each variant resolves to the same
/// sRGB value in light and dark (except where noted) to match the SwiftUI
/// behavior documented as "The SwiftUI standard colors".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardColor {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
    Gray,
    White,
    Black,
    Clear,
}

impl StandardColor {
    pub fn all() -> [Self; 16] {
        [
            Self::Red,
            Self::Orange,
            Self::Yellow,
            Self::Green,
            Self::Mint,
            Self::Teal,
            Self::Cyan,
            Self::Blue,
            Self::Indigo,
            Self::Purple,
            Self::Pink,
            Self::Brown,
            Self::Gray,
            Self::White,
            Self::Black,
            Self::Clear,
        ]
    }

    /// 12-color grid as shown in screenshot (excludes white/black/clear).
    pub fn grid12() -> [Self; 12] {
        [
            Self::Blue,
            Self::Brown,
            Self::Cyan,
            Self::Gray,
            Self::Green,
            Self::Mint,
            Self::Orange,
            Self::Pink,
            Self::Purple,
            Self::Red,
            Self::Teal,
            Self::Yellow,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Mint => "mint",
            Self::Teal => "teal",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
            Self::Indigo => "indigo",
            Self::Purple => "purple",
            Self::Pink => "pink",
            Self::Brown => "brown",
            Self::Gray => "gray",
            Self::White => "white",
            Self::Black => "black",
            Self::Clear => "clear",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Red => Color::from_rgb(255, 59, 48),     // #FF3B30
            Self::Orange => Color::from_rgb(255, 149, 0),   // #FF9500
            Self::Yellow => Color::from_rgb(255, 204, 0),   // #FFCC00
            Self::Green => Color::from_rgb(52, 199, 89),    // #34C759
            Self::Mint => Color::from_rgb(0, 199, 190),     // #00C7BE
            Self::Teal => Color::from_rgb(48, 176, 199),    // #30B0C7
            Self::Cyan => Color::from_rgb(50, 173, 230),    // #32ADE6
            Self::Blue => Color::from_rgb(0, 122, 255),     // #007AFF
            Self::Indigo => Color::from_rgb(88, 86, 214),   // #5856D6
            Self::Purple => Color::from_rgb(175, 82, 222),  // #AF52DE
            Self::Pink => Color::from_rgb(255, 45, 85),     // #FF2D55
            Self::Brown => Color::from_rgb(162, 132, 94),   // #A2845E
            Self::Gray => Color::from_rgb(142, 142, 147),   // #8E8E93
            Self::White => Color::from_rgb(255, 255, 255),
            Self::Black => Color::from_rgb(0, 0, 0),
            Self::Clear => Color::TRANSPARENT,
        }
    }
}

/// Palette widget — "Standard Colors — The SwiftUI standard colors".
///
/// Shows a 4×3 (12) grid matching the screenshot first row preview for
/// Standard Colors, with additional handling for light/dark contrast.
///
/// The preview background is black (#000000) like the screenshot cards.
pub struct StandardColors {
    id: WidgetId,
    columns: usize,
    swatch_size: f32,
    position_mode: PositionMode,
    position: Position,
}

impl StandardColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            columns: 4,
            swatch_size: 20.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }

    pub fn swatch_size(mut self, s: f32) -> Self {
        self.swatch_size = s;
        self
    }

    pub fn to_view(self) -> View {
        // 4 cols * 20 + gaps ~ 96 width
        View::new(self).with_frame(0.0, 0.0, 110.0, 86.0)
    }
}

impl Default for StandardColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for StandardColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 86); }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let colors = StandardColor::grid12();
        let cols = self.columns;
        let rows = (colors.len() + cols - 1) / cols;
        let s = self.swatch_size as i32;

        for r in 0..rows {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            row.set_halign(gtk::Align::Center);
            for c in 0..cols {
                let idx = r * cols + c;
                if idx >= colors.len() { break; }
                let col = colors[idx].color();
                let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
                sw.set_size_request(s, s);
                sw.add_css_class("std-swatch");
                let border = if colors[idx] == StandardColor::White {
                    if is_dark { "1px solid rgba(255,255,255,0.15)" } else { "1px solid rgba(0,0,0,0.12)" }
                } else if colors[idx] == StandardColor::Clear {
                    if is_dark { "1px dashed rgba(255,255,255,0.25)" } else { "1px dashed rgba(0,0,0,0.20)" }
                } else {
                    "none"
                };
                uikit::widget::apply_css(
                    &sw,
                    &format!(
                        ".std-swatch {{ background: {}; border: {}; border-radius: 3px; min-width: {}px; min-height: {}px; }}",
                        col.to_css(), border, s, s
                    ),
                );
                row.append(&sw);
            }
            container.append(&row);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(110.0, 86.0) }
}

impl Widget for StandardColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 110.0, 86.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_color_count() {
        assert_eq!(StandardColor::all().len(), 16);
        assert_eq!(StandardColor::grid12().len(), 12);
    }
    #[test]
    fn standard_blue_is_blue() {
        assert_eq!(StandardColor::Blue.color(), Color::from_rgb(0, 122, 255));
        assert_eq!(StandardColor::Clear.color().a, 0.0);
    }
}
