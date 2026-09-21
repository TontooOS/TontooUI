# Text

Static text label rendered through Parley and Vello. Layout is cached and
rebuilt only when content, size, color or wrap width change.

## Constructors

```rust
pub fn new(content: impl Into<String>) -> Self
```

Creates a 15 px white label at position (0, 0) with no wrapping.

## Builder

```rust
pub fn size(self, size: f32) -> Self
```

Logical font size in px. Marks the cached layout dirty.

```rust
pub fn color(self, color: Color) -> Self
```

Text color as a Peniko `Color`. Marks the cached layout dirty.

```rust
pub fn at(self, x: f32, y: f32) -> Self
```

Logical draw position (top-left of the laid out block).

```rust
pub fn wrap(self, max_width: f32) -> Self
```

Wraps lines at logical `max_width` px. Without `wrap`, the text stays on
one line and can overflow the window.

## Mutation

```rust
pub fn set_content(&mut self, content: impl Into<String>)
```

Replaces the text. Rebuilds the layout only when the string changed.

## Layout

```rust
pub fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32)
```

Logical width/height of the laid out text. Builds the layout on first call.

```rust
pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem)
```

Builds the layout when dirty and records it into `scene` at the stored
position. See [Renderer.md](Renderer.md) for the drawing pipeline.

## Usage / Example

```rust
let mut title = Text::new("TontooUI Renderer Test")
    .size(28.0)
    .color(Color::WHITE)
    .at(32.0, 36.0);
title.draw(&mut scene, &mut fonts);
```

## Cross References

- [Renderer.md](Renderer.md) – `FontSystem`, frame pipeline, `View` trait
- [TextInput.md](TextInput.md) – editable single-line text field
