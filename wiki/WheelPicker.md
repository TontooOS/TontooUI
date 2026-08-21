# WheelPicker

WheelPicker is a macOS/iOS style scroll wheel picker with spring physics,
snap-to-center locking, and a 3D fade/scale effect. Items are positioned
absolutely around a fixed center selection bar, so the drum animates cleanly
while dragging, flinging or scrolling.

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
|---|---|---|
| `selected_value()` | `&str` | Currently selected value |

## Rendering Model

The wheel is a `gtk::Overlay` sized `width x height` (default `280 x 176`).
Every item is an overlay child positioned with `margin_top` so its center
sits at the middle of the container when it is selected. A centered
"lock" bar (rounded highlight) marks the selection row.

Items are styled per frame from their distance to the center:

| Distance factor (0..=1) | Effect |
|---|---|
| 0 (selected) | Full opacity (1.0), bold, base font `18pt`, scale `1.0` |
| 1 (1.75 rows away) | 35% text alpha, normal weight, scale `0.85` |
| Middle | Linear interpolation between the two states |

Opacity (rendered as Pango foreground-alpha, so the numbers themselves fade)
and font scale follow the same curve as the macOS/iOS drum:
`opacity = 1 - factor * 0.65` and `scale = 1 - factor * 0.15`, where
`factor = (dist * ITEM_HEIGHT / 70).clamp(0, 1)`. All styling is applied via
native Pango attributes (`AttrSize`, `AttrFloat::new_scale`, `AttrWeight`,
`AttrString::new_family`) on every frame, so the fade/3D effect animates
smoothly while the drum moves.

## Physics Constants

The spring physics match the compositor's `WheelPickerState`:

| Constant | Value | Description |
|---|---|---|
| `SPRING_STIFFNESS` | 280.0 | Spring stiffness (higher = snappier) |
| `SPRING_DAMPING` | 26.0 | Damping factor (controls overshoot) |
| `ITEM_HEIGHT` | 40.0 | Height of each item row |

## Interaction

| Gesture | Behavior |
|---|---|
| Mouse drag | Items follow the pointer 1:1; drag velocity is tracked |
| Mouse release | Spring locks the wheel onto the nearest item (fling + settle) |
| Mouse wheel / trackpad | Scroll events move the spring target; the drum glides toward it with a visible spring animation |
| Scroll stop | ~120 ms after the last scroll event the target is rounded and the wheel locks onto the nearest index |

The scroll controller consumes the event (`Propagation::Stop`), so the wheel
"locks" scrolling while the pointer is over it, like a real iOS drum. The
accumulated scroll target (not the lagging offset) decides the final lock, so
a fast scroll keeps its momentum and does not snap back a step.

## ViewContent Implementation

WheelPicker implements `ViewContent`, so it can be wrapped in a `View`:

```rust
let view = View::new(picker).with_frame(0.0, 0.0, 280.0, 176.0);
```

## Usage / Example

```rust
use tontooui::prelude::*;

let picker = WheelPicker::new()
    .items(["1", "2", "3", "4", "5", "6"])
    .selected("3")
    .frame(280.0, 176.0)
    .on_change(|value| println!("Selected: {}", value));
```

## Animation Loop

WheelPicker runs a 60 FPS tick loop only while the drum is dragging,
scrolling, or the spring is still settling. As soon as everything settles the
loop stops itself and is restarted by the next gesture. An idle wheel therefore
does not relayout all item labels at 60 FPS forever.

## Features

- Spring physics snap-to-center with fling on release (stiffness: 280, damping: 26)
- Animated mouse-wheel / trackpad scrolling (drum glides toward the scroll target, then locks)
- 3D fading + font scaling (items fade and shrink as they move from center)
- Center selection lock bar
- macOS glass-style appearance (dark theme, `overflow: hidden` rounded drum)
- Drag, fling and mouse-wheel / trackpad interaction
- `on_change` fires only when the selected index actually changes

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- another spring-physics element
