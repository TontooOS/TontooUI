# Titlebar

Custom decoration bar: opaque background with top-only rounded corners, a
1 px bottom divider and a centered semibold title. No traffic lights. Window
dragging is handled by the shell (see Dragging).

## Colors

Dark colors are active; light constants exist for later.

| Token | Dark | Light |
|---|---|---|
| `TITLEBAR_BG_*` | `#2C2C2E` (lighter than body) | `#DEDEE1` (darker than body) |
| `TITLEBAR_TEXT_*` | `#F5F5F7` | `#1E1E1E` |
| `TITLEBAR_DIVIDER_*` | white 14% | black 12% |

```rust
pub const TITLEBAR_BG_DARK: Color;
pub const TITLEBAR_BG_LIGHT: Color;
pub const TITLEBAR_TEXT_DARK: Color;
pub const TITLEBAR_TEXT_LIGHT: Color;
pub const TITLEBAR_DIVIDER_DARK: Color;
pub const TITLEBAR_DIVIDER_LIGHT: Color;
```

## Height

```rust
pub enum TitlebarHeight {
    Standard,
    Mac,
}
```

| Variant | Height |
|---|---|
| `Standard` | 31 px (default: 17 px button size + 14 px padding) |
| `Mac` | 44 px |

```rust
pub fn px(self) -> f32
```

Height in logical px.

## Element

```rust
pub fn new(title: impl Into<String>) -> Self
```

Creates a `Standard` bar with centered 13 px semibold system-ui title
(system font, so SF Pro Display on TontooOS).

```rust
pub fn height(self, height: TitlebarHeight) -> Self
```

Selects the bar height.

```rust
pub fn set_title(&mut self, title: impl Into<String>)
```

Replaces the title. Rebuilds the layout only when the string changed.

```rust
pub fn set_rect(&mut self, x: f32, y: f32, width: f32)
```

Places the bar, usually spanning the content viewport. Rebuilds the layout
only when the width changed.

```rust
pub fn bounds(&self) -> (f32, f32, f32, f32)
```

Logical hit rect (x, y, width, height). Return it from
`View::drag_region` so the shell starts a window drag on press.

```rust
pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem)
```

Draws the background (`RoundedRect` with top-only radius via
`RoundedRectRadii`), the 1 px bottom divider (stops before the rounded
corners) and the vertically/horizontally centered title.

## Dragging

The shell checks `View::drag_region` on left press:

```rust
fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
    Some(self.bar.bounds())
}
```

A press inside starts `window.drag_window()` and keeps focus (no click is
forwarded to the view). Dragging needs compositor support for `xdg_toplevel`
move; failures are ignored silently.

## Usage / Example

```rust
let mut bar = Titlebar::new("TontooUI").height(TitlebarHeight::Standard);
bar.set_rect(viewport.x, viewport.y, viewport.width);
bar.draw(scene, fonts);
```

## Cross References

- [Renderer.md](Renderer.md) – `View` trait, `drag_region`, frame, `FontSystem`
- [Text.md](Text.md) – static text element
- [TextInput.md](TextInput.md) – editable text field
