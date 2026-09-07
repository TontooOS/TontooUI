//! Lists demo — riesige Menge 1:1 aus Screenshots
//! 32 Varianten direkt auf Background (#1d1d1d / #ececec), nur TontooUI API, Ampeln sichtbar.

use tontooui::prelude::*;
use tontooui::{List, ListSection, ListRow, ListStyle, OutlineGroup, DisclosureGroup};

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new().spacing(6.0)
        .child(HStack::new().spacing(0.0).child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap())))
        .child(preview)
        .child(Text::new(title).font_size(10.0).bold().color(title_c).max_width(190.0))
        .child(Text::new(desc).font_size(8.0).color(desc_c).max_width(190.0))
}

// ── 32 previews ──

fn p_outline() -> impl Widget { List::new().outline_group(OutlineGroup::new("Header").child(ListRow::new("Foo")).child(ListRow::new("Foo 1")).child(ListRow::new("Foo 2"))).list_style(ListStyle::Sidebar) }
fn p_disclosure() -> impl Widget { List::new().disclosure_group(DisclosureGroup::new("Header").child(ListRow::new("Foo")).child(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_edit_button() -> impl Widget {
    VStack::new().spacing(8.0)
        .child(EditButton::new())
        .child(List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))))
}
fn p_sidebar_style() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::Sidebar) }
fn p_inset_grouped() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::InsetGrouped) }
fn p_inset() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::Inset) }
fn p_elliptical() -> impl Widget { List::new().section(ListSection::new().row(ListRow::new("Button"))).section(ListSection::new().row(ListRow::new("Picker").detail("Option 1"))).list_style(ListStyle::Elliptical) }
fn p_carousel() -> impl Widget { List::new().section(ListSection::new().row(ListRow::new("Button")).row(ListRow::new("Picker").detail("Option 1"))).list_style(ListStyle::Carousel) }
fn p_bordered() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).list_style(ListStyle::Bordered) }
fn p_section_index() -> impl Widget { List::new().section(ListSection::new().header("A").row(ListRow::new("5")).row(ListRow::new("6")).row(ListRow::new("7")).row(ListRow::new("8"))).section_index_visible(true) }
fn p_move_disabled() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").move_disabled(true))).list_style(ListStyle::InsetGrouped) }
fn p_delete_disabled() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").delete_disabled(true))).list_style(ListStyle::InsetGrouped) }
fn p_refreshable() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).refreshable() }
fn p_swipe() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").swipe_action("Action"))).list_style(ListStyle::InsetGrouped) }
fn p_badge_prominence() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Increased").badge(1).badge_prominent()).row(ListRow::new("Standard").badge(2)).row(ListRow::new("Decreased").badge(3))).list_style(ListStyle::InsetGrouped) }
fn p_badge() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").badge(1)).row(ListRow::new("Bar").badge(2))).list_style(ListStyle::InsetGrouped) }
fn p_row_bg() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").row_background(Color::from_rgb(0,122,255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_hidden_row_sep() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_hidden(true)).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_hidden_section_sep() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_section_sep_tint() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_tint(Color::from_rgb(10,132,255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_row_sep_tint() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_tint(Color::from_rgb(10,132,255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_item_tint() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").tint(Color::from_rgb(10,132,255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped) }
fn p_section_margins() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).section_margins(16.0, 12.0)).list_style(ListStyle::InsetGrouped) }
fn p_compact_spacing() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).compact_spacing() }
fn p_custom_spacing() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).section(ListSection::new().header("Foo").row(ListRow::new("Foo"))).custom_section_spacing(24.0) }
fn p_row_spacing() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).row_spacing(12.0)).list_style(ListStyle::InsetGrouped) }
fn p_min_header() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).footer("Footer")).min_header_height(44.0) }
fn p_min_row() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).footer("Footer")).min_row_height(44.0) }
fn p_row_insets() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Header Only")).row(ListRow::new("Header")).row(ListRow::new("Normal"))).list_style(ListStyle::InsetGrouped) }
fn p_increased_header() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).footer("Footer").header_prominent()).list_style(ListStyle::InsetGrouped) }
fn p_background_prominence() -> impl Widget { List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).footer("Footer")).background_prominent().list_style(ListStyle::InsetGrouped) }

