# Sidebar

A macOS-style sidebar with traffic lights, search bar, and scrollable item list.
Clicking a row moves the Apple-blue selection immediately and fires `on_select`.
Use [`TabView.md`](TabView.md) when the selection should also swap the detail
content on the right (SwiftUI `.sidebarAdaptable` behavior).

## Constructor

```rust
pub fn new() -> Self
```

Creates a 220px wide sidebar. The background follows the color scheme when no
explicit color is set (dark `#2C2C2E` / light `#F5F5F7`).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `item` | `item(self, label, icon) -> Self` | Append a row (label + `SidebarIcon`, or PNG path without `coreicon`) |
| `section` | `section(self, title) -> Self` | Insert a section header above the following items |
| `selected` | `selected(self, index: usize) -> Self` | Highlight an item by index (headers are not counted) |
| `search_placeholder` | `search_placeholder(self, text) -> Self` | Placeholder for the search bar (`lang` key `sidebar.search_placeholder`) |
| `no_search` | `no_search(self) -> Self` | Remove the search bar |
| `background_color` | `background_color(self, c: Color) -> Self` | Sidebar fill (default: scheme-aware Apple fill) |
| `background_gradient` | `background_gradient(self, g: Gradient) -> Self` | Sidebar fill as a CoreIcon gradient (switches to the framed card look) |
| `border_color` | `border_color(self, c: Color) -> Self` | Outer border color (switches to the framed card look) |
| `glow_color` | `glow_color(self, c: Color) -> Self` | Outer glow (box-shadow) color (switches to the framed card look) |
| `selected_color` | `selected_color(self, c: Color) -> Self` | Background of the selected row (default Apple blue `#0A84FF`) |
| `font_size` | `font_size(self, size: f32) -> Self` | Label font size in px (default `13.5`) |
| `bold` | `bold(self, bold: bool) -> Self` | Bold labels (default `true`) |
| `text_color` | `text_color(self, c: Color) -> Self` | Label text color (default: scheme-aware, near-white in dark mode; selected rows stay white) |
| `icon_size` | `icon_size(self, size: f32) -> Self` | Icon size in px (default `24`) |
| `side_margins` | `side_margins(self, m: f32) -> Self` | Row inset from the left/right edges in px (default `12`) |
| `icon_spacing` | `icon_spacing(self, s: f32) -> Self` | Gap between icon and label in px (default `8`) |
| `row_spacing` | `row_spacing(self, s: f32) -> Self` | Vertical gap between rows in px (default `2`) |
| `row_height` | `row_height(self, h: f32) -> Self` | Row height in px (default `30`) |
| `selection_overhang` | `selection_overhang(self, o: f32) -> Self` | How far the selected blue pill extends past the row inset on the left in px (default `2`, clamped to `side_margins`; icon stays aligned) |
| `width` | `width(self, w: f32) -> Self` | Sidebar width |
| `height` | `height(self, h: f32) -> Self` | Sidebar height |
| `on_select` | `on_select(self, handler) -> Self` | Callback with the item index when a row is clicked |

## Getters

| Method | Returns | Description |
|---|---|---|
| `selected_index` | `usize` | Currently selected item index |
| `item_count` | `usize` | Number of selectable items (section headers are not counted) |

## Layout

```
+-----------------------------+
|  (red) (yellow) (green)     |  <-- traffic lights
| [ Search...           ]     |  <-- search bar (live filter)
|---------------------------- |
|  [icon] Wi-Fi               |  <-- selected: blue fill, white label
|  [icon] Bluetooth           |
|  Foo                        |  <-- section header
|  [icon] 1                   |
+-----------------------------+
```

## Selection Behavior

Clicking a row (left mouse button, `released` event) updates the highlight
immediately by toggling the `sb-sel` / `sb-lbl-sel` CSS classes on the rows —
no re-render is needed — and then calls the `on_select` handler with the item
index. Hovering a row shows a subtle fill (`sb-hover`). The selected icon has
no badge; like Apple, selection is purely the blue row background.

## Search Filter

Typing in the search field hides non-matching rows (case-insensitive substring
match on the label). A section header hides when none of its items match and
reappears when the query is cleared.

## Apple-Plain vs Framed Card

By default the sidebar is Apple-plain: square corners, no margin, no glow, and
a 1px trailing separator (dark `rgba(255,255,255,0.10)` /
light `rgba(0,0,0,0.12)`). Setting `background_gradient`, `border_color`, or
`glow_color` switches to the legacy framed card look (12px radius, 6px margin,
2px border, glow) with auto-adaptive frame colors.

## SidebarIcon

