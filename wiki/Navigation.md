# Navigation

SwiftUI-style Navigation category for TontooUI, recreating the navigation modifiers from the macOS 26 interface. The category currently contains the single modifier `NavigationSubtitle`. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## NavigationSubtitle

```rust
pub struct NavigationSubtitle { /* ... */ }

impl NavigationSubtitle {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self;
    pub fn subtitle(subtitle: impl Into<String>) -> Self;
    pub fn title(self, title: impl Into<String>) -> Self;
    pub fn subtitle_text(self, subtitle: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — configures the view's subtitle for purposes of navigation, using a localized string key. Mirrors `SwiftUI.View/navigationSubtitle(_:)`.

- `new(title, subtitle)` creates a preview with title `Foo` and subtitle `Bar` as in the screenshot (title `15px` Semibold, subtitle `11px` regular, plus a hairline separator).
- `subtitle(subtitle)` is a convenience that uses the default title `Foo`.
- `to_view` size is `260×64`.
- Adaptive colors: title `#FFFFFF` dark / `#1d1d1d` light; subtitle `rgba(255,255,255,0.62)` dark / `rgba(60,60,67,0.6)` light; separator `rgba(255,255,255,0.10)` dark / `rgba(0,0,0,0.08)` light.
- Directly on window — no extra card; the outer `VBox` is centered and contains two `Label`s plus a `Separator`, SF Pro.
- `is_interactive() == false`.

## Usage / Example

Run the gallery demo (recreates the single-card screenshot):

```bash
cargo run --example navigation
```

Minimal usage:

```rust
use tontooui::prelude::*;

// Modifier on a view with navigation title "Foo"
let subtitle = NavigationSubtitle::new("Foo", "Bar").to_view();
let custom = NavigationSubtitle::subtitle("Bar").title("Foo");

// As modifier-style — wrap a VStack that already has a title
let root = VStack::new()
    .spacing(8.0)
    .child(Text::new("Foo").font_size(15.0).bold())
    .child(NavigationSubtitle::new("Foo", "Bar").to_view());
```

Category folder layout:

```
src/elements/navigation/
  mod.rs                    // category root
  navigation_subtitle.rs    // NavigationSubtitle (modifier)
```

## Cross References

- [Text.md](Text.md) -- text rendering uses same SF Pro context
- [ControlGroup.md](ControlGroup.md) -- control groups also group navigation controls
