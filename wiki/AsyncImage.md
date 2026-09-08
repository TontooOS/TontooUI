# AsyncImage

SwiftUI-style AsyncImage category for TontooUI, recreating asynchronous URL image views. The category contains three `initializer` elements and one `modifier` element. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## CustomPlaceholderAsyncImage

```rust
pub struct CustomPlaceholderAsyncImage { /* ... */ }
impl CustomPlaceholderAsyncImage {
    pub fn new() -> Self;
    pub fn url(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — loads and displays a modifiable image from the specified URL load request with a custom placeholder.

- Preview: small centered cats group `3x 26x32` on a `180x110` phone card.
- Size `180x110`.

## CustomPhasesAsyncImage

```rust
pub enum AsyncImagePhase { Empty, Success, Failure }
pub struct CustomPhasesAsyncImage { /* ... */ }
impl CustomPhasesAsyncImage {
    pub fn new() -> Self;
    pub fn url(self, v: impl Into<String>) -> Self;
    pub fn phase(self, v: AsyncImagePhase) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — loads and displays a modifiable image from the specified URL load request with custom phases.

| Type | Values |
|---|---|
| `AsyncImagePhase` | `Empty` `Success` `Failure` |

- Preview: red phase band `180x44` on top plus small cats row below.
- Size `180x110`.

## AsyncURLImage

```rust
pub struct AsyncURLImage { /* ... */ }
impl AsyncURLImage {
    pub fn new() -> Self;
    pub fn url(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — loads and displays an image from the specified URL load request.

- Preview: full-bleed cats image `180x110` (`12px` radius).
- Size `180x110`.

## CustomSessionAsyncImage

```rust
pub struct CustomSessionAsyncImage { /* ... */ }
impl CustomSessionAsyncImage {
    pub fn new() -> Self;
    pub fn session(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a modifier that adds a URL session for asynchronous images contained in the view.

- Preview: full-bleed cats with a blue session tint overlay.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example async_images
```

Minimal usage:

```rust
use tontooui::prelude::*;

let custom = CustomPlaceholderAsyncImage::new().url("https://example.com/cats.png");
let phases = CustomPhasesAsyncImage::new().phase(AsyncImagePhase::Success);
let plain = AsyncURLImage::new().url("https://example.com/cats.png");
let session = CustomSessionAsyncImage::new().session("ephemeral");

let root = VStack::new()
    .spacing(8.0)
    .child(custom.to_view())
    .child(plain.to_view());
```

Category folder layout:

```
src/elements/async_images/
  mod.rs                // category root
  custom_placeholder.rs // CustomPlaceholderAsyncImage
  custom_phases.rs      // CustomPhasesAsyncImage + AsyncImagePhase
  async_url_image.rs    // AsyncURLImage
  custom_session.rs     // CustomSessionAsyncImage
```

## Cross References

- [Label.md](Label.md) -- `ImageLabel` pairs a static image with a title; async images load from URL
- [Shapes.md](Shapes.md) -- image clips can reuse shape glyphs
