# Button

SwiftUI-style push button for TontooUI, recreating the SwiftUI `Button` API
surface from the macOS 26 interface dumps. One element covers roles, visual
styles, tint, border shapes and sizing, plus the three system buttons
`RenameButton`, `EditButton` and `PasteButton`. Buttons are theme-aware
(dark / light) and follow the detected system scheme unless overridden.

## ButtonRole

```rust
pub enum ButtonRole {
    Destructive,
    Cancel,
    Confirm,
    Close,
}
```

Mirrors `SwiftUI.ButtonRole.Role` (destructive, cancel, confirm, close).

- `Destructive` tints the button with the system red.
- `Cancel`, `Confirm` and `Close` keep the system blue tint.
- A button created with an empty label resolves its label from the role.

### `ButtonRole::default_label`

```rust
pub fn default_label(self) -> &'static str
```

Returns the system default label for a role. Used automatically when
`Button::new("")` is combined with a role.

| Role | Default label |
|---|---|
| `Cancel` | `Cancel` |
| `Close` | `Close` |
| `Confirm` | `Done` |
| `Destructive` | `Delete` |

## ButtonStyle

```rust
pub enum ButtonStyle {
    Automatic,
    Plain,
    Bordered,
    BorderedProminent,
    Glass,
    GlassProminent,
}
```

Mirrors the SwiftUI style hierarchy (`PlainButtonStyle`,
`BorderedButtonStyle_Mac`, `GlassButtonStyle`, `GlassProminentButtonStyle`).

| Style | Background | Label |
|---|---|---|
| `Plain` | none | tint |
| `Bordered` | tint at low alpha + tint border | tint |
| `BorderedProminent` | solid tint | white |
| `Glass` | neutral glass (white/black at low alpha) + hairline border | tint |
| `GlassProminent` | solid tint | white |
| `Automatic` | resolves to `Glass` | tint |

## ButtonBorderShape

```rust
pub enum ButtonBorderShape {
    Automatic,
    Capsule,
    RoundedRectangle(f32),
    Circle,
}
```

Mirrors `SwiftUI.ButtonBorderShape.Guts`. `Automatic` and `Capsule` render a
fully rounded pill, `RoundedRectangle(radius)` a rounded rectangle, `Circle` a
perfect circle sized by the `height` (for icon-only buttons).

## ButtonSizing

```rust
pub enum ButtonSizing {
    Automatic,
    Fitted,
    Flexible,
}
```

Mirrors `SwiftUI.ButtonSizing.Value`. `Fitted` hugs the content, `Flexible`
expands the button horizontally to fill the available width
(`hexpand` + `Align::Fill`).

## Button

```rust
pub struct Button { /* ... */ }

impl Button {
    pub fn new(label: impl Into<String>) -> Self;
    pub fn role(self, role: ButtonRole) -> Self;
    pub fn style(self, style: ButtonStyle) -> Self;
    pub fn tint(self, color: Color) -> Self;
    pub fn border_shape(self, shape: ButtonBorderShape) -> Self;
    pub fn sizing(self, sizing: ButtonSizing) -> Self;
    pub fn icon(self, symbol: impl Into<String>) -> Self;
    pub fn size(self, width: f32, height: f32) -> Self;
    pub fn width(self, width: f32) -> Self;
    pub fn height(self, height: f32) -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn on_click(self, handler: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn effective_label(&self) -> String;
    pub fn to_view(self) -> View;
}
```

Builder API. Behavior notes:

- The tint defaults to system blue (`#0A84FF` dark / `#007AFF` light);
  `ButtonRole::Destructive` switches it to system red (`#FF453A` dark /
  `#FF3B30` light). `.tint(...)` overrides both.
- `effective_label()` returns the configured label, or the role default label
  when the label is empty.
- `.icon(...)` loads an SF Symbol PNG from the CoreIcon assets (feature
  `coreicon`), recolors it via its alpha mask and caches the result in the
  temp dir. An empty label with an icon renders an icon-only button.
- Hover raises brightness, pressing lowers it (mirrors the
  `isPressed` configuration behavior of `ButtonStyle.makeBody`).
- Invalid icon symbols are skipped silently (no icon is rendered).
- `Button` shadows the raw `uikit::Button` inside the prelude; the uikit
  version stays reachable as `uikit::widgets::Button`.

## RenameButton

```rust
pub struct RenameButton { /* ... */ }

impl RenameButton {
    pub fn new() -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn on_rename(self, handler: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

A plain blue button with a pencil icon labeled `Rename` that triggers a
standard rename action.

## EditButton

```rust
pub struct EditButton { /* ... */ }

impl EditButton {
    pub fn new() -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn is_editing(&self) -> bool;
    pub fn to_view(self) -> View;
}
```

Toggles between the `Edit` and `Done` labels on every click. `is_editing()`
reports the current state. The label is updated in place on the rendered
`GtkButton` (via a weak reference), so repeated toggling does not rebuild the
widget tree.

## PasteButton

```rust
pub struct PasteButton { /* ... */ }

impl PasteButton {
    pub fn new() -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn on_paste(self, handler: impl Fn(Option<String>) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

A prominent blue button with a clipboard icon labeled `Paste`. On click it
reads the system clipboard asynchronously and delivers the text to the
handler; the handler receives `None` when the pasteboard holds no text or the
read fails.

## Usage / Example

Run the full gallery demo (recreates the SwiftUI button example cards):

```bash
cargo run --example buttons
```

Minimal usage:

```rust
use tontooui::prelude::*;

let ok = Button::new("Tap Me")
    .style(ButtonStyle::BorderedProminent)
    .tint(Color::from_rgb(0, 122, 255))
    .on_click(|| println!("tapped"));

let cancel = Button::new("").role(ButtonRole::Cancel);

let delete = Button::new("")
    .role(ButtonRole::Destructive)
    .style(ButtonStyle::BorderedProminent);
```

## Cross References

- [TextInput.md](TextInput.md) -- text input element
- [Slider.md](Slider.md) -- spring-physics slider
- [Sidebar.md](Sidebar.md) -- sidebar with traffic lights and item list
- [ContentUnavailableView.md](ContentUnavailableView.md) -- empty state with icon, title, message
