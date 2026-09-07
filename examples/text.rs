//! TontooUI Text demo — Text category with Text Format element
//! Directly on window background (#1d1d1d dark / #ececec light), no card wrapper.
//! SF Pro font, adaptive.

use tontooui::prelude::*;
use tontooui::TextFormat;

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
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(260.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(260.0))
}

fn preview_text_format() -> impl Widget {
    TextFormat::demo()
}

fn main() {
    let mut app = App::new("TontooUI Text", 420, 320);

    let title = Text::new("Text").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let grid = HStack::new().spacing(24.0).child(cell(
        "Text Format",
        "Creates a text view that displays the formatted representation of a nonstring type",
        "initializer",
        preview_text_format(),
    ));

    // Also show single-value examples directly on window below the palette
    let singles = VStack::new().spacing(8.0)
        .child(
            HStack::new().spacing(12.0)
                .child(TextFormat::currency(1234.56).font_size(10.0))
                .child(TextFormat::percent(0.874).font_size(10.0))
                .child(TextFormat::measurement(123.0, "meters").font_size(10.0)),
        )
        .child(
            HStack::new().spacing(12.0)
                .child(TextFormat::temperature_c(25.5).font_size(10.0))
                .child(TextFormat::date(2025, 7, 21).font_size(10.0))
                .child(TextFormat::time(9, 25, 39, true).font_size(10.0)),
        );

    let root = VStack::new().spacing(18.0).child(title).child(grid).child(singles);

    app.set_root(root);
    app.run();
}
