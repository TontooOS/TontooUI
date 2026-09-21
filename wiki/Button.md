# Button

Standard button in `src/elements/buttons/button.rs`: rounded fill, optional
SF Symbol icon plus label, hover/pressed/disabled states and a press
callback. Icons come from CoreIcon (`COREICON_ASSETS_DIR` override or the
system resources on TontooOS); a missing icon draws the label alone.

## Colors

| Token | Value |
|---|---|
| `BUTTON_BG_DARK` | `#2C2C2E` |
| `BUTTON_BG_LIGHT` | `#E9E9EB` |
| `BUTTON_ACCENT` | `#007AFF` (default prominent/tinted accent) |

```rust
pub const BUTTON_RADIUS: f32;     // 8.0
pub const BUTTON_FONT_SIZE: f32;  // 13.0, macOS control size
pub const BUTTON_PAD_X: f32;      // 12.0
pub const BUTTON_PAD_Y: f32;      // 6.0
pub const BUTTON_GAP: f32;        // 6.0 icon-to-label
pub const BUTTON_ICON_SIZE: f32;  // 16.0 box, aspect kept
```

## Style

```rust
pub enum ButtonStyle {
    Automatic,
    Bordered,
    BorderedProminent,
    BorderedTinted,
    Plain,
}
```

`Automatic` resolves to `Bordered`. `BorderedProminent` fills the theme
accent with white text, `BorderedTinted` fills accent at 20% alpha with
accent text, `Plain` draws no background. Press lightens in dark mode and
darkens in light mode; `Plain` dims its text instead.

```rust
pub fn style(self, style: ButtonStyle) -> Self
pub fn set_theme(&mut self, accent: Color, dark: bool)
```

## Shape

```rust
pub enum ButtonShape {
    Automatic,
    RoundedRectangle,
    Capsule,
    Circle,
}
```

`Automatic` resolves to `RoundedRectangle`. `Capsule` and `Circle` use
half the smaller side as radius.

```rust
pub fn shape(self, shape: ButtonShape) -> Self
```

## Element

```rust
pub fn new(label: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn icon_size(self, px: f32) -> Self
pub fn on_press(self, callback: impl FnMut() + 'static) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn set_palette(&mut self, bg: Color, text: Color)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_focused(&mut self, focused: bool)
```

SF Symbol names use the CoreIcon spelling (e.g. `"hand.tap"`). Icons
tint to the label color and fit the icon box preserving aspect.
Unfocused windows desaturate the button like the rest of the palette.

Clicks fire on release inside the button (`mouse_down` arms,
`View::mouse_up` fires); the shell forwards both. Hover tracks through
`set_hover`.

## Usage / Example

Run `cargo run --example button` (needs the CoreIcon assets, e.g.
`COREICON_ASSETS_DIR=../CoreIcon/assets/icons` on dev checkouts): two
`Tap` buttons plus tinted, plain and capsule variants in a `VStack`, press
count in the titlebar.

```rust
let mut tap = Button::new("Tap with Label")
    .icon("hand.tap")
    .style(ButtonStyle::BorderedProminent)
    .on_press(|| println!("tapped"));
tap.set_theme(accent, true);
```

## Cross References

- [Layout.md](Layout.md) – stacks hosting buttons, `View` trait
- [Renderer.md](Renderer.md) – `ImageLoader`, `mouse_up`, `App`
- [Theme.md](Theme.md) – accent and mode driving styles
- [Titlebar.md](Titlebar.md) – bar used by the demo
