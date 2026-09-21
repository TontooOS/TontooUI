# Theme

System theme from the settings daemon: dark/light mode plus accent color.
`ThemeWatcher` polls the daemon (1 s interval, revision-guarded) and
crossfades the whole palette over 0.25 s, so mode switches animate live
instead of snapping.

## Mode and Accent

```rust
pub enum ThemeMode {
    Dark,
    Light,
}
```

```rust
pub enum Accent {
    Multicolor,
    Blue,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Cyan,
    Indigo,
    Purple,
    Purple2,
    Pink,
    Gray,
}
```

`Accent::from_str` falls back to `Multicolor` for unknown input.
`Accent::hex` returns the Settings app display hex; `Multicolor` renders
blue (`#007AFF`). `Accent::color` converts to a Peniko color.

```rust
pub struct Theme {
    pub mode: ThemeMode,
    pub accent: Accent,
}
```

Default is dark mode with multicolor accent.

## Palette

```rust
pub struct Palette {
    pub bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub titlebar_bg: Color,
    pub titlebar_text: Color,
    pub divider: Color,
    pub accent: Color,
}
```

```rust
pub fn palette(&self) -> Palette
```

Resolved per mode on `Theme`. Dark body is `#1d1d1d`, light body
`#ececec`; the titlebar sits slightly off the body (lighter in dark,
darker in light).

## Watcher

```rust
pub fn new() -> Self
pub fn theme(&self) -> Theme
pub fn poll(&mut self, now_secs: f64) -> bool
pub fn palette(&mut self, now_secs: f64) -> Palette
```

`poll` reads `customize_get` through CoreSettings at most once per
`THEME_POLL_SECONDS` (1.0 s) and starts a fade when the theme changed
(revision-guarded, so idle systems cost one cheap socket read per
second). A missing daemon keeps the current theme, never an error.
`palette` blends from the previous to the current palette with
`CubicOut` over `THEME_FADE_SECONDS` (0.25 s) and returns the exact
target once finished.

Unix only (daemon socket). Elsewhere the watcher serves defaults and
never polls.

```rust
pub fn set_focused(&mut self, focused: bool, now_secs: f64)
pub fn focused(&self) -> bool
```

Inactive window state like macOS: unfocused windows desaturate the whole
palette by luminance (gray glass instead of glass color, colorless text,
monochrome accent) through the same 0.25 s fade instead of snapping.
Forward the shell focus event here; refocusing restores the exact live
palette. Traffic lights dim separately in the bar (see
[Titlebar.md](Titlebar.md)).

```rust
pub fn desaturate(color: Color) -> Color
```

Grays a single color by luminance, keeping alpha. Future glass materials
and elements read the (possibly desaturated) palette, so they turn gray
and non-glass automatically when the window is inactive.

## Wiring

The shell reads `App::background` every frame for the window body; apps
override it with the watcher palette. Themed elements take palette
colors per frame:

```rust
self.watcher.poll(time_secs);
let palette = self.watcher.palette(time_secs);
self.bg = palette.bg;
self.bar.set_palette(palette.titlebar_bg, palette.titlebar_text, palette.divider);
```

`Titlebar::set_palette` rebuilds title glyphs on text color change, so
mid-fade colors stay crisp without extra work from callers. Rebuilt text
elements rejoin through their own `set_color`.

## Usage / Example

Poll the watcher per frame, override `App::background`, forward the
palette to themed views. Switch the daemon theme (`customize_set
{"theme": "light"}`) while the app runs and watch the 0.25 s crossfade.

## Cross References

- [Renderer.md](Renderer.md) – `App::background`, frame pipeline, `View`/`App`
- [Layout.md](Layout.md) – views holding themed content
- [Titlebar.md](Titlebar.md) – `set_palette`
- [Animation.md](Animation.md) – easing and tween drivers used by the fade
