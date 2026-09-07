# Picker

Picker is the SwiftUI-style picker element for TontooUI. It covers the entire SwiftUI `Picker` gallery — every `PickerStyle` from the Xcode 26 preview catalog plus `PickerSection`, `PickerDivider`, and the `wheelPickerItemHeight` / `horizontalRadioGroupLayout` modifiers. The existing `WheelPicker` drum remains available as a focused physics widget; `Picker` with `PickerStyle::Wheel` reuses its visual language in a unified API.

Appearance follows the TontooOS design system: `SF Pro Display` for every label, dark background `#1d1d1d` and light background `#ececec`, accent tint (default Tontoo blue `#0a84ff`, configurable) for selection highlights. The color scheme adapts automatically via `uikit::app::ColorScheme::detect_system()` or an explicit `color_scheme()` override.

## Types

### `PickerStyle`

| Variant | SwiftUI name | Description |
|---|---|---|
| `Automatic` | *(default)* | Automatic menu style (platform default). |
| `Wheel` | `WheelPickerStyle` | Scrollable wheel drum with center highlight. |
| `Segmented` | `SegmentedPickerStyle` | Segmented control pill. |
| `Palette` | `PalettePickerStyle` | Row of compact capsule elements. |
| `RadioGroup` | `RadioGroupPickerStyle` | Group of radio buttons (vertical or horizontal). |
| `NavigationLink` | `NavigationLinkPickerStyle` | Navigation link row presenting options in a list. |
| `Menu` | `MenuPickerStyle` | Menu button that shows options in a popover on press. |
| `Inline` | `InlinePickerStyle` | Options displayed inline with other views in the container. |
| `Tabs` | `TabsPickerStyle` | Options presented as segmented tabs. |

### `PickerItem`

One selectable row. Mirrors SwiftUI where each row can be `Text`, `Image`, `systemImage`, or a custom label.

```rust
pub struct PickerItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub image: Option<String>,
    pub system_image: Option<String>,
    pub is_custom: bool,
    pub tag: Option<String>,
}
```

| Method | Description |
|---|---|
| `PickerItem::new(title)` | Plain title row (`Text("Foo")`). |
| `.with_subtitle(s)` | Trailing count/detail text (e.g. `"1"`). |
| `.with_image(name)` | Bundled image (`Image("bar")`). |
| `.with_system_image(name)` | SF Symbol (`Image(systemName: "star.fill")`). |
| `.custom()` | Mark as custom emphasized label. |
| `.tag(t)` | Attach a SwiftUI `.tag()` value. |

### `PickerSection`

A SwiftUI `Section` inside a `Picker`.

```rust
pub struct PickerSection {
    pub title: Option<String>,
    pub items: Vec<PickerItem>,
    pub has_divider: bool,
}
```

| Method | Description |
|---|---|
| `PickerSection::new(title)` | Section with header title. |
| `PickerSection::untitled()` | Section without header. |
| `.item(item)` | Add one `PickerItem`. |
| `.items(iter)` | Add many `PickerItem`s. |
| `.divider()` | Mark section as followed by a divider. |

## Constructor

```rust
pub fn new(label: impl Into<String>) -> Self
```

