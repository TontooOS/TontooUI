//! TontooUI Navigation demo — NavigationSubtitle directly on window.

use tontooui::prelude::*;
use tontooui::NavigationSubtitle;

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

fn main() {
    let mut app = App::new("TontooUI Navigation", 420, 320);
    let title = Text::new("Navigation").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let grid = HStack::new().spacing(24.0).child(cell(
        "Navigation Subtitle",
        "Configures the view's subtitle for purposes of navigation, using a localized string key",
        "modifier",
        NavigationSubtitle::new("Foo", "Bar"),
    ));

    // Also show custom subtitle directly on window
    let extra = HStack::new().spacing(12.0)
        .child(NavigationSubtitle::new("TontooOS", "Version 26.1"));

    let root = VStack::new().spacing(18.0).child(title).child(grid).child(extra);
    app.set_root(root);
    app.run();
}
