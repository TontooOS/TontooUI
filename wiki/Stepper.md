# Stepper

Basic stepper in `src/elements/steppers/stepper.rs`: two chevron halves
sharing one rounded control. Each press moves the value by `step` inside
`min..=max`. A side that cannot move further draws its chevron dimmed and
ignores clicks, so a `0..=100` range grays out the down chevron at `0` and
the up chevron at `100`.

## Geometry

| Token | Value |
|---|---|
| `STEPPER_W` / `STEPPER_H` | 34 px wide, 56 px tall (swapped when horizontal) |
| `STEPPER_RADIUS` | 8 px corner radius |
| `STEPPER_DIV_INSET` | 6 px divider inset |
| `STEPPER_CHEV_STROKE` | 2 px round-cap chevron stroke |
| `STEPPER_DISABLED_ALPHA` | 0.35 alpha for the exhausted chevron |
| `STEPPER_BG_DARK` / `STEPPER_BG_LIGHT` | `#2C2C2E` / `#E9E9EB` |
| `STEPPER_DIVIDER_DARK` / `STEPPER_DIVIDER_LIGHT` | white 36 alpha / black 31 alpha |

The control draws a small contact shadow under the body, a pressed or
hovered half highlight clipped to the rounded body, a divider between the
halves and one chevron per half (up/down when vertical, left/right when
horizontal).

## Element

```rust
pub fn new(value: f64, min: f64, max: f64) -> Self
pub fn step(self, step: f64) -> Self
pub fn range(self, min: f64, max: f64) -> Self
pub fn orientation(self, orientation: StepperOrientation) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_change(self, callback: impl FnMut(f64) + 'static) -> Self
pub fn value(&self) -> f64
pub fn set_value(&mut self, value: f64)
pub fn set_step(&mut self, step: f64)
pub fn set_range(&mut self, min: f64, max: f64)
pub fn increment_enabled(&self) -> bool
pub fn decrement_enabled(&self) -> bool
pub fn increment(&mut self)
pub fn decrement(&mut self)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_disabled(&mut self, disabled: bool)
```

- `step(1.0)` (default) counts 1, 2, 3; `step(10.0)` counts 10, 20, 30.
  Non-positive steps keep the previous step.
- `range` normalizes swapped bounds and clamps the value into them.
- `increment_enabled` is false at `max`, `decrement_enabled` is false at
  `min`; either is false when the whole control is disabled.
- `set_value` clamps into `min..=max` and fires `on_change` only when the
  value actually changes.
- `set_theme` drives the pressed chevron (accent) plus the mode grays;
  unfocused windows desaturate the control like the rest of the palette.

### Orientation

```rust
pub enum StepperOrientation {
    Vertical,
    Horizontal,
}
```

`Vertical` (default) stacks the increment chevron on top and the
decrement chevron at the bottom with a horizontal divider, matching the
reference control. `Horizontal` puts decrement on the left and increment
on the right with a vertical divider. `measure` swaps the intrinsic size
accordingly.

## Interaction

`mouse_down` hits the top/bottom half (vertical) or left/right half
(horizontal) and steps immediately, firing `on_change`. Clicks on an
exhausted side are ignored and leave no pressed state. `mouse_up` clears
the pressed half. `set_hover` tracks the hovered half for a subtle
highlight. Forward `mouse_down`, `mouse_up` and `set_hover` from the app
(see `examples/stepper.rs`).

## Usage / Example

Run `cargo run --example stepper`: a basic 0-10 stepper, a 0-100 stepper
with `step(10.0)` and a horizontal 0-100 stepper with `step(5.0)`, each
next to a live value label.

```rust
let mut tens = Stepper::new(30.0, 0.0, 100.0).step(10.0);
tens.set_theme(accent, true);
```

## Cross References

- [Layout.md](Layout.md) – stacks hosting steppers, `View` trait
- [Renderer.md](Renderer.md) – frame loop, `App` mouse forwarding
- [Theme.md](Theme.md) – accent and mode driving the control
- [Slider.md](Slider.md) – stepped values and `on_change` pattern