Creates a picker with the given label (shown on the leading side or as the card title, e.g. `Picker("Foo", selection: 0)` in SwiftUI).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `titles` | `titles<I,S>(self, iter: I) -> Self` | Set items from string titles (shorthand). |
| `items` | `items<I>(self, iter: I) -> Self` | Set items from `PickerItem` iterator. |
| `item` | `item(self, item: PickerItem) -> Self` | Add one `PickerItem`. |
| `title_item` | `title_item(self, title) -> Self` | Add plain title item. |
| `system_image_item` | `system_image_item(self, title, system_image) -> Self` | Add title + SF Symbol. |
| `image_item` | `image_item(self, title, image) -> Self` | Add title + bundled image. |
| `custom_label_item` | `custom_label_item(self, title) -> Self` | Add custom emphasized label item. |
| `section` | `section(self, section: PickerSection) -> Self` | Add a `PickerSection`. |
| `section_with` | `section_with(self, title, items) -> Self` | Convenience: section from string titles. |
| `divider` | `divider(self) -> Self` | Insert divider after last item/section (`Divider` inside `Picker`). |
| `selected_index` | `selected_index(self, idx: usize) -> Self` | Initial selection by index (clamped). |
| `selected` | `selected(self, value) -> Self` | Initial selection by title value. |
| `style` | `style(self, style: PickerStyle) -> Self` | Picker style (`.pickerStyle()`). |
| `segmented` | `segmented(self) -> Self` | Alias for `style(Segmented)`. |
| `wheel` | `wheel(self) -> Self` | Alias for `style(Wheel)`. |
| `radio_group` | `radio_group(self) -> Self` | Alias for `style(RadioGroup)`. |
| `palette` | `palette(self) -> Self` | Alias for `style(Palette)`. |
| `menu` | `menu(self) -> Self` | Alias for `style(Menu)`. |
| `inline_picker` | `inline_picker(self) -> Self` | Alias for `style(Inline)`. |
| `tabs` | `tabs(self) -> Self` | Alias for `style(Tabs)`. |
| `navigation_link` | `navigation_link(self) -> Self` | Alias for `style(NavigationLink)`. |
| `wheel_item_height` | `wheel_item_height(self, h: f32) -> Self` | Drum row height modifier (default `40.0`). |
| `horizontal_radio_group` | `horizontal_radio_group(self, bool) -> Self` | Horizontal layout for radio group modifier. |
| `multiple_sources` | `multiple_sources(self, bool) -> Self` | Mark as bound to multiple sources. |
| `on_change_multiple` | `on_change_multiple(self, fn) -> Self` | Extra callback per source (multi-source demo). |
| `custom_value_label` | `custom_value_label(self, key, fn) -> Self` | Custom value label from `LocalizedStringKey` (`Custom Value Label Picker`). |
| `color_scheme` | `color_scheme(self, ColorScheme) -> Self` | Force dark/light (default: auto). |
| `accent_color` | `accent_color(self, Color) -> Self` | Selection highlight color. |
| `frame` | `frame(self, width, height) -> Self` | Set size. |
| `width` | `width(self, w: f32) -> Self` | Set width only. |
| `on_change` | `on_change(self, fn(String)) -> Self` | Callback on selection change. |

## Accessor Methods

| Method | Return | Description |
|---|---|---|
| `flat_items()` | `Vec<PickerItem>` | All items flattened (sections expanded). |
| `selected_value()` | `String` | Title of the selected item. |
| `selected_item()` | `Option<PickerItem>` | Selected item clone if any. |
| `display_label()` | `String` | Display label (applies custom value label generator when set). |
| `picker_style()` | `PickerStyle` | Current style variant. |
| `to_view()` | `View` | Wrap in a `View` (`View::new(picker).with_frame(...)`). |

## Gallery Mapping

Each gallery card in the screenshot maps to a `Picker` construction:

| Gallery card | Construction |
|---|---|
| Custom Value Label Picker | `Picker::new("Foo").titles([...]).custom_value_label("Current", \|v\| format!("Current: {v}"))` |
| Multiple Sources Picker | `Picker::new("Size").titles([...]).multiple_sources(true).on_change_multiple(...)` |
| Picker (title/image/systemImage/custom) | `Picker::new("Picker").item(PickerItem::new("Foo")).item(PickerItem::new("Bar").with_image("bar.png")).item(PickerItem::new("Foo").with_system_image("star.fill")).item(PickerItem::new("Foo").custom())` |
| `TabsPickerStyle` | `.style(PickerStyle::Tabs)` |
| `WheelPickerStyle` | `.style(PickerStyle::Wheel)` |
| `SegmentedPickerStyle` | `.style(PickerStyle::Segmented)` |
| `PalettePickerStyle` | `.style(PickerStyle::Palette)` |
| `RadioGroupPickerStyle` | `.style(PickerStyle::RadioGroup)` |
| `NavigationLinkPickerStyle` | `.style(PickerStyle::NavigationLink)` |
| `MenuPickerStyle` | `.style(PickerStyle::Menu)` |
| `InlinePickerStyle` | `.style(PickerStyle::Inline)` |
| Wheel Picker Item Height | `.wheel_item_height(52.0)` |
| Horizontal Radio Group Layout | `.horizontal_radio_group(true)` |
| Picker Section | `.section(PickerSection::new("Section").items(...))` |
| Picker Divider | `.divider()` |

