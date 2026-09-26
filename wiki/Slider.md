# Slider

Horizontal slider in `src/elements/sliders/slider.rs`: basic, stepped,
labeled, ticked, colored and glass variants. Clicking the track animates
the knob there; pressing the knob (or holding) follows the mouse directly.
While pressed the knob turns liquid glass and samples the in-app backdrop
blur (when the shell runs `App::wants_backdrop`).

## Geometry

| Token | Value |
|---|---|
| `SLIDER_TRACK_H` | 6 px line |
| `SLIDER_KNOB_D` | 28 px white knob with soft shadow |
| `SLIDER_KNOB_EXPAND_W` / `SLIDER_KNOB_EXPAND_H` | +4.8 / +3.8 px while pressed (20% larger glass knob) |
| `SLIDER_HEADER_SIZE` / `SLIDER_SMALL_SIZE` | 15 px semibold header / 11 px labels |
| `SLIDER_ANIM_SECONDS` | 0.25 s click-to-point animation |
| `SLIDER_TRACK_DARK` / `SLIDER_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `SLIDER_FILL` | `#007AFF` default value fill |

## Element

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn step(self, step: f64) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn min_label(self, label: impl Into<String>) -> Self
pub fn max_label(self, label: impl Into<String>) -> Self
pub fn value_text(self, f: impl Fn(f64) -> String + 'static) -> Self
pub fn show_ticks(self, show: bool) -> Self
pub fn fill(self, color: Color) -> Self
pub fn track_color(self, color: Color) -> Self
pub fn glass(self, glass: bool) -> Self
pub fn on_change(self, callback: impl FnMut(f64) + 'static) -> Self
pub fn value(&self) -> f64
pub fn is_dragging(&self) -> bool
pub fn set_value(&mut self, value: f64)
pub fn set_theme(&mut self, accent: Color, dark: bool, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
```

- `step(0.0)` (default) is continuous; otherwise the value snaps to
  `min + k * step`. Snapped values drive ticks and labels.
- `title` renders left of the track, `min_label`/`max_label` at the track
  ends, `value_text` centered above (rebuilt live, e.g.
  `|v| format!("Value: {v:.0}°")`).
- `show_ticks` draws a dot per step under the track (max 64).
- `glass(true)` swaps the solid track for translucent frost per
  LiquidGlass stage (same size, same layout).
- `on_change` fires on every value change, including programmatic
  `set_value` and animation landing.
- `is_dragging` is true while the knob is held; return it from
  `App::wants_backdrop` so the shell runs the backdrop blur pass.

## Interaction

`mouse_down` on the knob starts drag-follow (snapped live);
`mouse_down` on the track starts a 0.25 s `CubicOut` tween there (driven
by real frame deltas, so Hz-independent); `mouse_move` follows while
dragging; `mouse_up` ends the drag. The filled part, knob and ticks track
the animated display value, labels track the logical value. Forward all
three mouse methods from the app (see `examples/slider.rs`); the `View`
`mouse_down` / `mouse_up` / `set_hover` do the same, so sliders also
work nested inside `VStack` / `HStack` pages (hover moves drive a held
knob there).

While held the knob grows 20 % with a 0.18 s `CubicOut` ease
(`SLIDER_EXPAND_SECONDS`, reverses mid-flight) and turns into the
shared liquid glass lens
(`fill_lens_glass`): clear magnified center plus a thin 4 px blurred rim,
frost tint, bevel and chroma; on the capture pass the knob (and its shadow)
is omitted so the blur sees the track behind it. Without a backdrop pass
the knob keeps the solid frost tint only.

## Usage / Example

Run `cargo run --example slider`: basic, stepped with ticks, labeled
temperature, 1-5 rating with ticks, red/green/purple colors side by side
and a glass slider in a `VStack`.

```rust
let mut temperature = Slider::new(50.0, 0.0, 100.0)
    .title("Temperature")
    .min_label("0°")
    .max_label("100°")
    .value_text(|v| format!("Value: {v:.0}°"));
temperature.set_theme(accent, true, GlassAmount::Glass);
```

## Cross References

- [Layout.md](Layout.md) – stacks hosting sliders, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App::wants_backdrop`, backdrop blur
- [Animation.md](Animation.md) – tween drivers used by click-to-point
- [Theme.md](Theme.md) – accent, mode and glass stage
- [Glass.md](Glass.md) – glass stage rendering
