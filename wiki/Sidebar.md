# Sidebar

A macOS-style sidebar with traffic lights, search bar, and scrollable item list.
Rounded corners and an optional outer glow match the system Settings appearance.

## Constructor

```rust
pub fn new() -> Self
```

Creates a 260x600 sidebar with dark background, white-tinted border, and search bar.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `item` | `item(self, label, icon) -> Self` | Append a row (label + SF Symbol icon name) |
| `selected` | `selected(self, index: usize) -> Self` | Highlight an item by index |
| `search_placeholder` | `search_placeholder(self, text) -> Self` | Placeholder for the search bar |
| `no_search` | `no_search(self) -> Self` | Remove the search bar |
| `background_color` | `background_color(self, c: Color) -> Self` | Sidebar fill (Dark: #1d1d1f, Light: #ececec) |
| `border_color` | `border_color(self, c: Color) -> Self` | Outer border color |
| `glow_color` | `glow_color(self, c: Color) -> Self` | Outer glow (box-shadow) color |
| `selected_color` | `selected_color(self, c: Color) -> Self` | Background of the selected row |
| `icons_dir` | `icons_dir(self, dir) -> Self` | Path to icon PNG directory |
| `width` | `width(self, w: f32) -> Self` | Sidebar width |
| `height` | `height(self, h: f32) -> Self` | Sidebar height |
| `on_select` | `on_select(self, handler) -> Self` | Callback when an item is clicked |

## Getters

| Method | Returns | Description |
|---|---|---|
| `selected_index` | `usize` | Currently selected item index |

## Layout

```
+-----------------------------+
|  (red) (yellow) (green)     |  <-- traffic lights
| [ Search...           ]     |  <-- search bar
|---------------------------- |
|  [icon] Wi-Fi               |
|  [icon] Bluetooth           |
|  [icon] Network             |
|  ...                        |
+-----------------------------+
```

## Usage / Example

```rust
use tontooui::prelude::*;

let sidebar = Sidebar::new()
    .item("Wi-Fi", "wifi.circle.fill")
    .item("Bluetooth", "antenna.radiowaves.left.and.right")
    .item("Network", "globe")
    .item("VPN", "lock.shield")
    .item("Battery", "battery.100")
    .selected(0)
    .background_color(Color::from_rgb(30, 30, 32))
    .on_select(|i| println!("Selected: {}", i));
```

## Features

- Traffic lights drawn as CSS circles (red/yellow/green, 12px)
- Search bar with frosted translucent background
- Scrollable item list with icon PNGs + SF Pro Display labels
- Selection highlight with customizable color
- Rounded corners (12px) with white glow border
- Customizable background for Dark/Light mode adaptation

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- another interactive element
- [TextInput.md](TextInput.md) -- text input element