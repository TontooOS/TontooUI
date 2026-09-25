# Menu

Menu category in `src/elements/menu/`: `Menu` in `menu.rs` is a
simple dropdown with a fixed button text plus a chevron. Rows are
action buttons: every row click fires `on_action` (no selection
state, the button text never changes). `MenuPicker`
([Picker.md](Picker.md)) reuses this base and only adds the
selection on top (button text plus checkmark, `on_select` on
change).

## Geometry

| Token | Value |
|---|---|
| `MENU_BUTTON_H` / `MENU_BUTTON_RADIUS` | 24 px button, 6 px radius |
| `MENU_FONT_SIZE` | 13 px button, rows and leading label |
| `MENU_GAP` / `MENU_BTN_PAD_X` | 9 px label-to-button gap / 10 px text padding |
| `MENU_CHEV_W` / `MENU_CHEV_GAP` | 12 px chevron box / 8 px text-to-chevron gap |
| `MENU_PAD` / `MENU_RADIUS` | 6 px panel padding / 9 px panel radius |
| `MENU_ROW_H` / `MENU_ROW_SPACING` | 26 px rows, 2 px row gap |
| `MENU_CHECK_COL` / `MENU_TEXT_GAP` | 20 px check column / 6 px check-to-text gap, only when `checked` is set |
| `MENU_CHECK_W` / `MENU_CHECK_H` | 10 px by 7.5 px fixed checkmark glyph |
| `MENU_PANEL_GAP` | 4 px button-to-panel gap |
| `MENU_SHADOW_BLUR` | 24 px heavy edge shadow |
| `MENU_ACCENT` | `#007AFF` default hover fill |

## Chevron

```rust
pub enum MenuChevron {
    Down,
    Both,
}
```

`Down` draws the single down chevron of the simple dropdown
reference; `Both` draws the up/down pair of the macOS pop-up
button (used by the picker).

## Element

```rust
pub fn new(button: impl Into<String>, options: Vec<String>) -> Self
pub fn from_slice(button: impl Into<String>, options: &[&str]) -> Self
pub fn label(self, label: impl Into<String>) -> Self
pub fn chevron(self, chevron: MenuChevron) -> Self
pub fn checked(self, row: Option<usize>) -> Self
pub fn hover_fill(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_action(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn is_open(&self) -> bool
pub fn open(&mut self)
pub fn close(&mut self)
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn option(&self, index: usize) -> Option<&str>
pub fn last_action(&self) -> Option<usize>
pub fn button_text(&self) -> &str
pub fn button_rect(&self) -> (f32, f32, f32, f32)
pub fn menu_rect(&self) -> (f32, f32, f32, f32)
pub fn row_rect(&self, row: usize) -> Option<(f32, f32, f32, f32)>
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_button(&mut self, button: impl Into<String>)
pub fn set_chevron(&mut self, chevron: MenuChevron)
pub fn set_checked(&mut self, row: Option<usize>)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- The button is wide enough for the longest row, so opening the
  menu never resizes it. The closed button has no hover state.
- A click (press plus release) on the button toggles the menu;
  press plus release on the same row fires `on_action` with the
  row index and closes; any other release closes. `last_action`
  records the row without needing a callback. Empty option lists
  never open.
- `checked` draws the fixed checkmark on that row (picker use)
  and reserves the check column; `None` is plain action rows with
  text starting at the panel padding (no wasted left space).
- Hovering a row tints it with the hover fill (system accent unless
  set manually with `hover_fill`) and turns row text and checkmark
  white.
- The frosted glass panel (`Frosted` finish, no clear background)
  prefers below the button, falls back above it and always clamps
  into the `set_viewport` bounds (including a cap at the window
  size), so the glass never samples outside the window. Apps must
  call `set_viewport` every frame and return `is_open()` from
  `App::wants_backdrop` so the shell runs the blur pass while the
  menu is open.

## NestedMenu

```rust
pub enum MenuItem {
    Action(String),
    Submenu(String, Vec<MenuItem>),
    Section(String),
    Divider,
}
pub fn action(label: impl Into<String>) -> Self
pub fn submenu(label: impl Into<String>, items: Vec<MenuItem>) -> Self
pub fn section(title: impl Into<String>) -> Self
pub fn divider() -> Self
```

| Token | Value |
|---|---|
| `NESTED_CHEV_COL` / `NESTED_CHEV_GAP` | 16 px submenu chevron / 6 px text gap |
| `NESTED_DIV_H` | 9 px divider slot |
| `NESTED_SUB_GAP` | 4 px parent-to-child panel gap |
| `NESTED_ACCENT` | `#007AFF` default hover fill |

```rust
pub fn new(button: impl Into<String>, items: Vec<MenuItem>) -> Self
pub fn label(self, label: impl Into<String>) -> Self
pub fn hover_fill(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_action(self, callback: impl FnMut(Vec<usize>) + 'static) -> Self
pub fn is_open(&self) -> bool
pub fn open(&mut self)
pub fn close(&mut self)
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn last_action(&self) -> Option<&[usize]>
pub fn button_text(&self) -> &str
pub fn button_rect(&self) -> (f32, f32, f32, f32)
pub fn panel_rects(&self) -> &[(f32, f32, f32, f32)]
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_button(&mut self, button: impl Into<String>)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
```

- Rows share the menu metrics (`MENU_ROW_H`, `MENU_PAD`,
  `MENU_RADIUS`, 24 px shadow). Action rows fire `on_action` with
  the path from the root (e.g. `[0, 2]`); section titles and
  dividers are dead (no hover, clicks on them close).
- Hovering a submenu row opens its child panel beside the parent
  (right preferred, left fallback, then viewport clamp); deeper
  levels follow the hover chain. Each level gets its own frosted
  panel plus scrollbar-free clipped rows.
- The closed button has no hover state. Panels cap at the window
  size; scrolling is unnecessary. Apps must call `set_viewport`
  every frame and return `is_open()` from `App::wants_backdrop`.

## Usage / Example

Run `cargo run --example menu`: `Color` picker plus an `Options`
simple dropdown (`Option 1/2/3`, last action in the titlebar) over
sample text lines that show the frost blur behind the open panels.

```rust
let mut options = Menu::from_slice("Options", &["Option 1", "Option 2", "Option 3"])
    .on_action(|i| println!("option: {i}"));
options.set_theme(accent, true);
options.set_glass(ThemeMode::Dark, GlassAmount::Glass);
```

Run `cargo run --example nested`: `Share` (destination submenu)
and `File` (file/edit sections) dropdowns in a `VStack` with the
last action path in the titlebar plus a `Selected: ...` caption.

```rust
let mut share = NestedMenu::new(
    "Share",
    vec![
        MenuItem::section("Choose destination"),
        MenuItem::submenu("Messages", vec![
            MenuItem::action("John"),
            MenuItem::action("Jane"),
        ]),
        MenuItem::divider(),
        MenuItem::action("More..."),
    ],
);
share.set_theme(accent, true);
```

## Cross References

- [Picker.md](Picker.md) – `MenuPicker` selection on top of this base
- [Button.md](Button.md) – button metrics, press overlay
- [Glass.md](Glass.md) – `Frosted` panel, backdrop blur pass
- [Layout.md](Layout.md) – stacks hosting menus, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell, mouse forwarding
- [Theme.md](Theme.md) – accent, mode and Multicolor default
