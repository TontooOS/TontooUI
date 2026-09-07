# Gauge

Gauge is a SwiftUI-style value indicator for TontooOS. It shows a value within a closed range together with an optional label, current value label and minimum/maximum labels. Rendering is done with Cairo on a `DrawingArea` so the widget blends into the TontooOS dark background `#1d1d1d` (light `#ececec`) and uses `SF Pro Display` for all text.

## Constructor

```rust
pub fn new(value: f32) -> Self
```

Creates a gauge displaying `value` in the default range `0.0..=1.0`.

```rust
pub fn with_label(label: impl Into<String>, value: f32) -> Self
```

Convenience that mirrors SwiftUI `Gauge(value: ...) { Text("Foo") }`.

## GaugeStyle

```rust
pub enum GaugeStyle {
    Default,
    Circular,
    Linear,
    LinearCapacity,
    AccessoryLinear,
    AccessoryLinearCapacity,
    AccessoryCircular,
    AccessoryCircularCapacity,
}
```

| Variant | Description |
|---|---|
| `Default` | Linear capacity — filled bar from leading to trailing (used for the plain `Gauge`). |
| `LinearCapacity` | Filled bar from leading to trailing edges as value increases. |
| `Linear` | Bar with a circular marker at the current value point. |
| `Circular` | Open ring (240 degrees) with a colored marker at the current value. |
| `AccessoryLinear` | Thin accessory bar with a marker dot. |
| `AccessoryLinearCapacity` | Thin accessory capacity bar that fills. |
| `AccessoryCircular` | Small accessory open ring with marker. |
| `AccessoryCircularCapacity` | Small accessory closed ring partially filled to indicate the value. |

## GaugeTint

```rust
pub enum GaugeTint {
    Single(Color),
    Gradient(Vec<Color>),
}
```

Use `.tint(Color)` for a single color or `.tint_gradient(vec![...])` for a linear gradient fill (linear gauges interpolate across the full track width).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `value` | `value(self, v: f32) -> Self` | Set displayed value |
| `in_range` | `in_range(self, min: f32, max: f32) -> Self` | Set min/max range (also `range`) |
| `label` | `label(self, l: impl Into<String>) -> Self` | Label text (SwiftUI `label` closure) |
| `current_value_label` | `current_value_label(self, l: impl Into<String>) -> Self` | Center/current value label ("42", "0.42000") |
| `minimum_value_label` | `minimum_value_label(self, l: impl Into<String>) -> Self` | Left minimum label ("0") |
| `maximum_value_label` | `maximum_value_label(self, l: impl Into<String>) -> Self` | Right maximum label ("100") |
| `gauge_style` | `gauge_style(self, s: GaugeStyle) -> Self` | Set style (also `style`) |
| `tint` | `tint(self, c: Color) -> Self` | Single tint color for fill/marker |
| `tint_gradient` | `tint_gradient(self, colors: Vec<Color>) -> Self` | Gradient tint (linear gauges) |
| `width` | `width(self, w: f32) -> Self` | Preferred width |
| `height` | `height(self, h: f32) -> Self` | Override height (auto per style otherwise) |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Set width + height |
| `to_view` | `to_view(self) -> View` | Wrap in a `View` with auto frame |

Default size is `width 220` and auto height: `Circular 110`, `AccessoryCircular 56`, `AccessoryLinear 34`, otherwise `42`.

## Initializer Variants (SwiftUI parity)

| SwiftUI | TontooUI |
|---|---|
| `Gauge(value: 42, in: 0...100) { Text("Foo") }` | `Gauge::new(42.0).in_range(0.0, 100.0).label("Foo")` |
| `Gauge(value: 0.42) { Text("Foo") } currentValueLabel: { Text("0.42") }` | `Gauge::new(0.42).label("Foo").current_value_label("0.42000")` |
| `Gauge(value: 42, in: 0...100) { Text("Foo") } minimumValueLabel: { Text("0") } maximumValueLabel: { Text("100") }` | `Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").minimum_value_label("0").maximum_value_label("100")` |
| `Gauge(value: 42) { Text("Foo") } currentValueLabel: { Text("42") } minimumValueLabel: { Text("0") } maximumValueLabel: { Text("100") }` | `Gauge::new(42.0).in_range(0.0,100.0).label("Foo").current_value_label("42").minimum_value_label("0").maximum_value_label("100")` |

## Rendering

- **LinearCapacity / Default** — rounded rect track `rgba(0.22,0.22,0.23,1.0)` height `6`, fill with `tint` (or gradient via `cairo::LinearGradient`) width `norm * (width-12)`.
- **Linear** — track `height 5` white/gray, marker circle `r 6` white outer + dark inner, tinted border when custom tint set.
- **Accessory linear variants** — same but thinner (`2.5-4`) and smaller marker (`3.5`).
- **Circular** — open arc `135deg` to `405deg` (240deg sweep), stroke `5`, track gray, marker white ring + tint fill at `start + norm*sweep`.
- **AccessoryCircularCapacity** — closed ring `stroke 5`, full gray track, tint arc from top `-90deg` length `norm*360deg` with round caps.

All text uses `SF Pro Display`: label `11px rgba(235,235,245,0.85)`, min/max `9px rgba(235,235,245,0.55)`, current `10px`.

## ViewContent / Widget

`Gauge` implements both `ViewContent` and `Widget`, so it can be used directly in `VStack`/`HStack` or wrapped in a `View`:

```rust
let g = Gauge::new(0.42).label("Foo");
let view = View::new(g).with_frame(0.0, 0.0, 220.0, 42.0);
// or
VStack::new().child(Gauge::new(42.0).in_range(0.0, 100.0).gauge_style(GaugeStyle::Circular))
```

## Usage / Example

```rust
use tontooui::prelude::*;
use tontooui::{Gauge, GaugeStyle};

fn main() {
    let mut app = App::new("Gauges", 980, 900);

    // Min / Max / Current
    let g1 = Gauge::new(42.0).in_range(0.0, 100.0)
        .label("Foo").current_value_label("42")
        .minimum_value_label("0").maximum_value_label("100");

    // Circular style
    let g2 = Gauge::new(0.42).label("Foo").gauge_style(GaugeStyle::Circular);

    // Linear capacity with gradient tint
    let g3 = Gauge::new(42.0).in_range(0.0, 100.0)
        .gauge_style(GaugeStyle::LinearCapacity)
        .tint_gradient(vec![Color::from_rgb(255,204,0), Color::from_rgb(255,69,58)]);

    app.set_root(VStack::new().spacing(16.0).child(g1).child(g2).child(g3));
    app.run();
}
```

See `examples/gauges.rs` for all 11 replica cards: Min Max Current Value Gauge, Current Value Gauge, Gauge, Gauge Colors, CircularGaugeStyle, LinearGaugeStyle, AccessoryLinearCapacityGaugeStyle, AccessoryLinearGaugeStyle, LinearCapacityGaugeStyle, AccessoryCircularCapacityGaugeStyle, AccessoryCircularGaugeStyle.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- spring-physics slider (related linear control)
- [ProgressView.md](ProgressView.md) -- circular progress indicator (related ring drawing)
