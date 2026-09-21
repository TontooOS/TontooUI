use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use wgpu::PresentMode;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use super::text::FontSystem;

/// Window background. Dark mode base color per TontooOS convention.
pub const BACKGROUND: Color = Color::from_rgb8(0x1d, 0x1d, 0x1d);

/// Standard window corner radius in logical px. Follows the macOS 27 Golden
/// Gate direction: one fixed radius for all windows, tighter than Tahoe.
/// Physical pixels = value x window scale factor (20 pt is ~40 px at 2x).
pub const WINDOW_CORNER_RADIUS: f32 = 20.0;

/// Non-printable keys forwarded to the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Enter,
    Escape,
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
        viewport: Viewport,
        time_secs: f64,
    );
    fn mouse_down(&mut self, _x: f64, _y: f64) {}
    fn mouse_move(&mut self, _x: f64, _y: f64) {}
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
    scene: Scene,
    scale: f64,
    cursor_pos: (f64, f64),
    start: Instant,
}

struct Shell<V: App> {
    title: String,
    width: u32,
    height: u32,
    app: V,
    context: Option<RenderContext>,
    active: Option<Active>,
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

        active.scene.reset();
        let scale = active.scale as f32;
        active.fonts.scale = scale;

        // Window frame behind content: shadows plus rounded body. The surface
        // itself is cleared transparent so the corners stay see-through.
        super::frame::draw_behind(
            &mut active.scene,
            size.width,
            size.height,
            scale,
            if self.app.transparent_body() {
                None
            } else {
                Some(self.app.background())
            },
        );

        let (vx, vy, vw, vh) = super::frame::content_rect(
            size.width as f32 / scale,
            size.height as f32 / scale,
        );
        let elapsed = active.start.elapsed().as_secs_f64();
        self.app.draw(
            &mut active.scene,
            &mut active.fonts,
            Viewport {
                x: vx,
                y: vy,
                width: vw,
                height: vh,
            },
            elapsed,
        );

        // Frame lines above content so bars and fields never cover them.
        super::frame::draw_frame(&mut active.scene, size.width, size.height, scale);

        let surface = &mut active.surface;
        let devices = &context.devices;
        let device_handle = &devices[surface.dev_id];
        let params = RenderParams {
            base_color: Color::TRANSPARENT,
            width: size.width,
            height: size.height,
            antialiasing_method: AaConfig::Msaa8,
        };
        if let Err(err) = active.renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            &active.scene,
            &surface.target_view,
            &params,
        ) {
            eprintln!("render error: {err:?}");
            return;
        }

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
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(&self.title)
                        .with_decorations(false)
                        .with_transparent(true)
                        .with_inner_size(LogicalSize::new(self.width, self.height)),
                )
                .expect("create window"),
        );
        let scale = window.scale_factor();
        let size = window.inner_size();

        let mut context = self.context.take().unwrap_or_else(RenderContext::new);
        let surface = match futures::executor::block_on(context.create_surface(
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
        self.active = Some(Active {
            window,
            surface,
            renderer,
            fonts: FontSystem::new(),
            scene: Scene::new(),
            scale,
            cursor_pos: (0.0, 0.0),
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
                self.app.mouse_move(position.x / scale, position.y / scale);
            }
            WindowEvent::Focused(focused) => {
                self.app.set_focused(focused);
                active.window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    let scale = active.scale;
                    let x = (active.cursor_pos.0 / scale) as f32;
                    let y = (active.cursor_pos.1 / scale) as f32;
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
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                let mut redraw = true;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Backspace) => self.app.key(Key::Backspace),
                    PhysicalKey::Code(KeyCode::ArrowLeft) => self.app.key(Key::Left),
                    PhysicalKey::Code(KeyCode::ArrowRight) => self.app.key(Key::Right),
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
