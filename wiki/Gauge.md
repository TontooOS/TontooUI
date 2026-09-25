# Gauge

Gauge category in `src/elements/gauges/`: `Gauge` in `gauge.rs`
displays a value fraction as a rounded bar with a centered title
above it. Display-only (no mouse handling); value changes tween to
the new fill width. The fill follows the system accent (Multicolor
renders blue) unless the dev sets it manually with `fill`.

## Geometry

| Token | Value |
|---|---|
| `GAUGE_TRACK_H` / `GAUGE_RADIUS` | 12 px bar, 6 px radius (rounded ends) |
| `GAUGE_TITLE_SIZE` / `GAUGE_TITLE_GAP` | 13 px centered title / 8 px title gap |
| `GAUGE_ANIM_SECONDS` | 0.25 s fill tween |
| `GAUGE_TRACK_DARK` / `GAUGE_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `GAUGE_FILL` | `#007AFF` default value fill |

## Element

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn fill(self, color: Color) -> Self
pub fn track_color(self, color: Color) -> Self
pub fn value(&self) -> f64
pub fn fraction(&self) -> f32
pub fn set_value(&mut self, value: f64)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

- `new` clamps the value into `min..=max` (`min`/`max` swap-safe);
  `fraction` is 0.0 for an empty range.
- `set_value` clamps and tweens the fill with a 0.25 s `CubicOut`
  tween (driven by real frame deltas, so Hz-independent).
- `fill` wins over the system accent until cleared; `track_color`
  wins over the mode gray.
- Unfocused windows desaturate the gauge like the rest of the
  palette. No mouse methods: forward nothing (see
  `examples/gauge.rs`, titlebar only).

## Usage / Example

Run `cargo run --example gauge`: green `Progress`, theme-accent and
red `Storage` gauges in a `VStack`.

```rust
let mut progress = Gauge::new(0.6, 0.0, 1.0)
    .title("Progress")
    .fill(Color::from_rgb8(0x34, 0xc7, 0x59));
progress.set_theme(accent, true);
progress.set_value(0.8);
```

## Cross References

- [Slider.md](Slider.md) – value range, manual `fill`, click animation
- [Layout.md](Layout.md) – stacks hosting gauges, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell
- [Animation.md](Animation.md) – tween driver used by the fill
- [Theme.md](Theme.md) – accent, mode and Multicolor default
