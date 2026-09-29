# TontooUI – Wiki

SwiftUI-inspired declarative UI layer for TontooOS: elements, layout,
theme, animation and a Vello/WGPU renderer.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Animation | [Animation.md](Animation.md) | Frame clock, tweens, springs, decay |
| Alerts | [Alerts.md](Alerts.md) | Modal frosted alert with OK/Cancel actions |
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