fn main() {
    let mut app = App::new("Lists", 1100, 900);
    let title = Text::new("Lists").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let rows = vec![
        HStack::new().spacing(14.0)
            .child(cell("Outline Group", "A structure that computes views and disclosure groups on demand from an...", "initializer", p_outline()))
            .child(cell("Disclosure Group", "A view that shows or hides another content view, based on the state of a...", "initializer", p_disclosure()))
            .child(cell("Edit Button", "A button that toggles the edit mode environment value.", "initializer", p_edit_button()))
            .child(cell("SidebarListStyle", "The list style that describes the behavior and appearance of a sidebar list.", "style", p_sidebar_style())),
        HStack::new().spacing(14.0)
            .child(cell("InsetGroupedListStyle", "The list style that describes the behavior and appearance of an inset grouped list.", "style", p_inset_grouped()))
            .child(cell("InsetGroupedListStyle", "The list style that describes the behavior and appearance of an inset grouped list.", "style", p_inset_grouped()))
            .child(cell("InsetListStyle", "The list style that describes the behavior and appearance of an inset list.", "style", p_inset()))
            .child(cell("EllipticalListStyle", "The list style that describes the behavior and appearance of an elliptical list.", "style", p_elliptical())),
        HStack::new().spacing(14.0)
            .child(cell("CarouselListStyle", "The carousel list style.", "style", p_carousel()))
            .child(cell("BorderedListStyle", "The list style that describes the behavior and appearance of a list with standard...", "style", p_bordered()))
            .child(cell("Section Index Label and Visibility", "Sets the label that is used in a section index to point to this section, typically...", "modifier", p_section_index()))
            .child(cell("Move Disabled", "Adds a condition for whether the view’s view hierarchy is movable.", "modifier", p_move_disabled())),
        HStack::new().spacing(14.0)
            .child(cell("Delete Disabled", "Adds a condition for whether the view’s view hierarchy is deletable.", "modifier", p_delete_disabled()))
            .child(cell("Refreshable List", "Marks this view as refreshable.", "modifier", p_refreshable()))
            .child(cell("Swipe Action", "Adds custom swipe actions to a row in a list.", "modifier", p_swipe()))
            .child(cell("List Badge Prominence", "Specifies the prominence of badges created by this view.", "modifier", p_badge_prominence())),
        HStack::new().spacing(14.0)
            .child(cell("List Badge", "Generates a badge for the view from an integer value.", "modifier", p_badge()))
            .child(cell("List Row Background", "Places a custom background view behind a list row item.", "modifier", p_row_bg()))
            .child(cell("Hidden List Row Separator", "Sets the display mode to hidden for the separator associated with this specific...", "modifier", p_hidden_row_sep()))
            .child(cell("Hidden List Section Separator", "Sets whether to hide the separator associated with a list section.", "modifier", p_hidden_section_sep())),
        HStack::new().spacing(14.0)
            .child(cell("List Section Separator Tint", "Sets the tint color associated with a section.", "modifier", p_section_sep_tint()))
            .child(cell("List Row Separator Tint", "Sets the tint color associated with a row.", "modifier", p_row_sep_tint()))
            .child(cell("List Item Tint", "Sets a fixed tint color for content in a list.", "modifier", p_item_tint()))
            .child(cell("List Section Margins", "Set the section margins for the specific edges.", "modifier", p_section_margins())),
        HStack::new().spacing(14.0)
            .child(cell("Compact List Section Spacing", "Compact spacing between sections", "modifier", p_compact_spacing()))
            .child(cell("Custom List Section Spacing", "Sets the spacing between adjacent sections in a List to a custom value.", "modifier", p_custom_spacing()))
            .child(cell("List Row Spacing", "Sets the vertical spacing between two adjacent rows in a List.", "modifier", p_row_spacing()))
            .child(cell("Default Min List Header Height", "The default minimum height of a header in a list.", "modifier", p_min_header())),
        HStack::new().spacing(14.0)
            .child(cell("Default Min List Row Height", "The default minimum height of rows in a list.", "modifier", p_min_row()))
            .child(cell("List Row Insets", "Applies an inset to the rows in a list.", "modifier", p_row_insets()))
            .child(cell("Increased Header Prominence", "Sets the header prominence to increased for this view.", "modifier", p_increased_header()))
            .child(cell("List Background Prominence", "The prominence of the background underneath views associated with this...", "modifier", p_background_prominence())),
    ];

    let mut grid = VStack::new().spacing(14.0);
    for r in rows { grid = grid.child(r); }

    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
