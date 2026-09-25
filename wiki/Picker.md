# Picker

Picker category in `src/elements/pickers/`: `SegmentedPicker` in
`segmented.rs` (SwiftUI `Picker` with `.segmented` style),
`InlinePicker` in `inline.rs` (SwiftUI `Picker` with `.inline` style),
`MenuPicker` in `menu.rs` (SwiftUI `Picker` with `.menu` style) and
`DatePicker` in `date.rs` (graphical calendar). Segmented, inline
and menu carry an optional leading label; all select on release
inside the control and fire an `on_select` callback. Accent-driven
fills follow the system accent (Multicolor renders blue) unless the
dev sets them manually (`accent` for segmented/inline/date,
`accent` and `hover_fill` for menu). The date selection circle
defaults to the label color (black in light mode, like the
reference) instead.

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
| `MENU_PAD` / `MENU_RADIUS` | 6 px panel padding / 9 px panel radius |
| `MENU_ROW_H` / `MENU_ROW_SPACING` | 26 px rows, 2 px row gap |
| `MENU_CHECK_COL` / `MENU_TEXT_GAP` | 20 px check column / 6 px check-to-text gap |
| `MENU_CHECK_W` / `MENU_CHECK_H` | 10 px by 7.5 px fixed checkmark glyph |
| `MENU_PANEL_GAP` | 4 px button-to-panel gap |
| `MENU_SHADOW_BLUR` | 24 px heavy edge shadow |
| `MENU_ACCENT` | `#007AFF` default hover fill |
| `DATE_FIELD_H` / `DATE_FIELD_RADIUS` | 24 px field button, 6 px radius |
| `DATE_FIELD_FONT_SIZE` / `DATE_FIELD_PAD_X` | 13 px field and label / 10 px text padding |
| `DATE_FIELD_CHEV_W` / `DATE_FIELD_CHEV_GAP` | 12 px chevron box / 8 px text-to-chevron gap |
| `DATE_FIELD_GAP` / `DATE_PANEL_GAP` | 9 px label-to-field gap / 4 px field-to-panel gap |
| `DATE_CELL_W` / `DATE_CELL_H` | 32 px by 28 px day cells, 7 columns, 6 rows |
| `DATE_HEADER_H` / `DATE_WEEK_H` | 32 px title bar / 20 px weekday row |
| `DATE_PAD` / `DATE_RADIUS` | 8 px panel padding / 9 px panel radius |
| `DATE_TITLE_SIZE` / `DATE_DAY_SIZE` | 14 px title and day numbers |
| `DATE_WEEK_SIZE` / `DATE_LIST_SIZE` | 10 px weekday header / 13 px popup rows |
| `DATE_TITLE_GAP` | 8 px month-to-year gap |
| `DATE_SEL_R` | 12 px selection circle radius |
| `DATE_NAV_W` | 28 px month stepper hit width |
| `DATE_LIST_ROW_H` / `DATE_LIST_SPACING` | 26 px rows, 2 px row gap |
| `DATE_LIST_PAD` / `DATE_LIST_RADIUS` | 6 px popup padding / 9 px popup radius |
| `DATE_LIST_VISIBLE` | 8 visible popup rows |
| `DATE_LIST_CHECK_COL` / `DATE_LIST_TEXT_GAP` | 20 px check column / 6 px text gap |
| `DATE_SCROLL_W` / `DATE_POP_GAP` | 6 px scrollbar / 4 px header-to-popup gap |
| `DATE_YEAR_MIN` / `DATE_YEAR_MAX` | 1 to 3000 selectable years |
| `DATE_SHADOW_BLUR` | 24 px heavy edge shadow |
| `DATE_ACCENT` | `#007AFF` manual selection fill |
| `DATE_MONTHS` / `DATE_WEEKDAYS` | English month names / Monday-first headers |

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

Runs on the shared menu base ([Menu.md](Menu.md)): the base owns
button, frosted panel, rows, hover and viewport clamping, while the
picker only tracks the selection (button text plus checkmark) and
fires `on_select` on change. Same function as before.

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
  Rows draw inside a clip layer matching the panel, so text never
  spills outside it. Apps must call `set_viewport` every frame and
  return `is_open()` from `App::wants_backdrop` so the shell runs
  the blur pass while the menu is open.
