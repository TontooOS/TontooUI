# Text

SwiftUI-style Text category for TontooUI. The category groups text display elements. The foundational `Text`/`Label` widgets from UIKit (SF Pro, Pango, adaptive) live here logically, and the single initializer element `TextFormat` renders formatted representations of non-string types. All rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card — matching the `Color` and `Gauge` galleries.

## Text / Label

```rust
pub use uikit::widgets::{Text, Label};
```

UIKit's `Text` (type alias for `Label`) — displays text via `GtkLabel` + Pango.

```rust
pub struct Label { /* ... */ }

impl Label {
    pub fn new(content: impl Into<String>) -> Self;
    pub fn font_size(self, size: f32) -> Self;
    pub fn font_family(self, family: impl Into<String>) -> Self;
    pub fn bold(self) -> Self;
    pub fn color(self, color: Color) -> Self;
    pub fn max_width(self, width: f32) -> Self;
    pub fn at(self, x: f32, y: f32) -> Self;
    pub fn size(self, width: f32, height: f32) -> Self;
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View;
}
```

Mirrors `SwiftUI.Text` for plain strings. The TontooUI `Label`/`Text` are re-exported from this category for discoverability; existing code can keep using `uikit::prelude::Text` or `tontooui::prelude::Text`.

Behavior notes:

- `Text` is a type alias for `Label` (`pub type Text = Label`).
- `font_family` defaults to `SF Pro Display` (`Font::system` / `Font::default`).
- Color defaults to `#EBEBF5` at 92% white; override with `.color(...)`.
- `max_width` enables ellipsization.

## TextFormatKind

```rust
pub enum TextFormatKind {
    CurrencyUsd(f64),
    Percent(f64),
    Number(f64),
    IntegerGrouped(i64),
    Scientific(f64),
    Measurement { value: f64, unit: String },
    WeightKg(f64),
    TemperatureC(f64),
    TemperatureWords(f64),
    Time { h: u8, m: u8, s: u8, pm: bool },
    Date { year: i32, month: u8, day: u8 },
    DateNumeric { month: u8, day: u8 },
    Duration { minutes: u32, seconds: u32 },
    Custom(String),
}

impl TextFormatKind {
    pub fn formatted(&self) -> String;
}
```

Discriminant for each SwiftUI `FormatStyle`. `formatted()` returns the locale-aware string (e.g. `CurrencyUsd(1234.56)` → `"$1,234.56"`).

| Kind | Example input | Output |
|---|---|---|
| `CurrencyUsd(1234.56)` | `1234.56` | `$1,234.56` |
| `Percent(0.874)` | `0.874` | `87.4%` |
| `Number(3.14)` | `3.14` | `3.14` |
| `IntegerGrouped(8976543)` | `8976543` | `8,976,543` |
| `Scientific(6.022e23)` | `6.022e23` | `6.022E23` |
| `Measurement{123,"meters"}` | `123 meters` | `123 meters` |
| `WeightKg(70)` | `70` | `70kg` |
| `TemperatureC(25.5)` | `25.5` | `25.5°C` |
| `TemperatureWords(-30.1)` | `-30.1` | `-30.1 degrees Celsius` |
| `Time{h:9,m:25,s:39,pm:true}` | — | `9:25:39 PM` |
| `Date{2025,7,21}` | — | `Jul 21, 2025` |
| `DateNumeric{1,3}` | — | `1/03` |
| `Duration{1,31}` | — | `1:31` |
| `Custom("hello")` | — | `hello` |

## TextFormat

```rust
pub struct TextFormat { /* ... */ }

impl TextFormat {
    pub fn new(kind: TextFormatKind) -> Self;
    pub fn custom(text: impl Into<String>) -> Self;
    pub fn currency(value: f64) -> Self;
    pub fn percent(value: f64) -> Self;
    pub fn number(value: f64) -> Self;
    pub fn integer_grouped(value: i64) -> Self;
    pub fn scientific(value: f64) -> Self;
    pub fn measurement(value: f64, unit: impl Into<String>) -> Self;
    pub fn weight_kg(value: f64) -> Self;
    pub fn temperature_c(value: f64) -> Self;
    pub fn temperature_words(value: f64) -> Self;
    pub fn time(h: u8, m: u8, s: u8, pm: bool) -> Self;
    pub fn date(year: i32, month: u8, day: u8) -> Self;
    pub fn date_numeric(month: u8, day: u8) -> Self;
    pub fn duration(minutes: u32, seconds: u32) -> Self;
    pub fn demo() -> Self;
    pub fn font_size(self, size: f32) -> Self;
    pub fn formatted(&self) -> String;
    pub fn kind(&self) -> &TextFormatKind;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a text view that displays the formatted representation of a non-string type. Mirrors `SwiftUI.Text/init(_:format:)` and `Text(_:formatter:)`.

- Convenience constructors (`::currency`, `::percent`, etc.) map directly to `TextFormatKind` variants.
- `::demo()` renders the full 11-value palette from the reference screenshot in three rows, directly on the window (no card, no background). The three rows are exactly:

```
$1,234.56  87.4%  3.14  8,976,543  6.022E23
123 meters  70kg  25.5°C  -30.1 degrees Celsius
9:25:39 PM  Jul 21, 2025  1/03  1:31
```

- `::custom` wraps an already formatted string.
- `font_size` defaults to `11.0` for palette, overridable.
- `to_view` size: palette `280×74`, single value `len×0.6×size+12 × 18`.

Color is adaptive: white (`#FFFFFF`) on dark (`#1d1d1d`), `#1d1d1d` on light (`#ececec`), via `resolve_scheme`.

Rendering details:

- No extra background container — sits directly on the window like `Gauge` and the `Color` palettes.
- Uses `SF Pro Display`, `font-size 10px` per label (`11px` for singles when overridden).
- `GtkLabel` per value, centered in an `HBox` per row.
- `is_interactive() == false`.

## Usage / Example

Run the gallery demo (recreates the single-card screenshot):

```bash
cargo run --example text
```

Minimal usage:

```rust
use tontooui::prelude::*;

// Single formatted values
let price = TextFormat::currency(1234.56).font_size(11.0);
let pct = TextFormat::percent(0.874);
let temp = TextFormat::temperature_c(25.5);
let when = TextFormat::date(2025, 7, 21);

// As Views directly on window
let root = VStack::new()
    .spacing(8.0)
    .child(price.to_view())
    .child(pct.to_view());

// Full palette (like the screenshot card preview but without the card)
let palette = TextFormat::demo().to_view();
```

Category folder layout:

```
src/elements/text/
  mod.rs         // category root, re-exports Text/Label + TextFormat
  text_format.rs // TextFormat + TextFormatKind (initializer)
```

## Cross References

- [Color.md](Color.md) -- color palettes also render directly on window
- [Divider.md](Divider.md) -- separators use SF Pro context similarly
- [ContentUnavailableView.md](ContentUnavailableView.md) -- empty state with text
