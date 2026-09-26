# Sidebar

Sidebar category in `src/elements/sidebars/sidebar.rs`: full-height
app navigation in `Sidebar` with `SidebarItem` rows (tinted SF
Symbol plus label), embedded traffic lights (replacing the
titlebar decoration), a left `BasicToolbar` pill with two optional
slot buttons plus a single far-right toggle pill, and a
`SearchField` row below the pills. Selecting an item switches the
right-side page, which the sidebar owns. Expanded, the slot pill
sits top right after the traffic lights and the toggle at the far
right edge with the title left in the content; collapsed, both
show far right in the content and the title moves next to the
traffic lights.

## Geometry

| Token | Value |
|---|---|
| `SIDEBAR_W` / `SIDEBAR_MIN_W` | 240 px default width / 220 px minimum |
| `SIDEBAR_ROW_H` | 24 px item rows |
| `SIDEBAR_PAD` | 16 px sidebar inset |
| `SIDEBAR_ICON_SIZE` / `SIDEBAR_ICON_GAP` | 14 px icon box / 6 px gap |
| `SIDEBAR_LABEL_SIZE` / `SIDEBAR_TITLE_SIZE` | 9.5 px item labels / 19 px semibold toolbar title |
| `SIDEBAR_TRAFFIC_TOP` / `SIDEBAR_BAR_TOP` / `SIDEBAR_ITEMS_TOP` | 22 px lights top / pills row centered on lights / 108 px items top |
| `SIDEBAR_TOOLBAR_H` | 64 px content toolbar height (collapsed pills row) |
| `SIDEBAR_COLLAPSE_SECONDS` | 0.22 s collapse slide plus fade (`CubicOut`) |
| `SIDEBAR_MIN_W` / `SIDEBAR_MAX_W` | 120 px absolute floor / 480 px drag cap |
| `SIDEBAR_RESIZE_HIT` / `SIDEBAR_REOPEN_HIT` | 6 px edge grab half-width / 8 px collapsed reopen strip |
| `SIDEBAR_CLOSE_SLOP` | 48 px below minimum snaps shut |
| `SIDEBAR_PILL_GAP` | 12 px traffic cluster to left pill (clears glow) |
| `SIDEBAR_SEARCH_TOP` / `SIDEBAR_SEARCH_H` | 60 px search row top / 36 px search row height |

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
pub fn left_button(self, slot: usize, icon: impl Into<String>, on_press: impl FnMut() + 'static) -> Self
pub fn set_left_button(&mut self, slot: usize, icon: impl Into<String>, on_press: impl FnMut() + 'static) -> bool
pub fn clear_left_button(&mut self, slot: usize)
pub fn search_field(self, show: bool) -> Self
pub fn set_search_field(&mut self, show: bool)
pub fn on_search(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn search_text(&self) -> &str
pub fn set_search_text(&mut self, text: impl Into<String>)
pub fn search_text_cursor(&self) -> bool
pub fn toggle_button(self, show: bool) -> Self
pub fn set_toggle_button(&mut self, show: bool)
pub fn collapsible(self, collapsible: bool) -> Self
pub fn set_collapsible(&mut self, collapsible: bool)
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn on_collapse(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn set_title(&mut self, title: impl Into<String>)
pub fn clear_title(&mut self)
pub fn selected_index(&self) -> usize
pub fn select(&mut self, index: usize) -> bool
pub fn is_collapsed(&self) -> bool
pub fn is_animating(&self) -> bool
pub fn is_resizing(&self) -> bool
pub fn wants_resize_cursor(&self, x: f64, y: f64) -> bool
pub fn min_bar_w(&self) -> f32
pub fn set_width(&mut self, px: f32)
pub fn width_value(&self) -> f32
pub fn update_progress(&mut self, elapsed: f32)
pub fn collapse_sample(from: f32, to: f32, elapsed: f32) -> (f32, bool)
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
  [Toolbar.md](Toolbar.md)), both fixed at the top: the left pill
  holds two optional slots (slot 0 first, slot 1 second; each set
  via `left_button` with an icon plus press callback, removed via
  `clear_left_button`; unset slots stay absent and out-of-range
  slots are rejected), the toggle rides alone far right (shown by
  default, optional via `toggle_button`). An empty left pill is
  skipped entirely. `collapsible(false)` keeps the toggle visible
  but gray and ignores its clicks. Pill clicks land in
  shared pending state and apply on the next mouse-up or draw, so
  unit tests never need a draw in between.
- Item clicks select (firing `on_select` on change); programmatic
  `select` returns false out of range. Slot callbacks only fire
  their own press action (history stays the app's job, see the
  demo: slot 0 goes back, slot 1 jumps to Notifications).
- The single toggle flips the column and fires `on_collapse`;
  `set_collapsed` stays silent and snaps at once. `toggle_sidebar`
  slides plus fades over `SIDEBAR_COLLAPSE_SECONDS` (`CubicOut`,
  same `Tween` idiom as the alerts): the column shrinks from the
  right edge while its body fades, both pill layouts crossfade and
  the title plus page slide with the live width. `is_collapsed`
  flips at once while the visuals catch up; `is_animating` reports
  the flight, `update_progress` advances it (`draw` feeds the live
  clock, tests feed fake time) and `collapse_sample` samples the
  curve purely.
- Hovering the column edge shows the resize cursor
  (`wants_resize_cursor` maps to `CursorKind::ResizeColumn` in
  `App::cursor`, see the demo); holding and dragging resizes live
  between `min_bar_w` and `SIDEBAR_MAX_W`. `min_bar_w` fits
  traffic plus every shown pill and shrinks when the dev hides
  slots or the toggle, never below `SIDEBAR_MIN_W`. Dragging past
  the minimum snaps shut with the collapse animation; grabbing the
  content left strip while collapsed reopens the same way.
  `is_resizing` reports the drag and the divider turns accent
  while hovered or held.
- The search row (a `SearchField`, see [Textfield.md](Textfield.md))
  shows by default below the pills (hidden via `search_field`).
  Clicking it focuses typing there; clicking anywhere else hands
  focus back, and `text` plus `key` reach the row only while it
  holds focus, else the active page. Typing filters the item rows
  live by label substring and `on_search` fires with the full text
  on every edit; row clicks select the real item index. The app
  returns the I-beam from `App::cursor` while
  `search_text_cursor` holds.
- `wants_backdrop` stays true while the Lens pills are on screen;
  return it from `App::wants_backdrop` like the toolbar demo.
- `press` reports traffic hits for the shell `WindowCommand`
  mapping (never forward those presses to `mouse_down`).
  `drag_rect` cuts out the traffic cluster and the pill group:
  expanded it spans the sidebar traffic band up to the pills
  (pill presses must reach the app, never start a window-drag),
  collapsed the content band up to the far-right pill group.
- `page_text` and `page_key` reach the active page through the
  `View` protocol; anything beyond that downcasts through
  `page_mut`.

## Usage / Example

Run `cargo run --example sidebar`: five tinted items with text
pages, slot 0 going back through history, slot 1 jumping to
Notifications, a live-filter search row and the toggle collapsing.
No titlebar: the sidebar fills the viewport and owns the
decoration.

```rust
let mut bar = Sidebar::new(vec![
    SidebarItem::new("General", "gear"),
    SidebarItem::new("Storage", "internaldrive"),
])
.page(settings_form)
.left_button(0, "chevron.left", || go_back());
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
- [Textfield.md](Textfield.md) – search row field
- [Groupbox.md](Groupbox.md) – sidebar body fill
- [Form.md](Form.md) – settings pages for sidebar content
- [Layout.md](Layout.md) – `View` protocol (`text`, `key`, `mouse_wheel`)
- [Theme.md](Theme.md) – accent icons and mode colors
