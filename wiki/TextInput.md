# TextInput

Single-line text field with a rounded dark box, orange focus ring, placeholder
and blinking cursor. Click to focus, type to edit. The cursor is a byte index
into the UTF-8 string and always sits on a character boundary.

## Constructors

```rust
pub fn new() -> Self
```

Creates an empty 300 x 40 field at position (0, 0), unfocused, no placeholder.

## Builder

```rust
pub fn bounds(self, x: f32, y: f32, width: f32, height: f32) -> Self
```

Logical position and size of the field box.

```rust
pub fn placeholder(self, placeholder: impl Into<String>) -> Self
```

Hint shown in gray (`#8e8e93`) while the field is empty.

```rust
pub fn on_change(self, callback: impl FnMut(&str) + 'static) -> Self
```

Callback fired with the full text after every insert and backspace.

## State

```rust
pub fn text(&self) -> &str
```

Current content.

```rust
pub fn is_focused(&self) -> bool
```

Returns true after a click inside the field, false after a click outside.

## Input

```rust
pub fn mouse_down(&mut self, x: f64, y: f64)
```

Focuses the field when (`x`, `y`) is inside its bounds and moves the cursor
to the end; blurs otherwise. Coordinates are logical pixels.

```rust
pub fn insert(&mut self, text: &str)
```

Inserts printable text at the cursor when focused. Control characters are
filtered out, so pasted newlines never enter the single line. Empty input is
ignored. Fires `on_change`.

```rust
pub fn backspace(&mut self)
```

Deletes the character before the cursor when focused. Does nothing at
position 0. Fires `on_change`.

```rust
pub fn move_left(&mut self)
```

Moves the cursor one character left. Stops at position 0.

```rust
pub fn move_right(&mut self)
```

Moves the cursor one character right. Stops at the end of the text.

## Drawing

```rust
pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, blink_on: bool)
```

Draws the box, the focus ring when focused, the text (or placeholder)
vertically centered with 12 px padding, and the cursor when focused and
`blink_on` is true. The caller derives `blink_on` from elapsed time (the
`window` example uses a 530 ms phase). The cursor x position is measured
from a scratch layout of the text prefix, so it matches shaping exactly.

## Limitations

- Click-to-position is not implemented; clicking moves the cursor to the end.
- No text selection and no clipboard shortcuts yet.
- Overflow wraps inside the box instead of scrolling; keep fields wide enough
  or texts short for now.

## Usage / Example

```rust
let mut input = TextInput::new()
    .bounds(32.0, 120.0, 420.0, 44.0)
    .placeholder("Type here...")
    .on_change(|text| println!("Changed: {text}"));
```

Forward window events in the `View` implementation:

```rust
fn mouse_down(&mut self, x: f64, y: f64) { self.input.mouse_down(x, y); }
fn text(&mut self, text: &str) { self.input.insert(text); }
fn key(&mut self, key: Key) {
    match key {
        Key::Backspace => self.input.backspace(),
        Key::Left => self.input.move_left(),
        Key::Right => self.input.move_right(),
        Key::Enter | Key::Escape => self.input.mouse_down(-1.0, -1.0),
    }
}
```

## Cross References

- [Renderer.md](Renderer.md) – `View` trait, `Key` enum, event flow
- [Text.md](Text.md) – static text element
