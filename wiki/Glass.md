# Glass

`GlassContainer`: liquid glass lens container with a clear minified center,
a frosted edge band, liquid bevel rim (Lens finish: specular top light
melting into bottom depth shade; Frosted finish: uniform 1 px dark-gray
rim on every side), edge sheen (both finishes: subtle top light brightest
in the middle and fading toward the corners, much fainter at the bottom)
and a soft drop
shadow. Optional content draws on top. When the shell runs `App::wants_backdrop`, the center samples the sharp in-app
capture slightly minified while only a narrow rim band samples the
blurred capture, so content behind the glass shows through shrunk with
frost only at the very edge.

```rust
pub fn new() -> Self
pub fn bounds(self, x: f32, y: f32, width: f32, height: f32) -> Self
pub fn radius(self, px: f32) -> Self
pub fn tint(mut self, tint: Color) -> Self
pub fn glass_type(mut self, glass_type: GlassType) -> Self
pub fn content(self, child: impl View + 'static) -> Self
pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32)
pub fn set_radius(&mut self, px: f32)
pub fn set_tint(&mut self, tint: Color)
pub fn set_glass_type(&mut self, glass_type: GlassType)
pub fn set_auto_frost(&mut self, auto_frost: bool)
pub fn frost_value(&self) -> f32
pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T>
```

```rust
pub enum GlassType {
    Lens,
    Frosted,
}
```

- `Lens` (default): clear minified center, blur only on the narrow edge
  band. Slider and toggle knobs always use this finish.
- `Frosted`: same lens plus a heavy blur veil (`GLASS_FROST_VEIL`, 0.95)
  over the whole body, so behind shows through but stays unrecognizable,
  with the strongest frost at the edge.
- `set_auto_frost` (default on): the clear center gains a blur veil
  with the local backdrop busyness (capped at `AUTO_FROST_VEIL`,
  0.65), so text and icons stay readable over video and photos
  while plain backgrounds stay perfectly clear. The shell samples
  luma variance per 16 px tile (`BUSY_TILE`) about twice per second
  (`BUSY_EVERY_N_FRAMES`, 30) into a shared grid
  (`BusyGrid::amount_at`); `frost_for_busy` maps variance to frost
  (plain below `BUSY_LO`, full at `BUSY_HI`, smooth between) and
  the container eases toward it (`frost_value` reads the live
  value). No sample yet (or no backdrop) means clear.

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
stages control frost opacity, rim light and grain for the Lens finish;
the Frosted finish ignores the glass amount and always renders the
balanced stage (it still follows dark/light mode).

| Token | Value |
|---|---|
| `GLASS_TINT_DARK` | white 10% |
| `GLASS_TINT_LIGHT` | black 8% |
| `GLASS_SPECULAR` | white 45% top light (Lens finish) |
| `GLASS_DEPTH` | black 18% bottom shade (Lens finish) |
| `GLASS_FROSTED_RIM` | dark gray (`#3A3A3C`) 1 px rim on every side (Frosted finish) |
| `GLASS_SHEEN_TOP` / `GLASS_SHEEN_TOP_GLOW` | top edge sheen core and halo, center-weighted (both finishes) |
| `GLASS_SHEEN_BOTTOM` / `GLASS_SHEEN_BOTTOM_GLOW` | much fainter bottom counterpart (both finishes) |
| `GLASS_CHROMA_RED` / `GLASS_CHROMA_CYAN` | reserved (former rim split, unused) |
| `GLASS_EDGE_WIDTH` / `GLASS_EDGE_WIDTH_LARGE` | 2 px rim band, 3 px once the smaller side reaches `GLASS_LARGE_MIN_SIDE` (200 px) |
| `GLASS_ZOOM` | 0.80x lens zoom of the clear center (minify) |

## Lens

Top to bottom: the liquid bevel strokes specular white at the top and depth
shade at the bottom with a transparent middle, so the flanks stay clean;
under it `stroke_backdrop_edge` strokes a 2 px blurred rim (3 px on large glass) fully inside the
body outline; the center fills with the sharp capture minified around the
body center (`fill_backdrop_lens`, `GLASS_ZOOM`), so the minifier covers
the whole middle. Tiny bodies (smaller than twice the band) fall back to a
full `fill_backdrop` blur. The held toggle knob and the dragged slider knob
use the same lens via `fill_lens_glass` with a narrower 4 px rim.

## Frosted

No bevel: a single 1 px `GLASS_FROSTED_RIM`
stroke sits exactly on the body edge, the same dark gray on every
side (desaturated with the palette when the window is inactive).

## Edge sheen

Both finishes draw a 1 px line along the straight top edge just
inside the rim, brightest in the middle and fading out toward the
corners, over a wider faint halo that softens it; the bottom edge
gets a much fainter counterpart. Like the reference menu highlight,
only the middle reads brighter while the corners stay clean.

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
