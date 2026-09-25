# Progress

Progress category in `src/elements/progress/`: `LinearProgress` in
`linear.rs` is a thin rounded bar fed by the app. The app keeps
sending progress numbers while the shown fill chases the target at
a limited speed, so the bar always moves slowly and glides through
brief app stalls. Display-only (no mouse handling). The fill
follows the system accent (Multicolor renders blue) unless the dev
sets it manually with `fill`. `Spinner` in `spinner.rs` is the
indeterminate counterpart: twelve rotating spokes with a fade
trail, always gray unless the dev sets a color (no accent).

## Geometry

| Token | Value |
|---|---|
| `PROGRESS_TRACK_H` / `PROGRESS_RADIUS` | 8 px bar, 4 px radius (rounded ends) |
| `PROGRESS_TITLE_SIZE` / `PROGRESS_TITLE_GAP` | 13 px centered title / 8 px title gap |
| `PROGRESS_SPEED` | 0.12 fraction per second chase speed |
| `PROGRESS_TRACK_DARK` / `PROGRESS_TRACK_LIGHT` | `#3A3A3C` / `#E5E5E5` |
| `PROGRESS_FILL` | `#007AFF` default bar fill |

## Element

```rust
pub fn new() -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn speed(self, per_second: f64) -> Self
pub fn fill(self, color: Color) -> Self
pub fn track_color(self, color: Color) -> Self
pub fn on_complete(self, callback: impl FnMut() + 'static) -> Self
pub fn set_progress(&mut self, value: f64)
pub fn set_speed(&mut self, per_second: f64)
pub fn progress(&self) -> f64
pub fn displayed(&self) -> f64
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

- `set_progress` takes the app-reported target 0.0-1.0 (clamped);
  `progress` reads it back, `displayed` reads the lagging fill.
  Per frame the fill moves toward the target by at most
  `speed * dt` (dt capped, so background stalls never jump).
- `speed` trades smoothness/buffer against tightness: lower glides
  longer through stalls, higher tracks the app closer. Minimum
  0.001/s.
- `on_complete` fires once when the shown fill reaches full while
  the target is full; dropping the target below full re-arms it.
- `fill` wins over the system accent until cleared; `track_color`
  wins over the mode gray.
- Unfocused windows desaturate the bar like the rest of the
  palette. No mouse methods: forward nothing (see
  `examples/progress.rs`, titlebar only).

## Spinner

```rust
pub fn new() -> Self
pub fn color(self, color: Color) -> Self
pub fn text(self, text: impl Into<String>) -> Self
pub fn head(&self) -> usize
pub fn set_dark(&mut self, dark: bool)
```

| Token | Value |
|---|---|
| `SPINNER_SPOKES` | 8 spokes, one rough revolution per second |
| `SPINNER_R_OUT` / `SPINNER_R_IN` | 12 px outer / 7 px inner spoke radius |
| `SPINNER_SPOKE_W` | 3 px rounded spokes |
| `SPINNER_STEP_SECONDS` | 1/8 s per spoke step |
| `SPINNER_TAIL_ALPHA` | 0.15 lightest trail spoke |
| `SPINNER_TEXT_SIZE` / `SPINNER_TEXT_GAP` | 13 px caption / 8 px caption gap |
| `SPINNER_GRAY` | `#8E8E93` default spoke color |

- The head spoke is fully opaque; older spokes fade linearly down
  to `SPINNER_TAIL_ALPHA`. Steps advance with wall-clock time, so
  the spin is Hz-independent.
- `color` sets spokes and caption together; there is intentionally
  no accent following. `set_dark` only re-tints the caption gray.
- No mouse methods: forward nothing.

## Usage / Example

Run `cargo run --example progress`: pink bar fed by a fake app
that reports jumpy progress with stalls (the titlebar shows the
raw target percent), plus a gray `Loading...` spinner below.

```rust
let mut spin = Spinner::new().text("Loading...");
spin.set_dark(true);
```

```rust
let mut loading = LinearProgress::new()
    .speed(0.12)
    .fill(Color::from_rgb8(0xff, 0x2d, 0x99))
    .on_complete(|| println!("done"));
loading.set_theme(accent, true);
loading.set_progress(0.42);
```

## Cross References

- [Gauge.md](Gauge.md) – value bars, manual `fill`, fill animation
- [Slider.md](Slider.md) – value range, click animation
- [Layout.md](Layout.md) – stacks hosting progress bars, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` shell
- [Animation.md](Animation.md) – frame deltas driving the chase
- [Theme.md](Theme.md) – accent, mode and Multicolor default
