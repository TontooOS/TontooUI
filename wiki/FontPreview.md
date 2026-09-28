# FontPreview

Font preview in `src/elements/text/font_preview.rs`: `FontPreview`
renders a font family name in its own family plus a sample line,
stacked vertically (Font Book style row). Display-only (no mouse
handling).

## Geometry

| Token | Value |
|---|---|
| `FONT_PREVIEW_TITLE_SIZE` | `17.0` family name size |
| `FONT_PREVIEW_TITLE_WEIGHT` | `600.0` family name weight |
| `FONT_PREVIEW_SAMPLE_SIZE` | `28.0` sample size |
| `FONT_PREVIEW_SAMPLE_WEIGHT` | `400.0` sample weight |
| `FONT_PREVIEW_GAP` | `4.0` name-to-sample gap |
| `FONT_PREVIEW_DEFAULT_SAMPLE` | `"AaBbCcDdEeFfGg 0123456789"` default sample |

## FontPreview

```rust
pub fn new(family: impl Into<String>) -> Self
pub fn from_file(fonts: &mut FontSystem, path: &Path) -> std::io::Result<Self>
pub fn from_data(fonts: &mut FontSystem, data: Vec<u8>, fallback_family: &str) -> Self
pub fn sample(self, sample: impl Into<String>) -> Self
pub fn title_size(self, size: f32) -> Self
pub fn sample_size(self, size: f32) -> Self
pub fn show_family_label(self, show: bool) -> Self
pub fn width(self, px: f32) -> Self
pub fn set_family(&mut self, family: impl Into<String>)
pub fn set_sample(&mut self, sample: impl Into<String>)
pub fn set_title_size(&mut self, size: f32)
pub fn set_sample_size(&mut self, size: f32)
pub fn set_show_family_label(&mut self, show: bool)
pub fn set_width(&mut self, px: Option<f32>)
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn family_value(&self) -> &str
pub fn sample_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Both lines lay out through `FontSystem::layout_text_in_family`
  in the preview family with a `system-ui` fallback, so unknown
  families degrade to the system font instead of `.notdef` boxes.
- `from_file` loads a font file into `fonts` and previews its
  first registered family (read from the font name tables); it
  falls back to the file stem when nothing new registers and
  returns `Err` when the file cannot be read. `from_data` takes
  raw bytes with a `fallback_family` for the same case. No system
  install is needed in either case.
- Colors resolve `TextForeground::Primary` against the theme
  (`#D8D9D9` dark, `#272727` light), desaturated when unfocused.
- `width` fixes the box so longer content wraps; `None` (default)
  measures the intrinsic single-line size.
- Sizes below `1.0` clamp to `1.0`. Setters mark the layout dirty
  on change only. The cached layouts also rebuild automatically
  when the window scale factor changes (DPI move) or when the
  family changes, so glyphs stay sharp.
- `show_family_label(false)` hides the name row and measures only
  the sample line.

## Usage / Example

```rust
use tontooui::elements::{FontPreview, View};

let row = FontPreview::new("SF Pro");
let custom = FontPreview::new("Inter")
    .sample("The quick brown fox 0123456789")
    .sample_size(24.0)
    .width(320.0);
```

List every installed family with `FontRegistry::families` and map
each entry to one `FontPreview` row:

```rust
use coretext::FontRegistry;
use tontooui::elements::FontPreview;

let registry = FontRegistry::new();
let rows: Vec<FontPreview> = registry
    .families()
    .into_iter()
    .map(FontPreview::new)
    .collect();
```

Preview a font file straight from disk:

```rust
use std::path::Path;
use tontooui::elements::FontPreview;
use tontooui::renderer::FontSystem;

let mut fonts = FontSystem::new();
let row = FontPreview::from_file(&mut fonts, Path::new("/usr/share/fonts/MyFont.ttf"))?;
```

## Cross References

- [Text.md](Text.md) – `BasicText` styles and foregrounds behind
  the preview colors
- [Renderer.md](Renderer.md) – `FontSystem` layout and CoreText
  pipeline (`layout_text_in_family`, aligned layout API)
- [Theme.md](Theme.md) – palette behind `Primary`
