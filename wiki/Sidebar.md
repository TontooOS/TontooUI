# Sidebar

Sidebar category in `src/elements/sidebars/sidebar.rs`: full-height
app navigation in `Sidebar` with `SidebarItem` rows (tinted SF
Symbol plus label), embedded traffic lights (replacing the
titlebar decoration), a left `BasicToolbar` pill for back and dev
icons plus a single far-right toggle pill. Selecting an item
switches the right-side page, which the sidebar owns. Expanded,
the back pill sits top left in the sidebar and the toggle top
right with the title left in the content; collapsed, both show
far right in the content and the title moves next to the traffic
lights.

## Geometry

| Token | Value |
|---|---|
| `SIDEBAR_W` / `SIDEBAR_MIN_W` | 240 px default width / 220 px minimum |
| `SIDEBAR_ROW_H` | 46 px item rows |
| `SIDEBAR_PAD` | 16 px sidebar inset |
| `SIDEBAR_ICON_SIZE` / `SIDEBAR_ICON_GAP` | 22 px icon box / 12 px gap |
| `SIDEBAR_LABEL_SIZE` / `SIDEBAR_TITLE_SIZE` | 17 px item labels / 19 px semibold toolbar title |
| `SIDEBAR_TRAFFIC_TOP` / `SIDEBAR_BAR_TOP` / `SIDEBAR_ITEMS_TOP` | 22 px lights top / pills row centered on lights / 100 px items top |
| `SIDEBAR_TOOLBAR_H` | 64 px content toolbar height (collapsed pills row) |

Traffic geometry (`TRAFFIC_LEFT`, `TRAFFIC_SIZE`, `TRAFFIC_GAP`)
and colors come from [Titlebar.md](Titlebar.md); the sidebar body
reuses the `GroupBox` fill.

## SidebarItem

```rust
pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self
pub fn tint(self, color: Color) -> Self
pub fn label(&self) -> &str
pub fn set_label(&mut self, label: impl Into<String>)
```

Icon tint defaults to the theme accent.

## Sidebar

```rust
pub fn new(items: Vec<SidebarItem>) -> Self
pub fn page(self, page: impl View + 'static) -> Self
pub fn width(self, px: f32) -> Self
pub fn back_button(self, show: bool) -> Self
pub fn set_back_button(&mut self, show: bool)
pub fn toggle_button(self, show: bool) -> Self
pub fn set_toggle_button(&mut self, show: bool)
pub fn collapsible(self, collapsible: bool) -> Self
pub fn set_collapsible(&mut self, collapsible: bool)
pub fn toolbar_button(self, icon: impl Into<String>, on_press: impl FnMut() + 'static) -> Self
pub fn add_toolbar_button(&mut self, icon: impl Into<String>, on_press: impl FnMut() + 'static)
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn on_back(self, callback: impl FnMut() + 'static) -> Self
pub fn on_collapse(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn set_title(&mut self, title: impl Into<String>)
pub fn clear_title(&mut self)
pub fn selected_index(&self) -> usize
pub fn select(&mut self, index: usize) -> bool
pub fn is_collapsed(&self) -> bool
pub fn set_collapsed(&mut self, collapsed: bool)
pub fn toggle_sidebar(&mut self)
pub fn page_mut(&mut self, index: usize) -> Option<&mut dyn View>
pub fn active_page_mut(&mut self) -> Option<&mut dyn View>
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn wants_backdrop(&self) -> bool
pub fn press(&mut self, x: f64, y: f64) -> Option<TrafficAction>
pub fn drag_rect(&self) -> (f32, f32, f32, f32)
pub fn set_hover(&mut self, x: f32, y: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
pub fn page_text(&mut self, text: &str)
pub fn page_key(&mut self, key: Key) -> bool
```

- Pages match items by order; missing pages stay empty. The
  toolbar title follows the selected label until `set_title`
  overrides it (`clear_title` restores the follow mode).
- The toolbar holds two `BasicToolbar` pills (see
  [Toolbar.md](Toolbar.md)): back (optional via `back_button`) plus
  dev icons share the left pill, the toggle rides alone far right
  (optional via `toggle_button`). An empty left pill is skipped
  entirely. `collapsible(false)` keeps the toggle visible but gray
  and ignores its clicks. Pill clicks land in
  shared pending state and apply on the next mouse-up or draw, so
  unit tests never need a draw in between.
- Item clicks select (firing `on_select` on change); programmatic
  `select` returns false out of range. The back button only fires
  `on_back` (history stays the app's job, see the demo).
- The single toggle flips the column and fires `on_collapse`;
  `set_collapsed` stays silent.
- Dev icons carry their own press callbacks inside the left
  pill. The example adds one after construction so its callback
  can hold a shared sidebar handle.
- `wants_backdrop` stays true while the Lens pills are on screen;
  return it from `App::wants_backdrop` like the toolbar demo.
- `press` reports traffic hits for the shell `WindowCommand`
  mapping (never forward those presses to `mouse_down`).
  `drag_rect` cuts out the traffic cluster: expanded it spans the
  sidebar traffic band, collapsed the content band up to the
  far-right pill group.
- `page_text` and `page_key` reach the active page through the
  `View` protocol; anything beyond that downcasts through
  `page_mut` (see `examples/sidebar.rs` for the shared-handle
  pattern).

## Usage / Example

Run `cargo run --example sidebar`: five tinted items with text
pages, back history, a dev search button jumping to Notifications
and collapse. No titlebar: the sidebar fills the viewport and owns
the decoration.

```rust
let mut bar = Sidebar::new(vec![
    SidebarItem::new("General", "gear"),
    SidebarItem::new("Storage", "internaldrive"),
])
.page(settings_form);
bar.set_theme(accent, true);
```

Wire the shell like a titlebar app (see `examples/sidebar.rs`):

```rust
fn mouse_down(&mut self, x: f64, y: f64) {
    if let Some(action) = self.sidebar.borrow_mut().press(x, y) {
        match action {
            TrafficAction::Close => self.command = Some(WindowCommand::Close),
            TrafficAction::Minimize => self.command = Some(WindowCommand::Minimize),
            TrafficAction::Maximize => self.command = Some(WindowCommand::ToggleMaximize),
        }
    } else {
        self.sidebar.borrow_mut().mouse_down(x, y);
    }
}
fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
    Some(self.sidebar.borrow().drag_rect())
}
```

## Cross References

- [Titlebar.md](Titlebar.md) – traffic geometry, colors and actions
- [Toolbar.md](Toolbar.md) – toolbar pills, cells and actions
- [Images.md](Images.md) – SF Symbol row icons
- [Groupbox.md](Groupbox.md) – sidebar body fill
- [Form.md](Form.md) – settings pages for sidebar content
- [Layout.md](Layout.md) – `View` protocol (`text`, `key`, `mouse_wheel`)
- [Theme.md](Theme.md) – accent icons and mode colors
