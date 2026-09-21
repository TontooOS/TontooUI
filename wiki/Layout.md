# Layout

Stack containers that arrange elements: `VStack`, `HStack`, `ZStack` plus
`Spacer` for flexible space. Stacks measure children first (fonts needed
for text), then assign rects: fixed children keep intrinsic size, flex
children share the remaining space.

## Element

```rust
pub trait Element {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32);
    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32);
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem);
    fn flex(&self) -> f32 { 0.0 }
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

Intrinsic size, rect assignment and drawing in logical px. `flex` is the
share of remaining stack space (zero means fixed). `as_any_mut` powers
typed child access for state updates. All built-in elements (`Text`,
`TextInput`, `Titlebar`, stacks, `Spacer`) implement it.

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
pub fn child(self, child: impl Element + 'static) -> Self
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
pub fn child(self, child: impl Element + 'static) -> Self
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
pub fn child_mut<T: Element + 'static>(&mut self, index: usize) -> Option<&mut T>
```

Available on all three stacks. Typed access by position for state updates
(typing into a `TextInput`, changing a `Text`):

```rust
if let Some(input) = stack.child_mut::<TextInput>(2) {
    input.insert(text);
}
```

Returns `None` for a wrong index or type.

## Usage / Example

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
