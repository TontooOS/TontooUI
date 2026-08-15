# ProgressView

`ProgressView` is a macOS-style loading indicator with two modes. Without a value it renders an
indeterminate 8-tick spinner; with a `.value(v)` it renders a determinate circular progress ring.
Both modes support an optional label and sub-label below the indicator, matching the SwiftUI
`ProgressView` layout.

## Constructor

```rust
pub fn new() -> Self
```

Creates an indeterminate spinner with the default accent color and size (32 px diameter).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `value` | `value(self, v: f32) -> Self` | Set progress `0.0..=1.0` (clamped); switches to the determinate ring |
| `label` | `label(self, l: impl Into<String>) -> Self` | Set the main label below the indicator |
| `sub_label` | `sub_label(self, s: impl Into<String>) -> Self` | Set the secondary (smaller) label |
| `accent_color` | `accent_color(self, c: Color) -> Self` | Set spinner tick / progress ring color |
| `track_color` | `track_color(self, c: Color) -> Self` | Set the ring background track color |
| `size` | `size(self, s: f32) -> Self` | Set the indicator diameter |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Set size using the smaller side |

## Getters

| Method | Returns | Description |
|---|---|---|
| `progress` | `Option<f32>` | Current value, or `None` for the indeterminate spinner |
| `label_text` | `Option<&str>` | Main label, if set |
| `sub_label_text` | `Option<&str>` | Sub label, if set |

## Modes

| Mode | Condition | Rendering |
|---|---|---|
| Indeterminate | `value == None` | macOS spinner: 8 rounded ticks rotating in 8 discrete steps (45 deg every 125 ms), each tick with decreasing opacity |
| Determinate | `value == Some(v)` | Circular ring: background track plus an accent-colored arc from 12 o'clock covering `v * 100%`, rounded line caps |

## ViewContent Implementation

`ProgressView` implements `ViewContent`, so it can be wrapped in a `View`:

```rust
let view = View::new(spinner);
```

## Usage / Example

```rust
use tontooui::prelude::*;

// Indeterminate spinner
let spinner = ProgressView::new().label("Loading...");

// Determinate ring (42%)
let ring = ProgressView::new()
    .value(0.42)
    .label("Foo")
    .sub_label("bar")
    .accent_color(Color::from_rgb(0, 122, 255))
    .size(48.0);
```

## Features

- Single element, two modes (indeterminate spinner / determinate ring)
- 8-tick macOS spinner with stepped rotation and opacity trail
- Circular ring with rounded progress arc and configurable track color
- Labels rendered in SF Pro Display (13 px main, 11 px sub)
- Cairo-rendered via `DrawingArea` (WSLg compatible)

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- another spring-physics element
- [TextInput.md](TextInput.md) -- text input element