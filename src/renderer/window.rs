use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use vello::kurbo::Affine;
use vello::peniko::{Color, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use wgpu::PresentMode;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, ResizeDirection, Window, WindowAttributes};

use super::backdrop::BackdropBlur;
use super::backdrop_stream::CompositorBackdrop;
use super::images::{ImageCache, ImageLoader};
use super::text::FontSystem;

/// Window background. Dark mode base color per TontooOS convention.
pub const BACKGROUND: Color = Color::from_rgb8(0x1b, 0x20, 0x22);

/// Safety margin kept free around a clamped window in logical px.
/// winit only reports the full monitor size, so this reserves room
/// for taskbars and docks when fitting windows to the screen.
pub const SCREEN_MARGIN: f32 = 48.0;
/// Minimum window dimension in logical px used when clamping to the
/// screen, so tiny displays still yield a usable window.
pub const MIN_WINDOW: u32 = 320;

/// Grab width reaching into the window body in logical px. Deliberately
/// narrow so edge content stays usable: the overlay scrollbar thumb (up
/// to 10 px wide at the content edge) must keep its hover and drag
/// without the resize zone stealing it.
pub const RESIZE_INNER_HIT: f32 = 3.0;
/// Grab width reaching out of the window body into the transparent shadow
/// rim (see `frame.rs`) in logical px. The wider outer band keeps the
/// edge easy to grab even though the inner band is narrow.
pub const RESIZE_OUTER_HIT: f32 = 10.0;
/// Corner square half-size in logical px around each body corner. Corners
/// are checked first with this wider band so diagonal resizing wins over
/// the straight edges near the corners.
pub const RESIZE_CORNER_HIT: f32 = 22.0;
/// Standard window corner radius in logical px. Follows the macOS 27 Golden
/// Gate direction: one fixed radius for all windows, tighter than Tahoe.
/// Physical pixels = value x window scale factor (17 pt is ~34 px at 2x).
pub const WINDOW_CORNER_RADIUS: f32 = 17.0;

/// Non-printable keys forwarded to the app. The `Select*`, `Copy`,
/// `Cut`, `Paste`, `Undo` and `Redo` variants arrive for Ctrl
/// shortcuts (Ctrl+A/C/X/V/Z/Y and Ctrl+Shift+Z); text fields handle
/// them as select all, clipboard, undo/redo and extending caret
/// motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Enter,
    Escape,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
}

/// Modifier keys held while an input event happened.
///
/// Unlike the two booleans of `App::set_modifiers` this is the full
/// state and travels with every raw key and pointer button event, so
/// apps that speak protocols (a terminal sending Ctrl chords or mouse
/// reports) never have to cache modifier transitions themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// Command / Windows key (`super` is a Rust keyword).
    pub super_key: bool,
}

impl Modifiers {
    /// True when any modifier key is held.
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.super_key
    }
}

/// Key identity for `App::raw_key`.
///
/// The shell maps the physical key, so the same key reports the same
/// value on every layout. Printable keys carry the produced character
/// in `RawKey::Character` (already shifted, so `A` is upper case),
/// which is what an app needs to write the byte the user expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawKey {
    /// A printable key: letter, digit or symbol, carrying the produced
    /// character. With Ctrl or Alt held the character is the unshifted
    /// base (Ctrl+C is `Character('c')`).
    Character(char),
    Tab,
    /// Shift+Tab. Terminals expect `CSI Z` for this.
    BackTab,
    Enter,
    /// Numeric keypad enter, distinct from the main Enter key.
    KeypadEnter,
    Escape,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Up,
    Down,
    /// F1 to F12.
    Function(u8),
    /// Keypad digits `0` to `9`.
    KeypadDigit(u8),
    KeypadDot,
    KeypadPlus,
    KeypadMinus,
    KeypadStar,
    KeypadSlash,
    ContextMenu,
    NumLock,
    CapsLock,
    ScrollLock,
    Pause,
}

/// One raw key transition, forwarded to `App::raw_key`.
///
/// Every key event reaches the app here, including Ctrl chords, Tab,
/// the function keys and releases: the intent based `App::key` and
/// `App::text` hooks only cover what text fields need. `text` carries
/// the decoded string for printable input (including key repeat) and is
/// `None` for control keys and releases.
#[derive(Clone, Debug)]
pub struct KeyPress {
    pub key: RawKey,
    pub modifiers: Modifiers,
    pub text: Option<String>,
    pub pressed: bool,
    /// True for auto-repeat while the key is held down.
    pub repeat: bool,
}

/// Pointer button that produced an `App::mouse_button` event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButtonKind {
    Left,
    Middle,
    Right,
    /// Any further button, numbered from zero.
    Other(u16),
}

/// Pointer shape requested by content. The shell sets the winit
/// cursor from `App::cursor` after every move, unless the pointer sits on
/// a window resize zone (edges/corners take precedence); `Text` is the
/// I-beam over editable text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorKind {
    #[default]
    Default,
    Text,
    ResizeColumn,
    ResizeNorth,
    ResizeSouth,
    ResizeEast,
    ResizeWest,
    ResizeNorthEast,
    ResizeNorthWest,
    ResizeSouthEast,
    ResizeSouthWest,
}

impl From<ResizeDirection> for CursorKind {
    fn from(direction: ResizeDirection) -> Self {
        match direction {
            ResizeDirection::North => CursorKind::ResizeNorth,
            ResizeDirection::South => CursorKind::ResizeSouth,
            ResizeDirection::East => CursorKind::ResizeEast,
            ResizeDirection::West => CursorKind::ResizeWest,
            ResizeDirection::NorthEast => CursorKind::ResizeNorthEast,
            ResizeDirection::NorthWest => CursorKind::ResizeNorthWest,
            ResizeDirection::SouthEast => CursorKind::ResizeSouthEast,
            ResizeDirection::SouthWest => CursorKind::ResizeSouthWest,
        }
    }
}

