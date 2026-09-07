# TontooUI -- Wiki

TontooUI is a SwiftUI-inspired declarative UI layer for TontooOS, built on top of TontooUIKit. It provides pre-made elements with a clean builder API for building native GTK4 applications.

- Repository: tontoo-os/TontooLibs/TontooUI
- License: MIT
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki design system and conventions |
| Button | [Button.md](Button.md) | SwiftUI-style button: roles, styles, tint, border shapes, sizing + RenameButton/EditButton/PasteButton |
| Toolbar | [Toolbar.md](Toolbar.md) | Glass capsule toolbar: Toolbar, ToolbarItem, ToolbarSpacer, placements |
| Toggle | [Toggle.md](Toggle.md) | SwiftUI-style toggle: switch style and checkbox style |
| TextInput | [TextInput.md](TextInput.md) | Single-line text input field |
| WheelPicker | [WheelPicker.md](WheelPicker.md) | macOS/iOS scroll wheel picker |
| Picker | [Picker.md](Picker.md) | SwiftUI-style picker: wheel, segmented, palette, radio, menu, inline, tabs, navigation + sections/dividers |
| Slider | [Slider.md](Slider.md) | Spring-physics slider with white pill thumb, dark glass while pressed |
| ProgressView | [ProgressView.md](ProgressView.md) | Loading indicator: spinner/ring or linear bar (Circular/Linear, Light/Dark) |
| Sidebar | [Sidebar.md](Sidebar.md) | macOS-style sidebar with traffic lights, search, items, color/gradient background |
| ContentUnavailableView | [ContentUnavailableView.md](ContentUnavailableView.md) | Empty state with icon, title and hint message |
| Gauge | [Gauge.md](Gauge.md) | SwiftUI-style gauge: linear/circular, capacity/marker, accessory styles, tint/gradient |
| Menu | [Menu.md](Menu.md) | Liquid Glass menu (items, dividers, sections, nested) + ContextMenu with custom preview |
| ViewThatFits | [ViewThatFits.md](ViewThatFits.md) | Adaptive container that picks first child fitting available space (Vertical/Horizontal) |
| Divider | [Divider.md](Divider.md) | Separator line (Horizontal/Vertical, thickness, Light/Dark) |
| List | [List.md](List.md) | List with sections, rows, Outline/Disclosure, styles, separators, badges, swipe actions |
| GroupBox | [GroupBox.md](GroupBox.md) | Inset grouped container (label, custom background, Light/Dark) |
| ScrollView | [ScrollView.md](ScrollView.md) | Scrollable container with Hard/Soft edge effect (Light/Dark) |
| Color | [Color.md](Color.md) | Color category: opacity/gradient/variants + UIKit system palettes + semantic/standard colors |
| Text | [Text.md](Text.md) | Text category: formatted text for non-string types (initializer Text Format) |
| Material | [Material.md](Material.md) | Material category: frosted glass materials (type Materials) |
| Link | [Link.md](Link.md) | Link category: HelpLink, TextFieldLink, CustomPreview ShareLink, ShareLink, Link |
| ControlGroup | [ControlGroup.md](ControlGroup.md) | ControlGroup category: ControlGroup + Palette/Navigation/Menu/CompactMenu styles |
| Navigation | [Navigation.md](Navigation.md) | Navigation category: NavigationSubtitle modifier |
| View | [View.md](View.md) | View category: 10 modifiers — pickers, sheets, swipe, backgrounds, controlSize, glass |
| TabView | [TabView.md](TabView.md) | TabView category: 22 elements — sections, styles, customization, badges, accessories |
| Sheet | [Sheet.md](Sheet.md) | Sheet category: 12 modifiers — placement, dismiss, sizing, backgrounds, detents, presenters |
| Shapes | [Shapes.md](Shapes.md) | Shapes category: 7 shapes — circle, ellipse, capsule, rectangles, container-relative |
| Label | [Label.md](Label.md) | Label category: 3 initializers + 1 style — custom, image, system image, styles |
| LabeledContent | [LabeledContent.md](LabeledContent.md) | LabeledContent category: 3 initializers — custom, formatted, plain |

## Quick Start

```rust
use tontooui::prelude::*;

fn main() {
    let mut app = App::new("My App", 800, 600);
    app.set_root(
        VStack::new()
            .spacing(8.0)
            .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
            .child(TextInput::new("Enter text...")
                .on_change(|text| println!("Changed: {}", text)))
    );
    app.run();
}
```

See [TextInput.md](TextInput.md), [WheelPicker.md](WheelPicker.md), and [Slider.md](Slider.md) for details.

## Architecture

