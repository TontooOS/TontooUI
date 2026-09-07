//! ScrollView demo — 1:1 aus Screenshot, direkt auf Background, nur TontooUI API, Ampeln sichtbar.

use tontooui::prelude::*;

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

fn preview_hard_edge() -> impl Widget {
    // Content with items 5,6,7 as in screenshot — scrollable, pure TontooUI API
    let inner = VStack::new().spacing(6.0)
        .child(Text::new("Item 5").font_size(12.0))
        .child(Divider::horizontal().length(200.0))
        .child(Text::new("Item 6").font_size(12.0))
        .child(Divider::horizontal().length(200.0))
        .child(Text::new("Item 7").font_size(12.0))
        .child(Text::new("Item 8").font_size(12.0))
        .child(Text::new("Item 9").font_size(12.0));
    tontooui::ScrollView::new().content(inner).hard_edge().vertical(true)
}

fn main() {
    let mut app = App::new("ScrollView", 420, 480);
    let title = Text::new("ScrollView").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());
    let row = HStack::new().spacing(24.0)
        .child(cell("HardScrollEdgeEffect", "A scroll edge effect with a hard cutoff and dividing line", "style", preview_hard_edge()));
    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
