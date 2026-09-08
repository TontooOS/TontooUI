//! TontooUI ConcentricRectangle demo — both elements directly on window.

use tontooui::prelude::*;
use tontooui::{ConcentricRectangle, UniformConcentricRectangle};

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

fn main() {
    let mut app = App::new("TontooUI ConcentricRectangle", 1180, 900);
    let title = Text::new("ConcentricRectangle").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Uniform Concentric Rectangle",
            "Create a rectangle with the same corner style set on four corners.",
            "initializer",
            UniformConcentricRectangle::new(),
        ))
        .child(cell(
            "Concentric Rectangle",
            "A concentric rectangle whose corner radii are defined from the same circle.",
            "initializer",
            ConcentricRectangle::new(),
        ));

    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
