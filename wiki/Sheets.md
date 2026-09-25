# Sheets

Sheets category in `src/elements/sheets/`: `BasicSheet<V>` in
`basic.rs` is a modal card with any child content (text, buttons,
...), centered over the app with a dimmed backdrop. ESC closes it,
or a close button the app puts into the content. `Small` wraps the
intrinsic content size; `Quarter`, `Half` and `Large` fill 25%, 50%
or 75% of the viewport in both directions for rich content. The card
background is changeable. Entrance and exit fade through an engine
tween; the app triggers it with `show` and routes input while
visible. Like alerts, an open sheet blocks the red traffic light
(drive `Titlebar::set_modal_blocked` from `is_visible`) and clicks
outside are swallowed. Sheets are solid cards, so unlike frosted
alerts they need no backdrop blur pass.

## Geometry

| Token | Value |
|---|---|
| `SHEET_RADIUS` | 20 px card corner radius |
| `SHEET_FADE_SECONDS` | 0.25 s engine fade in/out |
| `SHEET_DIM_ALPHA` | 77 alpha black dim over the app behind the card |
| `SHEET_BG_DARK` / `SHEET_BG_LIGHT` | `#2C2C2E` / `#FFFFFF` default card fill |
| `SHEET_BUTTON_BG_DARK` | `#1E2022` bordered button fill on a dark card: darker than the card so buttons never melt into the sheet (light cards already contrast `#E9E9EB` buttons) |
| `SHEET_BORDER_DARK` / `SHEET_BORDER_LIGHT` | White 36 alpha / black 31 alpha 1 px border |
| `SHEET_SHADOW` / `SHEET_SHADOW_BLUR` / `SHEET_SHADOW_DY` | Black 64 alpha, 16 px blur, 4 px offset drop shadow |

## SheetSize

```rust
pub enum SheetSize { Small, Quarter, Half, Large }
pub fn fraction(&self) -> Option<f32>
```

- `Small` (default) is the intrinsic content size, centered.
  `Quarter`, `Half` and `Large` are 0.25, 0.5 and 0.75 of the
  viewport width and height, centered, with the content filling the
  card.

## BasicSheet

```rust
pub fn new(child: V) -> Self
pub fn size(self, size: SheetSize) -> Self
pub fn background(self, color: Color) -> Self
pub fn set_theme(&mut self, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_size(&mut self, size: SheetSize)
pub fn set_background(&mut self, color: Color)
pub fn clear_background(&mut self)
pub fn child_mut(&mut self) -> &mut V
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_open(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity_value(&self) -> f32
pub fn card_rect(&mut self, fonts: &mut FontSystem) -> (f32, f32, f32, f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn key(&mut self, key: Key) -> bool
```

- `show` fades in from transparent (`Tween` 0 to 1, `CubicOut`);
  `dismiss` fades out from the current opacity and hides itself when
  done. `is_open` is true only while fully open (accepting input);
  `is_visible` covers fading in, open and fading out — drive
  `Titlebar::set_modal_blocked` from it.
- `mouse_down`/`mouse_up` forward to the content through the `View`
  protocol only while fully open (buttons work via the `mouse_down`
  override on `View`); everything else is swallowed. `key` dismisses
  on ESC while visible and reports true; anything else reports
  false. The app forwards its `key` here.
- `background` wins over the theme until `clear_background`
  restores the theme fill. Unfocused windows desaturate the card
  like the palette.
- The card measures intrinsic content (fractions resolve against
  the viewport on draw) and centers in the viewport set via
  `set_viewport` (usually the content area below the titlebar).
  Missing viewport (zero size) draws nothing; a hidden sheet draws
  nothing.

## View mouse_down

```rust
fn mouse_down(&mut self, _x: f64, _y: f64) {}
```

- New default on the `View` protocol (mirrors `mouse_up`), so
  wrappers like sheets can forward presses to generic content.
  `Button` overrides it with its press tracking; other elements
  keep the noop.
- Stacks (`VStack`, `HStack`, `ZStack`) and wrappers (`Padding`,
  `Background`, `Frame`) forward `mouse_down`/`mouse_up` to their
  children, so nested buttons fire without direct wiring (this
  fixed the sheet Dismiss button).
- Content buttons on a dark card need `SHEET_BUTTON_BG_DARK` as
  their fill (via `set_palette`): the default bordered gray matches
  the card and would melt into it.

## Usage / Example

```rust
use tontooui::elements::{BasicSheet, Button, ButtonStyle, SheetSize, View, VStack};

let mut sheet = BasicSheet::new(
    VStack::new()
        .spacing(20.0)
        .child(BasicText::new("This is a sheet!"))
        .child(Button::new("Dismiss").style(ButtonStyle::Bordered)),
);
sheet.set_size(SheetSize::Half);
sheet.show();

// Per frame:
bar.set_modal_blocked(sheet.is_visible());
sheet.set_viewport(viewport.x, top, viewport.width, content_h);
sheet.draw(scene, fonts, images);

// Input: modal, only the sheet hears clicks while visible.
fn mouse_up(&mut self, x: f64, y: f64) {
    if sheet.is_visible() {
        sheet.mouse_up(x, y);
        return;
    }
    // ...normal app input
}

// ESC closes the open sheet.
fn key(&mut self, key: Key) {
    if sheet.is_visible() {
        sheet.key(key);
    }
}
```

See `examples/sheet.rs` for the full demo (small, 25%, 50% and 75%
sheets with a dismiss button, ESC handling, gray blocked red light).

## Cross References

- [Alerts.md](Alerts.md) – modal dialogs with the same dim/fade/block pattern
- [Button.md](Button.md) – close buttons inside sheet content
- [Titlebar.md](Titlebar.md) – red light block while modal
- [Animation.md](Animation.md) – engine tween behind the fade
- [Layout.md](Layout.md) – `View` protocol incl. `mouse_down`
