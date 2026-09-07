//! TontooUI Color demo — all 10 Color category elements
//! Shows every element from the screenshot grid: 4 + 4 + 2 layout.
//! Colors sit directly on the app background (#1d1d1d dark / #ececec light),
//! no card wrapper — pure TontooUI API. SF Pro font, adaptive.

use tontooui::prelude::*;
use tontooui::{
    ColorGradient, ColorOpacity, ColorVariants, SemanticColors, StandardColors,
    UIKitContentBackgroundColors, UIKitFillColors, UIKitLabelColors, UIKitSeparatorColors,
    UIKitTextColors,
};

// ── Cell helper — badge + preview + title + desc (mirrors gauges.rs) ──
fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(0.0).child(
                Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap()),
            ),
        )
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

// ── preview builders ──
fn preview_opacity() -> impl Widget {
    ColorOpacity::new(Color::from_rgb(10, 132, 255))
}

fn preview_gradient() -> impl Widget {
    ColorGradient::new(vec![
        Color::from_rgb(10, 132, 255),
        Color::from_rgb(0, 80, 160),
        Color::from_rgb(0, 40, 100),
    ])
}

fn preview_variants() -> impl Widget {
    ColorVariants::new()
}

fn preview_separator() -> impl Widget {
    UIKitSeparatorColors::new()
}

fn preview_background() -> impl Widget {
    UIKitContentBackgroundColors::new()
}

fn preview_text() -> impl Widget {
    UIKitTextColors::new()
}

fn preview_fill() -> impl Widget {
    UIKitFillColors::new()
}

fn preview_label() -> impl Widget {
    UIKitLabelColors::new()
}

fn preview_semantic() -> impl Widget {
    SemanticColors::new()
}

fn preview_standard() -> impl Widget {
    StandardColors::new()
}

fn main() {
    let mut app = App::new("TontooUI Colors", 1120, 900);

    let title = Text::new("Color").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Color Opacity",
            "The Color opacity modifier",
            "style",
            preview_opacity(),
        ))
        .child(cell(
            "Color Gradient",
            "The Color gradient modifier",
            "modifier",
            preview_gradient(),
        ))
        .child(cell(
            "Color Variants",
            "HierarchicalShapeStyle, a shape style that maps to one of the numbered...",
            "modifier",
            preview_variants(),
        ))
        .child(cell(
            "UIKit Separator Colors",
            "The UIKit separator colors that are also used in other components, such as the...",
            "type",
            preview_separator(),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "UIKit Content Background colors",
            "The UIKit colors that are also used in other components like in List and...",
            "type",
            preview_background(),
        ))
        .child(cell(
            "UIKit Text Colors",
            "The UIKit text colors that are also used in other components, such as the...",
            "type",
            preview_text(),
        ))
        .child(cell(
            "UIKit Fill Colors",
            "The UIKit fill colors that indicate the system's fill level",
            "type",
            preview_fill(),
        ))
        .child(cell(
            "UIKit Label Colors",
            "The UIKit label colors for displaying text",
            "type",
            preview_label(),
        ));

    let row3 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Semantic Colors",
            "The semantic colors that adapt to light and dark mode",
            "type",
            preview_semantic(),
        ))
        .child(cell(
            "Standard Colors",
            "The SwiftUI standard colors",
            "type",
            preview_standard(),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2).child(row3);
    let root = VStack::new().spacing(18.0).child(title).child(grid);

    app.set_root(root);
    app.run();
}
