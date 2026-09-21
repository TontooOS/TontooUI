# Layout

Stack containers that arrange elements: `VStack`, `HStack`, `ZStack` plus
`Spacer` for flexible space. Stacks measure children first (fonts needed
for text), then assign rects: fixed children keep intrinsic size, flex
children share the remaining space.

## View

```rust
pub trait View {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32);
    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32);
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem);
    fn flex(&self) -> f32 { 0.0 }
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

Everything visible is a `View`. Intrinsic size, rect assignment and drawing
in logical px. `flex` is the share of remaining stack space (zero means
fixed). `as_any_mut` powers typed child access for state updates. All
built-in views (`Text`, `TextInput`, `Titlebar`, stacks, `Spacer`,
modifiers) implement it. An `App` (see [Renderer.md](Renderer.md)) owns the
root view tree and forwards events into it.

`Text`, `TextInput` and `Titlebar` keep their inherent `draw` methods, so
direct callers work unchanged; stacks use the trait through
`Box<dyn Element>`. `TextInput` drawn through a stack never blinks (stacks
have no clock); direct callers pass the blink state explicitly.

```rust
pub enum Align {
    Leading,
    Center,
    Trailing,
}
```

Cross-axis alignment (`Leading` default). `ZStack` uses it on both axes.

## VStack

```rust
pub fn new() -> Self
pub fn spacing(self, px: f32) -> Self
pub fn align(self, align: Align) -> Self
pub fn child(self, child: impl View + 'static) -> Self
```

Default spacing 8 px, default align `Leading`. Children stack top to
bottom at intrinsic height; flex children share the leftover height. Width
is capped at the stack width and positioned per `align`.

## HStack

Same builders as `VStack`, mirrored: children sit left to right at
intrinsic width, flex children share the leftover width, height is capped
and positioned per `align`.

## ZStack

```rust
pub fn new() -> Self
pub fn align(self, align: Align) -> Self
pub fn child(self, child: impl View + 'static) -> Self
```

Overlay: every child gets the stack rect at intrinsic size, positioned per
`align` (default `Center`) on both axes. Draw order follows child order.

## Spacer

```rust
pub fn new() -> Self
pub fn min_size(self, px: f32) -> Self
pub fn factor(self, factor: f32) -> Self
```

Empty flex space. Measures `(min, min)`, draws nothing, reports `factor`
(default 1.0) from `flex` so stacks distribute remaining space
proportionally.

## Child Access

```rust
pub fn child_mut<T: View + 'static>(&mut self, index: usize) -> Option<&mut T>
```

Available on all three stacks. Typed access by position for state updates
(typing into a `TextInput`, changing a `Text`):

```rust
if let Some(input) = stack.child_mut::<TextInput>(2) {
    input.insert(text);
}
```

Returns `None` for a wrong index or type.

## Modifiers

Single-child wrappers, composable like everything else.

```rust
pub struct Padding;
impl Padding {
    pub fn all(child: impl View + 'static, px: f32) -> Self;
}
```

Uniform padding on every side. Measures child plus `2 * px`, offsets the
child rect inward on place.

```rust
pub struct Background;
impl Background {
    pub fn new(child: impl View + 'static, color: Color) -> Self;
    pub fn radius(self, px: f32) -> Self;
}
```

Rounded background behind the child, sized to the placed rect (same size
the child gets). Draws the rect first, then the child.

```rust
pub struct Frame;
impl Frame {
    pub fn new(child: impl View + 'static, width: f32, height: f32) -> Self;
}
```

Fixed-size box. Measures `(width, height)`; the child keeps intrinsic
size, top-leading aligned.

All three expose `child_mut::<T>()` for typed access to the wrapped view,
so nesting stays transparent:

```rust
root.child_mut::<Background>(0)?
    .child_mut::<VStack>()?
    .child_mut::<TextInput>(1)?
    .insert(text);
```

## Usage / Example

Run `cargo run --example multi_view`: an `HStack` with two `Background`
panels side by side (left: title, subtitle, `Spacer`, echo; right: input),
each padded, with nested `child_mut` state routing.

```rust
let mut stack = VStack::new()
    .spacing(8.0)
    .child(Text::new("Title").size(28.0).color(Color::WHITE))
    .child(TextInput::new().placeholder("Type here..."))
    .child(Spacer::new());

stack.place(fonts, x, y, width, height);
stack.draw(scene, fonts);
```

## Cross References

- [Renderer.md](Renderer.md) – `FontSystem`, frame, `View` trait
- [Text.md](Text.md) – static text element
- [TextInput.md](TextInput.md) – editable text field
- [Titlebar.md](Titlebar.md) – decoration bar
