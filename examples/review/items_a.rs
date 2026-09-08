//! Review items: Button, Toggle, Slider, ProgressView, ContentUnavailableView,
//! GroupBox, ScrollView, ViewThatFits, Toolbar.

use super::ReviewItem;
use tontooui::prelude::*;

pub fn items(out: &mut Vec<ReviewItem>) {
    // ── Button (11) ──
    out.push(ReviewItem { id: "button_role_defaults", category: "Button", title: "Role Button With Default Label", desc: "Creates a button that displays a default label.", badge: "initializer", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("").role(ButtonRole::Close).icon("xmark").style(ButtonStyle::Plain))
            .child(Button::new("").role(ButtonRole::Close).icon("xmark").style(ButtonStyle::Glass))
            .child(Button::new("").role(ButtonRole::Confirm).icon("checkmark").style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::Circle).height(36.0))
            .child(Button::new("").role(ButtonRole::Destructive).icon("xmark.bin").style(ButtonStyle::Glass)),
    ) });
    out.push(ReviewItem { id: "button_initializers", category: "Button", title: "Button", desc: "Three ways to initialize a button.", badge: "initializer", make: || WidgetNode::new(
        HStack::new().spacing(10.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain))
            .child(Button::new("Tap Me").icon("hand.tap").style(ButtonStyle::Plain))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass))
            .child(Button::new("Tap Me").icon("hand.tap").style(ButtonStyle::Glass)),
    ) });
    out.push(ReviewItem { id: "button_glass_styles", category: "Button", title: "Glass Button Styles", desc: "A button style that applies glass border artwork based on the button's context.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Glass))
            .child(Button::new("Tinted Tap Me").style(ButtonStyle::Glass).tint(Color::from_rgb(255, 69, 58)))
            .child(Button::new("Tap Me").style(ButtonStyle::GlassProminent))
            .child(Button::new("Tap Me").style(ButtonStyle::GlassProminent).tint(Color::from_rgb(255, 69, 58))),
    ) });
    out.push(ReviewItem { id: "button_tint", category: "Button", title: "Button Tint", desc: "Sets the button's tint color.", badge: "modifier", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(Color::from_rgb(10, 132, 255)))
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(Color::from_rgb(255, 69, 58)))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass).tint(Color::from_rgb(10, 132, 255)))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).tint(Color::from_rgb(10, 132, 255))),
    ) });
    out.push(ReviewItem { id: "button_roles", category: "Button", title: "Button Roles", desc: "A value that describes the purpose of a button.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain))
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).role(ButtonRole::Destructive))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass).role(ButtonRole::Cancel))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).role(ButtonRole::Destructive)),
    ) });
    out.push(ReviewItem { id: "button_styles", category: "Button", title: "Button Styles", desc: "A type that applies standard interaction behavior and a custom appearance.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain))
            .child(Button::new("Tap Me").style(ButtonStyle::Bordered))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass)),
    ) });
    out.push(ReviewItem { id: "button_sizing", category: "Button", title: "Button Sizing", desc: "The preferred sizing behavior of buttons in the view hierarchy.", badge: "modifier", make: || WidgetNode::new(
        VStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).sizing(ButtonSizing::Fitted))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).sizing(ButtonSizing::Flexible)),
    ) });
    out.push(ReviewItem { id: "rename_button", category: "Button", title: "RenameButton", desc: "A button that triggers a standard rename action.", badge: "initializer", make: || WidgetNode::new(
        HStack::new().spacing(10.0)
            .child(Text::new("foo").font_size(15.0))
            .child(RenameButton::new()),
    ) });
    out.push(ReviewItem { id: "edit_button", category: "Button", title: "EditButton", desc: "A button that toggles the edit mode environment value.", badge: "initializer", make: || WidgetNode::new(
        EditButton::new(),
    ) });
    out.push(ReviewItem { id: "paste_button", category: "Button", title: "PasteButton", desc: "A system button that reads items from the pasteboard.", badge: "initializer", make: || WidgetNode::new(
        PasteButton::new(),
    ) });
    out.push(ReviewItem { id: "button_border_shape", category: "Button", title: "Button Border Shape", desc: "A shape used to draw a button's border.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::Capsule))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::RoundedRectangle(8.0)))
            .child(Button::new("").style(ButtonStyle::BorderedProminent).icon("hand.tap").border_shape(ButtonBorderShape::Circle).height(36.0)),
    ) });

    // ── Toggle (2) ──
    out.push(ReviewItem { id: "switch_toggle_style", category: "Toggle", title: "SwitchToggleStyle", desc: "A toggle style that displays a leading label and a trailing switch.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(10.0)
            .child(Toggle::new("Foo1").value(true).width(150.0))
            .child(Toggle::new("Foo2").width(150.0))
            .child(Toggle::new("Wi-Fi").value(true).width(150.0)),
    ) });
    out.push(ReviewItem { id: "checkbox_toggle_style", category: "Toggle", title: "CheckboxToggleStyle", desc: "A toggle style that displays a checkbox followed by its label.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(10.0)
            .child(Toggle::new("Foo1").style(ToggleStyle::Checkbox).width(150.0))
            .child(Toggle::new("Foo2").style(ToggleStyle::Checkbox).value(true).width(150.0))
            .child(Toggle::new("Sync iCloud").style(ToggleStyle::Checkbox).width(150.0)),
    ) });

    // ── Slider (6) ──
    out.push(ReviewItem { id: "slider_tick_foreach", category: "Slider", title: "SliderTickContentForEach", desc: "A type of slider content that creates content by iterating over a collection.", badge: "initializer", make: || WidgetNode::new(
        Slider::new(0.0, 100.0).value(55.0).ticks(10).width(220.0),
    ) });
    out.push(ReviewItem { id: "slider_custom_ticks", category: "Slider", title: "Custom Slider Ticks", desc: "Creates a slider to select a value from a given range, subject to a step increment.", badge: "modifier", make: || WidgetNode::new(
        Slider::new(0.0, 100.0).value(100.0).step(1.0).ticks(20).width(220.0),
    ) });
    out.push(ReviewItem { id: "slider_min_max", category: "Slider", title: "Stepped Min Max Label Slider", desc: "Creates a slider to select a value from a given range, subject to a step increment.", badge: "modifier", make: || WidgetNode::new(
        Slider::new(0.0, 100.0).value(30.0).step(1.0).ticks(10).min_label("0").max_label("100").width(190.0),
    ) });
    out.push(ReviewItem { id: "slider_stepped", category: "Slider", title: "Stepped Slider", desc: "Creates a slider to select a value from a given range, subject to a step increment.", badge: "initializer", make: || WidgetNode::new(
        Slider::new(0.0, 100.0).value(45.0).step(1.0).ticks(10).width(220.0),
    ) });
    out.push(ReviewItem { id: "slider_plain", category: "Slider", title: "Slider", desc: "Creates a slider to select a value from a given range.", badge: "initializer", make: || WidgetNode::new(
        Slider::new(0.0, 100.0).value(35.0).width(220.0),
    ) });
    out.push(ReviewItem { id: "slider_color", category: "Slider", title: "Slider Color", desc: "Customizing the color of the slider.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(Slider::new(0.0, 100.0).value(50.0).accent_color(Color::from_rgb(255, 69, 58)).width(220.0))
            .child(Slider::new(0.0, 100.0).value(40.0).accent_color(Color::from_rgb(255, 69, 58)).ticks(10).min_label("0").max_label("100").width(190.0)),
    ) });

    // ── ProgressView (4) ──
    out.push(ReviewItem { id: "progress_initializers", category: "ProgressView", title: "ProgressView", desc: "The different ProgressView initializers.", badge: "initializer", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(ProgressView::new().value(0.42).label("Foo").progress_view_style(ProgressViewStyle::Linear).width(180.0))
            .child(ProgressView::new().value(0.65).progress_view_style(ProgressViewStyle::Linear).width(180.0))
            .child(ProgressView::new().value(0.42).label("Foo").sub_label("bar").progress_view_style(ProgressViewStyle::Linear).width(180.0)),
    ) });
    out.push(ReviewItem { id: "progress_colors", category: "ProgressView", title: "ProgressView Colors", desc: "Customizing the colors of the ProgressView.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(ProgressView::new().value(0.5).label("Foo").sub_label("Foo").tint(Color::from_rgb(255, 69, 58)).progress_view_style(ProgressViewStyle::Circular).size(36.0))
            .child(ProgressView::new().value(0.7).label("Foo").sub_label("bar").tint(Color::from_rgb(255, 69, 58)).progress_view_style(ProgressViewStyle::Linear).width(180.0)),
    ) });
    out.push(ReviewItem { id: "progress_circular", category: "ProgressView", title: "CircularProgressViewStyle", desc: "The style of a progress view that uses a circular gauge.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(12.0)
            .child(ProgressView::new().progress_view_style(ProgressViewStyle::Circular).size(36.0))
            .child(ProgressView::new().label("Foo").progress_view_style(ProgressViewStyle::Circular).size(36.0))
            .child(ProgressView::new().label("Foo").sub_label("bar").value(0.4).progress_view_style(ProgressViewStyle::Circular).size(36.0)),
    ) });
    out.push(ReviewItem { id: "progress_linear", category: "ProgressView", title: "LinearProgressViewStyle", desc: "A progress view that visually indicates its progress using a horizontal bar.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(12.0)
            .child(ProgressView::new().label("Foo").progress_view_style(ProgressViewStyle::Linear).width(180.0))
            .child(ProgressView::new().value(0.55).label("Foo").sub_label("bar").progress_view_style(ProgressViewStyle::Linear).width(180.0)),
    ) });

    // ── ContentUnavailableView (3) ──
    out.push(ReviewItem { id: "cuv_unavailable", category: "ContentUnavailableView", title: "Content Unavailable", desc: "A placeholder for empty results with optional button.", badge: "initializer", make: || WidgetNode::new(
        ContentUnavailableView::new().title("No Mail").message("New mails you receive will appear here.").icon("tray").button("Switch Account").width(220.0).height(170.0),
    ) });
    out.push(ReviewItem { id: "cuv_search_text", category: "ContentUnavailableView", title: "Content Unavailable Search Text", desc: "A pre-built placeholder for empty search results with search text.", badge: "type", make: || WidgetNode::new(
        ContentUnavailableView::new().query("foo").message("Check the spelling or try a new search.").icon("magnifyingglass").width(220.0).height(170.0),
    ) });
    out.push(ReviewItem { id: "cuv_search", category: "ContentUnavailableView", title: "Content Unavailable Search", desc: "A pre-built placeholder for empty search results.", badge: "type", make: || WidgetNode::new(
        ContentUnavailableView::new().title("No Results").query("").message("Check the spelling or try a new search.").icon("magnifyingglass").width(220.0).height(170.0),
    ) });

    // ── GroupBox (3) ──
    out.push(ReviewItem { id: "groupbox_label", category: "GroupBox", title: "GroupBox With Label", desc: "A GroupBox with Label.", badge: "initializer", make: || WidgetNode::new(
        GroupBox::with_label("Hello World").child(Text::new("Lorem ipsum dolor sit amet.").font_size(10.0).max_width(200.0)).width(260.0),
    ) });
    out.push(ReviewItem { id: "groupbox_plain", category: "GroupBox", title: "GroupBox", desc: "A simple GroupBox.", badge: "initializer", make: || WidgetNode::new(
        GroupBox::new().child(Text::new("Lorem ipsum dolor sit amet.").font_size(10.0).max_width(200.0)).width(260.0),
    ) });
    out.push(ReviewItem { id: "groupbox_bg", category: "GroupBox", title: "GroupBox Background", desc: "Set a custom GroupBox background.", badge: "modifier", make: || WidgetNode::new(
        GroupBox::new().background(Color::from_hex("#0A84FF").unwrap()).child(Text::new("Lorem ipsum dolor sit amet.").font_size(10.0).color(Color::WHITE).max_width(200.0)).width(260.0),
    ) });

    // ── ScrollView (1) ──
    out.push(ReviewItem { id: "scroll_hard_edge", category: "ScrollView", title: "HardScrollEdgeEffect", desc: "A scroll edge effect with a hard cutoff and dividing line.", badge: "style", make: || WidgetNode::new(
        tontooui::ScrollView::new().content(
            VStack::new().spacing(6.0)
                .child(Text::new("Item 5").font_size(12.0))
                .child(Divider::horizontal().length(200.0))
                .child(Text::new("Item 6").font_size(12.0))
                .child(Divider::horizontal().length(200.0))
                .child(Text::new("Item 7").font_size(12.0))
        ).hard_edge().vertical(true),
    ) });

    // ── ViewThatFits (2) ──
    out.push(ReviewItem { id: "vtf_vertical", category: "ViewThatFits", title: "ViewThatFits Vertical", desc: "A view that adapts to the available vertical space by providing the first child that fits.", badge: "initializer", make: || WidgetNode::new(
        ViewThatFits::vertical()
            .child(Text::new("Size w300 h30 — first fitting wins").font_size(11.0))
            .child(Text::new("Fallback").font_size(11.0)),
    ) });
    out.push(ReviewItem { id: "vtf_both", category: "ViewThatFits", title: "ViewThatFits", desc: "A view that adapts to the available space by providing the first child view that fits.", badge: "initializer", make: || WidgetNode::new(
        ViewThatFits::new()
            .child(Text::new("Available width 300").font_size(11.0))
            .child(Text::new("Fallback").font_size(11.0)),
    ) });

    // ── Toolbar (3) ──
    out.push(ReviewItem { id: "toolbar_spacer", category: "Toolbar", title: "ToolbarSpacer", desc: "A standard space item in toolbars.", badge: "initializer", make: || WidgetNode::new(
        Toolbar::new()
            .item(ToolbarItem::new("chevron.up"))
            .item(ToolbarItem::new("chevron.down"))
            .spacer(ToolbarSpacer::fixed())
            .item(ToolbarItem::new("ellipsis")),
    ) });
    out.push(ReviewItem { id: "toolbar_shared_bg", category: "Toolbar", title: "ToolbarItem Shared Background", desc: "Controls the visibility of the glass background effect on items in the toolbar.", badge: "modifier", make: || WidgetNode::new(
        Toolbar::new()
            .item(ToolbarItem::new("person.2.circle"))
            .spacer(ToolbarSpacer::fixed())
            .item(ToolbarItem::new("face.smiling"))
            .spacer(ToolbarSpacer::fixed())
            .item(ToolbarItem::new("heart").shared_background(false)),
    ) });
    out.push(ReviewItem { id: "toolbar_title_placement", category: "Toolbar", title: "Title Toolbar Item Placement", desc: "Places content in the title area.", badge: "modifier", make: || WidgetNode::new(
        Toolbar::new()
            .item(ToolbarItem::new("").placement(ToolbarItemPlacement::Principal).content(
                VStack::new().spacing(0.0)
                    .child(Text::new("2").font_size(13.0))
                    .child(Text::new("3").font_size(13.0))
                    .child(Text::new("1").font_size(13.0))
                    .child(Text::new("4").font_size(13.0))
            ))
            .item(ToolbarItem::new("plus"))
            .item(ToolbarItem::new("gearshape")),
    ) });
}
