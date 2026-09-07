//! TontooUI View demo — all 10 View modifiers directly on window.

use tontooui::prelude::*;
use tontooui::{
    AppStoreOverlay, BackgroundExtensionEffect, ControlSizeView, GlassEffect,
    ManageSubscriptionsSheet, MusicPicker, NavigationContainerBackground,
    NavigationSplitViewBackground, SwipeAction, SwipeContainer,
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
    let mut app = App::new("TontooUI View", 1180, 900);
    let title = Text::new("View").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Music Picker",
            "Presents a music picker to select items from the Apple Music catalog and the...",
            "modifier",
            MusicPicker::new(),
        ))
        .child(cell(
            "App Store Overlay",
            "Presents a StoreKit overlay when a given condition is true.",
            "modifier",
            AppStoreOverlay::new("com.example.app"),
        ))
        .child(cell(
            "Manage Subscriptions Sheet",
            "Opens the manage subscriptions sheet.",
            "modifier",
            ManageSubscriptionsSheet::new(),
        ))
        .child(cell(
            "Swipe Container",
            "Only allows a single active swipe within a container.",
            "modifier",
            SwipeContainer::new(),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Swipe Action",
            "Adds custom swipe actions to a row in a list or container, notifying you when th...",
            "modifier",
            SwipeAction::new(),
        ))
        .child(cell(
            "Navigation Split View Container ...",
            "A background placement behind the content of a NavigationSplitView.",
            "modifier",
            NavigationSplitViewBackground::new(),
        ))
        .child(cell(
            "Navigation Container Background",
            "Sets the container background of the enclosing container using a view.",
            "modifier",
            NavigationContainerBackground::new(),
        ))
        .child(cell(
            "ControlSize",
            "A control version that is the default size",
            "modifier",
            ControlSizeView::new(),
        ));

    let row3 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Background Extension Effect",
            "Adds the background extension effect to the view. The view will be duplicate...",
            "modifier",
            BackgroundExtensionEffect::new(),
        ))
        .child(cell(
            "Glass effect",
            "Applies the Liquid Glass effect to a view",
            "modifier",
            GlassEffect::new(),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2).child(row3);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
