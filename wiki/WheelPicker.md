# WheelPicker

WheelPicker is a macOS/iOS style scroll wheel picker with spring physics, snap-to-center behavior, and 3D fading effect. Items fade and scale based on their distance from the center selection bar.

## Constructor

```rust
pub fn new() -> Self
```

Creates an empty wheel picker.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `items` | `items<I, S>(self, iter: I) -> Self` | Set items from iterator |
| `item` | `item(self, text: impl Into<String>) -> Self` | Add a single item |
| `selected` | `selected(self, value: impl Into<String>) -> Self` | Set initial selection by value |
| `selected_index` | `selected_index(self, index: usize) -> Self` | Set initial selection by index |
| `accent_color` | `accent_color(self, color: Color) -> Self` | Set highlight color |
| `frame` | `frame(self, width: f32, height: f32) -> Self` | Set size |
| `on_change` | `on_change(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self` | Callback on selection change |

## Accessor Methods

| Method | Return | Description |
|---|---|---|---|
| `selected_value()` | `&str` | Currently selected value |

## Physics Constants

The spring physics match the compositor's `WheelPickerState`:

| Constant | Value | Description |
|---|---|---|
| `SPRING_STIFFNESS` | 280.0 | Spring stiffness (higher = snappier) |
| `SPRING_DAMPING` | 26.0 | Damping factor (controls overshoot) |
| `ITEM_HEIGHT` | 40.0 | Height of each item row |

## ViewContent Implementation

WheelPicker implements `ViewContent`, so it can be wrapped in a `View`:

```rust
let view = View::new(picker).with_frame(0.0, 0.0, 280.0, 200.0);
```

## Usage / Example

```rust
use tontooui::prelude::*;

let picker = WheelPicker::new()
    .items(["1", "2", "3", "4", "5", "6"])
    .selected("3")
    .frame(280.0, 200.0)
    .on_change(|value| println!("Selected: {}", value));
```

## Features

- Spring physics snap-to-center (stiffness: 280, damping: 26)
- 3D fading/scaling effect (items fade as they move from center)
- Center selection highlight bar
- macOS glass-style appearance (dark theme)
- Drag + scroll interaction via `GestureClick` and `EventControllerMotion`

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- another spring-physics element
