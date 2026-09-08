//! Review items: Picker (16), List (31).

use super::ReviewItem;
use tontooui::prelude::*;

fn base_section() -> ListSection {
    ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))
}

pub fn items(out: &mut Vec<ReviewItem>) {
    // ── Picker (16) ──
    out.push(ReviewItem { id: "picker_custom_label", category: "Picker", title: "Custom Value Label Picker", desc: "Creates a picker that generates its label from a localized string key.", badge: "initializer", make: || WidgetNode::new(
        Picker::new("Foo").titles(["1", "2", "3"]).selected_index(0).custom_value_label("Current", |v| format!("Current: {v}")).style(PickerStyle::Menu),
    ) });
    out.push(ReviewItem { id: "picker_multi_source", category: "Picker", title: "Multiple Sources Picker", desc: "Creates a picker that updates the selected property for all sources.", badge: "initializer", make: || WidgetNode::new(
        Picker::new("Size for all pets").titles(["small", "medium", "large"]).selected_index(1).multiple_sources(true).style(PickerStyle::Menu),
    ) });
    out.push(ReviewItem { id: "picker_assets", category: "Picker", title: "Picker", desc: "Creates pickers with title, image, systemImage, and custom label.", badge: "initializer", make: || WidgetNode::new(
        Picker::new("Picker")
            .item(PickerItem::new("Foo").with_subtitle("1"))
            .item(PickerItem::new("Bar").with_subtitle("1"))
            .item(PickerItem::new("Bar").with_image("bar.png").with_subtitle("1"))
            .item(PickerItem::new("Foo").with_system_image("star.fill").with_subtitle("1"))
            .selected_index(0).style(PickerStyle::Inline),
    ) });
    out.push(ReviewItem { id: "picker_tabs", category: "Picker", title: "TabsPickerStyle", desc: "A picker style that presents options as segmented tabs.", badge: "style", make: || WidgetNode::new(
        Picker::new("Tabs").titles(["1", "Bar", "2", "3"]).selected_index(0).style(PickerStyle::Tabs),
    ) });
    out.push(ReviewItem { id: "picker_wheel", category: "Picker", title: "WheelPickerStyle", desc: "A picker style that presents the options in a scrollable wheel.", badge: "style", make: || WidgetNode::new(
        Picker::new("Wheel").titles(["1", "Bar", "2", "3"]).selected_index(0).style(PickerStyle::Wheel).frame(220.0, 160.0),
    ) });
    out.push(ReviewItem { id: "picker_segmented", category: "Picker", title: "SegmentedPickerStyle", desc: "A picker style that presents options in a segmented control.", badge: "style", make: || WidgetNode::new(
        Picker::new("Segmented").titles(["1", "Bar", "2", "3"]).selected_index(0).style(PickerStyle::Segmented),
    ) });
    out.push(ReviewItem { id: "picker_palette", category: "Picker", title: "PalettePickerStyle", desc: "A picker style that presents options as a row of compact elements.", badge: "style", make: || WidgetNode::new(
        Picker::new("Palette").titles(["1", "2", "3", "Bar"]).selected_index(2).style(PickerStyle::Palette),
    ) });
    out.push(ReviewItem { id: "picker_radio", category: "Picker", title: "RadioGroupPickerStyle", desc: "A picker style that presents options as a group of radio buttons.", badge: "style", make: || WidgetNode::new(
        Picker::new("Radio").titles(["1", "2", "3"]).selected_index(2).style(PickerStyle::RadioGroup),
    ) });
    out.push(ReviewItem { id: "picker_navlink", category: "Picker", title: "NavigationLinkPickerStyle", desc: "A picker style represented by a navigation link.", badge: "style", make: || WidgetNode::new(
        Picker::new("Foo").titles(["1", "Bar", "2"]).selected_index(0).style(PickerStyle::NavigationLink),
    ) });
    out.push(ReviewItem { id: "picker_menu", category: "Picker", title: "MenuPickerStyle", desc: "A picker style that presents options as a menu.", badge: "style", make: || WidgetNode::new(
        Picker::new("Foo").titles(["1", "Bar", "2"]).selected_index(0).style(PickerStyle::Menu),
    ) });
    out.push(ReviewItem { id: "picker_inline", category: "Picker", title: "InlinePickerStyle", desc: "A PickerStyle where each option is displayed inline.", badge: "style", make: || WidgetNode::new(
        Picker::new("Foo").titles(["1", "Bar", "2", "3"]).selected_index(0).style(PickerStyle::Inline),
    ) });
    out.push(ReviewItem { id: "picker_wheel_height", category: "Picker", title: "Wheel Picker Item Height", desc: "Sets the default wheel-style picker item height.", badge: "modifier", make: || WidgetNode::new(
        Picker::new("Wheel Height").titles(["1", "Bar", "2"]).selected_index(0).style(PickerStyle::Wheel).wheel_item_height(52.0).frame(220.0, 160.0),
    ) });
    out.push(ReviewItem { id: "picker_h_radio", category: "Picker", title: "Horizontal Radio Group Layout", desc: "Sets the style for radio group pickers to be horizontal.", badge: "modifier", make: || WidgetNode::new(
        Picker::new("Horizontal Radio").titles(["FooBar 1", "2", "3"]).selected_index(0).style(PickerStyle::RadioGroup).horizontal_radio_group(true),
    ) });
    out.push(ReviewItem { id: "picker_section", category: "Picker", title: "Picker Section", desc: "A Section inside a Picker.", badge: "content", make: || WidgetNode::new(
        Picker::new("Foo").section(PickerSection::new("Section").items(vec![PickerItem::new("1"), PickerItem::new("2"), PickerItem::new("3")])).style(PickerStyle::Menu),
    ) });
    out.push(ReviewItem { id: "picker_divider", category: "Picker", title: "Picker Divider", desc: "A Picker with a Divider.", badge: "content", make: || WidgetNode::new(
        Picker::new("Foo").titles(["1", "2", "3"]).selected_index(0).divider().style(PickerStyle::Menu),
    ) });
    out.push(ReviewItem { id: "wheel_picker", category: "Picker", title: "WheelPicker", desc: "The spring-physics wheel drum picker.", badge: "initializer", make: || WidgetNode::new(
        WheelPicker::new().items(["1", "2", "3", "4", "5", "6"]).selected("3").item_height(40.0).frame(180.0, 140.0),
    ) });

    // ── List (31 — gallery duplicate InsetGrouped card listed once) ──
    out.push(ReviewItem { id: "list_outline", category: "List", title: "Outline Group", desc: "A structure that computes views and disclosure groups on demand.", badge: "initializer", make: || WidgetNode::new(
        List::new().outline_group(OutlineGroup::new("Header").child(ListRow::new("Foo")).child(ListRow::new("Foo 1")).child(ListRow::new("Foo 2"))).list_style(ListStyle::Sidebar),
    ) });
    out.push(ReviewItem { id: "list_disclosure", category: "List", title: "Disclosure Group", desc: "A view that shows or hides another content view.", badge: "initializer", make: || WidgetNode::new(
        List::new().disclosure_group(DisclosureGroup::new("Header").child(ListRow::new("Foo")).child(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_edit_button", category: "List", title: "Edit Button", desc: "A button that toggles the edit mode environment value.", badge: "initializer", make: || WidgetNode::new(
        VStack::new().spacing(8.0).child(EditButton::new()).child(List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")))),
    ) });
    out.push(ReviewItem { id: "list_sidebar", category: "List", title: "SidebarListStyle", desc: "The list style for a sidebar list.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::Sidebar),
    ) });
    out.push(ReviewItem { id: "list_inset_grouped", category: "List", title: "InsetGroupedListStyle", desc: "The list style for an inset grouped list.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_inset", category: "List", title: "InsetListStyle", desc: "The list style for an inset list.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Bar1")).row(ListRow::new("Bar2")).footer("Footer")).list_style(ListStyle::Inset),
    ) });
    out.push(ReviewItem { id: "list_elliptical", category: "List", title: "EllipticalListStyle", desc: "The list style for an elliptical list.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().row(ListRow::new("Button"))).section(ListSection::new().row(ListRow::new("Picker").detail("Option 1"))).list_style(ListStyle::Elliptical),
    ) });
    out.push(ReviewItem { id: "list_carousel", category: "List", title: "CarouselListStyle", desc: "The carousel list style.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().row(ListRow::new("Button")).row(ListRow::new("Picker").detail("Option 1"))).list_style(ListStyle::Carousel),
    ) });
    out.push(ReviewItem { id: "list_bordered", category: "List", title: "BorderedListStyle", desc: "The list style for a list with standard borders.", badge: "style", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).list_style(ListStyle::Bordered),
    ) });
    out.push(ReviewItem { id: "list_section_index", category: "List", title: "Section Index Label and Visibility", desc: "Sets the section index label and visibility.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("A").row(ListRow::new("5")).row(ListRow::new("6")).row(ListRow::new("7")).row(ListRow::new("8"))).section_index_visible(true),
    ) });
    out.push(ReviewItem { id: "list_move_disabled", category: "List", title: "Move Disabled", desc: "Adds a condition for whether the view hierarchy is movable.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").move_disabled(true))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_delete_disabled", category: "List", title: "Delete Disabled", desc: "Adds a condition for whether the view hierarchy is deletable.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").delete_disabled(true))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_refreshable", category: "List", title: "Refreshable List", desc: "Marks this view as refreshable.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(base_section()).refreshable(),
    ) });
    out.push(ReviewItem { id: "list_swipe", category: "List", title: "Swipe Action", desc: "Adds custom swipe actions to a row in a list.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar").swipe_action("Action"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_badge_prominence", category: "List", title: "List Badge Prominence", desc: "Specifies the prominence of badges created by this view.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Increased").badge(1).badge_prominent()).row(ListRow::new("Standard").badge(2)).row(ListRow::new("Decreased").badge(3))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_badge", category: "List", title: "List Badge", desc: "Generates a badge for the view from an integer value.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").badge(1)).row(ListRow::new("Bar").badge(2))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_row_bg", category: "List", title: "List Row Background", desc: "Places a custom background view behind a list row item.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").row_background(Color::from_rgb(0, 122, 255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_hide_row_sep", category: "List", title: "Hidden List Row Separator", desc: "Hides the separator for a specific row.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_hidden(true)).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_hide_section_sep", category: "List", title: "Hidden List Section Separator", desc: "Sets whether to hide the separator of a list section.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(base_section()).section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_section_sep_tint", category: "List", title: "List Section Separator Tint", desc: "Sets the tint color associated with a section.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_tint(Color::from_rgb(10, 132, 255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_row_sep_tint", category: "List", title: "List Row Separator Tint", desc: "Sets the tint color associated with a row.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").separator_tint(Color::from_rgb(10, 132, 255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_item_tint", category: "List", title: "List Item Tint", desc: "Sets a fixed tint color for content in a list.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo").tint(Color::from_rgb(10, 132, 255))).row(ListRow::new("Bar"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_section_margins", category: "List", title: "List Section Margins", desc: "Set the section margins for the specific edges.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).section_margins(16.0, 12.0)).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_compact_spacing", category: "List", title: "Compact List Section Spacing", desc: "Compact spacing between sections.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(base_section()).section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar"))).compact_spacing(),
    ) });
    out.push(ReviewItem { id: "list_custom_spacing", category: "List", title: "Custom List Section Spacing", desc: "Sets the spacing between adjacent sections to a custom value.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(base_section()).section(ListSection::new().header("Foo").row(ListRow::new("Foo"))).custom_section_spacing(24.0),
    ) });
    out.push(ReviewItem { id: "list_row_spacing", category: "List", title: "List Row Spacing", desc: "Sets the vertical spacing between two adjacent rows.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).row_spacing(12.0)).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_min_header", category: "List", title: "Default Min List Header Height", desc: "The default minimum height of a header in a list.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).footer("Footer")).min_header_height(44.0),
    ) });
    out.push(ReviewItem { id: "list_min_row", category: "List", title: "Default Min List Row Height", desc: "The default minimum height of rows in a list.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).row(ListRow::new("Bar")).footer("Footer")).min_row_height(44.0),
    ) });
    out.push(ReviewItem { id: "list_row_insets", category: "List", title: "List Row Insets", desc: "Applies an inset to the rows in a list.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Header Only")).row(ListRow::new("Header")).row(ListRow::new("Normal"))).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_header_prominence", category: "List", title: "Increased Header Prominence", desc: "Sets the header prominence to increased for this view.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).footer("Footer").header_prominent()).list_style(ListStyle::InsetGrouped),
    ) });
    out.push(ReviewItem { id: "list_bg_prominence", category: "List", title: "List Background Prominence", desc: "The prominence of the background underneath associated views.", badge: "modifier", make: || WidgetNode::new(
        List::new().section(ListSection::new().header("Header").row(ListRow::new("Foo")).footer("Footer")).background_prominent().list_style(ListStyle::InsetGrouped),
    ) });
}