```
tontooui (SwiftUI-style layer)
 |
 +-- Button          (roles, styles, tint, border shapes, sizing)
 +-- Toolbar         (glass capsule bar: items, spacers, placements)
 +-- Toggle          (switch style + checkbox style)
 +-- RenameButton    (system rename action)
 +-- EditButton      (Edit/Done toggle)
 +-- PasteButton     (clipboard read)
 +-- TextInput       (placeholder, password, on_change, on_submit)
 +-- WheelPicker     (spring physics, snap-to-center, 3D fade)
 +-- Picker          (all picker styles: wheel, segmented, palette, radio, menu, inline, tabs, nav + sections/dividers)
 +-- PickerItem      (title / image / systemImage / custom label per row)
 +-- PickerSection   (section inside a picker)
  +-- Slider          (spring physics, white pill thumb, dark glass while pressed)
   +-- ProgressView    (Circular spinner/ring or Linear bar, tint, Light/Dark)
   +-- Sidebar         (traffic lights, search, selectable item list)
   +-- ContentUnavailableView (empty state with icon, title, message, button)
   +-- Gauge           (linear/circular/capacity/accessory styles, tint/gradient)
   +-- Menu            (Liquid Glass items/dividers/sections/nested) + ContextMenu (custom preview)
   +-- ViewThatFits    (Vertical/Horizontal adaptive, picks first fitting child)
   +-- Divider         (Horizontal/Vertical separator, thickness, Light/Dark)
   +-- List            (Plain/Inset/InsetGrouped/Sidebar/Elliptical/Carousel/Bordered + Outline/Disclosure + badges/separators/swipe)
   +-- GroupBox        (label + inset rounded rect, custom background, Light/Dark)
   +-- ScrollView      (vertical/horizontal, Hard vs Soft edge, Light/Dark)
   +-- Color           (opacity/gradient/variants + UIKit label/fill/separator/background/text + semantic/standard)
   +-- Text            (Text + TextFormat initializer for formatted non-string)
   +-- Material        (Materials — ultraThin … ultraThick, bar)
   +-- Link            (HelpLink, TextFieldLink, CustomPreview ShareLink, ShareLink, Link)
   +-- ControlGroup    (ControlGroup + Palette/Navigation/Menu/CompactMenu styles)
   +-- Navigation      (NavigationSubtitle modifier)
   +-- View            (MusicPicker, AppStoreOverlay, ManageSubscriptionsSheet, SwipeContainer, SwipeAction, NavSplitBg, NavContainerBg, ControlSize, BgExtension, Glass)
   +-- TabView         (TabSection, TabBarOnly, TabView, SearchRole, Grouped/Page/Vertical/Sidebar styles, BottomAccessory, Collapse, Customization, Sidebar Footer/Header/BottomBar, Actions, Badge, HiddenIndex, Value, HideOnScroll)
  +-- Sheet           (Placement, DisableDismiss, PageSize, FittedSizing, CornerRadius, PrioritizeScrolling, BgInteraction, Bg, DragIndicator, Size/Detents, Item, Boolean)
  +-- Shapes          (Circle, Ellipse, Capsule, Rectangle, RoundedRectangle, UnevenRoundedRectangle, ContainerRelativeShape)
  +-- Label           (CustomLabel, ImageLabel, SystemImageLabel, LabelStyles)
  +-- LabeledContent  (CustomLabeledContent, FormattedLabeledContent, LabeledContent)
 |
 +-- uikit (backend)
      +-- View, Widget, ViewContent
      +-- App, ViewController
      +-- Layout (VStack, HStack, ZStack)
      +-- Animation (Animator, Spring)
```

## Performance Notes

TontooUI is designed so long-running apps do not accumulate work over time:

- `Slider` and `WheelPicker` run their 60 FPS tick loop only while the physics
  are active (dragging, scrolling, wobbling or settling). As soon as the element
  settles the loop stops itself and is restarted by the next interaction, so an
  idle element does not wake the main loop or invalidate layout at 60 FPS
  forever.
- `Slider` applies its per-frame thumb styling through a single cached
  `gtk::CssProvider` that is updated in place, instead of registering a new
  CSS provider on every frame.

## Cross References

- [Button.md](Button.md) -- SwiftUI-style button element
- [Toolbar.md](Toolbar.md) -- glass capsule toolbar elements
- [Toggle.md](Toggle.md) -- switch/checkbox toggle element
- [TextInput.md](TextInput.md) -- text input element
- [WheelPicker.md](WheelPicker.md) -- scroll wheel picker
- [Picker.md](Picker.md) -- all picker styles (wheel, segmented, palette, radio, menu, inline, tabs, navigation)
- [Slider.md](Slider.md) -- spring-physics slider
- [ProgressView.md](ProgressView.md) -- loading indicator (Circular/Linear)
- [Sidebar.md](Sidebar.md) -- sidebar with traffic lights and item list
- [ContentUnavailableView.md](ContentUnavailableView.md) -- empty state with icon, title, message, button
- [Gauge.md](Gauge.md) -- gauge with all SwiftUI styles and tint support
- [Menu.md](Menu.md) -- Liquid Glass menu + context menu
- [ViewThatFits.md](ViewThatFits.md) -- adaptive fitting container
- [Divider.md](Divider.md) -- separator line
- [List.md](List.md) -- list with all styles and 32 modifiers
- [GroupBox.md](GroupBox.md) -- inset grouped container
- [ScrollView.md](ScrollView.md) -- scrollable with edge effect
- [Color.md](Color.md) -- color system (opacity/gradient/variants + UIKit palettes + semantic/standard)
- [Text.md](Text.md) -- text and formatted text initializer
- [Material.md](Material.md) -- frosted glass materials
- [Link.md](Link.md) -- link & sharing controls
- [ControlGroup.md](ControlGroup.md) -- control groups and styles
- [Navigation.md](Navigation.md) -- navigation subtitle modifier
- [View.md](View.md) -- view modifiers (music picker, sheets, swipe, backgrounds, glass)
- [TabView.md](TabView.md) -- tab view containers, styles, modifiers (22 elements)
- [Sheet.md](Sheet.md) -- sheet presentations and modifiers (12 elements)
- [Shapes.md](Shapes.md) -- shape views (7 elements)
- [Label.md](Label.md) -- label initializers and styles (4 elements)
- [LabeledContent.md](LabeledContent.md) -- labeled informational views (3 elements)