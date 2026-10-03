# TontooUI – Wiki

SwiftUI-inspired declarative UI layer for TontooOS: elements, layout,
theme, animation and a Vello/WGPU renderer.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 27.0.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Animation | [Animation.md](Animation.md) | Frame clock, tweens, springs, decay |
| Alerts | [Alerts.md](Alerts.md) | Modal frosted alert with OK/Cancel actions |
| BarSwitcher | [BarSwitcher.md](BarSwitcher.md) | Single-select toolbar with icon/text/both cells |
| BackdropStream | [BackdropStream.md](BackdropStream.md) | Compositor desktop backdrop stream for client blur |
| Button | [Button.md](Button.md) | Standard button with CoreIcon SF Symbols |
| Colors | [Colors.md](Colors.md) | System colors and linear/radial/angular gradients |
| ContentUnavailable | [ContentUnavailable.md](ContentUnavailable.md) | Empty-state placeholder with refresh |
| Divider | [Divider.md](Divider.md) | Full-bleed horizontal and vertical dividers |
| Gauge | [Gauge.md](Gauge.md) | Basic, linear, circular and capacity gauges |
| Glass | [Glass.md](Glass.md) | Liquid glass container and backdrop blur |
| Gestures | [Gestures.md](Gestures.md) | Tap, long press, drag and magnify areas |
| Groupbox | [Groupbox.md](Groupbox.md) | Basic group box with centered text |
| Images | [Images.md](Images.md) | SF Symbol, app resource, URL and overlay card |
| Layout | [Layout.md](Layout.md) | VStack, HStack, ZStack, modifiers, `View` |
| List | [List.md](List.md) | Static text list with row dividers |
| Link | [Link.md](Link.md) | Blue link opening the default browser |
| Label | [Label.md](Label.md) | Icon, image, styled and icon-only labels |
| Menu | [Menu.md](Menu.md) | Simple dropdown with action rows, picker base |
| Outline | [Outline.md](Outline.md) | File tree with fade reveal, chevrons and single-select |
| Material | [Material.md](Material.md) | Translucent material veils in five thicknesses |
| Renderer | [Renderer.md](Renderer.md) | Window shell, frame pipeline, backdrop blur |
| Picker | [Picker.md](Picker.md) | Segmented, inline, menu and date pickers |
| Progress | [Progress.md](Progress.md) | Linear progress bar with chase buffer |
| Scrollbar | [Scrollbar.md](Scrollbar.md) | Overlay side bar with fade, drag and page jump |
| ScrollView | [ScrollView.md](ScrollView.md) | Clipped scroll container with integrated scrollbar |
| Sheets | [Sheets.md](Sheets.md) | Modal sheet with sizes, custom background, ESC |
| Sidebar | [Sidebar.md](Sidebar.md) | App navigation with traffic, toolbar, pages and collapse |
| Shapes | [Shapes.md](Shapes.md) | Rectangle, circle, rounded, capsule and custom shapes |
| Slider | [Slider.md](Slider.md) | Slider with steps, labels, ticks, glass |
| Stepper | [Stepper.md](Stepper.md) | Basic stepper with step size, range and limit dimming |
| Form | [Form.md](Form.md) | Settings form with sections, text, toggle, picker and button rows |
| FontPreview | [FontPreview.md](FontPreview.md) | Font family name plus sample line in its own family |
| Table | [Table.md](Table.md) | Basic table with sort, scroll, select and inline edit |
| Theme | [Theme.md](Theme.md) | Live dark/light, accent, glass stage |
| Text | [Text.md](Text.md) | Basic text with styles and foregrounds |
| Textfield | [Textfield.md](Textfield.md) | Slim and large single-line fields |
| Titlebar | [Titlebar.md](Titlebar.md) | Custom decoration bar with drag region |
| Toggle | [Toggle.md](Toggle.md) | Switch, button and checkbox styles |
| Toolbar | [Toolbar.md](Toolbar.md) | Small clear-glass icon toolbar |
| Video | [Video.md](Video.md) | File/URL playback via MediaKit, no next/previous |

## Quick Start

