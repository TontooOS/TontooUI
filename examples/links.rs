//! TontooUI Link demo — All Link category elements directly on window.

use tontooui::prelude::*;
use tontooui::{CustomPreviewShareLink, HelpLink, Link, ShareLink, TextFieldLink};

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
    let mut app = App::new("TontooUI Links", 1120, 720);

    let title = Text::new("Link").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "HelpLink",
            "A button with a standard appearance that opens app-specific help",
            "initializer",
            HelpLink::new().on_activate(|| println!("HelpLink")),
        ))
        .child(cell(
            "TextFieldLink",
            "A control that requests text input from the user when pressed",
            "initializer",
            TextFieldLink::new("Set Text").on_submit(|s| println!("TextFieldLink: {}", s)),
        ))
        .child(cell(
            "Custom Preview Item ShareLink",
            "Creates an instance, with a custom label, that presents the share interface",
            "initializer",
            CustomPreviewShareLink::new("Share Cats", "Derpy Cats"),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "ShareLink",
            "A view that controls a sharing presentation",
            "initializer",
            ShareLink::new(vec!["Share ...".into(), "Explore SwiftUI".into(), "Foo".into()]),
        ))
        .child(cell(
            "Link",
            "A control for navigating to a URL",
            "initializer",
            Link::new("Explore SwiftUI", "https://explore.swiftui.com")
                .on_activate(|url| println!("Link -> {}", url)),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
