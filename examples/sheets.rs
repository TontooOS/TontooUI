//! TontooUI Sheet demo — all 12 Sheet modifiers directly on window.

use tontooui::prelude::*;
use tontooui::{
    BooleanSheet, DisableSheetDismissSwipe, FittedSheetSizing, ItemSheet,
    PageScreenSheetSize, PrioritizeSheetContentScrolling, SheetBackground,
    SheetBackgroundInteraction, SheetCornerRadius, SheetDragIndicatorVisibility,
    SheetPlacement, SheetSize,
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
    let mut app = App::new("TontooUI Sheet", 1180, 900);
    let title = Text::new("Sheet").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Sheet Placement",
            "Sets the placement of a presentation within the presenting view.",
            "modifier",
            SheetPlacement::new(),
        ))
        .child(cell(
            "Disable Sheet Dismiss Swipe",
            "Conditionally prevents interactive dismissal of presentations.",
            "modifier",
            DisableSheetDismissSwipe::new(),
        ))
        .child(cell(
            "Page Screen Sheet Size",
            "On devices smaller than a page of paper, such as iPhone, page sizing fills.",
            "modifier",
            PageScreenSheetSize::new(),
        ))
        .child(cell(
            "Fitted Sheet Sizing",
            "Sets the sizing of the containing presentation.",
            "modifier",
            FittedSheetSizing::new(),
        ));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Sheet Corner Radius",
            "Requests that the presentation have a specific corner radius.",
            "modifier",
            SheetCornerRadius::new(),
        ))
        .child(cell(
            "Prioritize Sheet Content Scrolling",
            "Configure the behavior of swipe gestures on a presentation.",
            "modifier",
            PrioritizeSheetContentScrolling::new(),
        ))
        .child(cell(
            "Sheet Background Interaction",
            "Controls whether people can interact with the view behind.",
            "modifier",
            SheetBackgroundInteraction::new(),
        ))
        .child(cell(
            "Sheet Background",
            "Sets the presentation background of the enclosing sheet.",
            "modifier",
            SheetBackground::new(),
        ));

    let row3 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Sheet Drag Indicator Visibility",
            "Sets the visibility of the drag indicator on top of a sheet.",
            "modifier",
            SheetDragIndicatorVisibility::new(),
        ))
        .child(cell(
            "Sheet Size",
            "Sets the available detents for the enclosing sheet.",
            "modifier",
            SheetSize::new(),
        ))
        .child(cell(
            "Item Sheet",
            "Presents a sheet using the given item as a data source.",
            "modifier",
            ItemSheet::new(),
        ))
        .child(cell(
            "Boolean Sheet",
            "Presents a sheet when a binding to a Boolean value is true.",
            "modifier",
            BooleanSheet::new(),
        ));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2).child(row3);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
