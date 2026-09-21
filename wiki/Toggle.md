# Toggle

Boolean toggle in `src/elements/toggles/toggle.rs`: switch, button and
checkbox styles with an optional label, an optional leading SF Symbol
(CoreIcon) badge and an `on_toggle` callback. Clicking the row toggles
the state on release; the switch knob slides over with a 0.20 s
`CubicOut` tween while the track crossfades from gray to the on-color.
The on-color follows the system accent (Multicolor renders blue, like
sliders) unless the dev sets it manually with `fill`.

## Geometry

| Token | Value |
|---|---|
| `TOGGLE_SWITCH_W` / `TOGGLE_SWITCH_H` | 51 px / 31 px track (iOS measure) |
| `TOGGLE_KNOB_D` | 27 px white knob with soft shadow |
| `TOGGLE_ANIM_SECONDS` | 0.20 s knob slide |
| `TOGGLE_BOX` / `TOGGLE_BOX_RADIUS` | 22 px box, 6 px radius |
| `TOGGLE_ICON_BOX` / `TOGGLE_ICON_RADIUS` | 28 px badge, 7 px radius |
| `TOGGLE_ICON_GLYPH` | 16 px glyph inside the badge |
| `TOGGLE_LABEL_SIZE` | 17 px row label |
| `TOGGLE_GAP` | 8 px badge/box/control-to-label gap |
| `TOGGLE_OFF_DARK` / `TOGGLE_OFF_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `TOGGLE_ACCENT` | `#007AFF` default on-color |

## Style

```rust
pub enum ToggleStyle {
    Switch,
    Button,
    Checkbox,
}
```

`Switch` draws the label left and the 51x31 track right (with the badge
before the label when an icon is set). `Checkbox` draws the 22 px box
left and the label right. `Button` draws a rounded button (8 px radius,
same padding as `Button`) that fills gray when off and accent with
white text when on; the icon renders inside the button.

```rust
pub fn style(self, style: ToggleStyle) -> Self
pub fn set_theme(&mut self, accent: Color, dark: bool)
```

## Element

```rust
pub fn new(label: impl Into<String>) -> Self
pub fn on(self, on: bool) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn fill(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_toggle(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn is_on(&self) -> bool
pub fn set_on(&mut self, on: bool)
pub fn toggle(&mut self)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_focused(&mut self, focused: bool)
pub fn set_hover(&mut self, x: f32, y: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- `on` sets the initial state without animation and without firing
  `on_toggle`; `set_on` sets it immediately and fires `on_toggle` when
  the state changed; `toggle` flips it with knob animation and fires
  `on_toggle`.
- `icon` uses the CoreIcon spelling (e.g. `"airplane"`, `"wifi"`). For
  switch/checkbox it draws the gray settings-row badge; for button
  style the glyph renders inside the button. A missing icon draws the
  row without badge.
- `fill` wins over the system accent until cleared. The on-color
  follows the system default/color unless the dev sets it by hand.
- `set_theme` takes the palette accent plus the dark mode flag, like
  `Slider::set_theme` without the glass stage.
- Unfocused windows desaturate the toggle like the rest of the palette.

## Interaction

`mouse_down` inside the row arms the toggle; `mouse_up` inside flips
the state with animation (`View::mouse_up` does the same for boxed
children). Pressing inside and releasing outside keeps the state.
`set_hover` tracks the hover position for the button press/hover
overlay. Forward `mouse_down`, `set_hover` and `mouse_up` from the app
(see `examples/toggle.rs`). The animation is driven by real frame
deltas, so it is Hz-independent.

## Usage / Example

Run `cargo run --example toggle` (needs the CoreIcon assets, e.g.
`COREICON_ASSETS_DIR=../CoreIcon/assets/icons` on dev checkouts):
switch, button and checkbox rows, `Airplane Mode` / `Wi-Fi` /
`Bluetooth` settings rows and a fixed green switch in a `VStack`, on
count in the titlebar.

```rust
let mut wifi = Toggle::new("Wi-Fi")
    .icon("wifi")
    .on(true)
    .on_toggle(|on| println!("wifi: {on}"));
wifi.set_theme(accent, true);
```

## Cross References

- [Slider.md](Slider.md) – accent fill, manual `fill`, click animation
- [Button.md](Button.md) – button metrics, icon loading, press overlay
- [Layout.md](Layout.md) – stacks hosting toggles, `View` trait
- [Renderer.md](Renderer.md) – `ImageLoader`, frame loop, `App`
- [Animation.md](Animation.md) – tween driver used by the knob slide
- [Theme.md](Theme.md) – accent, mode and Multicolor default
