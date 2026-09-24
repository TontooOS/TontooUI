# Glass

`GlassContainer`: liquid glass lens container with a clear minified center,
a frosted edge band, liquid bevel rim (specular top light melting into bottom
depth shade), chromatic edge split (red outside, cyan inside) and a soft drop
shadow. Optional content draws on top. When the shell runs
`App::wants_backdrop`, the center samples the sharp in-app capture slightly
minified while only a narrow rim band samples the blurred capture, so content
behind the glass shows through shrunk with frost only at the very edge.

```rust
pub fn new() -> Self
pub fn bounds(self, x: f32, y: f32, width: f32, height: f32) -> Self
pub fn radius(self, px: f32) -> Self
pub fn tint(mut self, tint: Color) -> Self
pub fn content(self, child: impl View + 'static) -> Self
pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32)
pub fn set_tint(&mut self, tint: Color)
pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T>
```

Defaults: 320 x 180, 24 px radius, dark frost tint. `set_tint` overrides
manually; `set_theme` follows the system glass stage.

## Stages

The `glass` daemon setting (LiquidGlass slider) drives the look:

| Stage | Dark | Light |
|---|---|---|
| `Less` | Mostly opaque dark (black 59%), brighter rim, grainy black outer edge | Mostly opaque light (white 59%), brighter rim, grainy edge |
| `Glass` | Balanced frost (white 10%) | Balanced frost (black 8%) |
| `Much` | Unchanged balanced frost (white 10%) | Lighter frost (white 5%) |

Less glass adds a scattered black dash ring outside the crisp rim. The
stages control frost opacity, rim light and grain; the lens layers the sharp
minified capture in the center and the blurred capture as an edge band under
the frost when `App::wants_backdrop` is enabled.

| Token | Value |
|---|---|
| `GLASS_TINT_DARK` | white 10% |
| `GLASS_TINT_LIGHT` | black 8% |
| `GLASS_SPECULAR` | white 45% top light |
| `GLASS_DEPTH` | black 18% bottom shade |
| `GLASS_CHROMA_RED` / `GLASS_CHROMA_CYAN` | faint rim split |
| `GLASS_EDGE_WIDTH` | 6 logical px thin frosted rim band |
| `GLASS_ZOOM` | 0.80x lens zoom of the clear center (minify) |

## Lens

Top to bottom: the liquid bevel strokes specular white at the top and depth
shade at the bottom with a transparent middle, so the flanks stay clean;
under it `stroke_backdrop_edge` strokes a 6 px blurred rim fully inside the
body outline; the center fills with the sharp capture minified around the
body center (`fill_backdrop_lens`, `GLASS_ZOOM`), so the minifier covers
the whole middle. Tiny bodies (smaller than twice the band) fall back to a
full `fill_backdrop` blur. The held toggle knob and the dragged slider knob
use the same lens via `fill_lens_glass` with a narrower 4 px rim.

Desktop pixels behind a transparent window still need the compositor (it
owns those pixels); `App::transparent_body` skips the window background
fill so only frame lines, bars and glass show over the desktop. Enable
`App::wants_backdrop` for in-app blur of content the shell itself draws
(window body, titlebar, tracks). On the capture pass the whole container
(body and children) is omitted so the blur sees only what sits behind it.

## Usage / Example

Run `cargo run --example glass`: transparent window, titlebar on top,
one centered empty glass container. Nothing else. The example returns
true from `wants_backdrop` so the glass body blurs the in-app titlebar
and frame content behind it.

## Cross References

- [Renderer.md](Renderer.md) – `App::transparent_body`, `App::wants_backdrop`, backdrop blur, `View`/`App`
- [Layout.md](Layout.md) – stacks and modifiers for glass content
- [Theme.md](Theme.md) – frost tints per mode, gray inactive state
