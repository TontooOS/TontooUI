# Titlebar

Custom decoration bar: opaque background with top-only rounded corners, a
1 px bottom divider and a centered semibold title. No traffic lights. Window
dragging is handled by the shell (see Dragging).

## Colors

Dark colors are active; light constants exist for later.

| Token | Dark | Light |
|---|---|---|
| `TITLEBAR_BG_*` | `#2C2C2E` (lighter than body) | `#DEDEE1` (darker than body) |
| `TITLEBAR_TEXT_*` | `#D8D9D9` | `#272727` |
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
pub fn set_palette(&mut self, bg: Color, text: Color, divider: Color)
```

Live theme colors. Rebuilds title glyphs only when the text color
changed (see [Theme.md](Theme.md)).

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

The shell checks `View::drag_region` on left press. The bar exposes the
bounds minus the traffic light cluster so button clicks never drag:

```rust
fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
    Some(self.bar.drag_rect())
}
```

A press inside starts `window.drag_window()` and keeps focus (no click is
forwarded to the view). Dragging needs compositor support for `xdg_toplevel`
move; failures are ignored silently.

```rust
pub fn drag_rect(&self) -> (f32, f32, f32, f32)
```

`bounds` minus the left traffic light cluster (18 px margin + 3 x 17 px
buttons + 2 x 10 px gaps = 89 px).

The maximize glyph is the TontooOS expand logo (two shapes, 500x500
viewBox), uniformly scaled into the 68% box so it never stretches.

## Traffic Lights

17 px circles, no border, no shadow. Hovering the group on a focused
window shows glyphs at 68% size: filled rounded bars for x (dark red)
and minus (dark amber), the expand logo (near-black) for maximize. An
unfocused window shows all gray with no glyphs, even on hover. A modal
blocked close light is gray with no glyph and ignores clicks.

| Token | Value |
|---|---|
| `TRAFFIC_SIZE` / `TRAFFIC_GAP` / `TRAFFIC_LEFT` | 17 px / 10 px / 18 px |
| `TRAFFIC_CLOSE` | `#FF5F56` |
| `TRAFFIC_MINIMIZE` | `#FFBD2E` |
| `TRAFFIC_MAXIMIZE` | `#27C93F` |
| `TRAFFIC_INACTIVE` | `#888888` |
| `TRAFFIC_GLYPH` | black 60% (maximize logo) |
| `TRAFFIC_GLYPH_CLOSE` | `#8A1F1A` (x bars) |
| `TRAFFIC_GLYPH_MINIMIZE` | `#8A6800` (minus bar) |

```rust
pub enum TrafficAction {
    Close,
    Minimize,
    Maximize,
}
```

```rust
pub fn set_hover(&mut self, x: f32, y: f32)
```

Group hover from the logical cursor position (3 px tolerance around each
button).

```rust
pub fn set_focused(&mut self, focused: bool)
```

Dims all buttons to `TRAFFIC_INACTIVE` when the window loses focus.
Forward `View::set_focused` here.

```rust
pub fn press(&mut self, x: f32, y: f32) -> Option<TrafficAction>
```

Click handling. Map the result to a `WindowCommand` and return it from
`View::poll_window_command`; the shell executes close, minimize and
maximize toggle.

```rust
fn mouse_move(&mut self, x: f64, y: f64) { self.bar.set_hover(x as f32, y as f32); }
fn mouse_down(&mut self, x: f64, y: f64) {
    match self.bar.press(x as f32, y as f32) {
        Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
        Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
        Some(TrafficAction::Maximize) => self.command = Some(WindowCommand::ToggleMaximize),
        None => self.input.mouse_down(x, y),
    }
}
```

## Usage / Example

```rust
let mut bar = Titlebar::new("TontooUI").height(TitlebarHeight::Standard);
bar.set_rect(viewport.x, viewport.y, viewport.width);
bar.draw(scene, fonts);
```

## Cross References

- [Renderer.md](Renderer.md) – `View` trait, `drag_region`, frame, `FontSystem`
- [Theme.md](Theme.md) – live palette with fade animation
