//! TontooUI ProductView demo — all 6 ProductView elements directly on window.

use tontooui::prelude::*;
use tontooui::{
    CompactProductViewStyle, CustomIconProductView, LargeProductViewStyle,
    PlaceholderIconProductView, ProductViewElement, RegularProductViewStyle,
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
    let mut app = App::new("TontooUI ProductView", 1180, 900);
    let title = Text::new("ProductView").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Placeholder Icon Product View",
            "Creates a view to load an individual product with a placeholder icon.",
            "initializer",
            PlaceholderIconProductView::new(),
        ))
        .child(cell(
            "Custom Icon Product View",
            "Creates a view to load an individual product with a custom icon.",
            "initializer",
            CustomIconProductView::new(),
        ))
        .child(cell(
            "Product View",
            "Creates a view to load and merchandise an individual product.",
            "initializer",
            ProductViewElement::new(),
        ))
        .child(cell(
            "CompactProductViewStyle",
            "A style for a product view for layouts with less available space.",
            "style",
            CompactProductViewStyle::new(),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "RegularProductViewStyle",
            "A style for a product view that uses a standard layout.",
            "style",
            RegularProductViewStyle::new(),
        ))
        .child(cell(
            "LargeProductViewStyle",
            "A style for a product view where the purchase is the hero content.",
            "style",
            LargeProductViewStyle::new(),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
