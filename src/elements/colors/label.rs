//! UIKit Label Colors — label / secondaryLabel / tertiaryLabel / quaternaryLabel.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// UIKit label colors — mirrors `UIColor.label`, `secondaryLabel`,
/// `tertiaryLabel`, `quaternaryLabel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIKitLabelColor {
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,
}

impl UIKitLabelColor {
    pub fn all() -> [Self; 4] {
        [
            Self::Label,
            Self::SecondaryLabel,
            Self::TertiaryLabel,
            Self::QuaternaryLabel,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::SecondaryLabel => "secondaryLabel",
            Self::TertiaryLabel => "tertiaryLabel",
            Self::QuaternaryLabel => "quaternaryLabel",
        }
    }

    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::Label, false) => Color::from_rgb(0, 0, 0),
            (Self::Label, true) => Color::from_rgb(255, 255, 255),
            (Self::SecondaryLabel, false) => Color::from_rgba(60, 60, 67, 153), // 60%
            (Self::SecondaryLabel, true) => Color::from_rgba(235, 235, 245, 153),
            (Self::TertiaryLabel, false) => Color::from_rgba(60, 60, 67, 77), // 30%
            (Self::TertiaryLabel, true) => Color::from_rgba(235, 235, 245, 77),
            (Self::QuaternaryLabel, false) => Color::from_rgba(60, 60, 67, 46), // 18%
            (Self::QuaternaryLabel, true) => Color::from_rgba(235, 235, 245, 46),
        }
    }
}

/// Palette widget — "UIKit Label Colors — Label Secondary Label Tertiary ...".
pub struct UIKitLabelColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl UIKitLabelColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 48.0)
    }
}

impl Default for UIKitLabelColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for UIKitLabelColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 48); }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let row1 = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row1.set_halign(gtk::Align::Center);
        for variant in [UIKitLabelColor::Label, UIKitLabelColor::SecondaryLabel] {
            let lbl = gtk::Label::new(Some(variant.label()));
            // pretty print: "Label" / "Secondary Label"
            let text = match variant {
                UIKitLabelColor::Label => "Label",
                UIKitLabelColor::SecondaryLabel => "Secondary Label",
                _ => "",
            };
            lbl.set_text(text);
            lbl.add_css_class("label-lbl");
            uikit::widget::apply_css(
                &lbl,
                &format!(
                    ".label-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}",
                    variant.color(is_dark).to_css()
                ),
            );
            row1.append(&lbl);
        }
        container.append(&row1);

        let row2 = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row2.set_halign(gtk::Align::Center);
        for variant in [UIKitLabelColor::TertiaryLabel, UIKitLabelColor::QuaternaryLabel] {
            let text = match variant {
                UIKitLabelColor::TertiaryLabel => "Tertiary Label",
                UIKitLabelColor::QuaternaryLabel => "Quaternary Label",
                _ => "",
            };
            let lbl = gtk::Label::new(Some(text));
            lbl.add_css_class("label-lbl2");
            uikit::widget::apply_css(
                &lbl,
                &format!(
                    ".label-lbl2 {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}",
                    variant.color(is_dark).to_css()
                ),
            );
            row2.append(&lbl);
        }
        container.append(&row2);

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(180.0, 48.0) }
}

impl Widget for UIKitLabelColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 180.0, 48.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn label_alpha_descending() {
        let a = UIKitLabelColor::Label.color(true).a;
        let b = UIKitLabelColor::SecondaryLabel.color(true).a;
        let c = UIKitLabelColor::TertiaryLabel.color(true).a;
        let d = UIKitLabelColor::QuaternaryLabel.color(true).a;
        assert!(a > b && b > c && c > d);
    }
}