```rust
use tontooui::elements::Titlebar;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, run};
use tontooui::Scene;

struct Hello {
    bar: Titlebar,
}

impl App for Hello {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut tontooui::renderer::ImageLoader<'_>,
        viewport: Viewport,
        _t: f64,
    ) {
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
    }
}

fn main() {
    run("Hello", 800, 600, Hello {
        bar: Titlebar::new("Hello"),
    }).unwrap();
}
```

See [Renderer.md](Renderer.md) for the shell and [Layout.md](Layout.md)
for the `View` tree.

## Changelog

- 2026-10-02: `BACKDROP_SCALE` is now `1`, so the compositor stream delivers
  full resolution. Affordable because the compositor only captures when the
  content behind the window actually changed: a panel opening over a still
  desktop pays for exactly one readback, and divisors above one only help
  while something behind the window animates. See
  [BackdropStream.md](BackdropStream.md).
- 2026-10-03: The wheel reaches every element again. `View::mouse_wheel`
  has a do-nothing default and parents only reach their children through
  the trait, but no scrollable element forwarded it in its `impl View`
  block, so anything held as a `Box<dyn View>` swallowed the wheel: a
  `ScrollView` behind a stack or a custom wrapper view (Weather paints
  its condition gradient that way) drew its scrollbar thumb but never
  scrolled. `ScrollView`, `Scrollbar`, `Sidebar`, `Menu`, `NestedMenu`,
  `MenuButton`, `ContextMenu`, `Form`, `BasicTable` and `DatePicker` now
  forward it; custom wrapper views must do the same, see
  [Layout.md](Layout.md) and [ScrollView.md](ScrollView.md).

- 2026-10-03: Sidebar row hit test plus destructive menu rows.
  `Sidebar::item_at(x, y)` returns the real item index under a point
  (column width, collapse state and search filter applied), the same
  test `mouse_down` uses, so context menus can resolve the row a
  right-click landed on: a right-click arrives as `App::context_click`
  and never selects. `Menu::destructive(row)` (`set_destructive`,
  `is_destructive`, `MENU_DESTRUCTIVE_LIGHT` / `_DARK`) paints one row
  in the macOS system red for `Delete`-style actions. Added for the
  Weather app, which removes a location through a sidebar context menu
  and an `ActionAlert`. See [Sidebar.md](Sidebar.md) and
  [Menu.md](Menu.md).

- 2026-10-02: Two backdrop fixes plus an optional content title band.
  `readback_variance` no longer leaks a mapped buffer: the
  variance staging buffer is taken out of `VarTargets` for the
  whole map, so a failed or late `map_async` cannot leave it
  mapped and trip wgpu's `Queue::submit` validation (Buffer ...
  is still mapped) on the next frame; `release_map` unmaps and
  hands it back once the callback lands. `Sidebar::toolbar(false)` (
  `set_toolbar` / `has_toolbar`) hides the `SIDEBAR_TOOLBAR_H`r
  content title band so the page starts at the top. The pill group
  keeps its own layout: a collapsed column still parks the slot pill
  plus toggle at the far right of the window on the traffic row, and
  the row is painted after the page so it floats on a full-bleed
  background instead of disappearing under it. See
  [BackdropStream.md](BackdropStream.md), [Sidebar.md](Sidebar.md) and
  [Renderer.md](Renderer.md).

- 2026-10-02: Protocol level input and a live window title in the
  shell. `App::raw_key` delivers every key transition as a `KeyPress`
  (`RawKey` identity plus the full `Modifiers` state, decoded text
  and repeat flag) before the intent hooks `key` / `text` keep
  running; `App::mouse_button` reports every button, including
  middle, on press and release with its modifiers; `App::window_title`
  sets the real window title once per frame. Added for the Terminal
  app, which needs Tab, F-keys, Ctrl chords, Alt as an escape prefix
  and mouse reports. See [Renderer.md](Renderer.md).


