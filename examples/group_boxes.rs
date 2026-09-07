//! GroupBox demo — 3 Varianten direkt auf Background, nur TontooUI API, Ampeln sichtbar.

use tontooui::prelude::*;
use tontooui::GroupBox;

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new().spacing(8.0)
        .child(HStack::new().spacing(0.0).child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap())))
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

fn preview_with_label() -> impl Widget {
    GroupBox::with_label("Hello World")
        .child(Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat.").font_size(10.0).color(Color::from_hex("#a1a1aa").unwrap()).max_width(200.0))
        .width(260.0)
}

fn preview_plain() -> impl Widget {
    GroupBox::new()
        .child(Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.").font_size(10.0).color(Color::from_hex("#a1a1aa").unwrap()).max_width(200.0))
        .width(260.0)
}

fn preview_background() -> impl Widget {
    GroupBox::new()
        .background(Color::from_hex("#0A84FF").unwrap())
        .child(Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat.").font_size(10.0).color(Color::WHITE).max_width(200.0))
        .width(260.0)
}

fn main() {
    let mut app = App::new("GroupBox", 900, 420);
    let title = Text::new("GroupBox").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());
    let row = HStack::new().spacing(24.0)
        .child(cell("GroupBox With Label", "A GroupBox with Label", "initializer", preview_with_label()))
        .child(cell("GroupBox", "A simple GroupBox", "initializer", preview_plain()))
        .child(cell("GroupBox Background", "Set a custom GroupBox background", "modifier", preview_background()));
    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