impl CursorKind {
    fn winit_cursor(self) -> CursorIcon {
        match self {
            CursorKind::Default => CursorIcon::Default,
            CursorKind::Text => CursorIcon::Text,
            CursorKind::ResizeColumn | CursorKind::ResizeEast | CursorKind::ResizeWest => {
                CursorIcon::EwResize
            }
            CursorKind::ResizeNorth | CursorKind::ResizeSouth => CursorIcon::NsResize,
            CursorKind::ResizeNorthEast | CursorKind::ResizeSouthWest => CursorIcon::NeswResize,
            CursorKind::ResizeNorthWest | CursorKind::ResizeSouthEast => CursorIcon::NwseResize,
        }
    }
}

/// Resize zone at the given logical pointer position for a logical window
/// size of `width` x `height`. Hits the visible body border (which sits
/// `MARGIN` inside the transparent window, see `frame.rs`): corners first
/// with the wider `RESIZE_CORNER_HIT` band, then straight edges. The band
/// is asymmetric (`RESIZE_INNER_HIT` into content, `RESIZE_OUTER_HIT`
/// into the shadow rim) so edge content like the overlay scrollbar keeps
/// working. Returns `None` inside content, far outside the body, or when
/// the window is too small to hold a body.
pub fn resize_direction_at(x: f32, y: f32, width: f32, height: f32) -> Option<ResizeDirection> {
    let margin = super::frame::MARGIN;
    let body_l = margin;
    let body_t = margin;
    let body_r = width - margin;
    let body_b = height - margin;
    if body_r <= body_l || body_b <= body_t {
        return None;
    }
    // Ignore presses far away from the body (deep content or far outside
    // the shadow rim).
    if x < body_l - RESIZE_OUTER_HIT
        || x > body_r + RESIZE_OUTER_HIT
        || y < body_t - RESIZE_OUTER_HIT
        || y > body_b + RESIZE_OUTER_HIT
    {
        return None;
    }
    // Signed distance from each body edge (negative = inside content).
    let dist_left = x - body_l;
    let dist_right = body_r - x;
    let dist_top = y - body_t;
    let dist_bottom = body_b - y;
    /// True when a signed edge distance sits inside the grab band
    /// (negative = outside in the shadow rim, positive = inside content).
    fn on_edge(dist: f32) -> bool {
        dist >= -RESIZE_OUTER_HIT && dist <= RESIZE_INNER_HIT
    }
    let near_left = dist_left.abs() <= RESIZE_CORNER_HIT;
    let near_right = dist_right.abs() <= RESIZE_CORNER_HIT;
    let near_top = dist_top.abs() <= RESIZE_CORNER_HIT;
    let near_bottom = dist_bottom.abs() <= RESIZE_CORNER_HIT;
    // Corners win: diagonal resize. Require the tight edge band on both
    // axes so the corner squares do not swallow long edge stretches.
    let on_left = on_edge(dist_left);
    let on_right = on_edge(dist_right);
    let on_top = on_edge(dist_top);
    let on_bottom = on_edge(dist_bottom);
    if near_left && near_top && (on_left || on_top) {
        return Some(ResizeDirection::NorthWest);
    }
    if near_right && near_top && (on_right || on_top) {
        return Some(ResizeDirection::NorthEast);
    }
    if near_left && near_bottom && (on_left || on_bottom) {
        return Some(ResizeDirection::SouthWest);
    }
    if near_right && near_bottom && (on_right || on_bottom) {
        return Some(ResizeDirection::SouthEast);
    }
    // Straight edges.
    if on_left && y > body_t && y < body_b {
        return Some(ResizeDirection::West);
    }
    if on_right && y > body_t && y < body_b {
        return Some(ResizeDirection::East);
    }
    if on_top && x > body_l && x < body_r {
        return Some(ResizeDirection::North);
    }
    if on_bottom && x > body_l && x < body_r {
        return Some(ResizeDirection::South);
    }
    None
}

/// Touch contact phases forwarded to the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Window operations requested by content (e.g. traffic lights).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Close,
    Minimize,
    ToggleMaximize,
}

