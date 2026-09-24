# Picker

Picker category in `src/elements/pickers/`: `SegmentedPicker` in
`segmented.rs` (SwiftUI `Picker` with `.segmented` style),
`InlinePicker` in `inline.rs` (SwiftUI `Picker` with `.inline` style)
and `MenuPicker` in `menu.rs` (SwiftUI `Picker` with `.menu` style).
All carry an optional leading label, select on release inside the
control and fire an `on_select` callback. Accent-driven fills follow
the system accent (Multicolor renders blue) unless the dev sets them
manually (`accent` for segmented/inline, `accent` and `hover_fill`
for menu).

## Geometry

| Token | Value |
|---|---|
| `SEGMENTED_HEIGHT` / `SEGMENTED_RADIUS` | 24 px track, 6 px radius |
| `SEGMENTED_PAD` / `SEGMENTED_PILL_RADIUS` | 1.5 px pill inset, 5 px pill radius |
| `SEGMENTED_FONT_SIZE` / `SEGMENTED_LABEL_SIZE` | 11 px segments / 13 px leading label |
| `SEGMENTED_GAP` / `SEGMENTED_PAD_X` | 9 px label-to-track gap / 12 px text padding |
| `SEGMENTED_MIN_SEG_W` | 54 px minimum segment width |
| `SEGMENTED_ANIM_SECONDS` | 0.20 s pill slide |
| `SEGMENTED_PRESSED_DARK` / `SEGMENTED_PRESSED_LIGHT` | `#636366` / `#D1D1D6` hold highlight |
| `SEGMENTED_TRACK_DARK` / `SEGMENTED_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `SEGMENTED_ACCENT` | `#007AFF` default selected fill |
| `INLINE_RADIO_R` / `INLINE_DOT_R` | 9 px radio, 3.75 px white center dot |
| `INLINE_ROW_H` / `INLINE_ROW_SPACING` | 24 px rows, 3 px row gap |
| `INLINE_GAP_X` / `INLINE_RADIO_GAP` | 12 px label-to-options gap, 7.5 px radio-to-text gap |
| `INLINE_FONT_SIZE` | 13 px options and leading label |
| `INLINE_ANIM_SECONDS` | 0.15 s dot pop |
| `INLINE_OFF_DARK` / `INLINE_OFF_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `INLINE_ACCENT` | `#007AFF` default selected fill |
| `MENU_BUTTON_H` / `MENU_BUTTON_RADIUS` | 24 px button, 6 px radius |
| `MENU_FONT_SIZE` | 13 px button, rows and leading label |
| `MENU_GAP` / `MENU_BTN_PAD_X` | 9 px label-to-button gap / 10 px text padding |
| `MENU_CHEV_W` / `MENU_CHEV_GAP` | 12 px chevron box / 8 px text-to-chevron gap |
| `MENU_PAD` / `MENU_RADIUS` | 6 px panel padding / 6 px panel radius |
| `MENU_ROW_H` / `MENU_ROW_SPACING` | 26 px rows, 2 px row gap |
| `MENU_CHECK_COL` / `MENU_TEXT_GAP` | 20 px check column / 6 px check-to-text gap |
| `MENU_CHECK_W` / `MENU_CHECK_H` | 10 px by 7.5 px fixed checkmark glyph |
| `MENU_PANEL_GAP` | 4 px button-to-panel gap |
| `MENU_SHADOW_BLUR` | 24 px heavy edge shadow |
| `MENU_ACCENT` | `#007AFF` default hover fill |

## SegmentedPicker

