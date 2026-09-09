# Toggle

SwiftUI-style boolean toggle for TontooUI, recreating the non-list SwiftUI
toggle styles from the macOS 26 dumps: `SwitchToggleStyle` (leading label with
a trailing switch) and `CheckboxToggleStyle` (checkbox followed by its label).
The whole row is clickable, the state flips in place without rebuilding the
widget tree, and the element is theme-aware (dark / light).

## ToggleStyle

```rust
pub enum ToggleStyle {
    Switch,
    Checkbox,
}
```

Mirrors the SwiftUI toggle style hierarchy (`SwitchToggleStyle`,
`CheckboxToggleStyle`).

| Style | Layout | Control |
|---|---|---|
| `Switch` | label leading, switch trailing (row expands when `width` set) | 51x31 pill, green when on, gray when off, white 27px round knob with 2px inset |
| `Checkbox` | checkbox leading, label trailing | 20x20 rounded square, system blue filled with white checkmark when on, hairline outline when off |

## Toggle

```rust
pub struct Toggle { /* ... */ }

impl Toggle {
    pub fn new(label: impl Into<String>) -> Self;
    pub fn style(self, style: ToggleStyle) -> Self;
    pub fn value(self, on: bool) -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn width(self, w: f32) -> Self;
    pub fn on_change(self, handler: impl Fn(bool) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Builder API. Behavior notes:

- The default style is `Switch`, the default state is off.
- The whole row handles the click (one `GestureClick` on the row container),
  mirroring the SwiftUI toggle row; there are no nested buttons, so no
  double-toggle can occur.
- On flip the track/knob or checkbox is repainted in place (CSS provider on
  the control widget itself, per the GTK4 self-scoping rule) and
  `on_change` fires with the new state.
- The switch knob slides via `margin-start` (2 px off, 22 px on, 20 px travel).
- The checkmark glyph is loaded from the CoreIcon assets (feature `coreicon`)
  and recolored white; without the feature the checkbox toggles without a
  visible glyph.
- The color scheme resolves through `elements::resolve_scheme`: explicit
  `.color_scheme(...)` wins, then the running app's scheme, then system
  detection.

## Usage / Example

Run the demo:

```bash
cargo run --example toggles
```

Minimal usage:

```rust
use tontooui::prelude::*;

let wifi = Toggle::new("Wi-Fi")
    .value(true)
    .on_change(|on| println!("wifi: {}", on));

let sync = Toggle::new("Sync iCloud")
    .style(ToggleStyle::Checkbox);
```

## Cross References

- [Button.md](Button.md) -- SwiftUI-style button element (shares the icon
  pipeline)
- [Slider.md](Slider.md) -- spring-physics slider (same scheme resolution)