/// Logical content area inside the window frame.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// App hosted in a `Window`. The app owns a tree of element `View`s
/// (see `elements::View`) and forwards events into it. Coordinates are
/// logical pixels relative to the window; place content inside `viewport`.
pub trait App {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    );
    fn mouse_down(&mut self, _x: f64, _y: f64) {}
    fn mouse_up(&mut self, _x: f64, _y: f64) {}
    fn mouse_move(&mut self, _x: f64, _y: f64) {}
    /// Modifier keys currently held (Ctrl for multi-select, Shift for
    /// range select). The shell calls this on modifier changes;
    /// default ignores. Tables read it through `set_modifiers`.
    fn set_modifiers(&mut self, _ctrl: bool, _shift: bool) {}
    /// Pointer shape at the given logical position. Called after
    /// every pointer move when the pointer is not on a window resize
    /// zone (edges/corners take precedence and show resize arrows);
    /// default is the arrow. Text fields return `Text` while hovered
    /// so the cursor turns into an I-beam; the sidebar returns
    /// `ResizeColumn` over its resize edge.
    fn cursor(&self, _x: f64, _y: f64) -> CursorKind {
        CursorKind::Default
    }
    /// Scroll wheel delta in logical px (right/down positive).
    fn mouse_wheel(&mut self, _dx: f64, _dy: f64) {}
    /// Right-click press in logical px (context menus).
    fn context_click(&mut self, _x: f64, _y: f64) {}
    /// Touch contact change in logical px (long-press detection).
    fn touch(&mut self, _phase: TouchPhase, _x: f64, _y: f64) {}
    fn set_focused(&mut self, _focused: bool) {}
    fn text(&mut self, _text: &str) {}
    fn key(&mut self, _key: Key) {}
    /// Every key transition with the full modifier state, called before
    /// the intent hooks `key` / `text` (which keep running afterwards
    /// for existing apps). Use this for protocol level input: terminal
    /// chords, Tab, function keys, keypad, Ctrl+letter.
    fn raw_key(&mut self, _press: &KeyPress) {}
    /// Pointer button press or release with the full modifier state,
    /// called for every button (including middle and right) next to the
    /// intent hooks `mouse_down` / `mouse_up` / `context_click`. Window
    /// resize drags and title bar drags report no button event, so
    /// content never sees those.
    fn mouse_button(
        &mut self,
        _button: MouseButtonKind,
        _pressed: bool,
        _x: f64,
        _y: f64,
        _modifiers: Modifiers,
    ) {
    }
    /// Live title for the real window (task switcher, compositor),
    /// read once per frame. `Some` replaces the title the window was
    /// created with, `None` keeps the current one. An app that mirrors
    /// its own in-window title bar (a terminal following OSC titles)
    /// returns the same string from here.
    fn window_title(&self) -> Option<&str> {
        None
    }
    /// Draggable region for window moving: (x, y, width, height) in logical
    /// px. A press inside starts a window drag instead of a click.
    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }
    /// Window operation requested by content. Consumed once per call.
    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        None
    }
    /// Window body background. Read every frame so theme changes apply
    /// live; defaults to the dark base color.
    fn background(&self) -> Color {
        BACKGROUND
    }
    /// Transparent body: skips the window background fill so only frame
    /// lines, bars and glass show over the desktop.
    fn transparent_body(&self) -> bool {
        false
    }
    /// When true the shell runs a second capture pass: draw without glass
    /// bodies, blur that into an offscreen texture, then draw again with
    /// the blur available to glass views via `ImageLoader::backdrop`.
    /// Return true only while backdrop glass is on screen (e.g. a slider
    /// knob is held) so idle frames stay single-pass.
    fn wants_backdrop(&self) -> bool {
        false
    }
}

/// Open a window and run `app` until the window closes.
pub fn run(
    title: &str,
    width: u32,
    height: u32,
    app: impl App + 'static,
) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut shell = Shell::new(title.to_owned(), width, height, app);
    event_loop.run_app(&mut shell)?;
    Ok(())
}

struct Active {
    window: Arc<Window>,
    surface: RenderSurface<'static>,
    renderer: Renderer,
    fonts: FontSystem,
    images: ImageCache,
    scene: Scene,
    backdrop: BackdropBlur,
    /// Live compositor backdrop subscription, `None` on compositors that
    /// do not offer the stream.
    backdrop_stream: Option<CompositorBackdrop>,
    /// True while the backdrop texture holds compositor pixels instead of an
    /// in-app capture, which removes the second Vello pass.
    backdrop_external: bool,
    scale: f64,
    cursor_pos: (f64, f64),
    last_cursor: CursorKind,
    start: Instant,
}

/// Modifier state from a winit `KeyModifiersState`.
fn modifiers_from(state: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        super_key: state.super_key(),
    }
}

/// Physical key to [`RawKey`], independent of the produced text. Returns
/// `None` for keys that produce a character, so the caller can fall back
/// to the decoded text (which knows about Shift and AltGr).
fn raw_key_from_code(code: KeyCode, modifiers: Modifiers) -> Option<RawKey> {
    // Shift+Tab reports as `BackTab` by the backend on most platforms,
    // but the Shift+Tab chord itself is what apps need.
    let back_tab = matches!(code, KeyCode::Tab) && modifiers.shift;
    let key = match code {
        KeyCode::Tab if back_tab => RawKey::BackTab,
        KeyCode::Tab => RawKey::Tab,
        KeyCode::Enter => RawKey::Enter,
        KeyCode::NumpadEnter => RawKey::KeypadEnter,
        KeyCode::Escape => RawKey::Escape,
        KeyCode::Backspace => RawKey::Backspace,
        KeyCode::Delete => RawKey::Delete,
        KeyCode::Insert => RawKey::Insert,
        KeyCode::Home => RawKey::Home,
        KeyCode::End => RawKey::End,
        KeyCode::PageUp => RawKey::PageUp,
        KeyCode::PageDown => RawKey::PageDown,
        KeyCode::ArrowLeft => RawKey::Left,
        KeyCode::ArrowRight => RawKey::Right,
        KeyCode::ArrowUp => RawKey::Up,
        KeyCode::ArrowDown => RawKey::Down,
        KeyCode::F1 => RawKey::Function(1),
        KeyCode::F2 => RawKey::Function(2),
        KeyCode::F3 => RawKey::Function(3),
        KeyCode::F4 => RawKey::Function(4),
        KeyCode::F5 => RawKey::Function(5),
        KeyCode::F6 => RawKey::Function(6),
        KeyCode::F7 => RawKey::Function(7),
        KeyCode::F8 => RawKey::Function(8),
        KeyCode::F9 => RawKey::Function(9),
        KeyCode::F10 => RawKey::Function(10),
        KeyCode::F11 => RawKey::Function(11),
        KeyCode::F12 => RawKey::Function(12),
        KeyCode::Numpad0 => RawKey::KeypadDigit(0),
        KeyCode::Numpad1 => RawKey::KeypadDigit(1),
        KeyCode::Numpad2 => RawKey::KeypadDigit(2),
        KeyCode::Numpad3 => RawKey::KeypadDigit(3),
        KeyCode::Numpad4 => RawKey::KeypadDigit(4),
        KeyCode::Numpad5 => RawKey::KeypadDigit(5),
        KeyCode::Numpad6 => RawKey::KeypadDigit(6),
        KeyCode::Numpad7 => RawKey::KeypadDigit(7),
        KeyCode::Numpad8 => RawKey::KeypadDigit(8),
        KeyCode::Numpad9 => RawKey::KeypadDigit(9),
        KeyCode::NumpadDecimal => RawKey::KeypadDot,
        KeyCode::NumpadAdd => RawKey::KeypadPlus,
        KeyCode::NumpadSubtract => RawKey::KeypadMinus,
        KeyCode::NumpadMultiply => RawKey::KeypadStar,
        KeyCode::NumpadDivide => RawKey::KeypadSlash,
        KeyCode::ContextMenu => RawKey::ContextMenu,
        KeyCode::NumLock => RawKey::NumLock,
        KeyCode::CapsLock => RawKey::CapsLock,
        KeyCode::ScrollLock => RawKey::ScrollLock,
        KeyCode::Pause => RawKey::Pause,
        _ => return None,
    };
    Some(key)
}