```rust
pub fn new(label: impl Into<String>, options: Vec<String>) -> Self
pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self
pub fn selected(self, index: usize) -> Self
pub fn accent(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn selected_index(&self) -> usize
pub fn selected_label(&self) -> Option<&str>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn select(&mut self, index: usize)
pub fn set_selected(&mut self, index: usize)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- `selected` sets the initial segment without animation and without
  firing `on_select`; `set_selected` sets it immediately and fires
  `on_select` when the selection changed; `select` slides the pill
  with a 0.20 s `CubicOut` tween and fires `on_select`.
- Segments share the track equally; at intrinsic size each keeps at
  least its text width plus padding or `SEGMENTED_MIN_SEG_W`.
- The pill carries a soft shadow; hairline dividers render only
  between unselected neighbors, like the macOS segmented control.
- There is no hover state. `mouse_down` inside the track arms the
  control and shows the gray hold highlight on the pressed segment
  (`SEGMENTED_PRESSED_DARK` / `SEGMENTED_PRESSED_LIGHT`); the
  highlight follows the pointer while held. `mouse_up` on the same
  segment switches to it (`View::mouse_up` does the same). Press
  inside and release outside keeps the selection.
- Empty option lists draw nothing and never select; out-of-range
  indices clamp to the last segment.

## InlinePicker

```rust
pub fn new(label: impl Into<String>, options: Vec<String>) -> Self
pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self
pub fn selected(self, index: usize) -> Self
pub fn accent(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn selected_index(&self) -> usize
pub fn selected_label(&self) -> Option<&str>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn select(&mut self, index: usize)
pub fn set_selected(&mut self, index: usize)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- `selected` sets the initial row without animation and without
  firing `on_select`; `set_selected` sets it immediately and fires
  `on_select` when the selection changed; `select` pops the center
  dot with a 0.15 s `CubicOut` tween and fires `on_select`.
- The leading label shares the first row baseline; rows below align
  in the options column. Hits in the spacing gap between rows select
  nothing.
- Unselected rows show a mode-gray dot plus a hover ring; the
  selected row shows the accent dot with a white center and a small
  soft shadow.
- `disabled` rows never arm; unfocused windows desaturate the picker
  like the rest of the palette.

## MenuPicker

```rust
pub fn new(label: impl Into<String>, options: Vec<String>) -> Self
pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self
pub fn selected(self, index: usize) -> Self
pub fn hover_fill(self, color: Color) -> Self
pub fn accent(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn is_open(&self) -> bool
pub fn open(&mut self)
pub fn close(&mut self)
pub fn selected_index(&self) -> usize
pub fn selected_label(&self) -> Option<&str>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn select(&mut self, index: usize)
pub fn set_selected(&mut self, index: usize)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- The closed pop-up button (current value plus up/down chevrons) has
  no hover state. A click (press plus release) on it opens the menu;
  clicking it again closes the menu.
- The open menu is a `Frosted` glass panel (blurred everywhere, no
  clear background) with a heavy 24 px edge shadow. The selected row
  shows a fixed-size checkmark (10 by 7.5 px, same at any menu size).
- Hovering a row tints it with the hover fill (system accent unless
  set manually with `hover_fill`) and turns row text and checkmark
  white. Press plus release on the same row selects it and closes;
  any other release closes without changing.
- The panel prefers below the button, falls back above it and is
  always clamped into the `set_viewport` bounds (including a cap at
  the window size), so the glass never samples outside the window.
  Apps must call `set_viewport` every frame and return `is_open()`
  from `App::wants_backdrop` so the shell runs the blur pass while
  the menu is open.
- Empty option lists draw only the label and never open; out-of-range
  indices clamp to the last option.

## Usage / Example

Run `cargo run --example segmented`: `Options` (`One`, `Two`,
`Three`), `Size` and a fixed green `Theme` picker in a `VStack`
with the selection in the titlebar plus a `Selected: ...` caption.

Run `cargo run --example inline`: `Size` (`Small`, `Medium`,
`Large`, `Extra Large`) in a `VStack` with the selection in the
titlebar plus a `Selected: ...` caption.

Run `cargo run --example menu`: `Color` (`Red`, `Green`, `Blue`,
`Yellow`, `Purple`) pop-up button in a `VStack` with the selection
in the titlebar plus a `Selected: ...` caption; the open glass menu
floats above the caption and sample text lines that show the frost
blur behind it.

```rust
let mut options = SegmentedPicker::from_slice("Options", &["One", "Two", "Three"])
    .on_select(|i| println!("segment: {i}"));
options.set_theme(accent, true);

let mut size = InlinePicker::from_slice("Size", &["Small", "Medium", "Large", "Extra Large"])
    .on_select(|i| println!("size: {i}"));
size.set_theme(accent, true);

let mut color = MenuPicker::from_slice("Color", &["Red", "Green", "Blue"])
    .on_select(|i| println!("color: {i}"));
color.set_theme(accent, true);
color.set_glass(ThemeMode::Dark, GlassAmount::Glass);
```

## Cross References

- [Slider.md](Slider.md) – accent fill, manual `accent`, click animation
- [Toggle.md](Toggle.md) – settings rows, icon badges, `on_toggle` callback
- [Button.md](Button.md) – control metrics, press overlay
- [Layout.md](Layout.md) – stacks hosting pickers, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell, mouse forwarding
- [Glass.md](Glass.md) – `Frosted` menu panel, backdrop blur pass
- [Animation.md](Animation.md) – tween drivers used by pill slide and dot pop
- [Theme.md](Theme.md) – accent, mode and Multicolor default