- 2026-10-02: `BasicText` gained `size(px)` / `weight(w)` plus
  `clear_size` / `clear_weight`, the `set_size(Option<f32>)` /
  `set_weight(Option<f32>)` setters and the `size_value` /
  `weight_value` readers. `TextStyle` tops out at `LargeTitle` (34 px),
  so display numbers set the size directly; `style` still picks the type
  scale. See [Text.md](Text.md).

- 2026-10-02: Desktop backdrop stream (`src/renderer/backdrop_stream.rs`).
  TontooUI now blurs the *desktop* instead of only its own content:
  `CompositorBackdrop::attach` wraps winit's `wl_display` in a guest
  Wayland backend, binds `tontoo_ui_manager`, hands the compositor a
  `memfd` and subscribes with `set_backdrop`. The compositor captures the
  elements below the window at half resolution, writes them into the
  shared file and sends a `backdrop` event; TontooUI upsamples the rect and
  pushes it into `BackdropBlur` through the new `upload_content`, so the
  existing WGSL blur, lens, frost and rim code is untouched. A live stream
  also removes the second Vello pass. `App::wants_backdrop` keeps working
  as the fallback for compositors without the global. New deps:
  `wayland-scanner`, `wayland-backend`, `libc` plus a vendored
  `protocol/tontoo_ui.xml`. See [BackdropStream.md](BackdropStream.md).
- 2026-10-02: Fixed three compile errors in the `BasicText` unit tests
  (`measure` needs a mutable receiver, `clear_size` consumes the builder).
  The test target did not build before. See [Text.md](Text.md).

- 2026-09-30: List icons plus outline upgrades. `ListRow::icon`
  plus `icon_tint` draw an 18px SF Symbol ahead of item labels
  (row height stays put); `OutlineNode::icon_tint` tints one node
  icon over the group tint; `DisclosureGroup::header_icon` plus
  `header_icon_tint` prefix the header title;
  `DisclosureGroup::trailing_chevron` moves the chevron to the row
  end with the same morph and hit box. See [List.md](List.md) and
  [Outline.md](Outline.md).

- 2026-09-30: `BarSwitcher` element (`src/elements/barswitcher.rs`).
  Single-select toolbar in the `BasicToolbar` capsule look: each cell
  holds an icon, a text label or both, the selected cell keeps the
  gray hover-style highlight, clicks move the selection and fire
  `on_select` (change only) for view swapping. Demo in
  `examples/barswitcher.rs`. See [BarSwitcher.md](BarSwitcher.md).

- 2026-09-30: Window resize zone is asymmetric (3 px into content,
  10 px into the shadow rim) so the overlay scrollbar stays usable.
  See [Renderer.md](Renderer.md).

- 2026-09-30: Window resizing for undecorated windows. Hovering a
  body edge or corner shows the matching resize arrow and a left
  press starts an OS resize drag (`resize_direction_at`,
  `RESIZE_HIT`, `RESIZE_CORNER_HIT`, eight `CursorKind::Resize*`
  variants); maximized windows keep the app cursor.
  See [Renderer.md](Renderer.md).

- 2026-09-29: `slider` example with a vendored `FileImage` photo
  background (`examples/assets/slider-bg.jpg`, `ImageFit::Cover`,
  shell-clipped to the rounded window) and three `BasicToolbar`
  rows (leading, center, trailing).
  See [Slider.md](Slider.md).

- 2026-09-29: Window corner radius is `17.0` logical px
  (`WINDOW_CORNER_RADIUS`, was `20.0`), matching the GTK theme.
  See [Renderer.md](Renderer.md).

- 2026-09-29: `VideoPlayer` element (`elements/video/`). File/URL
  playback through MediaKit `frame_at` with transport controls
  (play/pause, stop, scrubber, time, mute, fullscreen) and
  deliberately no next/previous. See [Video.md](Video.md).
- 2026-09-29: `GlassTextField` element. Single-line input with the
  `SearchField` clear (`Lens`) glass capsule but no magnifier icon
  (full-width text, `GLASS_FIELD_FONT_SIZE` 14 px,
  `GLASS_FIELD_PAD_X` 14 px). Same editing contract as the basic
  field, needs `wants_backdrop` while visible.
  See [Textfield.md](Textfield.md).
