# View

SwiftUI-style View category for TontooUI, recreating view modifiers from the macOS 26 interface. The category contains ten `modifier` elements that cover pickers, overlays, swipe, navigation backgrounds, control sizing, and visual effects. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`. Core backing is in `uikit::view_modifiers::ViewModifierExt` — TontooUI provides the palette previews.

## ViewModifiers (core)

```rust
// UIKit core — ViewModifierExt
pub trait ViewModifierExt {
    fn musicPicker(self, presented: bool) -> Self;
    fn appStoreOverlay(self, presented: bool, app_id: impl Into<String>) -> Self;
    fn manageSubscriptionsSheet(self, presented: bool) -> Self;
    fn swipeContainer(self, single_active: bool) -> Self;
    fn swipeAction(self, edge: SwipeEdge, label: impl Into<String>, destructive: bool) -> Self;
    fn navigationSplitViewBackground(self, style: ContainerBackground) -> Self;
    fn navigationContainerBackground(self, color: Color) -> Self;
    fn controlSize(self, size: ControlSize) -> Self;
    fn backgroundExtensionEffect(self, enabled: bool) -> Self;
    fn glassEffect(self, tint: Option<Color>) -> Self;
}
```

Core stores `ViewModifiers` per `ViewId` in a global map; the `View::to_gtk` path reads them. The TontooUI palette widgets below call these modifiers in their `render()` (so the state is visible in `get_modifiers(view_id)`) and then return their own `gtk::Widget` preview directly on window.

| Type | Values |
|---|---|
| `ControlSize` | `Mini` `Small` `Regular` `Large` `ExtraLarge` |
| `SwipeEdge` | `Leading` `Trailing` |
| `ContainerBackground` | `Default` `Blue` `Custom` |

## MusicPicker

```rust
pub struct MusicPicker { /* ... */ }
impl MusicPicker {
    pub fn new() -> Self;
    pub fn presented(self, shown: bool) -> Self;
    pub fn on_select(self, f: impl Fn(String) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — presents a music picker to select items from the Apple Music catalog.

- Preview: top bar `✕ Bar ✓` (`8px` radius, `#2c2c2e`), search `⌕ Your Library 🎙` pill, plus `Library` / `Playlists` labels, all directly on window.
- Size `220×120`.

## AppStoreOverlay

```rust
pub struct AppStoreOverlay { /* ... */ }
impl AppStoreOverlay {
    pub fn new(app_id: impl Into<String>) -> Self;
    pub fn presented(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — presents a StoreKit overlay when a given condition is true.

- Preview: `200×44` card `12px` radius with `36×36` blue `A` icon + `App Store` / `Developer Preview` text, directly on window.
- Size `220×70`.

## ManageSubscriptionsSheet

```rust
pub struct ManageSubscriptionsSheet { /* ... */ }
impl ManageSubscriptionsSheet {
    pub fn new() -> Self;
    pub fn presented(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — opens the manage subscriptions sheet.

- Preview: centered `10px` `You don't have any subscriptions.` directly on window (`220×40`).

## SwipeContainer

```rust
pub struct SwipeContainer { /* ... */ }
impl SwipeContainer {
    pub fn new() -> Self;
    pub fn single_active(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — only allows a single active swipe within a container.

- Preview: two `200×34` cards (`10px` radius) each with `Group1 Foo1/Bar1 + Delete` red pill (`32×14`), directly on window.
- Size `220×80`.

## SwipeAction

```rust
pub struct SwipeAction { /* ... */ }
impl SwipeAction {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds custom swipe actions to a row in a list or container.

- Preview: header `Trailing Swipe  Leading Swipe` + `Stateful Swipe  Swipe state: active` (`9px` dim) and a row `Swipe to Delete` + red `Delete` (`44×16`), directly on window.
- Size `220×70`.

## NavigationSplitViewBackground

```rust
pub struct NavigationSplitViewBackground { /* ... */ }
impl NavigationSplitViewBackground {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a background placement behind the content of a `NavigationSplitView`.

- Preview: `200×80` blue `#2a7fff` `10px` radius with small `a` label bottom-left, directly on window.
- Size `220×100`.

## NavigationContainerBackground

```rust
pub struct NavigationContainerBackground { /* ... */ }
impl NavigationContainerBackground {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the container background of the enclosing container using a view.

- Preview: `200×80` blue `#2a7fff` `10px` radius with centered `Foo` `11px` white, directly on window.
- Size `220×100`.

## ControlSizeView

```rust
pub struct ControlSizeView { /* ... */ }
impl ControlSizeView {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a control version that is the default size. Mirrors `SwiftUI.ControlSize`.

- Preview: vertical stack of 5 `Tap Me` pills (`22,26,30,36,42 × 70`, `999px` radius, `#0A84FF`) with `9–13px` white `SF Pro`, directly on window.
- Size `120×150`.
- Re-exports core `ControlSize` enum (`Mini` … `ExtraLarge`).

## BackgroundExtensionEffect

```rust
pub struct BackgroundExtensionEffect { /* ... */ }
impl BackgroundExtensionEffect {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds the background extension effect to the view. The view will be duplicated behind the extended background.

- Preview: two rows of 3 cats `60×44` (`8px` radius, `#d8b48a` …) — top row `opacity 1.0`, bottom row `0.45` to hint duplication, directly on window.
- Size `220×110`.

## GlassEffect

```rust
pub struct GlassEffect { /* ... */ }
impl GlassEffect {
    pub fn new() -> Self;
    pub fn tint(self, color: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — applies the Liquid Glass effect to a view. Uses `uikit::shader::glass` via `ShaderView` in production; the palette preview simulates it with a frosted `20px` radius box.

- Preview: `220×110` `Overlay` with 3 cats `66×80` (`#d8b48a` …) and a centered glass pill `120×40` (`rgba(255,255,255,0.28)` + `1px` white border + `box-shadow`, 3 × `Foo` white pills `28×18` inside), directly on window.
- Size `220×110`.
- `tint` defaults to `None` (white frosted, `0.28` dark / `0.22` light).

## Usage / Example

Run the gallery demo (recreates the 10-card screenshot):

```bash
cargo run --example views
```

Minimal usage:

```rust
use tontooui::prelude::*;
use uikit::view_modifiers::ViewModifierExt; // core backing

let picker = MusicPicker::new().on_select(|item| println!("{}", item));
let overlay = AppStoreOverlay::new("com.example.app").presented(true);
let glass = GlassEffect::new().tint(Color::new(1.0,1.0,1.0,0.28));

// Core modifier chain also works on a raw View
let raw = uikit::prelude::View::empty()
    .musicPicker(true)
    .glassEffect(Some(Color::new(1.0,1.0,1.0,0.25)))
    .controlSize(uikit::view_modifiers::ControlSize::Large);

let root = VStack::new()
    .spacing(8.0)
    .child(picker.to_view())
    .child(glass.to_view());
```

Category folder layout:

```
src/elements/views/
  mod.rs                               // category root
  music_picker.rs                      // MusicPicker
  app_store_overlay.rs                 // AppStoreOverlay
  manage_subscriptions_sheet.rs        // ManageSubscriptionsSheet
  swipe_container.rs                   // SwipeContainer
  swipe_action.rs                      // SwipeAction
  navigation_split_view_background.rs  // NavigationSplitViewBackground
  navigation_container_background.rs   // NavigationContainerBackground
  control_size.rs                      // ControlSizeView
  background_extension_effect.rs       // BackgroundExtensionEffect
  glass_effect.rs                      // GlassEffect
uikit/src/view_modifiers.rs            // core backing
```

## Cross References

- [ScrollView.md](ScrollView.md) -- `SwipeContainer` uses scrollable container semantics
- [Material.md](Material.md) -- `GlassEffect` uses frosted material + shader
- [ControlGroup.md](ControlGroup.md) -- `ControlSize` also affects control groups