- Empty option lists draw only the label and never open; out-of-range
  indices clamp to the last option.

## DatePicker

```rust
pub fn new() -> Self
pub fn label(self, label: impl Into<String>) -> Self
pub fn selected(self, year: i32, month: u32, day: u32) -> Self
pub fn accent(self, color: Color) -> Self
pub fn hover_fill(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_select(self, callback: impl FnMut(i32, u32, u32) + 'static) -> Self
pub fn is_open(&self) -> bool
pub fn open(&mut self)
pub fn close(&mut self)
pub fn formatted(&self) -> String
pub fn selected_date(&self) -> (i32, u32, u32)
pub fn viewed(&self) -> (i32, u32)
pub fn set_selected(&mut self, year: i32, month: u32, day: u32)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn step_month(&mut self, delta: i32)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn key(&mut self, key: Key)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
```

- The closed field button (current date as `"16 July 2026"` plus
  up/down chevrons, optional leading label) has no hover state, like
  `MenuPicker`. A click (press plus release) on it opens the
  calendar; clicking it again closes it. The button keeps a stable
  width (`"30 September 3000"`), so opening never resizes the layout.
- The open calendar is a `Frosted` glass panel floating above the
  background (heavy 24 px edge shadow). It prefers below the field,
  falls back above it and is always clamped into the `set_viewport`
  bounds, so the glass never samples outside the window.
- The panel shows a 7-column Monday-first grid with a fixed 6-row
  height (no resize jitter across months), gray weekday header and
  blank cells outside the month. Clicking a day selects it
  (`set_selected` clamps the day and fires `on_select` when the date
  changed) and closes the calendar, like a menu row click.
- The header shows the month and the year as two menus plus
  `<`/`>` steppers that move one month (wrapping years) and keep the
  calendar open. Clicking the month opens all twelve months;
  clicking the year opens years `DATE_YEAR_MIN` to `DATE_YEAR_MAX`
  (1-3000) with the current year visible on open. Lists show 8 rows
  with a scrollbar, hover in the accent fill (or the fixed
  `hover_fill`) and a checkmark on the current entry; a row click
  picks it and keeps the calendar open, header taps switch lists,
  anything else closes the whole calendar (or `Escape` closes the
  list first, then the calendar). Popup rows draw inside a clip
  layer matching the list area, so scrolled text never spills over
  the panel edge.
- Hovering a day, the month/year zones or the steppers tints them
  with the accent color. Hover only exists inside the open calendar.
- Date math is dependency-free (civil algorithms, Gregorian leap
  rule); the default selection is today from the system clock.
- Apps must call `set_viewport` every frame, forward `mouse_wheel`
  (year scrolling) and return `is_open()` from `App::wants_backdrop`
  so the shell runs the blur pass while the calendar is open.

```rust
pub fn days_in_month(year: i32, month: u32) -> u32
pub fn first_weekday(year: i32, month: u32) -> u32
```

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

Run `cargo run --example date`: `"Date"` pop-up field defaulting to
today (16 July 2026 in the reference) with a `Selected: ...`
caption. Click the field to float the calendar above the caption and
sample text (frost blur test), click the month or year for popup
lists (scroll the years with the wheel), use `<`/`>` to step months,
click a day to select and close.

## Cross References

- [Menu.md](Menu.md) – shared dropdown base underneath the picker
- [Slider.md](Slider.md) – accent fill, manual `accent`, click animation
- [Toggle.md](Toggle.md) – settings rows, icon badges, `on_toggle` callback
- [Button.md](Button.md) – control metrics, press overlay
- [Layout.md](Layout.md) – stacks hosting pickers, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell, mouse forwarding
- [Glass.md](Glass.md) – `Frosted` menu panel, backdrop blur pass
- [Animation.md](Animation.md) – tween drivers used by pill slide and dot pop
- [Theme.md](Theme.md) – accent, mode and Multicolor default