- 2026-09-29: Re-exported Vello scene types from the crate root
  (`Scene`, `Color`, `kurbo`, `peniko`). Apps now write
  `use tontooui::Scene` instead of `use vello::Scene`; no direct
  `use vello` import is needed in app code. See [Renderer.md](Renderer.md).

- 2026-09-28: Empty-state text follows the theme. `ContentUnavailable`
  and `CustomContentUnavailable` forward the mode to their inner
  title/message texts (they kept the dark default, rendering gray in
  light mode); the `all_elements` demo themes all three variants.
  See [ContentUnavailable.md](ContentUnavailable.md).
- 2026-09-28: `TextInputAlert` element. Modal alert with a clear
  (`Lens`) glass input pill (no icon), autofocus, Enter confirms
  with OK, typed text via `text_value`; wired into the
  `all_elements` demo (own section). See [Alerts.md](Alerts.md).
- 2026-09-28: `all_elements` example. One scrollable page with every
  TontooUI element in a single `View` (`ScrollView` plus a flat `VStack`
  with captions); modal elements (alerts, sheet, color picker popup,
  context menu) open through demo buttons.
- 2026-09-28: Stacks clip children to the placed rect. `VStack`,
  `HStack` and `ZStack` cut overflowing content off via a clip layer
  (same pattern as `ScrollView`) instead of spilling past their bounds;
  unplaced stacks draw unclipped as before. See [Layout.md](Layout.md).
- 2026-09-28: Fixed `accent_gallery` row helper skipping every second
  element type in mixed rows (Gauge, Stepper, SecureField, MenuButton
  never received the theme). It now iterates by `len` and skips type
  mismatches, the documented `child_mut` pattern.
- 2026-09-28: `accent_gallery` example. One page with every
  accent-following element plus an accent switcher (daemon "Auto" or
  local preview of all thirteen accents). See [Theme.md](Theme.md).
- 2026-09-28: Push-based theme updates. `ThemeWatcher` holds one
  persistent `subscribe` connection and applies pushed
  `customize_changed` events immediately (no polling traffic while idle),
  with throttled revision polling as fallback. See [Theme.md](Theme.md).
- 2026-09-28: Faster theme switch detection. `THEME_POLL_SECONDS` is 0.2 s
  instead of 1.0 s, so dark/light changes picked up from the daemon start
  the crossfade almost immediately (revision-guarded, still one cheap
  socket read per interval when idle). See [Theme.md](Theme.md).
- 2026-09-28: Image backend moved from the third-party `image`/`png`
  crates to CoreImage (`coreimage::TiImage::load`, `from_bytes` and
  `thumbnail` in `src/renderer/images.rs`; `coreimage::codecs::png::encode`
  for the hue wheel in `src/elements/colors/picker.rs`). The `image` and
  `png` dependencies were removed. See [Renderer.md](Renderer.md) and
  [Colors.md](Colors.md).
- 2026-09-28: `FontPreview::from_file` and `from_data` preview
  font files/bytes without a system install
  (`FontSystem::register_font_file`, `register_font_data`).
- 2026-09-28: New `FontPreview` element (family name plus sample
  line in its own family, `FontSystem::layout_text_in_family` with
  `system-ui` fallback). See [FontPreview.md](FontPreview.md).

- 2026-09-28: Clipboard uses Foundation `NSPasteboard` (native
  Wayland/X11); the `arboard` dependency is removed, unifying the
  `image` crate to a single version (fixes duplicate-symbol cdylib
  link). Layer-shell backend (`run_layer`, one `App` per output) and
  transparency-only `BarMenu` for shell bars.
- 2026-09-26: Text stack moved to CoreText (`CTFrame` layouts,
  `hit_byte` links, `decorations()` underlines; `parley` dependency
  removed). `FontSystem` keeps its name and gains
  `layout_text_aligned`, `layout_rich_text_aligned`, `framesetter`
  and `draw_with_brush` on CoreText.
- 2026-09-26: `UrlImage` downloads through NetworkKit
  (`networkkit::http`) instead of a direct `ureq` client. The `ureq`
  dependency is removed.
