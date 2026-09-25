# Outline Group

Outline category in `src/elements/outlines/outline.rs`:
`BasicOutlineGroup` renders a file-tree of `OutlineNode`s with
chevron folders, tinted SF Symbol icons, single-select rows and
animated expand/collapse, like the reference Documents tree.
Toggling tweens the children height (`OUTLINE_ANIM_SECONDS`,
`CubicOut`) while the block fades in/out through a clip layer and
the chevron morphs from `>` to `v`; rows below glide down. Node
paths are index vectors from the roots (e.g. `vec![0, 1]` is the
second child of the first root).

## Geometry

| Token | Value |
|---|---|
| `OUTLINE_ROW_H` | 32 px row height |
| `OUTLINE_INDENT` | 26 px indent per tree level |
| `OUTLINE_CHEV_SLOT` | 22 px reserved chevron slot (files leave it empty so icons align) |
| `OUTLINE_CHEV_W` / `OUTLINE_CHEV_H` | 7 px / 10 px chevron glyph |
| `OUTLINE_CHEV_STROKE` | 1.8 px round-cap chevron stroke |
| `OUTLINE_LABEL_SIZE` | 15 px labels |
| `OUTLINE_ICON_SIZE` / `OUTLINE_ICON_GAP` | 20 px SF Symbol box / 8 px gap |
| `OUTLINE_ANIM_SECONDS` | 0.25 s expand/collapse (height plus fade) |
| `OUTLINE_SELECTED_ALPHA` | 0.35 accent fill for selected rows |
| `OUTLINE_ACCENT` | `#007AFF` default icon tint |

## OutlineNode

```rust
pub fn folder(label: impl Into<String>) -> Self
pub fn file(label: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn expanded(self, expanded: bool) -> Self
pub fn child(self, node: OutlineNode) -> Self
pub fn children(self, nodes: Vec<OutlineNode>) -> Self
pub fn label(&self) -> &str
pub fn set_label(&mut self, label: impl Into<String>)
pub fn is_folder(&self) -> bool
pub fn has_children(&self) -> bool
pub fn is_expanded(&self) -> bool
pub fn progress(&self) -> f32
pub fn child_nodes(&self) -> &[OutlineNode]
```

- Folders expand while they have children (files never do);
  childless folders show no chevron and ignore toggles. Custom
  symbols (e.g. `"tag"`) still expand with children.
- `expanded` snaps without animation and applies in any builder
  order (children added later still open the node).
- `progress` is 0.0 closed, 1.0 open, between mid-tween; children
  fade and reveal with it.

## BasicOutlineGroup

```rust
pub fn new(roots: Vec<OutlineNode>) -> Self
pub fn selectable(self, selectable: bool) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn on_toggle(self, callback: impl FnMut(Vec<usize>, bool) + 'static) -> Self
pub fn on_select(self, callback: impl FnMut(Vec<usize>) + 'static) -> Self
pub fn selected_path(&self) -> Option<Vec<usize>>
pub fn select(&mut self, path: Vec<usize>) -> bool
pub fn clear_selection(&mut self)
pub fn node(&self, path: &[usize]) -> Option<&OutlineNode>
pub fn toggle(&mut self, path: Vec<usize>) -> bool
pub fn set_expanded(&mut self, path: &[usize], expanded: bool) -> bool
pub fn is_expanded(&self, path: &[usize]) -> bool
pub fn expand_all(&mut self)
pub fn collapse_all(&mut self)
pub fn content_height(&self) -> f32
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- Clicking a folder row selects it and toggles it; clicking a file
  row selects it. Clicks on empty space keep the selection.
  `toggle` fires `on_toggle`, programmatic `set_expanded` does
  not; `select` and `clear_selection` fire `on_select` on change
  (empty on clear). `expand_all` and `collapse_all` snap instantly
  without callbacks.
- Icons tint with the theme accent unless `icon_color` wins;
  unfocused windows desaturate like the palette.
- `measure` reports the content width with the animated height, so
  the group pages inside a `ScrollView` and rows below glide.

## Usage / Example

Run `cargo run --example outline`: the reference Documents tree
with the selection in the titlebar.

```rust
let mut tree = BasicOutlineGroup::new(vec![
    OutlineNode::folder("Documents")
        .expanded(true)
        .child(OutlineNode::file("Resume.pdf")),
]);
tree.set_theme(accent, true);
```

## Cross References

- [List.md](List.md) – `DisclosureGroup` single-level expand pattern
- [Images.md](Images.md) – SF Symbol row icons
- [Animation.md](Animation.md) – tween drivers behind the reveal
- [ScrollView.md](ScrollView.md) – paging long trees
- [Layout.md](Layout.md) – `View` protocol
- [Theme.md](Theme.md) – accent icons and unfocused desaturation
