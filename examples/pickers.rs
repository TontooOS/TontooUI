//! TontooUI Pickers demo — all Picker elements from the SwiftUI Picker gallery.
//!
//! Each card mirrors one screenshot in the gallery:
//! Custom Value Label, Multiple Sources, Title/Image/SystemImage/CustomLabel,
//! Tabs, Wheel, Segmented, Palette, RadioGroup, NavigationLink, Menu, Inline,
//! Wheel Item Height, Horizontal Radio, Picker Section, Picker Divider.
//!
//! This example uses **only TontooUI** (`tontooui::prelude::*`).
//! UIKit/GTK are hidden behind TontooUI — `Picker`/`WheelPicker`/`PickerCard`
//! render via GTK internally, but the demo itself never imports `gtk` or `uikit`.
//! The window bar (traffic lights / Ampeln) is shown — standard TontooUI chrome.

use tontooui::prelude::*;

// ── Builders for each gallery entry ────────────────────────────────────

fn custom_value_label_picker() -> Picker {
    Picker::new("Foo")
        .titles(["1", "2", "3"])
        .selected_index(0)
        .custom_value_label("Current", |v| format!("Current: {v}"))
        .style(PickerStyle::Menu)
        .on_change(|v| println!("custom label picked: {v}"))
}

fn multiple_sources_picker() -> Picker {
    Picker::new("Size for all pets")
        .titles(["small", "medium", "large"])
        .selected_index(1)
        .multiple_sources(true)
        .on_change(|v| println!("main source: {v}"))
        .on_change_multiple(|v| println!("Bella source: {v}"))
        .on_change_multiple(|v| println!("Max source: {v}"))
        .style(PickerStyle::Menu)
}

fn picker_with_assets() -> Picker {
    Picker::new("Picker")
        .item(PickerItem::new("Foo").with_subtitle("1"))
        .item(PickerItem::new("Bar").with_subtitle("1"))
        .item(PickerItem::new("Bar").with_image("bar.png").with_subtitle("1"))
        .item(PickerItem::new("Foo").with_system_image("star.fill").with_subtitle("1"))
        .selected_index(0)
        .style(PickerStyle::Inline)
}

fn tabs_picker() -> Picker {
    Picker::new("Tabs")
        .titles(["1", "Bar", "2", "3"])
        .selected_index(0)
        .style(PickerStyle::Tabs)
        .on_change(|v| println!("tabs: {v}"))
}

fn wheel_picker_demo() -> Picker {
    Picker::new("Wheel")
        .titles(["1", "Bar", "2", "3"])
        .selected_index(0)
        .style(PickerStyle::Wheel)
        .frame(220.0, 160.0)
}

fn segmented_picker() -> Picker {
    Picker::new("Segmented")
        .titles(["1", "Bar", "2", "3"])
        .selected_index(0)
        .style(PickerStyle::Segmented)
        .on_change(|v| println!("segmented: {v}"))
}

fn palette_picker() -> Picker {
    Picker::new("Palette")
        .titles(["1", "2", "3", "Bar"])
        .selected_index(2)
        .style(PickerStyle::Palette)
}

fn radio_group_picker() -> Picker {
    Picker::new("Radio")
        .titles(["1", "2", "3"])
        .selected_index(2)
        .style(PickerStyle::RadioGroup)
}

fn navigation_link_picker() -> Picker {
    Picker::new("Foo")
        .titles(["1", "Bar", "2"])
        .selected_index(0)
        .style(PickerStyle::NavigationLink)
}

fn menu_picker() -> Picker {
    Picker::new("Foo")
        .titles(["1", "Bar", "2"])
        .selected_index(0)
        .style(PickerStyle::Menu)
}

fn inline_picker() -> Picker {
    Picker::new("Foo")
        .titles(["1", "Bar", "2", "3"])
        .selected_index(0)
        .style(PickerStyle::Inline)
}

fn wheel_item_height_picker() -> Picker {
    Picker::new("Wheel Height")
        .titles(["1", "Bar", "2"])
        .selected_index(0)
        .style(PickerStyle::Wheel)
        .wheel_item_height(52.0)
        .frame(220.0, 160.0)
}

