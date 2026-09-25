# Images

Images category in `src/elements/images/`: `SFSymbolImage` in
`symbol.rs` renders an SF Symbol glyph with size and an optional
color, `AppImage` in `app.rs` loads a raster from the app resources,
`UrlImage` in `url.rs` downloads a raster asynchronously with a
`Spinner` placeholder and an `Error {code}` state, and
`ImageOverlay` in `overlay.rs` is a photo card with a bottom
gradient, a caption and an optional badge. All four are display-only
(no mouse handling). Shared fit math and resource lookup live in
`mod.rs`; untinted raster upload (`raster`, `raster_file`) lives on
`ImageLoader` in `src/renderer/images.rs` next to the tinted
SF Symbol path (`get`).

## ImageFit

```rust
pub enum ImageFit { Cover, Fit }
```

- `Cover` (default) scales the raster to cover the frame and crops
  the overflow, centered. `Fit` scales it inside the frame and
  letterboxes the rest, centered.
- `fit_rect(iw, ih, w, h, fit)` returns the drawn size and centered
  offset `(draw_w, draw_h, dx, dy)`; degenerate inputs stay empty.

## Geometry

| Token | Value |
|---|---|
| `IMAGE_SYMBOL_SIZE` | 24 px default `SFSymbolImage` box |
| `IMAGE_RADIUS` | 12 px default raster frame corner radius |
| `IMAGE_TEXT_SIZE` | 13 px overlay caption and URL error text |
| `IMAGE_BADGE_SIZE` | 20 px overlay badge symbol box |
| `IMAGE_PLACEHOLDER_DARK` / `IMAGE_PLACEHOLDER_LIGHT` | `#2C2C2E` / `#E5E5E5` missing-file fill |

## SFSymbolImage

```rust
pub fn new(name: impl Into<String>) -> Self
pub fn size(self, px: f32) -> Self
pub fn color(self, color: Color) -> Self
pub fn set_theme(&mut self, text: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn tint(&self) -> Color
pub fn name_value(&self) -> &str
pub fn size_value(&self) -> f32
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Glyph artwork comes from CoreIcon (`COREICON_ASSETS_DIR` override
  or the system resources on TontooOS) through the tinted
  `ImageLoader::get` path, uploaded at ~2x for crisp supersampling.
  The artwork keeps its aspect ratio and centers in the box.
- Without `.color()` the glyph follows the theme text color via
  `set_theme`; a manual color wins like on dividers. Unfocused
  windows desaturate the glyph.
- A missing symbol draws nothing but keeps its box; `size` clamps
  to >= 0.

## AppImage

```rust
pub fn new(name: impl Into<String>, width: f32, height: f32) -> Self
pub fn fit(self, fit: ImageFit) -> Self
pub fn radius(self, px: f32) -> Self
pub fn set_theme(&mut self, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_fit(&mut self, fit: ImageFit)
pub fn set_radius(&mut self, px: f32)
pub fn path(&self) -> PathBuf
pub fn name_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `name` resolves through `resolve_resource_path`: the
  `$APP_RESOURCES_DIR` override (with and without `.png`), then the
  relative `assets/` dir (dev / `cargo run`), then the name as a
  literal path.
- Decoded without tinting (photos stay as-is) through
  `ImageLoader::raster_file`, cached per path, uploaded at ~2x and
  drawn into a rounded clip (`radius`, default `IMAGE_RADIUS`).
- A missing or undecodable file draws the theme placeholder box
  (`IMAGE_PLACEHOLDER_DARK` / `IMAGE_PLACEHOLDER_LIGHT`).

## UrlImage

```rust
pub fn new(url: impl Into<String>, width: f32, height: f32) -> Self
pub fn fit(self, fit: ImageFit) -> Self
pub fn radius(self, px: f32) -> Self
pub fn spinner_color(self, color: Color) -> Self
pub fn retry(&mut self)
pub fn set_theme(&mut self, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_fit(&mut self, fit: ImageFit)
pub fn set_radius(&mut self, px: f32)
pub fn state_value(&self) -> &'static str
pub fn error_text(&self) -> Option<String>
pub fn url_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- The download starts lazily on the first draw on a background
  thread (`ureq` blocking client + `mpsc` channel), so the UI never
  stalls. Each frame polls the channel without blocking.
- While loading the frame shows a centered `Spinner` (the existing
  progress element, recolorable via `spinner_color`). Once the bytes
  arrive they decode through the cached `ImageLoader::raster` path
  under the `url:` key and draw like `AppImage`.
- HTTP error statuses become `Error {code}` text (e.g. `Error 404`
  via `ureq::Error::StatusCode`); transport failures and undecodable
  bodies become plain `Error`. `state_value` returns `loading`,
  `loaded` or `error`; `retry` restarts the download.
- No network happens in unit tests: state transitions and error
  text are covered without requests.

## ImageOverlay

```rust
pub enum OverlaySource { File(PathBuf), Resource(String) }
pub fn file(path: impl Into<PathBuf>, width: f32, height: f32) -> Self
pub fn resource(name: impl Into<String>, width: f32, height: f32) -> Self
pub fn caption(self, text: impl Into<String>) -> Self
pub fn badge(self, symbol: impl Into<String>) -> Self
pub fn badge_color(self, color: Color) -> Self
pub fn radius(self, px: f32) -> Self
pub fn set_theme(&mut self, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_caption(&mut self, text: Option<String>)
pub fn set_badge(&mut self, symbol: Option<String>)
pub fn set_badge_color(&mut self, color: Color)
pub fn set_radius(&mut self, px: f32)
pub fn source_value(&self) -> &OverlaySource
pub fn caption_value(&self) -> Option<&str>
pub fn badge_value(&self) -> Option<&str>
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- The raster (file or app resource, same lookup as `AppImage`)
  draws cover-fit into the rounded card; a bottom gradient
  (transparent to black 170 alpha over the lower half) keeps the
  caption readable. Missing files draw the theme placeholder with
  the caption in theme text.
- The caption sits bottom left with a 12 px inset and wraps inside
  the card. The badge is an SF Symbol top right with a 12 px inset
  in `badge_color` (white by default, desaturated when unfocused).

## Usage / Example

```rust
use tontooui::elements::{AppImage, ImageOverlay, SFSymbolImage, UrlImage, View, VStack};
use vello::peniko::Color;

let stack = VStack::new()
  .spacing(28.0)
  .child(SFSymbolImage::new("star.fill").size(40.0).color(Color::from_rgb8(0xff, 0x9f, 0x0a)))
  .child(AppImage::new("demo-star", 200.0, 130.0))
  .child(UrlImage::new("https://picsum.photos/400/260", 200.0, 130.0))
  .child(ImageOverlay::resource("demo-star", 220.0, 140.0).caption("Demo star").badge("heart.fill"));
```

Wire the live theme per frame (see `examples/image.rs`):

```rust
symbol.set_theme(palette.text, dark);
symbol.set_focused(focused);
image.set_theme(dark);
```

## Cross References

- [Progress.md](Progress.md) – `Spinner` used for the loading state
- [Renderer.md](Renderer.md) – `ImageLoader` cache and upload pipeline
- [Theme.md](Theme.md) – theme text color and unfocused desaturation
- [Layout.md](Layout.md) – stacks size and place images via `View`
