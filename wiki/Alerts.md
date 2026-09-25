# Alerts

Alerts category in `src/elements/alerts/`: `BasicAlert` in `basic.rs`
is a modal dialog over the app with a dimmed backdrop, a frosted
LiquidGlass card, a centered title, a message and one (OK) or two
(Cancel + OK) action buttons, `ActionAlert` in `action.rs` is the
action variant with a leading-aligned title and message plus exactly
two side-by-side buttons with custom tints (e.g. gray Cancel plus
red Delete), and `ConfirmationDialog` in `confirm.rs` is the choice
variant with only a centered title and a vertical stack of
full-width option buttons plus a trailing cancel, and `IconAlert` in
`icon.rs` is the icon variant with an SF Symbol on the left, a
leading-aligned title plus message on its right, and any number of
full-width action buttons stacked below. None can be
dismissed by clicking outside — only the buttons close them.
Entrance and exit fade through an engine tween; the app triggers
them with `show` (e.g. from its own buttons). The action variants
report their button as an `AlertEvent`; the plain `BasicAlert`
reports no such event. While visible the app drives
`Titlebar::set_modal_blocked`, which turns the red light gray and
unclickable. Alert buttons react to clicks only: hover does nothing
(`Button::hover_effect(false)`).

## Geometry

| Token | Value |
|---|---|
| `ALERT_WIDTH` / `ALERT_RADIUS` | 315 px card width / 18 px corner radius (middle size) |
| `ALERT_PAD` | 18 px inner padding |
| `ALERT_TITLE_SIZE` / `ALERT_MESSAGE_SIZE` | 12.75 px semibold title / 11.25 px message, wrapping |
| `ALERT_TITLE_GAP` / `ALERT_MESSAGE_GAP` | 6 px title gap / 15 px button gap |
| `ALERT_BUTTON_H` / `ALERT_BUTTON_GAP` | 33 px button height / 9 px button gap |
| `ALERT_ICON_SIZE` / `ALERT_ICON_GAP` | 44 px SF icon box / 12 px icon-text gap |
| `ALERT_FADE_SECONDS` | 0.25 s engine fade in/out (shared by all variants) |
| `ALERT_DIM_ALPHA` | 77 alpha black dim over the app behind the card |
| `ALERT_TITLE_DARK` / `ALERT_TITLE_LIGHT` | White / `#272727` title text |
| `ALERT_MESSAGE_DARK` / `ALERT_MESSAGE_LIGHT` | White 220 alpha / dark 220 alpha message text |
| `ALERT_ACCENT` | `#007AFF` OK button fill |
| `ALERT_CANCEL` | `#8E8E93` Cancel button tint |

## AlertAction / AlertButton

```rust
pub enum AlertAction { Ok, Cancel }
pub struct AlertButton { pub label: String, pub action: AlertAction, pub color: Option<Color> }
pub fn ok(label: impl Into<String>) -> Self
pub fn cancel(label: impl Into<String>) -> Self
pub fn color(self, color: Color) -> Self
```

- Without `.color()`, `Ok` renders prominent blue (capsule) and
  `Cancel` renders tinted gray (translucent fill, `ALERT_CANCEL`
  label). A custom color renders the tinted style with that tint
  (translucent fill, colored label), like a red delete button, and
  wins over the role default.

## BasicAlert

```rust
pub fn ok(title: impl Into<String>, message: impl Into<String>) -> Self
pub fn buttons(title: impl Into<String>, message: impl Into<String>, buttons: Vec<AlertButton>) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_message(&mut self, message: impl Into<String>)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn viewport(&self) -> (f32, f32, f32, f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_open(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity_at(&self, elapsed: f32) -> f32
pub fn opacity_value(&self) -> f32
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertAction>
```

- `buttons` with one entry fills the row; two entries share it
  (Cancel left, OK right, in def order). Empty falls back to a single
  OK; longer lists clamp to the first two.
- `show` fades in from transparent (`Tween` 0 to 1, `CubicOut`);
  `dismiss` fades out from the current opacity and hides itself when
  done. `is_open` is true only while fully open (accepting clicks);
  `is_visible` covers fading in, open and fading out — drive
  `Titlebar::set_modal_blocked` from it.
- `mouse_down`/`mouse_up` forward to the buttons only while fully
  open; everything else (outside clicks, mid-fade clicks) is
  swallowed and returns `None`. Each button fires its action once per
  click through its press callback.
- The card measures `ALERT_WIDTH` by content height (wrapping title
  and message inside the padding) and centers in the viewport set via
  `set_viewport` (usually the content area below the titlebar). Draw
  paints the dim, the frosted `GlassContainer` card, the texts and
  the buttons inside one opacity layer, so the whole overlay fades
  as one.
- Missing viewport (zero size) draws nothing; a hidden alert draws
  nothing.

## ActionAlert

```rust
pub struct AlertEvent { pub index: usize, pub action: AlertAction, pub label: String }
pub fn new(title: impl Into<String>, message: impl Into<String>, left: AlertButton, right: AlertButton) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_message(&mut self, message: impl Into<String>)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_open(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity_value(&self) -> f32
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertEvent>
```