fn horizontal_radio_picker() -> Picker {
    Picker::new("Horizontal Radio")
        .titles(["FooBar 1", "2", "3"])
        .selected_index(0)
        .style(PickerStyle::RadioGroup)
        .horizontal_radio_group(true)
}

fn picker_section_demo() -> Picker {
    Picker::new("Foo")
        .section(PickerSection::new("Section").items(vec![PickerItem::new("1"), PickerItem::new("2"), PickerItem::new("3")]))
        .style(PickerStyle::Menu)
}

fn picker_divider_demo() -> Picker {
    Picker::new("Foo")
        .titles(["1", "2", "3"])
        .selected_index(0)
        .divider()
        .style(PickerStyle::Menu)
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new("TontooUI Pickers", 1080, 860);
    // Standard TontooUI Window-Bar mit Ampeln (rot/gelb/grün) — wie jede normale App
    // `no_window_bar()` bewusst entfernt, damit du die Ampeln siehst

    // Each card is a pure TontooUI `PickerCard` (GTK hidden inside the library)
    let row1 = HStack::new()
        .spacing(14.0)
        .child(PickerCard::new("initializer", "Custom Value Label Picker", "Creates a picker that generates its label from a localized string key.", custom_value_label_picker()))
        .child(PickerCard::new("initializer", "Multiple Sources Picker", "Creates a picker that updates the selected property for all sources.", multiple_sources_picker()))
        .child(PickerCard::new("initializer", "Picker", "Creates pickers with title, image, systemImage, and custom label.", picker_with_assets()))
        .child(PickerCard::new("style", "TabsPickerStyle", "A picker style that presents options as segmented tabs.", tabs_picker()));

    let row2 = HStack::new()
        .spacing(14.0)
        .child(PickerCard::new("style", "WheelPickerStyle", "A picker style that presents the options in a scrollable wheel.", wheel_picker_demo()))
        .child(PickerCard::new("style", "SegmentedPickerStyle", "A picker style that presents options in a segmented control.", segmented_picker()))
        .child(PickerCard::new("style", "PalettePickerStyle", "A picker style that presents options as a row of compact elements.", palette_picker()))
        .child(PickerCard::new("style", "RadioGroupPickerStyle", "A picker style that presents options as a group of radio buttons.", radio_group_picker()));

    let row3 = HStack::new()
        .spacing(14.0)
        .child(PickerCard::new("style", "NavigationLinkPickerStyle", "A picker style represented by a navigation link.", navigation_link_picker()))
        .child(PickerCard::new("style", "MenuPickerStyle", "A picker style that presents options as a menu.", menu_picker()))
        .child(PickerCard::new("style", "InlinePickerStyle", "A PickerStyle where each option is displayed inline.", inline_picker()))
        .child(PickerCard::new("modifier", "Wheel Picker Item Height", "Sets the default wheel-style picker item height.", wheel_item_height_picker()));

    let row4 = HStack::new()
        .spacing(14.0)
        .child(PickerCard::new("modifier", "Horizontal Radio Group Layout", "Sets the style for radio group pickers to be horizontal.", horizontal_radio_picker()))
        .child(PickerCard::new("content", "Picker Section", "A Section inside a Picker.", picker_section_demo()))
        .child(PickerCard::new("content", "Picker Divider", "A Picker with a Divider.", picker_divider_demo()))
        // Reuse existing animated WheelPicker — now lives in `src/pickers/wheel_picker.rs`
        .child(PickerCard::new("legacy", "WheelPicker (existing)", "The original spring-physics wheel drum (WheelPicker).", WheelPicker::new().items(["1","2","3","4","5","6"]).selected("3").item_height(40.0).frame(180.0, 140.0)));

    let root = VStack::new()
        .spacing(14.0)
        .child(Text::new("Picker").font_size(26.0).bold())
        .child(Text::new("All SwiftUI Picker styles — SF Pro Display, #1d1d1d dark / #ececec light, live selection. Only TontooUI, UIKit hidden.").font_size(12.0).color(Color::from_rgb(142, 142, 147)))
        .child(row1)
        .child(row2)
        .child(row3)
        .child(row4);

    app.set_root(root);
    app.run();
}