## Rendering Model

`Picker` implements `ViewContent` and `Widget`. Rendering dispatches by `PickerStyle`:

- **Menu/Automatic** — `gtk::MenuButton` + `gtk::Popover` with rows, section headers, dividers and checkmarks. Label shows `display_label()` + selection value.
- **Segmented** — `gtk::ToggleButton` row in a pill container; only one active, group behaviour enforced via `toggled` handlers.
- **Palette** — row of `gtk::Button` chips; selected chip gets accent fill and border.
- **RadioGroup** — `gtk::CheckButton` radio group; horizontal orientation when `horizontal_radio_group(true)`.
- **NavigationLink** — card row with title + value + chevron, plus expandable `ListBox`-like list below; clicking a row updates selection.
- **Inline** — embedded list card (rounded `#2c2c2e`/`#ffffff` background) with inline selectable rows and section titles.
- **Tabs** — segmented tab bar with tab buttons + placeholder content showing selection.
- **Wheel** — `gtk::Overlay` drum (`#2c2c2e`/`#ffffff` via `palette_for`) with absolutely-positioned labels using the `wheel_item_height` for `y` offsets; Pango fade/scale per distance-from-center (same curve as `WheelPicker`), click-to-select.

All styles use `SF Pro Display`, `ColorScheme`-aware palettes (`#1d1d1d` dark, `#ececec` light), and fire `on_change` (and any `on_change_multiple` handlers) only via GTK click/toggled callbacks.

## Usage / Example

```rust
use tontooui::prelude::*;

// Segmented control
let seg = Picker::new("Options")
    .titles(["1", "Bar", "2", "3"])
    .selected_index(0)
    .style(PickerStyle::Segmented)
    .on_change(|v| println!("picked {v}"));

// Custom value label (LocalizedStringKey-style)
let custom = Picker::new("Foo")
    .titles(["1", "2", "3"])
    .custom_value_label("Current", |v| format!("Current: {v}"));

// Mixed asset rows inside a section with a divider
let rich = Picker::new("Picker")
    .section(
        PickerSection::new("Greetings")
            .item(PickerItem::new("Foo").with_subtitle("1"))
            .item(PickerItem::new("Bar").with_system_image("star.fill"))
            .item(PickerItem::new("Custom").custom())
            .divider(),
    )
    .style(PickerStyle::Menu);

// Wheel with custom item height
let wheel = Picker::new("Wheel")
    .titles(["1", "Bar", "2"])
    .style(PickerStyle::Wheel)
    .wheel_item_height(52.0)
    .frame(260.0, 176.0);

let view = View::new(seg).with_frame(0.0, 0.0, 320.0, 44.0);
```

## Demo

`examples/pickers.rs` renders the full 14-card gallery in a 4-per-row grid, each card showing the style preview above its title and description. Run:

```bash
cargo run --example pickers
```

Also see `examples/demo.rs` which embeds a `WheelPicker` (the physics drum) alongside the new `Picker`.

## Cross References

- [WheelPicker.md](WheelPicker.md) — the standalone spring-physics wheel drum
- [MAIN.md](MAIN.md) — library overview
- [Slider.md](Slider.md) — another spring-physics element
- [Toggle.md](Toggle.md) — switch/checkbox toggles (similar color-scheme handling)
