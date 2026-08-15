# Slider

Slider is a macOS-style slider with spring physics, pill-shaped white thumb, and grow/squish animation. The thumb scales up on press and wobbles on release using elastic easing.

## Constructor

```rust
pub fn new(min: f32, max: f32) -> Self
```

Creates a new slider with the given range.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `value` | `value(self, v: f32) -> Self` | Set initial value |
| `step` | `step(self, s: f32) -> Self` | Set step increment |
| `label` | `label(self, l: impl Into<String>) -> Self` | Set header label |
| `accent_color` | `accent_color(self, c: Color) -> Self` | Set the fill bar and value label color (thumb stays white) |
| `track_color` | `track_color(self, c: Color) -> Self` | Set the track bar background color |
| `width` | `width(self, w: f32) -> Self` | Set width |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Set size |
| `on_change` | `on_change(self, h: impl Fn(f32) + Send + Sync + 'static) -> Self` | Callback on value change |

## Physics Constants

| Constant | Value | Description |
|---|---|---|
| `SPRING_K` | 47.25 | Spring stiffness for squish animation |
| `DAMP` | 10.8 | Damping for squish spring |
| `VALUE_SPEED` | 18.0 | Speed of value following |
| `WOBBLE_DURATION` | 0.259 | Duration of wobble animation (seconds) |

## ViewContent Implementation

Slider implements `ViewContent`, so it can be wrapped in a `View`:

```rust
let view = View::new(slider).with_frame(0.0, 0.0, 300.0, 80.0);
```

## Usage / Example

```rust
use tontooui::prelude::*;

let slider = Slider::new(0.0, 100.0)
    .value(50.0)
    .step(1.0)
    .label("Volume")
    .accent_color(Color::new(0.016, 0.525, 0.941, 1.0))
    .track_color(Color::from_rgb(51, 51, 51))
    .width(300.0)
    .on_change(|val| println!("Value: {}", val));
```

## Features

- Spring physics value following (stiffness: 47.1, damping: 10.8)
- Pill-shaped white thumb (1.5x wider than tall)
- Thumb grows 1.25x on press with spring animation
- Squish/stretch on drag (velocity-based)
- Elastic wobble on release
- CSS-only rendering (no cairo -- WSLg compatible)
- Full-width click/drag hit zone

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [WheelPicker.md](WheelPicker.md) -- another spring-physics element
- [TextInput.md](TextInput.md) -- text input element
