# Gauge

Gauge category in `src/elements/gauges/`: `Gauge` in `gauge.rs`
displays a value fraction as a rounded bar with a centered title
above it, `LinearGauge` in `linear.rs` shows a value label left of
a thin track line with a knob marker. Both are display-only (no
mouse handling); value changes tween to the new position. The
`Gauge` fill follows the system accent (Multicolor renders blue)
unless the dev sets it manually with `fill`; `LinearGauge` stays
monochrome in every theme.

## Geometry

| Token | Value |
|---|---|
| `GAUGE_TRACK_H` / `GAUGE_RADIUS` | 12 px bar, 6 px radius (rounded ends) |
| `GAUGE_TITLE_SIZE` / `GAUGE_TITLE_GAP` | 13 px centered title / 8 px title gap |
| `GAUGE_ANIM_SECONDS` | 0.25 s fill tween |
| `GAUGE_SIDE_SIZE` / `GAUGE_SIDE_GAP` | 11 px side labels / 8 px label gap |
| `GAUGE_VALUE_SIZE` / `GAUGE_VALUE_GAP` | 13 px value caption / 8 px caption gap |
| `GAUGE_TRACK_DARK` / `GAUGE_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `GAUGE_FILL` | `#007AFF` default value fill |

## Element

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn min_label(self, label: impl Into<String>) -> Self
pub fn max_label(self, label: impl Into<String>) -> Self
pub fn value_text(self, f: impl Fn(f64) -> String + 'static) -> Self
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
- `min_label`/`max_label` draw left and right of the bar (the track
  shrinks between them); `value_text` draws a live caption centered
  under the bar (e.g. `|v| format!("{v:.0}°")` tracks the logical
  value). Without labels the gauge is the plain bar from the
  reference.
- `set_value` clamps and tweens the fill with a 0.25 s `CubicOut`
  tween (driven by real frame deltas, so Hz-independent).
- `fill` wins over the system accent until cleared; `track_color`
  wins over the mode gray.
- Unfocused windows desaturate the gauge like the rest of the
  palette. No mouse methods: forward nothing (see
  `examples/gauge.rs`, titlebar only).

## LinearGauge

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn value_text(self, f: impl Fn(f64) -> String + 'static) -> Self
pub fn value(&self) -> f64
pub fn fraction(&self) -> f32
pub fn set_value(&mut self, value: f64)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

| Token | Value |
|---|---|
| `LINEAR_TRACK_H` | 6 px track line |
| `LINEAR_KNOB_R` / `LINEAR_RING_R` / `LINEAR_DOT_R` | 8 / 5.5 / 2.5 px fixed knob |
| `LINEAR_VALUE_SIZE` / `LINEAR_VALUE_GAP` | 15 px value label / 10 px label gap |

- `value_text` draws left of the track (e.g. `|v| format!("{v:.0}%")`
  tracks the logical value); without it the gauge is just the line
  plus knob. An optional `title` centers above like `Gauge`.
- `set_value` clamps and tweens the knob with a 0.25 s `CubicOut`
  tween (driven by real frame deltas, so Hz-independent).
- Track, knob and label use the label color on the mode background
  (black on light, white on dark); `set_theme` takes the accent for
  API parity but the gauge stays monochrome.
- Unfocused windows desaturate the gauge like the rest of the
  palette. No mouse methods: forward nothing.

## CircularGauge

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn label(self, label: impl Into<String>) -> Self
pub fn value_text(self, f: impl Fn(f64) -> String + 'static) -> Self
pub fn fill(self, color: Color) -> Self
pub fn value(&self) -> f64
pub fn fraction(&self) -> f32
pub fn set_value(&mut self, value: f64)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

| Token | Value |
|---|---|
| `CIRC_RING_R` / `CIRC_TRACK_W` | 35 px ring radius / 6 px stroke |
| `CIRC_START` / `CIRC_SWEEP` | 135 deg start / 270 deg sweep (gap at bottom) |
| `CIRC_VALUE_SIZE` / `CIRC_LABEL_SIZE` | 17 px value / 7.5 px caption |
| `CIRC_KNOB_R` / `CIRC_KNOB_RING_R` / `CIRC_DOT_R` | 4 / 2.75 / 1.25 px fixed knob |
| `CIRC_FILL` | `#007AFF` manual ring fill |

- The dial is a fixed 84 px square: full 270-degree arc (no gray
  track, like the reference) with round caps, a knob marker at the
  value angle, centered value text and the caption below it inside
  the bottom gap.
- `set_value` clamps and tweens the knob along the arc with a
  0.25 s `CubicOut` tween (driven by real frame deltas, so
  Hz-independent).
- Ring and knob default to the label color (black in light mode,
  like the reference); `fill` wins over it. `set_theme` takes the
  accent for API parity but the dial stays monochrome unless filled
  by hand.
- Unfocused windows desaturate the dial like the rest of the
  palette. No mouse methods: forward nothing.

## CapacityGauge

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn value_text(self, f: impl Fn(f64) -> String + 'static) -> Self
pub fn fill(self, color: Color) -> Self
pub fn track_color(self, color: Color) -> Self
pub fn value(&self) -> f64
pub fn fraction(&self) -> f32
pub fn set_value(&mut self, value: f64)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

| Token | Value |
|---|---|
| `CAP_RING_R` / `CAP_TRACK_W` | 30 px ring radius / 5 px stroke |
| `CAP_START` / `CAP_SWEEP` | top start / full circle sweep |
| `CAP_VALUE_SIZE` | 15 px centered value |
| `CAP_TRACK_DARK` / `CAP_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `CAP_FILL` | `#007AFF` manual value fill |

- The dial is a fixed 73 px square: gray background track, full
  circle, with the value arc sweeping clockwise from the top and
  round caps, plus centered value text. No knob, no caption, like
  the reference.
- `set_value` clamps and tweens the arc with a 0.25 s `CubicOut`
  tween (driven by real frame deltas, so Hz-independent).
- Arc and track default to label color and mode gray (black on
  light, like the reference); `fill`/`track_color` win over them.
  `set_theme` takes the accent for API parity but the dial stays
  monochrome unless set by hand.
- Unfocused windows desaturate the dial like the rest of the
  palette. No mouse methods: forward nothing.

## Usage / Example

Run `cargo run --example gauge`: green `Progress`, theme-accent,
red `Storage` and labeled green `Temperature` (`0°`/`100°`,
`72°`) gauges in a `VStack`, a `60%` linear gauge, a `70%`
`Battery` dial and a `65%` capacity ring below.

```rust
let mut progress = Gauge::new(0.6, 0.0, 1.0)
    .title("Progress")
    .fill(Color::from_rgb8(0x34, 0xc7, 0x59));
progress.set_theme(accent, true);
progress.set_value(0.8);

let mut temperature = Gauge::new(72.0, 0.0, 100.0)
    .title("Temperature")
    .min_label("0°")
    .max_label("100°")
    .value_text(|v| format!("{v:.0}°"));

let mut level = LinearGauge::new(60.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
level.set_theme(accent, true);
level.set_value(80.0);

let mut battery = CircularGauge::new(70.0, 0.0, 100.0)
    .label("Battery")
    .value_text(|v| format!("{v:.0}%"));
battery.set_theme(accent, false);

let mut capacity = CapacityGauge::new(65.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
capacity.set_theme(accent, false);
```

## Cross References

- [Slider.md](Slider.md) – value range, manual `fill`, click animation
- [Layout.md](Layout.md) – stacks hosting gauges, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell
- [Animation.md](Animation.md) – tween driver used by the fill
- [Theme.md](Theme.md) – accent, mode and Multicolor default
