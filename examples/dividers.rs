//! Divider demo — eigene Kategorie, direkt auf Background, nur TontooUI API

use tontooui::prelude::*;
use tontooui::Divider;

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(HStack::new().spacing(0.0).child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap())))
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(180.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(180.0))
}

fn preview_horizontal() -> impl Widget {
    VStack::new().spacing(12.0)
        .child(Text::new("Foo").font_size(11.0))
        .child(Divider::horizontal().length(180.0))
        .child(Text::new("Bar").font_size(11.0))
}

fn preview_vertical() -> impl Widget {
    HStack::new().spacing(12.0)
        .child(Text::new("Left").font_size(11.0))
        .child(Divider::vertical().length(60.0))
        .child(Text::new("Right").font_size(11.0))
}

fn preview_thick() -> impl Widget {
    VStack::new().spacing(8.0)
        .child(Divider::horizontal().thickness(2.0).length(180.0).color(Color::from_hex("#0A84FF").unwrap()))
        .child(Divider::horizontal().thickness(1.0).length(180.0))
}

fn preview_custom() -> impl Widget {
    VStack::new().spacing(10.0)
        .child(Divider::horizontal().length(120.0).thickness(1.0))
        .child(HStack::new().spacing(8.0)
            .child(Divider::vertical().length(40.0))
            .child(VStack::new().spacing(4.0)
                .child(Text::new("Section").font_size(10.0).bold())
                .child(Divider::horizontal().length(100.0))
                .child(Text::new("Content").font_size(11.0))
            )
        )
}

fn main() {
    let mut app = App::new("Divider", 900, 400);
    let title = Text::new("Divider").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());
    let row = HStack::new().spacing(24.0)
        .child(cell("Divider Horizontal", "A horizontal line that separates content", "initializer", preview_horizontal()))
        .child(cell("Divider Vertical", "A vertical line that separates content", "initializer", preview_vertical()))
        .child(cell("Divider Thick", "Custom thickness and tint", "style", preview_thick()))
        .child(cell("Divider Section", "Dividers inside sections", "modifier", preview_custom()));
    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
