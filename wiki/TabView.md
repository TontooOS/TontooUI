# TabView

SwiftUI-style TabView category for TontooUI, recreating the `TabView` containers, styles, and modifiers from the macOS 26 interface. The category contains 23 elements (5 `initializer`, 5 `style`, 13 `modifier`). All elements render directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## Tab

```rust
pub struct Tab { /* ... */ }
impl Tab {
    pub fn new(content: impl Widget + 'static) -> Self;
    pub fn title(self, title: impl Into<String>) -> Self;
    pub fn system_image(self, name: impl Into<String>) -> Self;
    pub fn image(self, path: impl Into<String>) -> Self;
    pub fn label_text(&self) -> &str;
    pub fn has_sidebar_label(&self) -> bool;
}
```

Initializer — a single tab with title, image/systemImage and detail content.
`content` is any `Widget` (e.g. `Text`) shown in the detail area when the tab
is selected. `title` sets the sidebar label (empty = icon-only/empty row, like
SwiftUI's `Tab { Text("0") }`). `system_image` takes an SF Symbol name and
renders a monochrome sidebar glyph (transparent background, white/black tint —
like Apple's sidebar tabs, not Settings-style color tiles).
`image` takes a PNG file path; missing files render as an empty icon slot
instead of a broken image.

## TabSection

```rust
pub struct TabSection { /* ... */ }
impl TabSection {
    pub fn new() -> Self;
    pub fn header(self, title: impl Into<String>) -> Self;
    pub fn tab(self, tab: Tab) -> Self;
    pub fn header_text(&self) -> Option<&str>;
    pub fn tabs(&self) -> &[Tab];
    pub fn tab_count(&self) -> usize;
    pub fn to_view(self) -> View;
}
```

Initializer — a container that you can use to add hierarchy within a tab view.
`header("Foo")` mirrors SwiftUI's `TabSection("Foo")`. Without tabs the element
keeps its `180x80` preview (pill bar and hierarchy hint, directly on window).
With tabs, `to_view` renders a static sidebar fragment (header plus rows,
first tab highlighted).

## TabBarOnlyTabViewStyle

```rust
pub struct TabBarOnlyTabViewStyle { /* ... */ }
impl TabBarOnlyTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a tab view style that displays a tab bar when possible. `180×80`.

## TabView

```rust
pub struct TabView { /* ... */ }
impl TabView {
    pub fn new() -> Self;
    pub fn title(self, title: impl Into<String>) -> Self;
    pub fn tab(self, tab: Tab) -> Self;
    pub fn section(self, section: TabSection) -> Self;
    pub fn selected(self, index: usize) -> Self;
    pub fn on_select(self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self;
    pub fn sidebar_width(self, w: f32) -> Self;
    pub fn show_toggle(self, show: bool) -> Self;
    pub fn tab_count(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn to_view(self) -> View;
}
```

Initializer — creates Tabs with title, image, systemImage and custom Label.
Without tabs the element keeps its `180×80` preview (three tabs `◉ ▭ ⬡` with
badge). With tabs it renders the functional Apple `.sidebarAdaptable`
container: a 48px title bar (traffic lights, sidebar toggle, bold title),
a sidebar (tabs and section headers, Apple-blue selection, monochrome
glyphs) with a 1px separator, and the selected tab's detail content on the
right. Clicking a row swaps the detail view immediately and fires `on_select`
with the flat tab index (top-level tabs first, then section tabs in order).
The toggle button collapses the sidebar. Empty sections are ignored.
`to_view` sizes the functional container to `780×520`.

## SearchTabRole

```rust
pub struct SearchTabRole { /* ... */ }
impl SearchTabRole {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — on Liquid Glass, this tab is placed in its own group. In left-to-right (LTR) layout...

## GroupedTabViewStyle

```rust
pub struct GroupedTabViewStyle { /* ... */ }
impl GroupedTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a tab view style that displays a tab bar that groups its tabs together. `180×80`.

## PageTabViewStyle

```rust
pub struct PageTabViewStyle { /* ... */ }
impl PageTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a TabViewStyle that displays a paged scrolling TabView. `180×80`.

## VerticalPageTabViewStyle

```rust
pub struct VerticalPageTabViewStyle { /* ... */ }
impl VerticalPageTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a TabViewStyle that displays a vertical TabView interaction and appearance.

## SidebarAdaptableTabViewStyle

```rust
pub struct SidebarAdaptableTabViewStyle { /* ... */ }
impl SidebarAdaptableTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a tab bar style that adapts to each platform.

## TabViewBottomAccessoryPlacement

```rust
pub struct TabViewBottomAccessoryPlacement { /* ... */ }
impl TabViewBottomAccessoryPlacement {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a placement of the bottom accessory in a tab view. You can use this to adjust...

## DefaultCollapsedTabSection

```rust
pub struct DefaultCollapsedTabSection { /* ... */ }
impl DefaultCollapsedTabSection {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the default expansion state for the section containing this tab to collapse...

## TabViewCustomizationBehavior

```rust
pub struct TabViewCustomizationBehavior { /* ... */ }
impl TabViewCustomizationBehavior {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — configures the customization behavior of customizable tab view content.

## TabViewCustomization

```rust
pub struct TabViewCustomization { /* ... */ }
impl TabViewCustomization {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — specifies the customizations to apply to the sidebar representation of the tab...

## TabViewSideBarFooter

```rust
pub struct TabViewSideBarFooter { /* ... */ }
impl TabViewSideBarFooter {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds a custom footer to the sidebar of a tab view. `180×60`.

## TabViewSideBarBottomBar

```rust
pub struct TabViewSideBarBottomBar { /* ... */ }
impl TabViewSideBarBottomBar {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds a custom bottom bar to the sidebar of a tab view.

## DefaultAdaptableTabBarPlacement

```rust
pub struct DefaultAdaptableTabBarPlacement { /* ... */ }
impl DefaultAdaptableTabBarPlacement {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — specifies the default placement for the tabs in a tab view using the adaptable...

## TabBarSectionActions

```rust
pub struct TabBarSectionActions { /* ... */ }
impl TabBarSectionActions {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds custom actions to a tab section.

## TabViewSideBarHeader

```rust
pub struct TabViewSideBarHeader { /* ... */ }
impl TabViewSideBarHeader {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — adds a custom header to the sidebar of a tab view.

## TabBadge

```rust
pub struct TabBadge { /* ... */ }
impl TabBadge {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — generates a badge for a tab from an integer value. Shows red `1` badge on pill.

## HiddenIndexPageTabViewStyle

```rust
pub struct HiddenIndexPageTabViewStyle { /* ... */ }
impl HiddenIndexPageTabViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a TabViewStyle that displays a paged scrolling TabView with a hidden index.

## ValueTabView

```rust
pub struct ValueTabView { /* ... */ }
impl ValueTabView {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — creates a tab view that uses a builder to create and specify selection values for...

## HideTabBarOnScrollDown

```rust
pub struct HideTabBarOnScrollDown { /* ... */ }
impl HideTabBarOnScrollDown {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — minimize the tab bar when downwards scrolling starts. Minimizing is supporte...

## BottomAccessory

```rust
pub struct BottomAccessory { /* ... */ }
impl BottomAccessory {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a modifier to place content above the tabs. `180×60`.

All `TabView` elements use the same preview shell: title `10px Semibold`, pill `140×26` (`#2c2c2e`/`#ffffff`, `999px` radius), icons `9px` dim, blue badge `14×14`, hint `7px` dim, directly on window — no extra card. Each `to_view` size is `180×80` (or `180×60` for footer/header/badge variants).

## Usage / Example

Functional sidebar tabs (ports the SwiftUI `.sidebarAdaptable` reference):

```rust
use tontooui::prelude::*;
use tontooui::{Tab, TabSection, TabView};

let view = TabView::new()
    .title("ExploreSwiftUISandbox")
    .tab(Tab::new(Text::new("0").font_size(28.0)))
    .section(TabSection::new().header("Foo").tab(
        Tab::new(Text::new("1").font_size(28.0))
            .title("1")
            .system_image("1.circle"),
    ))
    .tab(Tab::new(Text::new("2").font_size(28.0))
        .title("2")
        .image("cats24x24"))
    .tab(Tab::new(Text::new("3").font_size(28.0))
        .title("3")
        .system_image("3.circle"))
    .selected(0)
    .on_select(|i| println!("Tab selected: {}", i))
    .to_view();
```

Run the live demo (needs a display; hide the system bar so the
container draws its own title bar, like the Apple reference):

```bash
cargo run --example sidebar_tabs
```

Run the gallery demo (recreates the 22-card screenshots):

```bash
cargo run --example tab_views
```

Minimal usage:

```rust
use tontooui::prelude::*;

let section = TabSection::new().to_view();
let tabview = TabView::new().to_view();
let badge = TabBadge::new().to_view();
let hide = HideTabBarOnScrollDown::new().to_view();

let root = VStack::new()
    .spacing(8.0)
    .child(section)
    .child(tabview);
```

Category folder layout:

```
src/elements/tab_views/
  mod.rs
  tab.rs
  tab_section.rs
  tab_bar_only_style.rs
  tab_view.rs
  search_tab_role.rs
  grouped_style.rs
  page_style.rs
  vertical_page_style.rs
  sidebar_adaptable_style.rs
  bottom_accessory_placement.rs
  default_collapsed_section.rs
  customization_behavior.rs
  customization.rs
  side_bar_footer.rs
  side_bar_bottom_bar.rs
  default_adaptable_placement.rs
  section_actions.rs
  side_bar_header.rs
  badge.rs
  hidden_index_page_style.rs
  value_tab_view.rs
  hide_on_scroll.rs
  bottom_accessory.rs
```

## Cross References

- [List.md](List.md) -- `TabSection` hierarchy mirrors `ListSection`
- [Sidebar.md](Sidebar.md) -- standalone sidebar (traffic lights, search, sections, Apple-blue selection)
- [View.md](View.md) -- view modifiers category
