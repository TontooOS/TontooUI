//! Review items: Menu, Color, ControlGroup, Link, Material, Navigation, Text,
//! TextInput.

use super::ReviewItem;
use tontooui::prelude::*;
use tontooui::NavigationControlGroupStyle;

fn std_menu() -> Menu {
    Menu::new("Menu")
        .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
        .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
}

pub fn items(out: &mut Vec<ReviewItem>) {
    // ── Menu (8) ──
    out.push(ReviewItem { id: "menu_action_button", category: "Menu", title: "Menu Action Button", desc: "Acts like a button and opens the menu on a secondary gesture.", badge: "initializer", make: || WidgetNode::new(std_menu()) });
    out.push(ReviewItem { id: "menu_plain", category: "Menu", title: "Menu", desc: "A control for presenting a menu of actions.", badge: "initializer", make: || WidgetNode::new(
        Menu::new("Menu").entry(MenuEntry::Item(MenuItem::new("Button 1"))),
    ) });
    out.push(ReviewItem { id: "context_menu_preview", category: "Menu", title: "Context Menu Preview", desc: "A context menu with custom preview.", badge: "modifier", make: || WidgetNode::new(
        ContextMenu::new(Text::new("Long-press / Right-click").font_size(11.0))
            .preview(Text::new("Preview").font_size(11.0).bold())
            .entries(vec![MenuEntry::Item(MenuItem::new("Button 1"))]),
    ) });
    out.push(ReviewItem { id: "context_menu", category: "Menu", title: "Context Menu", desc: "Adds a context menu to a view.", badge: "modifier", make: || WidgetNode::new(
        ContextMenu::new(Button::new("Buttone")).entries(vec![MenuEntry::Item(MenuItem::new("Button 1"))]),
    ) });
    out.push(ReviewItem { id: "menu_fixed_order", category: "Menu", title: "Menu Fixed Order", desc: "Order items from top to bottom.", badge: "modifier", make: || WidgetNode::new(
        Menu::new("Menu")
            .entry(MenuEntry::Item(MenuItem::new("Button 1")))
            .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
            .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3"))),
    ) });
    out.push(ReviewItem { id: "menu_nested", category: "Menu", title: "Nested Menu", desc: "A Menu inside a Menu.", badge: "content", make: || WidgetNode::new(
        Menu::new("Menu")
            .submenu("Other", vec![MenuEntry::Item(MenuItem::new("Button 1"))])
            .entry(MenuEntry::Item(MenuItem::new("Button 1"))),
    ) });
    out.push(ReviewItem { id: "menu_divider", category: "Menu", title: "Menu Divider", desc: "A Menu with a Divider.", badge: "content", make: || WidgetNode::new(
        Menu::new("Menu")
            .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
            .entry(MenuEntry::Divider)
            .entry(MenuEntry::Item(MenuItem::new("Button 1"))),
    ) });
    out.push(ReviewItem { id: "menu_section", category: "Menu", title: "Menu Section", desc: "A Section inside a Menu.", badge: "content", make: || WidgetNode::new(
        Menu::new("Menu").section_items(Some("foo".to_string()), vec![
            MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")),
            MenuEntry::Item(MenuItem::new("Button 1")),
        ]),
    ) });

    // ── Color (10) ──
    out.push(ReviewItem { id: "color_opacity", category: "Color", title: "Color Opacity", desc: "The Color opacity modifier.", badge: "style", make: || WidgetNode::new(ColorOpacity::new(Color::from_rgb(10, 132, 255))) });
    out.push(ReviewItem { id: "color_gradient", category: "Color", title: "Color Gradient", desc: "The Color gradient modifier.", badge: "modifier", make: || WidgetNode::new(ColorGradient::new(vec![Color::from_rgb(10, 132, 255), Color::from_rgb(0, 80, 160), Color::from_rgb(0, 40, 100)])) });
    out.push(ReviewItem { id: "color_variants", category: "Color", title: "Color Variants", desc: "HierarchicalShapeStyle, a shape style that maps to one of the numbered content styles.", badge: "modifier", make: || WidgetNode::new(ColorVariants::new()) });
    out.push(ReviewItem { id: "color_separator", category: "Color", title: "UIKit Separator Colors", desc: "The UIKit separator colors used in components such as List.", badge: "type", make: || WidgetNode::new(UIKitSeparatorColors::new()) });
    out.push(ReviewItem { id: "color_background", category: "Color", title: "UIKit Content Background colors", desc: "The UIKit colors used in List and Form backgrounds.", badge: "type", make: || WidgetNode::new(UIKitContentBackgroundColors::new()) });
    out.push(ReviewItem { id: "color_text", category: "Color", title: "UIKit Text Colors", desc: "The UIKit text colors used in components such as TextField.", badge: "type", make: || WidgetNode::new(UIKitTextColors::new()) });
    out.push(ReviewItem { id: "color_fill", category: "Color", title: "UIKit Fill Colors", desc: "The UIKit fill colors that indicate the system's fill level.", badge: "type", make: || WidgetNode::new(UIKitFillColors::new()) });
    out.push(ReviewItem { id: "color_label", category: "Color", title: "UIKit Label Colors", desc: "The UIKit label colors for displaying text.", badge: "type", make: || WidgetNode::new(UIKitLabelColors::new()) });
    out.push(ReviewItem { id: "color_semantic", category: "Color", title: "Semantic Colors", desc: "The semantic colors that adapt to light and dark mode.", badge: "type", make: || WidgetNode::new(SemanticColors::new()) });
    out.push(ReviewItem { id: "color_standard", category: "Color", title: "Standard Colors", desc: "The SwiftUI standard colors.", badge: "type", make: || WidgetNode::new(StandardColors::new()) });

    // ── ControlGroup (6) ──
    out.push(ReviewItem { id: "control_group", category: "ControlGroup", title: "ControlGroup", desc: "Creates a new ControlGroup with the specified children.", badge: "initializer", make: || WidgetNode::new(ControlGroup::new()) });
    out.push(ReviewItem { id: "control_group_palette", category: "ControlGroup", title: "PaletteControlGroupStyle", desc: "A control group style that presents its content as a palette.", badge: "style", make: || WidgetNode::new(PaletteControlGroupStyle::new()) });
    out.push(ReviewItem { id: "control_group_nav", category: "ControlGroup", title: "NavigationControlGroupStyle", desc: "The navigation control group style.", badge: "style", make: || WidgetNode::new(NavigationControlGroupStyle::new()) });
    out.push(ReviewItem { id: "control_group_menu", category: "ControlGroup", title: "MenuControlGroupStyle", desc: "A control group style that presents its content as a menu.", badge: "style", make: || WidgetNode::new(MenuControlGroupStyle::new()) });
    out.push(ReviewItem { id: "control_group_compact", category: "ControlGroup", title: "CompactMenuControlGroupStyle", desc: "A control group style that presents its content as a compact menu.", badge: "style", make: || WidgetNode::new(CompactMenuControlGroupStyle::new()) });
    out.push(ReviewItem { id: "control_group_palette_api", category: "ControlGroup", title: "ControlGroup Palette via API", desc: "Palette style via ControlGroup + ControlGroupStyle::Palette.", badge: "style", make: || WidgetNode::new(ControlGroup::new().style(ControlGroupStyle::Palette)) });

    // ── Link (5) ──
    out.push(ReviewItem { id: "help_link", category: "Link", title: "HelpLink", desc: "A button with a standard appearance that opens app-specific help.", badge: "initializer", make: || WidgetNode::new(HelpLink::new()) });
    out.push(ReviewItem { id: "textfield_link", category: "Link", title: "TextFieldLink", desc: "A control that requests text input from the user when pressed.", badge: "initializer", make: || WidgetNode::new(TextFieldLink::new("Set Text")) });
    out.push(ReviewItem { id: "share_link_preview", category: "Link", title: "Custom Preview Item ShareLink", desc: "Creates an instance, with a custom label, that presents the share interface.", badge: "initializer", make: || WidgetNode::new(CustomPreviewShareLink::new("Share Cats", "Derpy Cats")) });
    out.push(ReviewItem { id: "share_link", category: "Link", title: "ShareLink", desc: "A view that controls a sharing presentation.", badge: "initializer", make: || WidgetNode::new(ShareLink::new(vec!["Share ...".into(), "Explore SwiftUI".into(), "Foo".into()])) });
    out.push(ReviewItem { id: "link_url", category: "Link", title: "Link", desc: "A control for navigating to a URL.", badge: "initializer", make: || WidgetNode::new(Link::new("Explore SwiftUI", "https://explore.swiftui.com")) });

    // ── Material (1) / Navigation (1) / Text (1) / TextInput (1) ──
    out.push(ReviewItem { id: "materials", category: "Material", title: "Materials", desc: "All materials.", badge: "type", make: || WidgetNode::new(Materials::new()) });
    out.push(ReviewItem { id: "navigation_subtitle", category: "Navigation", title: "Navigation Subtitle", desc: "Configures the view's subtitle for purposes of navigation.", badge: "modifier", make: || WidgetNode::new(NavigationSubtitle::new("Foo", "Bar")) });
    out.push(ReviewItem { id: "text_format", category: "Text", title: "Text Format", desc: "Creates a text view that displays the formatted representation of a non-string type.", badge: "initializer", make: || WidgetNode::new(TextFormat::demo()) });
    out.push(ReviewItem { id: "text_input", category: "TextInput", title: "TextInput", desc: "Single-line text input field.", badge: "initializer", make: || WidgetNode::new(TextInput::new("Enter text...")) });
}
