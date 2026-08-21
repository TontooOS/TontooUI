# CircularGauge

CircularGauge is a recreation of the SwiftUI "Accessory Circular Gauge"
(iOS/macOS style). It draws a 270-degree open ring with round line caps, a
marker dot that travels clockwise from the lower-left to the lower-right, an
optional center readout (float, percent or static text), optional min/max
labels and an optional caption below the gauge. The displayed value can be
updated at runtime through `set_value`, which redraws the marker and the
center readout live. There is no light/dark toggle: the element follows the
given or detected color scheme automatically.

## Constructor

```rust
pub fn new() -> Self
```

Creates a gauge with value `0.0`, no center readout, no labels and the default
diameter of 96 px.

## Types

### CircularGaugeCenter

```rust
pub enum CircularGaugeCenter {
    None,
    Float(u8),
    Percent,
    Text(String),
}
```

| Variant | Meaning |
|---|---|
| `None` | No center readout (default) |
| `Float(decimals)` | The raw value with the given number of decimals, e.g. `0.420000` |
| `Percent` | The value rounded to a whole percent, e.g. `42` |
| `Text(s)` | A fixed custom string |

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `value` | `value(self, v: f32) -> Self` | Value `0.0..=1.0` (clamped) that positions the marker |
| `label` | `label(self, l: impl Into<String>) -> Self` | Caption below the gauge |
| `center` | `center(self, c: CircularGaugeCenter) -> Self` | Center readout |
| `min_label` | `min_label(self, l: impl Into<String>) -> Self` | Label at the lower-left inside the ring |
| `max_label` | `max_label(self, l: impl Into<String>) -> Self` | Label at the lower-right inside the ring |
| `size` | `size(self, s: f32) -> Self` | Diameter in pixels (default: 96) |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Size from a frame (uses the smaller side) |
| `color_scheme` | `color_scheme(self, c: ColorScheme) -> Self` | Force dark/light (default: system detection) |

## Runtime Update

```rust
pub fn set_value(&self, v: f32)
```

Clamps `v` to `0.0..=1.0`, stores it and — if the gauge was already rendered
with `to_gtk()` — moves the marker and updates the center readout live.

## Getter Methods

| Method | Returns |
|---|---|
| `current_value()` | `f32` -- the current value |
| `label_text()` | `Option<&str>` -- caption, if set |
| `center_readout()` | `&CircularGaugeCenter` -- readout configuration |
| `min_label_text()` | `Option<&str>` -- lower-left label, if set |
| `max_label_text()` | `Option<&str>` -- lower-right label, if set |

## Drawing Helper

```rust
pub fn draw_gauge(cr: &cairo::Context, w: i32, h: i32, value: f32, dark: bool)
```

Draws the ring and marker into an arbitrary cairo context. Used by the element
itself and useful for custom containers.

## Behavior

- Geometry mirrors the SwiftUI reference viewBox: ring radius 35, stroke 5.5,
  marker radius 4.5 with a 3 px contrasting stroke, arc from 135 to 405
  degrees (270 degrees, gap at the bottom), all scaled to the widget size.
- Light mode: black ring, white marker fill with black stroke. Dark mode:
  white ring, black marker fill with white stroke.
- Center readout: `Float` renders small (9 px, regular), `Percent` and `Text`
  render large (22 px, bold). Min/max labels render at 11 px in muted zinc
  tones (`#71717a` light, `#a1a1aa` dark).
- All text uses `SF Pro Display`.
- The element is not interactive (no mouse or keyboard handling).

## Usage / Example

```rust
use tontooui::prelude::*;

// Static gauge
let gauge = CircularGauge::new()
    .value(0.42)
    .center(CircularGaugeCenter::Percent)
    .min_label("0")
    .max_label("100")
    .label("Foo");

// Live-updated gauge
let gauge = CircularGauge::new().center(CircularGaugeCenter::Float(6));
let widget = gauge.to_gtk();
gauge.set_value(0.42); // moves the marker + readout after rendering
```

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [ProgressView.md](ProgressView.md) -- the other circular element (ring drawing via cairo)
- [Slider.md](Slider.md) -- interactive element for driving a gauge value
- [examples/circular_gauge.rs](../examples/circular_gauge.rs) -- demo built on the uikit App API (custom root widget, VStack/HStack layout): three gauges, live slider and dark-mode toggle