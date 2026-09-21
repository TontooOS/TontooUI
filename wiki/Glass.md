# Glass

`GlassContainer`: liquid glass container with frosted body, liquid bevel
rim (specular top light melting into bottom depth shade), chromatic edge
split (red outside, cyan inside) and a soft drop shadow. Optional content
draws on top.

```rust
pub fn new() -> Self
pub fn bounds(self, x: f32, y: f32, width: f32, height: f32) -> Self
pub fn radius(self, px: f32) -> Self
pub fn tint(mut self, tint: Color) -> Self
pub fn content(self, child: impl View + 'static) -> Self
pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32)
pub fn set_tint(&mut self, tint: Color)
pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T>
```

Defaults: 320 x 180, 24 px radius, dark frost tint. `set_tint` switches
themes live (dark frost vs. light frost, gray when inactive).

| Token | Value |
|---|---|
| `GLASS_TINT_DARK` | white 10% |
| `GLASS_TINT_LIGHT` | black 8% |
| `GLASS_SPECULAR` | white 45% top light |
| `GLASS_DEPTH` | black 18% bottom shade |
| `GLASS_CHROMA_RED` / `GLASS_CHROMA_CYAN` | faint rim split |

True backdrop blur and refraction need the compositor (it owns the
desktop pixels behind a transparent window); this kit does everything
downstream of that: tint, bevel, rim light, chroma and shadow. Shell
support: `App::transparent_body` skips the window background fill so only
frame lines, bars and glass show over the desktop.

## Usage / Example

Run `cargo run --example glass`: transparent window, titlebar on top,
one centered empty glass container. Nothing else.

## Cross References

- [Renderer.md](Renderer.md) – `App::transparent_body`, frame, `View`/`App`
- [Layout.md](Layout.md) – stacks and modifiers for glass content
- [Theme.md](Theme.md) – frost tints per mode, gray inactive state