/// Full key identity for one winit keyboard event: the physical mapping
/// wins for named keys, the decoded text names the printable ones, and a
/// bare letter/digit physical key fills in when Ctrl or Alt swallowed the
/// text (Ctrl+C arrives without any text but must still be reported).
fn raw_key_from_event(code: KeyCode, text: Option<&str>, modifiers: Modifiers) -> Option<RawKey> {
    if let Some(key) = raw_key_from_code(code, modifiers) {
        return Some(key);
    }
    if let Some(text) = text {
        if let Some(ch) = text.chars().find(|ch| !ch.is_control()) {
            return Some(RawKey::Character(ch));
        }
    }
    base_character(code)
}

/// Unmodified character a physical key produces, or `None` for keys
/// without one. Only letters and digits are needed: symbols already
/// arrive through the decoded text.
fn base_character(code: KeyCode) -> Option<RawKey> {
    let letter = match code {
        KeyCode::KeyA => 'a',
        KeyCode::KeyB => 'b',
        KeyCode::KeyC => 'c',
        KeyCode::KeyD => 'd',
        KeyCode::KeyE => 'e',
        KeyCode::KeyF => 'f',
        KeyCode::KeyG => 'g',
        KeyCode::KeyH => 'h',
        KeyCode::KeyI => 'i',
        KeyCode::KeyJ => 'j',
        KeyCode::KeyK => 'k',
        KeyCode::KeyL => 'l',
        KeyCode::KeyM => 'm',
        KeyCode::KeyN => 'n',
        KeyCode::KeyO => 'o',
        KeyCode::KeyP => 'p',
        KeyCode::KeyQ => 'q',
        KeyCode::KeyR => 'r',
        KeyCode::KeyS => 's',
        KeyCode::KeyT => 't',
        KeyCode::KeyU => 'u',
        KeyCode::KeyV => 'v',
        KeyCode::KeyW => 'w',
        KeyCode::KeyX => 'x',
        KeyCode::KeyY => 'y',
        KeyCode::KeyZ => 'z',
        KeyCode::Digit0 => '0',
        KeyCode::Digit1 => '1',
        KeyCode::Digit2 => '2',
        KeyCode::Digit3 => '3',
        KeyCode::Digit4 => '4',
        KeyCode::Digit5 => '5',
        KeyCode::Digit6 => '6',
        KeyCode::Digit7 => '7',
        KeyCode::Digit8 => '8',
        KeyCode::Digit9 => '9',
        _ => return None,
    };
    Some(RawKey::Character(letter))
}

/// winit mouse button to [`MouseButtonKind`].
fn mouse_button_kind(button: MouseButton) -> MouseButtonKind {
    match button {
        MouseButton::Left => MouseButtonKind::Left,
        MouseButton::Middle => MouseButtonKind::Middle,
        MouseButton::Right => MouseButtonKind::Right,
        MouseButton::Back => MouseButtonKind::Other(3),
        MouseButton::Forward => MouseButtonKind::Other(4),
        MouseButton::Other(index) => MouseButtonKind::Other(index),
    }
}

struct Shell<V: App> {
    title: String,
    width: u32,
    height: u32,
    app: V,
    context: Option<RenderContext>,
    active: Option<Active>,
    modifiers: Modifiers,
}

