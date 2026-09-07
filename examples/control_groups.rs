//! TontooUI ControlGroup demo — all 5 ControlGroup elements directly on window.

use tontooui::prelude::*;
use tontooui::{
    CompactMenuControlGroupStyle, ControlGroup, ControlGroupStyle, MenuControlGroupStyle,
    NavigationControlGroupStyle, PaletteControlGroupStyle,
};

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
    let mut app = App::new("TontooUI ControlGroup", 1120, 720);
    let title = Text::new("ControlGroup").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "ControlGroup",
            "Creates a new ControlGroup with the specified children",
            "initializer",
            ControlGroup::new(),
        ))
        .child(cell(
            "PaletteControlGroupStyle",
            "A control group style that presents its content as a palette.",
            "style",
            PaletteControlGroupStyle::new(),
        ))
        .child(cell(
            "NavigationControlGroupStyle",
            "The navigation control group style",
            "style",
            NavigationControlGroupStyle::new(),
        ))
        .child(cell(
            "MenuControlGroupStyle",
            "A control group style that presents its content as a menu when the user...",
            "style",
            MenuControlGroupStyle::new(),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "CompactMenuControlGroupStyle",
            "A control group style that presents its content as a compact menu when the...",
            "style",
            CompactMenuControlGroupStyle::new(),
        ))
        .child(cell(
            "ControlGroup (Palette via API)",
            "Palette style via ControlGroup + ControlGroupStyle::Palette",
            "style",
            ControlGroup::new().style(ControlGroupStyle::Palette),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
