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

/// Grab width around the window body border in logical px. The window is
/// transparent with a `MARGIN` shadow rim (see `frame.rs`); the visible body
/// edge sits `MARGIN` inside the window, so the hit band extends to both
/// sides of that edge for easy grabbing.
pub const RESIZE_HIT: f32 = 10.0;
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
/// with the wider `RESIZE_CORNER_HIT` band, then straight edges with
/// `RESIZE_HIT`. Returns `None` inside content, far outside the body, or
/// when the window is too small to hold a body.
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
    if x < body_l - RESIZE_HIT
        || x > body_r + RESIZE_HIT
        || y < body_t - RESIZE_HIT
        || y > body_b + RESIZE_HIT
    {
        return None;
    }
    let near_left = (x - body_l).abs() <= RESIZE_CORNER_HIT;
    let near_right = (x - body_r).abs() <= RESIZE_CORNER_HIT;
    let near_top = (y - body_t).abs() <= RESIZE_CORNER_HIT;
    let near_bottom = (y - body_b).abs() <= RESIZE_CORNER_HIT;
    // Corners win: diagonal resize. Require the tight edge band on both
    // axes so the corner squares do not swallow long edge stretches.
    let on_left = (x - body_l).abs() <= RESIZE_HIT;
    let on_right = (x - body_r).abs() <= RESIZE_HIT;
    let on_top = (y - body_t).abs() <= RESIZE_HIT;
    let on_bottom = (y - body_b).abs() <= RESIZE_HIT;
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
    scale: f64,
    cursor_pos: (f64, f64),
    last_cursor: CursorKind,
    start: Instant,
}

struct Shell<V: App> {
    title: String,
    width: u32,
    height: u32,
    app: V,
    context: Option<RenderContext>,
    active: Option<Active>,
    ctrl: bool,
    shift: bool,
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
            ctrl: false,
            shift: false,
        }
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
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

        if self.app.wants_backdrop() {
            if active.backdrop.size() != Some((size.width, size.height)) {
                active.backdrop.take_image(&mut active.renderer);
                active.backdrop
                    .ensure_size(&device_handle.device, size.width, size.height);
            }

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
                loader.set_capture_pass(true);
                loader.set_busy(active.backdrop.busy_handle());
                self.app.draw(&mut active.scene, &mut active.fonts, &mut loader, viewport, elapsed);
            }
            if clip_body {
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
        self.active = Some(Active {
            window,
            surface,
            renderer,
            fonts: FontSystem::new(),
            images: ImageCache::new(),
            scene: Scene::new(),
            backdrop,
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
                match (button, state) {
                    (MouseButton::Left, ElementState::Pressed) => {
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
                if event.state != ElementState::Pressed {
                    return;
                }
                // Ctrl shortcuts translate to editing intents before
                // the regular key/text handling below (which would
                // otherwise see the bare letters).
                if self.ctrl {
                    let combo = match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyA) => Some(Key::SelectAll),
                        PhysicalKey::Code(KeyCode::KeyC) => Some(Key::Copy),
                        PhysicalKey::Code(KeyCode::KeyX) => Some(Key::Cut),
                        PhysicalKey::Code(KeyCode::KeyV) => Some(Key::Paste),
                        PhysicalKey::Code(KeyCode::KeyZ) if self.shift => Some(Key::Redo),
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
                if self.shift && !self.ctrl {
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
                let state = modifiers.state();
                self.ctrl = state.control_key();
                self.shift = state.shift_key();
                self.app.set_modifiers(self.ctrl, self.shift);
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