| Constructor | Description |
|---|---|
| `SidebarIcon::sf(symbol, color)` | SF Symbol with a solid background color and white foreground |
| `SidebarIcon::sf_gradient(symbol, gradient)` | SF Symbol with a gradient background and white foreground |
| `SidebarIcon::file(path)` | Finished artwork (e.g. an app icon PNG) used as-is: center-cropped to a square and Lanczos-downscaled to 3x the display size, no tile, no recolor |

```rust
use tontooui::prelude::*;

let chat = Sidebar::new()
    .item("Chat", SidebarIcon::file("../CoreIcon/examples/app_icon_demo/images_dark.png"));
```

Missing files render as an empty icon slot. Without the `coreicon`
feature, `item` takes a PNG path string directly instead of `SidebarIcon`.

## Usage / Example

```rust
use tontooui::prelude::*;

let sidebar = Sidebar::new()
    .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
    .item("Bluetooth", SidebarIcon::sf("antenna.radiowaves.left.and.right", Color::from_rgb(0, 122, 255)))
    .section("General")
    .item("About", SidebarIcon::sf("info.circle", Color::from_rgb(142, 142, 147)))
    .selected(0)
    .background_color(Color::from_rgb(30, 30, 32))
    .on_select(|i| println!("Selected: {}", i));
```

With a gradient background (requires the `coreicon` feature, default):

```rust
use tontooui::prelude::*;

let sidebar = Sidebar::new()
    .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
    .selected(0)
    .background_gradient(coreicon::Gradient::linear_two(
        coreicon::Color::new(0.09, 0.09, 0.11, 1.0),
        coreicon::Color::new(0.04, 0.10, 0.22, 1.0),
    ))
    .on_select(|i| println!("Selected: {}", i));
```

The gradient direction maps from `GradientDirection` to a CSS
`linear-gradient(...)` (`CenterRadial` renders a `radial-gradient` instead).
`background_gradient` overrides `background_color`; the solid color stays as a
fallback.

## Playground

Try every knob live (bold, text/icon size, text color, margins, gaps, row
height) without rebuilding:

```bash
cargo run --example sidebar_playground
```

## Features

- Traffic lights drawn as CSS circles (red/yellow/green, 12px)
- Search bar styled like `TextInput` with a blue focus accent, scheme-aware
  (dark `#3a3a3c` fill / light `#ffffff` fill), 14px radius, no border
- Scrollable item list with icon PNGs + SF Pro Display labels. Icons are
  served as 3x supersampled files (Lanczos3 downscale of the 1024px master,
  e.g. 84px for a 28px row) so thin glyph strokes stay crisp instead of
  going soft in GTK's single-step downscale. Before downscaling, glyph
  strokes are thickened by ~1px (alpha + bright-stroke dilation), which keeps
  them solid white instead of dissolving into gray at small sizes. Labels are
  left-aligned, expand to the row width and truncate with an end ellipsis
  (`EllipsizeMode::End`) instead of overflowing the row
- Apple-blue selection highlight (`#0A84FF`, 7px radius, white label),
  customizable via `selected_color`
- Section headers (`section(title)`, 11px semibold, dimmed) grouping items
- Scheme-aware sidebar fill (dark `#2C2C2E` / light `#F5F5F7`)
- Gradient background via `background_gradient(coreicon::Gradient)` with
  direction-aware CSS gradients (linear or radial). The gradient covers the
  full sidebar including the area behind the item list: the `ScrolledWindow`
  (`sb-scroll`), its internal `GtkViewport` (`sb-viewport`) and the list box
  (`sb-list`) each carry their own `background-color: transparent;
  background-image: none` provider, because the app-level CSS otherwise
  paints every `scrolledwindow`/`viewport` with the solid window background
  (`#1d1d1d` dark / `#ececec` light) which would hide the gradient behind
  the items. A container-level USER-priority rule backs the same selectors
  up for late-created internal nodes. Rows stay untouched so the selected
  row keeps its `selected_color` fill and the gradient shines through the
  highlight as well.
- Auto-adaptive frame: the border and glow take their color from the
  background fill. With a gradient, the frame uses the gradient's own hue
  (average of the stop colors), strongly lightened for dark fills and
  slightly darkened for light fills, always at low alpha so the frame stays
  subtle; with a solid color it is plain white on dark fills and plain
  black on light fills. Explicit `border_color` / `glow_color` calls override
  this.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [TabView.md](TabView.md) -- functional tab container with sidebar + detail content
- [Slider.md](Slider.md) -- another interactive element
- [TextInput.md](TextInput.md) -- text input element
