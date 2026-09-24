# Picker

Picker category in `src/elements/pickers/`: `SegmentedPicker` in
`segmented.rs` (SwiftUI `Picker` with `.segmented` style) and
`InlinePicker` in `inline.rs` (SwiftUI `Picker` with `.inline` style).
Both carry an optional leading label, select on release inside the
control and fire an `on_select` callback. The selected fill follows
the system accent (Multicolor renders blue) unless the dev sets it
manually with `accent`.

## Geometry

| Token | Value |
|---|---|
| `SEGMENTED_HEIGHT` / `SEGMENTED_RADIUS` | 32 px track, 8 px radius |
| `SEGMENTED_PAD` / `SEGMENTED_PILL_RADIUS` | 2 px pill inset, 6.5 px pill radius |
| `SEGMENTED_FONT_SIZE` / `SEGMENTED_LABEL_SIZE` | 15 px segments / 17 px leading label |
| `SEGMENTED_GAP` / `SEGMENTED_PAD_X` | 12 px label-to-track gap / 16 px text padding |
| `SEGMENTED_MIN_SEG_W` | 72 px minimum segment width |
| `SEGMENTED_ANIM_SECONDS` | 0.20 s pill slide |
| `SEGMENTED_TRACK_DARK` / `SEGMENTED_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `SEGMENTED_ACCENT` | `#007AFF` default selected fill |
| `INLINE_RADIO_R` / `INLINE_DOT_R` | 12 px radio, 5 px white center dot |
| `INLINE_ROW_H` / `INLINE_ROW_SPACING` | 32 px rows, 4 px row gap |
| `INLINE_GAP_X` / `INLINE_RADIO_GAP` | 16 px label-to-options gap, 10 px radio-to-text gap |
| `INLINE_FONT_SIZE` | 17 px options and leading label |
| `INLINE_ANIM_SECONDS` | 0.15 s dot pop |
| `INLINE_OFF_DARK` / `INLINE_OFF_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `INLINE_ACCENT` | `#007AFF` default selected fill |

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
- `mouse_down` inside the track arms the control; `mouse_up` inside
  selects the segment under the cursor (`View::mouse_up` does the
  same). Press inside and release outside keeps the selection.
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
  selected row shows the accent dot with a white center.
- `disabled` rows never arm; unfocused windows desaturate the picker
  like the rest of the palette.

## Usage / Example

Run `cargo run --example segmented`: `Options` (`One`, `Two`,
`Three`), `Size` and a fixed green `Theme` picker in a `VStack`
with the selection in the titlebar plus a `Selected: ...` caption.

Run `cargo run --example inline`: `Size` (`Small`, `Medium`,
`Large`, `Extra Large`) in a `VStack` with the selection in the
titlebar plus a `Selected: ...` caption.

```rust
let mut options = SegmentedPicker::from_slice("Options", &["One", "Two", "Three"])
    .on_select(|i| println!("segment: {i}"));
options.set_theme(accent, true);

let mut size = InlinePicker::from_slice("Size", &["Small", "Medium", "Large", "Extra Large"])
    .on_select(|i| println!("size: {i}"));
size.set_theme(accent, true);
```

## Cross References

- [Slider.md](Slider.md) – accent fill, manual `accent`, click animation
- [Toggle.md](Toggle.md) – settings rows, icon badges, `on_toggle` callback
- [Button.md](Button.md) – control metrics, press overlay
- [Layout.md](Layout.md) – stacks hosting pickers, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell, mouse forwarding
- [Animation.md](Animation.md) – tween drivers used by pill slide and dot pop
- [Theme.md](Theme.md) – accent, mode and Multicolor default