impl<V: App> Shell<V> {
    fn new(title: String, width: u32, height: u32, app: V) -> Self {
        Self {
            title,
            width,
            height,
            app,
            context: None,
            active: None,
            modifiers: Modifiers::default(),
        }
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if let Some(title) = self.app.window_title() {
            let title = title.to_string();
            if title != self.title {
                self.title = title.clone();
                active.window.set_title(&title);
            }
        }
        if let Some(command) = self.app.poll_window_command() {
            match command {
                WindowCommand::Close => event_loop.exit(),
                WindowCommand::Minimize => active.window.set_minimized(true),
                WindowCommand::ToggleMaximize => {
                    active.window.set_maximized(!active.window.is_maximized());
                }
            }
        }
        let context = self.context.as_ref().expect("context exists");
        let size = active.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let scale = active.scale as f32;
        active.fonts.scale = scale;

        let (vx, vy, vw, vh) = super::frame::content_rect(
            size.width as f32 / scale,
            size.height as f32 / scale,
        );
        let elapsed = active.start.elapsed().as_secs_f64();
        let devices = &context.devices;
        let device_handle = &devices[active.surface.dev_id];
        let params = RenderParams {
            base_color: Color::TRANSPARENT,
            width: size.width,
            height: size.height,
            antialiasing_method: AaConfig::Msaa8,
        };
        let viewport = Viewport {
            x: vx,
            y: vy,
            width: vw,
            height: vh,
        };
        let background = if self.app.transparent_body() {
            None
        } else {
            Some(self.app.background())
        };

        // Desktop backdrop stream. The subscription follows
        // `wants_backdrop`, so a window that never shows glass never makes the
        // compositor capture anything.
        let app_wants = self.app.wants_backdrop();
        if let Some(stream) = active.backdrop_stream.as_mut() {
            stream.set_enabled(app_wants, size.width, size.height);
            stream.poll();
            let frame = stream.take_frame();
            if !stream.is_alive() {
                // The connection died (or the compositor never offered the
                // stream): drop it and keep the in-app capture pass.
                active.backdrop_stream = None;
                active.backdrop_external = false;
            } else if let Some(frame) = frame {
                if active.backdrop.size() != Some((frame.width, frame.height)) {
                    active.backdrop.take_image(&mut active.renderer);
                    active
                        .backdrop
                        .ensure_size(&device_handle.device, frame.width, frame.height);
                }
                active.backdrop_external = active.backdrop.upload_content(
                    &device_handle.queue,
                    &frame.pixels,
                    frame.width,
                    frame.height,
                );
            }
        }

        if app_wants || active.backdrop_external {
            if !active.backdrop_external
                && active.backdrop.size() != Some((size.width, size.height))
            {
                active.backdrop.take_image(&mut active.renderer);
                active
                    .backdrop
                    .ensure_size(&device_handle.device, size.width, size.height);
            }
            let clip_body = background.is_some();
            let capture_clip = clip_body && !active.backdrop_external;

            if !active.backdrop_external {
                // Pass 1: capture without glass bodies (no frame lines: they
                // sit above content and must not appear under glass).
                active.scene.reset();
                super::frame::draw_behind(
                    &mut active.scene,
                    size.width,
                    size.height,
                    scale,
                    background,
                );
                // Clip content to the rounded body so square views never
                // spill over the corners. Transparent bodies skip the clip
                // (nothing to round against).
                if capture_clip {
                    let clip = super::frame::body_shape(size.width, size.height, scale);
                    active
                        .scene
                        .push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
                }
                {
                    let mut loader = ImageLoader::new(
                        &mut active.renderer,
                        &device_handle.device,
                        &device_handle.queue,
                        &mut active.images,
                    );
                    loader.set_capture_pass(true);
                    loader.set_busy(active.backdrop.busy_handle());
                    self.app.draw(
                        &mut active.scene,
                        &mut active.fonts,
                        &mut loader,
                        viewport,
                        elapsed,
                    );
                }
                if capture_clip {
                    active.scene.pop_layer();
                }
                if let Err(err) = active.renderer.render_to_texture(
                    &device_handle.device,
                    &device_handle.queue,
                    &active.scene,
                    active.backdrop.content_view(),
                    &params,
                ) {
                    eprintln!("backdrop capture error: {err:?}");
                    return;
                }
            }

            active
                .backdrop
                .run(&device_handle.device, &device_handle.queue);
            let backdrop_image = active.backdrop.sync_image(&mut active.renderer);
            let backdrop_sharp = active.backdrop.sync_sharp_image(&mut active.renderer);

            // Pass 2: final frame with the blur available to glass.
            active.scene.reset();
            super::frame::draw_behind(
                &mut active.scene,
                size.width,
                size.height,
                scale,
                background,
            );
            if clip_body {
                let clip = super::frame::body_shape(size.width, size.height, scale);
                active
                    .scene
                    .push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
            }
            {
                let mut loader = ImageLoader::new(
                    &mut active.renderer,
                    &device_handle.device,
                    &device_handle.queue,
                    &mut active.images,
                );
                loader.set_capture_pass(false);
                loader.set_backdrop(backdrop_image);
                loader.set_backdrop_sharp(backdrop_sharp);
                loader.set_busy(active.backdrop.busy_handle());
                self.app.draw(&mut active.scene, &mut active.fonts, &mut loader, viewport, elapsed);
            }
            if clip_body {
                active.scene.pop_layer();
            }
            super::frame::draw_frame(&mut active.scene, size.width, size.height, scale);
            if let Err(err) = active.renderer.render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &active.scene,
                &active.surface.target_view,
                &params,
            ) {
                eprintln!("render error: {err:?}");
                return;
            }
        } else {
            active.scene.reset();
            super::frame::draw_behind(
                &mut active.scene,
                size.width,
                size.height,
                scale,
                background,
            );
            let clip_body = background.is_some();
            if clip_body {
                let clip = super::frame::body_shape(size.width, size.height, scale);
                active
                    .scene
                    .push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
            }
            {
                let mut loader = ImageLoader::new(
                    &mut active.renderer,
                    &device_handle.device,
                    &device_handle.queue,
                    &mut active.images,
                );
                self.app.draw(&mut active.scene, &mut active.fonts, &mut loader, viewport, elapsed);
            }
            if clip_body {
                active.scene.pop_layer();
            }
            super::frame::draw_frame(&mut active.scene, size.width, size.height, scale);
            if let Err(err) = active.renderer.render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &active.scene,
                &active.surface.target_view,
                &params,
            ) {
                eprintln!("render error: {err:?}");
                return;
            }
        }

        // TEMP TIMING TEST via TONTOOUI_DELAY_FIRST: stall once before
        // the first present to replicate the debug-binary timing.
        if std::env::var("TONTOOUI_DELAY_FIRST").is_ok() {
            static DELAYED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !DELAYED.swap(true, std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        let surface = &mut active.surface;
        let frame = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                context.configure_surface(surface);
                active.window.request_redraw();
                return;
            }
        };
        let device = &device_handle.device;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("blit"),
        });
        surface.blitter.copy(
            device,
            &mut encoder,
            &surface.target_view,
            &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        device_handle.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl<V: App> ApplicationHandler for Shell<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.active.is_some() {
            return;
        }
        // No window may start bigger than the screen: clamp the
        // requested size to the primary monitor minus a safety
        // margin. Without a monitor (e.g. headless) keep the request.
        let max_logical = event_loop.primary_monitor().map(|m| {
            let logical: winit::dpi::LogicalSize<f64> =
                m.size().to_logical(m.scale_factor());
            (
                (logical.width - SCREEN_MARGIN as f64).max(MIN_WINDOW as f64),
                (logical.height - SCREEN_MARGIN as f64).max(MIN_WINDOW as f64),
            )
        });
        let (width, height) = match max_logical {
            Some((max_w, max_h)) => (
                (self.width.max(1) as f64).min(max_w) as u32,
                (self.height.max(1) as f64).min(max_h) as u32,
            ),
            None => (self.width.max(1), self.height.max(1)),
        };
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(&self.title)
                        .with_decorations(false)
                        .with_transparent(true)
                        .with_resizable(true)
                        .with_min_inner_size(LogicalSize::new(MIN_WINDOW, MIN_WINDOW))
                        .with_inner_size(LogicalSize::new(width, height)),
                )
                .expect("create window"),
        );
        // Never resizable beyond the screen either.
        if let Some((max_w, max_h)) = max_logical {
            window.set_max_inner_size(Some(LogicalSize::new(max_w as u32, max_h as u32)));
        }
        let scale = window.scale_factor();
        let size = window.inner_size();

        let mut context = self.context.take().unwrap_or_else(RenderContext::new);
        let mut surface = match futures::executor::block_on(context.create_surface(
            window.clone(),
            size.width.max(1),
            size.height.max(1),
            PresentMode::AutoVsync,
        )) {
            Ok(surface) => surface,
            Err(err) => {
                eprintln!("create surface failed (no compatible GPU?): {err:?}");
                event_loop.exit();
                return;
            }
        };
        // Transparent margin: vello configures `Auto` (resolves to `Opaque`),
        // which renders the margin solid black with square outer corners.
        super::frame::ensure_transparent_alpha(&context, &mut surface);
        let renderer = Renderer::new(
            &context.devices[surface.dev_id].device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                ..Default::default()
            },
        )
        .expect("create renderer");

        // The window Arc is owned (moved into the surface target), so the
        // surface does not borrow from our locals and can live 'static.
        let surface: RenderSurface<'static> = surface;

        self.context = Some(context);
        let backdrop_device = &self.context.as_ref().expect("context").devices[surface.dev_id].device;
        let backdrop = BackdropBlur::new(backdrop_device);

        // Desktop backdrop stream: the compositor hands us the pixels behind
        // this window so glass can blur the desktop instead of the app's own
        // second render pass. Optional, so a failure here is not fatal.
        let output_size = event_loop
            .primary_monitor()
            .map(|monitor| monitor.size())
            .unwrap_or(winit::dpi::PhysicalSize::new(size.width, size.height));
        let buffer_width = output_size.width.max(size.width).max(1);
        let buffer_height = output_size.height.max(size.height).max(1);
        let backdrop_stream = CompositorBackdrop::attach(&*window, buffer_width, buffer_height);

        self.active = Some(Active {
            window,
            surface,
            renderer,
            fonts: FontSystem::new(),
            images: ImageCache::new(),
            scene: Scene::new(),
            backdrop,
            backdrop_stream,
            backdrop_external: false,
            scale,
            cursor_pos: (0.0, 0.0),
            last_cursor: CursorKind::Default,
            start: Instant::now(),
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Copied before the window borrow below: the event handlers read
        // the modifier snapshot through this local, never through `self`.
        let modifiers = self.modifiers;
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if active.window.id() != window_id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(context) = self.context.as_ref() {
                        context.resize_surface(&mut active.surface, size.width, size.height);
                    }
                }
                active.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                active.scale = scale_factor;
                active.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                active.cursor_pos = (position.x, position.y);
                let scale = active.scale;
                let x = position.x / scale;
                let y = position.y / scale;
                self.app.mouse_move(x, y);
                // Window resize zones win over content cursors: hovering a
                // body edge or corner shows the matching resize arrow.
                // Maximized windows cannot resize, keep the app cursor.
                let size = active.window.inner_size();
                let cursor = if active.window.is_maximized() {
                    self.app.cursor(x, y)
                } else {
                    let (w, h) = (size.width as f64 / scale, size.height as f64 / scale);
                    match resize_direction_at(x as f32, y as f32, w as f32, h as f32) {
                        Some(direction) => CursorKind::from(direction),
                        None => self.app.cursor(x, y),
                    }
                };
                if cursor != active.last_cursor {
                    active.last_cursor = cursor;
                    active.window.set_cursor(cursor.winit_cursor());
                }
            }
            WindowEvent::Focused(focused) => {
                self.app.set_focused(focused);
                active.window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let scale = active.scale;
                let x = (active.cursor_pos.0 / scale) as f32;
                let y = (active.cursor_pos.1 / scale) as f32;
                let pressed = state == ElementState::Pressed;
                // Left presses can be claimed by the window itself (edge
                // resize, title bar drag). Those return early and report
                // no button event, so content never sees a drag.
                if button == MouseButton::Left && pressed {
                    // Edge/corner press starts an OS resize drag instead
                    // of a content click (maximized windows excepted).
                    if !active.window.is_maximized() {
                        let size = active.window.inner_size();
                        let (w, h) = (
                            size.width as f64 / scale,
                            size.height as f64 / scale,
                        );
                        if let Some(direction) =
                            resize_direction_at(x, y, w as f32, h as f32)
                        {
                            let _ = active.window.drag_resize_window(direction);
                            return;
                        }
                    }
                    if let Some((rx, ry, rw, rh)) = self.app.drag_region() {
                        if x >= rx && x <= rx + rw && y >= ry && y <= ry + rh {
                            // Titlebar drag: moving keeps focus, no click.
                            let _ = active.window.drag_window();
                            return;
                        }
                    }
                }
                self.app.mouse_button(
                    mouse_button_kind(button),
                    pressed,
                    x as f64,
                    y as f64,
                    modifiers,
                );
                match (button, state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        self.app.mouse_down(x as f64, y as f64);
                        active.window.request_redraw();
                    }
                    (MouseButton::Left, ElementState::Released) => {
                        self.app.mouse_up(x as f64, y as f64);
                        active.window.request_redraw();
                    }
                    (MouseButton::Right, ElementState::Pressed) => {
                        self.app.context_click(x as f64, y as f64);
                        active.window.request_redraw();
                    }
                    _ => {}
                }
            }
            WindowEvent::Touch(touch) => {
                let scale = active.scale;
                let phase = match touch.phase {
                    winit::event::TouchPhase::Started => TouchPhase::Started,
                    winit::event::TouchPhase::Moved => TouchPhase::Moved,
                    winit::event::TouchPhase::Ended => TouchPhase::Ended,
                    winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
                };
                self.app.touch(
                    phase,
                    touch.location.x / scale,
                    touch.location.y / scale,
                );
                active.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // The raw hook sees every transition (including releases
                // and Ctrl chords that produce no text) and runs before
                // the intent hooks below, which still fire for existing
                // apps.
                if let PhysicalKey::Code(code) = event.physical_key {
                    let text = event.text.as_ref().map(|text| text.as_str());
                    if let Some(key) = raw_key_from_event(code, text, modifiers) {
                        let press = KeyPress {
                            key,
                            modifiers,
                            text: text.map(|text| text.to_string()),
                            pressed: event.state == ElementState::Pressed,
                            repeat: event.repeat,
                        };
                        let repeats = press.pressed;
                        self.app.raw_key(&press);
                        if repeats {
                            active.window.request_redraw();
                        }
                    }
                }
                if event.state != ElementState::Pressed {
                    return;
                }
                // Ctrl shortcuts translate to editing intents before
                // the regular key/text handling below (which would
                // otherwise see the bare letters).
                if modifiers.ctrl {
                    let combo = match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyA) => Some(Key::SelectAll),
                        PhysicalKey::Code(KeyCode::KeyC) => Some(Key::Copy),
                        PhysicalKey::Code(KeyCode::KeyX) => Some(Key::Cut),
                        PhysicalKey::Code(KeyCode::KeyV) => Some(Key::Paste),
                        PhysicalKey::Code(KeyCode::KeyZ) if modifiers.shift => Some(Key::Redo),
                        PhysicalKey::Code(KeyCode::KeyZ) => Some(Key::Undo),
                        PhysicalKey::Code(KeyCode::KeyY) => Some(Key::Redo),
                        _ => None,
                    };
                    if let Some(key) = combo {
                        self.app.key(key);
                        active.window.request_redraw();
                        return;
                    }
                }
                // Shift+arrows extend the text selection instead of
                // moving the caret.
                if modifiers.shift && !modifiers.ctrl {
                    let extend = match event.physical_key {
                        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(Key::SelectLeft),
                        PhysicalKey::Code(KeyCode::ArrowRight) => Some(Key::SelectRight),
                        PhysicalKey::Code(KeyCode::ArrowUp) => Some(Key::SelectUp),
                        PhysicalKey::Code(KeyCode::ArrowDown) => Some(Key::SelectDown),
                        _ => None,
                    };
                    if let Some(key) = extend {
                        self.app.key(key);
                        active.window.request_redraw();
                        return;
                    }
                }
                let mut redraw = true;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Backspace) => self.app.key(Key::Backspace),
                    PhysicalKey::Code(KeyCode::ArrowLeft) => self.app.key(Key::Left),
                    PhysicalKey::Code(KeyCode::ArrowRight) => self.app.key(Key::Right),
                    PhysicalKey::Code(KeyCode::ArrowUp) => self.app.key(Key::Up),
                    PhysicalKey::Code(KeyCode::ArrowDown) => self.app.key(Key::Down),
                    PhysicalKey::Code(KeyCode::Enter) | PhysicalKey::Code(KeyCode::NumpadEnter) => {
                        self.app.key(Key::Enter)
                    }
                    PhysicalKey::Code(KeyCode::Escape) => self.app.key(Key::Escape),
                    _ => {
                        if let Some(text) = event.text.as_ref() {
                            self.app.text(text.as_str());
                        } else {
                            redraw = false;
                        }
                    }
                }
                if redraw {
                    active.window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers_from(modifiers.state());
                self.app.set_modifiers(self.modifiers.ctrl, self.modifiers.shift);
                active.window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scale = active.scale;
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x as f64 * 20.0, y as f64 * 20.0),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x / scale, pos.y / scale),
                };
                self.app.mouse_wheel(dx, dy);
                active.window.request_redraw();
            }
            WindowEvent::RedrawRequested => self.render(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Some(active) = self.active.as_ref() {
            active.window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Modifiers with only the named keys held.
    fn mods(ctrl: bool, shift: bool, alt: bool) -> Modifiers {
        Modifiers {
            shift,
            ctrl,
            alt,
            super_key: false,
        }
    }

    const NONE: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
    };

    #[test]
    fn named_keys_map_from_physical_code() {
        assert_eq!(raw_key_from_code(KeyCode::Tab, NONE), Some(RawKey::Tab));
        assert_eq!(raw_key_from_code(KeyCode::Enter, NONE), Some(RawKey::Enter));
        assert_eq!(
            raw_key_from_code(KeyCode::NumpadEnter, NONE),
            Some(RawKey::KeypadEnter)
        );
        assert_eq!(
            raw_key_from_code(KeyCode::PageUp, NONE),
            Some(RawKey::PageUp)
        );
        assert_eq!(
            raw_key_from_code(KeyCode::F12, NONE),
            Some(RawKey::Function(12))
        );
        assert_eq!(
            raw_key_from_code(KeyCode::Numpad7, NONE),
            Some(RawKey::KeypadDigit(7))
        );
        assert_eq!(
            raw_key_from_code(KeyCode::NumpadDecimal, NONE),
            Some(RawKey::KeypadDot)
        );
    }

    #[test]
    fn shift_tab_is_back_tab() {
        assert_eq!(
            raw_key_from_code(KeyCode::Tab, mods(false, true, false)),
            Some(RawKey::BackTab)
        );
    }

    #[test]
    fn printable_keys_use_decoded_text() {
        // Shift is already applied by the platform text.
        assert_eq!(
            raw_key_from_event(KeyCode::KeyA, Some("A"), mods(false, true, false)),
            Some(RawKey::Character('A'))
        );
        // AltGr layouts produce a symbol instead of the base letter.
        assert_eq!(
            raw_key_from_event(KeyCode::KeyE, Some("@"), mods(false, false, true)),
            Some(RawKey::Character('@'))
        );
    }

    #[test]
    fn ctrl_chords_fall_back_to_base_character() {
        // Ctrl+C produces no text at all, but the key must still be
        // reported so an app can send its control byte.
        let key = raw_key_from_event(KeyCode::KeyC, None, mods(true, false, false));
        assert_eq!(key, Some(RawKey::Character('c')));
        let digit = raw_key_from_event(KeyCode::Digit4, None, mods(true, false, false));
        assert_eq!(digit, Some(RawKey::Character('4')));
    }

    #[test]
    fn control_text_is_ignored_for_identity() {
        // A control byte in the text must not become a Character.
        let key = raw_key_from_event(KeyCode::KeyL, Some("\u{c}"), NONE);
        assert_eq!(key, Some(RawKey::Character('l')));
    }

    #[test]
    fn unmapped_keys_report_none() {
        assert_eq!(raw_key_from_event(KeyCode::F13, None, NONE), None);
        assert_eq!(raw_key_from_event(KeyCode::NumpadEqual, None, NONE), None);
    }

    #[test]
    fn mouse_buttons_keep_their_kind() {
        assert_eq!(mouse_button_kind(MouseButton::Left), MouseButtonKind::Left);
        assert_eq!(
            mouse_button_kind(MouseButton::Middle),
            MouseButtonKind::Middle
        );
        assert_eq!(mouse_button_kind(MouseButton::Right), MouseButtonKind::Right);
        assert_eq!(
            mouse_button_kind(MouseButton::Other(3)),
            MouseButtonKind::Other(3)
        );
    }

    #[test]
    fn modifiers_track_every_key() {
        assert!(!NONE.any());
        assert!(mods(true, false, false).any());
        assert!(mods(false, false, true).any());
        assert!(Modifiers {
            super_key: true,
            ..Modifiers::default()
        }
        .any());
    }

    const W: f32 = 800.0;
    const H: f32 = 600.0;

    #[test]
    fn corners_resize_diagonally() {
        use ResizeDirection::*;
        let margin = super::super::frame::MARGIN;
        assert_eq!(resize_direction_at(margin, margin, W, H), Some(NorthWest));
        assert_eq!(
            resize_direction_at(W - margin, margin, W, H),
            Some(NorthEast)
        );
        assert_eq!(
            resize_direction_at(margin, H - margin, W, H),
            Some(SouthWest)
        );
        assert_eq!(
            resize_direction_at(W - margin, H - margin, W, H),
            Some(SouthEast)
        );
    }

    #[test]
    fn edges_resize_straight() {
        use ResizeDirection::*;
        let margin = super::super::frame::MARGIN;
        assert_eq!(
            resize_direction_at(margin, H / 2.0, W, H),
            Some(West)
        );
        assert_eq!(
            resize_direction_at(W - margin, H / 2.0, W, H),
            Some(East)
        );
        assert_eq!(
            resize_direction_at(W / 2.0, margin, W, H),
            Some(North)
        );
        assert_eq!(
            resize_direction_at(W / 2.0, H - margin, W, H),
            Some(South)
        );
    }

    #[test]
    fn content_has_no_resize_zone() {
        assert_eq!(resize_direction_at(W / 2.0, H / 2.0, W, H), None);
        assert_eq!(resize_direction_at(200.0, 200.0, W, H), None);
    }

    #[test]
    fn scrollbar_stays_usable() {
        use ResizeDirection::*;
        let margin = super::super::frame::MARGIN;
        // Overlay scrollbar thumb (up to 10 px wide at the content edge):
        // hovering it must not show a resize cursor or steal the press.
        assert_eq!(resize_direction_at(W - margin - 8.0, H / 2.0, W, H), None);
        assert_eq!(resize_direction_at(W - margin - 5.0, H / 2.0, W, H), None);
        // The outermost content pixels still grab the edge.
        assert_eq!(
            resize_direction_at(W - margin - 2.0, H / 2.0, W, H),
            Some(East)
        );
        // Outer shadow rim grabs too.
        assert_eq!(
            resize_direction_at(W - margin + 8.0, H / 2.0, W, H),
            Some(East)
        );
    }

    #[test]
    fn resize_cursor_mapping() {
        use ResizeDirection::*;
        assert_eq!(CursorKind::from(SouthEast), CursorKind::ResizeSouthEast);
        assert_eq!(
            CursorKind::from(SouthEast).winit_cursor(),
            CursorIcon::NwseResize
        );
        assert_eq!(
            CursorKind::from(NorthEast).winit_cursor(),
            CursorIcon::NeswResize
        );
        assert_eq!(CursorKind::from(North).winit_cursor(), CursorIcon::NsResize);
        assert_eq!(CursorKind::from(East).winit_cursor(), CursorIcon::EwResize);
    }
}