- Same modal core as `BasicAlert` (frosted card, dim, engine fade,
  `show`/`dismiss`, viewport centering), but the title and message
  are leading-aligned and the row always holds exactly two
  side-by-side buttons with custom tints.
- A press returns the button as an `AlertEvent` (`index` 0 left or 1
  right, plus its action and label) once per click; the plain
  `BasicAlert` reports no such event. Clicks outside or mid-fade are
  swallowed and return `None`.

## ConfirmationDialog

```rust
pub fn new(title: impl Into<String>, options: Vec<AlertButton>) -> Self
pub fn cancel(self, label: impl Into<String>) -> Self
pub fn no_cancel(self) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn defs_value(&self) -> &[AlertButton]
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_open(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity_value(&self) -> f32
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertEvent>
```

- Same modal core as the other variants (frosted card, dim, engine
  fade, `show`/`dismiss`, viewport centering), but with only a
  centered title and a vertical stack of full-width option buttons
  plus a trailing cancel — made for more than two actions.
- Empty options fall back to a single OK; `cancel` renames the
  trailing cancel, `no_cancel` drops it (options only).
- The first option renders prominent blue (or its custom tint), the
  rest render tinted gray (or their custom tint). A press returns the
  button as an `AlertEvent` (stack index, action, label) once per
  click; clicks outside or mid-fade are swallowed.

## IconAlert

```rust
pub fn new(icon: impl Into<String>, title: impl Into<String>, message: impl Into<String>, actions: Vec<AlertButton>) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn set_icon_color(&mut self, color: Option<Color>)
pub fn icon_value(&self) -> &str
pub fn icon_color_value(&self) -> Option<Color>
pub fn cancel(self, label: impl Into<String>) -> Self
pub fn no_cancel(self) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, text: Color, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_message(&mut self, message: impl Into<String>)
pub fn defs_value(&self) -> &[AlertButton]
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_open(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity_value(&self) -> f32
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertEvent>
```

- Same modal core as the other variants (frosted card, dim, engine
  fade, `show`/`dismiss`, viewport centering), but with an SF Symbol
  (`ALERT_ICON_SIZE` box, top-aligned) on the left, a
  leading-aligned title plus message on its right, and any number of
  full-width action buttons stacked below.
- Without `.icon_color()` the glyph follows the theme text color via
  `set_theme` (which takes the text color, not the accent); a custom
  tint wins. Empty actions fall back to a single OK; a trailing
  Cancel is added by default (`cancel` renames it, `no_cancel` drops
  it).
- Every button renders tinted gray (or its custom tint), like the
  system prompt. A press returns the button as an `AlertEvent`
  (stack index, action, label) once per click; clicks outside or
  mid-fade are swallowed.

## Titlebar modal block

```rust
pub fn set_modal_blocked(&mut self, blocked: bool)
pub fn modal_blocked(&self) -> bool
```

- While blocked the red (close) light renders `TRAFFIC_INACTIVE`
  gray with no hover glyph, and `press` returns `None` for close
  hits. Minimize and maximize keep working.

## Button hover effect

```rust
pub fn hover_effect(self, enabled: bool) -> Self
pub fn set_hover_effect(&mut self, enabled: bool)
```

- Default on. Off (used by alert buttons) ignores `set_hover` and
  skips the hover fill: clicks still press, hover shows nothing.

## Usage / Example

```rust
use tontooui::elements::{ActionAlert, AlertButton, BasicAlert, View};

// Triggered from an app button:
alert.show();

// Per frame:
bar.set_modal_blocked(alert.is_visible());
alert.set_viewport(viewport.x, top, viewport.width, content_h);
alert.draw(scene, fonts, images);

// Input: modal, only the alert hears clicks while visible.
fn mouse_up(&mut self, x: f64, y: f64) {
    if alert.is_visible() {
        match alert.mouse_up(x, y) {
            Some(AlertAction::Ok) | Some(AlertAction::Cancel) => alert.dismiss(),
            None => {}
        }
        return;
    }
    // ...normal app input
}
```

Action variant with a custom-tinted delete button:

```rust
use tontooui::elements::{ActionAlert, AlertButton};
use vello::peniko::Color;

let mut alert = ActionAlert::new(
    "Delete Item?",
    "Are you sure you want to delete this item?",
    AlertButton::cancel("Cancel"),
    AlertButton::ok("Delete").color(Color::from_rgb8(0xff, 0x3b, 0x30)),
);
alert.show();

// Press reports the button as an event:
if let Some(event) = alert.mouse_up(x, y) {
    // event.index (0 left, 1 right), event.action, event.label.
    alert.dismiss();
}
```

See `examples/alert.rs` for the full demo (OK, OK/Cancel, action,
confirmation and icon alerts over buttons plus a toolbar, gray
blocked red light).

## Cross References

- [Glass.md](Glass.md) – frosted container used for the card
- [Button.md](Button.md) – action buttons and the hover effect toggle
- [Titlebar.md](Titlebar.md) – red light block while modal
- [Animation.md](Animation.md) – engine tween behind the fade
- [Layout.md](Layout.md) – `View` measure/place/draw contract
