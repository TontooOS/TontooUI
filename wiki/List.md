# List

List is a SwiftUI-style list with sections, disclosure and outline groups. It renders a vertical stack of sections with headers, footers and rows, supporting all SwiftUI list styles and modifiers. Background is transparent — rows sit directly on the app background (`#1d1d1d` Dark / `#ececec` Light) — with Light/Dark adaptive colors and SF Pro typography.

## Constructor

```rust
pub fn new() -> Self
```

Creates an empty list. Add sections via `.section()` or style via `.list_style()`.

## Types

```rust
pub enum ListStyle { Plain, Inset, InsetGrouped, Sidebar, Elliptical, Carousel, Bordered }
pub struct ListSection { header: Option<String>, footer: Option<String>, rows: Vec<ListRow> }
pub struct ListRow { label: String, detail: Option<String>, badge: Option<u32> }
pub struct OutlineGroup { header: String, children: Vec<ListRow> }
pub struct DisclosureGroup { header: String, children: Vec<ListRow> }
```

## Builder Methods — List

| Method | Signature | Description |
|---|---|---|
| `section` | `section(self, s: ListSection) -> Self` | Add section |
| `sections` | `sections(self, v: Vec<ListSection>) -> Self` | Replace sections |
| `list_style` | `list_style(self, s: ListStyle) -> Self` | Set style |
| `refreshable` | `refreshable(self) -> Self` | Show spinner (pull-to-refresh) |
| `section_index_visible` | `section_index_visible(self, v: bool) -> Self` | Show section index label ("A") |
| `background_prominent` | `background_prominent(self) -> Self` | Prominent background card |
| `compact_spacing` | `compact_spacing(self) -> Self` | Compact spacing between sections |
| `custom_section_spacing` | `custom_section_spacing(self, s: f32) -> Self` | Custom spacing |
| `min_row_height` | `min_row_height(self, h: f32) -> Self` | Minimum row height |
| `min_header_height` | `min_header_height(self, h: f32) -> Self` | Minimum header height |
| `outline_group` | `outline_group(self, g: OutlineGroup) -> Self` | Hierarchical outline |
| `disclosure_group` | `disclosure_group(self, g: DisclosureGroup) -> Self` | Collapsible disclosure |

## Builder Methods — ListSection

| Method | Signature | Description |
|---|---|---|
| `header` | `header(self, h: impl Into<String>) -> Self` | Header text |
| `footer` | `footer(self, f: impl Into<String>) -> Self` | Footer text |
| `row` | `row(self, r: ListRow) -> Self` | Add row |
| `rows` | `rows(self, v: Vec<ListRow>) -> Self` | Replace rows |
| `header_prominent` | `header_prominent(self) -> Self` | Increased prominence (bold, strong color) |
| `section_margins` | `section_margins(self, h: f32, v: f32) -> Self` | Margins |
| `row_spacing` | `row_spacing(self, s: f32) -> Self` | Vertical spacing between rows |
| `min_header_height` | `min_header_height(self, h: f32) -> Self` | Minimum header height |

## Builder Methods — ListRow

| Method | Signature | Description |
|---|---|---|
| `new` | `new(label: impl Into<String>) -> Self` | Create row |
| `detail` | `detail(self, d: impl Into<String>) -> Self` | Secondary detail |
| `badge` | `badge(self, n: u32) -> Self` | Red pill badge |
| `badge_prominent` | `badge_prominent(self) -> Self` | Prominent badge |
| `row_background` | `row_background(self, c: Color) -> Self` | Custom row background |
| `separator_hidden` | `separator_hidden(self, h: bool) -> Self` | Hide row separator |
| `section_separator_hidden` | `section_separator_hidden(self, h: bool) -> Self` | Hide section separator |
| `separator_tint` | `separator_tint(self, c: Color) -> Self` | Separator tint |
| `tint` | `tint(self, c: Color) -> Self` | Row text tint |
| `move_disabled` | `move_disabled(self, v: bool) -> Self` | Disable move (dimmed) |
| `delete_disabled` | `delete_disabled(self, v: bool) -> Self` | Disable delete (dimmed) |
| `swipe_action` | `swipe_action(self, label: impl Into<String>) -> Self` | Blue swipe button |

## Behavior

- Styles: `InsetGrouped`/`Inset` use card (`#2c2c2e` Dark / `#ffffff` Light + `10px` radius + border), `Sidebar` sidebar look, `Elliptical` `24px` pill, `Carousel` horizontal cards `100×60`, `Bordered` outer `1px` border, `Plain` transparent.
- Separators: `1px` `separator { background }` Dark `#3a3a3d` Light `#d1d1d6`, tintable per row/section, hidden via `separator_hidden`.
- Row height default `36px`, header `22px`, SF Pro 13px label, 11px footer/header-muted `rgba(235,235,245,0.55)` Dark / `rgba(60,60,67,0.60)` Light.
- Badge: `999px` red `#ff3b30` white 11px bold.
- Swipe action: blue `0A84FF` pill button.
- Refreshable shows `GtkSpinner` centered above sections.

## Usage / Example

```rust
use tontooui::prelude::*;

// InsetGrouped with header/footer, badge and hidden separator
let list = List::new()
    .list_style(ListStyle::InsetGrouped)
    .section(
        ListSection::new()
            .header("Header").footer("Footer")
            .row(ListRow::new("Bar1"))
            .row(ListRow::new("Bar2").badge(2))
    )
    .section(
        ListSection::new()
            .header("Header").footer("Footer")
            .row(ListRow::new("Foo").separator_hidden(true))
    );
```

See `examples/lists.rs` for the 1:1 demo (32 variants: OutlineGroup, DisclosureGroup, EditButton, Sidebar, InsetGrouped, Inset, Elliptical, Carousel, Bordered, Section Index, Move/Delete Disabled, Refreshable, SwipeAction, Badges, Backgrounds, Separators, Tints, Margins, Spacings, Header/Row heights, Insets, Prominence) directly on background, pure TontooUI API, Ampeln visible.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [ViewThatFits.md](ViewThatFits.md) -- adaptive container used with lists
- [Divider.md](Divider.md) -- separator line (also `ListRow` separator)
