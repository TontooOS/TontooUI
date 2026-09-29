//! Wayland layer-shell backend: top-anchored shell bars (menubar).
//!
//! Normal apps keep using [`super::window::run`]. Shell bars use
//! [`run_layer`]: every Wayland output gets its own layer surface (top
//! edge, exclusive zone, no keyboard focus) driving one [`App`] instance,
//! so one process serves all monitors. Rendering reuses the Vello/WGPU
//! pipeline from `window.rs` (offscreen texture, blit, present).
//!
//! Without a Wayland compositor `run_layer` returns an error instead of
//! hanging. There is deliberately no backdrop blur here: bars and their
//! menus paint plain translucent fills (see `BarMenu`), never glass.

use std::error::Error;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::time::Instant;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat,
    output::{OutputHandler, OutputInfo, OutputState},
    reexports::client::{
        globals::{registry_queue_init, GlobalList},
        protocol::{wl_output, wl_pointer, wl_seat, wl_surface},
        Connection, Proxy, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability as SeatCapability, SeatHandler, SeatState,
    },
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
};
use vello::kurbo::Rect;
use vello::peniko::{Brush, Color, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use wgpu::PresentMode;

use super::images::{ImageCache, ImageLoader};
use super::text::FontSystem;
use super::window::{App, Viewport, WindowCommand};

/// Linux evdev code of the left mouse button.
pub const LAYER_BUTTON_LEFT: u32 = 0x110;

/// Options of one layer bar surface.
#[derive(Clone, Debug)]
pub struct LayerBarOptions {
    /// Layer-shell namespace, e.g. `"menubar"`.
    pub namespace: String,
    /// Bar height in logical px. Doubles as the exclusive zone, so
    /// maximized windows never overlap the bar.
    pub height: u32,
}

impl LayerBarOptions {
    pub fn new(namespace: impl Into<String>, height: u32) -> Self {
        Self {
            namespace: namespace.into(),
            height: height.max(1),
        }
    }
}

/// One Wayland output offered to the factory.
#[derive(Clone, Debug)]
pub struct LayerOutput {
    /// Output name (`"eDP-1"`, `"HDMI-A-1"`, ...), or `"output-N"`.
    pub name: String,
    /// Logical width in px.
    pub width: u32,
    /// Output scale factor.
    pub scale: i32,
}

/// Exclusive zone for a bar height: the bar reserves its own strip.
pub fn exclusive_zone_for_height(height: u32) -> i32 {
    height.max(1) as i32
}

/// Logical size from physical pixels and the output scale.
pub fn logical_size(physical: u32, scale: i32) -> u32 {
    if scale <= 1 {
        physical
    } else {
        (physical as f32 / scale as f32).round().max(1.0) as u32
    }
}

/// Physical size from logical pixels and the output scale.
pub fn physical_size(logical: u32, scale: f32) -> u32 {
    ((logical as f32 * scale).round().max(1.0)) as u32
}

/// Output label: advertised name, or `output-{index}` when missing.
pub fn output_label(name: Option<&str>, index: usize) -> String {
    match name {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => format!("output-{index}"),
    }
}

/// Map a pointer button event to a press (`true`) or release (`false`).
/// Only the left button drives clicks; everything else is ignored.
pub fn button_press(code: u32, pressed: bool) -> Option<bool> {
    if code == LAYER_BUTTON_LEFT {
        Some(pressed)
    } else {
        None
    }
}

/// Run layer bar surfaces, one [`App`] per Wayland output.
///
/// The factory is called for every output (and for hotplugged ones) and
/// returns the bar app plus its options, or `None` to skip the output.
/// Surfaces without keyboard focus drive the app with pointer events only.
/// Returns when the last surface closes, or an error without a Wayland
/// compositor (no `WAYLAND_DISPLAY`).
pub fn run_layer(
    make: impl FnMut(LayerOutput) -> Option<(Box<dyn App>, LayerBarOptions)> + 'static,
) -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<LayerState>(&conn)?;
    let qh = queue.handle();

    let mut state = LayerState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor: CompositorState::bind(&globals, &qh)
            .map_err(|e| format!("no wl_compositor: {e:?}"))?,
        layer_shell: LayerShell::bind(&globals, &qh)
            .map_err(|e| format!("no wlr-layer-shell (not a layer-shell compositor?): {e:?}"))?,
        context: RenderContext::new(),
        windows: Vec::new(),
        factory: Some(
            Box::new(make)
                as Box<
                    dyn FnMut(LayerOutput) -> Option<(Box<dyn App>, LayerBarOptions)>,
                >,
        ),
        pointers: Vec::new(),
        start: Instant::now(),
        exit: false,
        connection: conn,
        _globals: globals,
    };
    // Initial roundtrip: outputs announce themselves through `new_output`,
    // which builds one layer surface per accepted output.
    queue.roundtrip(&mut state)?;
    // Safety net for outputs that arrived without usable info yet.
    let outputs: Vec<(wl_output::WlOutput, OutputInfo)> = state
        .output_state
        .outputs()
        .filter_map(|o| state.output_state.info(&o).map(|info| (o, info)))
        .collect();
    for (output, info) in outputs {
        state.ensure_window(&qh, &output, &info);
    }
    // First frames: commit every surface so the compositor starts sending
    // frame callbacks, which then drive continuous rendering.
    for window in &state.windows {
        window.wl.frame(&qh, window.wl.clone());
        window.wl.commit();
    }
    queue.flush()?;

    while !state.exit {
        queue.blocking_dispatch(&mut state)?;
    }
    Ok(())
}

/// Raw Wayland handles for wgpu surface creation. The compositor owns the
/// objects; the pointers stay valid while our surfaces live.
#[derive(Clone, Copy, Debug)]
struct LayerHandles {
    display: NonNull<c_void>,
    surface: NonNull<c_void>,
}

// The loop is single-threaded; handles only cross into wgpu with the window.
unsafe impl Send for LayerHandles {}
unsafe impl Sync for LayerHandles {}

impl HasDisplayHandle for LayerHandles {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: the display pointer comes from our live Wayland
        // connection and outlives every surface built from it.
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                self.display,
            )))
        })
    }
}

impl HasWindowHandle for LayerHandles {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // SAFETY: the surface pointer is our live layer surface, kept
        // alive in `LayerWindow` for the whole surface lifetime.
        Ok(unsafe {
            WindowHandle::borrow_raw(RawWindowHandle::Wayland(WaylandWindowHandle::new(
                self.surface,
            )))
        })
    }
}

struct LayerWindow {
    wl: wl_surface::WlSurface,
    layer: LayerSurface,
    output: wl_output::WlOutput,
    output_name: String,
    surface: RenderSurface<'static>,
    renderer: Renderer,
    fonts: FontSystem,
    images: ImageCache,
    scene: Scene,
    app: Box<dyn App>,
    // Kept alive: wgpu may retain the handles.
    _handles: LayerHandles,
    logical_w: u32,
    logical_h: u32,
    scale: f32,
    cursor: (f64, f64),
    pending_size: Option<(u32, u32)>,
}

struct LayerState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    context: RenderContext,
    windows: Vec<LayerWindow>,
    factory: Option<Box<dyn FnMut(LayerOutput) -> Option<(Box<dyn App>, LayerBarOptions)>>>,
    pointers: Vec<wl_pointer::WlPointer>,
    start: Instant,
    exit: bool,
    connection: Connection,
    _globals: GlobalList,
}

impl LayerState {
    fn window_for_surface(&mut self, surface: &wl_surface::WlSurface) -> Option<&mut LayerWindow> {
        self.windows.iter_mut().find(|w| w.wl == *surface)
    }

    /// Build the layer surface for an output unless the factory skips it
    /// or one already exists.
    fn ensure_window(
        &mut self,
        qh: &QueueHandle<LayerState>,
        output: &wl_output::WlOutput,
        info: &OutputInfo,
    ) {
        let name = output_label(info.name.as_deref(), self.windows.len());
        if self.windows.iter().any(|w| w.output == *output) {
            return;
        }
        let (logical_w, _) = info.logical_size.unwrap_or((0, 0));
        if logical_w <= 0 {
            return;
        }
        let summary = LayerOutput {
            name: name.clone(),
            width: logical_w.max(1) as u32,
            scale: info.scale_factor.max(1),
        };
        let Some((app, options)) = self.factory.as_mut().and_then(|f| f(summary)) else {
            return;
        };
        let width = logical_w.max(1) as u32;
        match self.create_window(qh, output, name, width, app, &options) {
            Ok(window) => {
                window.wl.frame(qh, window.wl.clone());
                window.wl.commit();
                self.windows.push(window);
            }
            Err(err) => eprintln!("layer surface error: {err:?}"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_window(
        &mut self,
        qh: &QueueHandle<LayerState>,
        output: &wl_output::WlOutput,
        output_name: String,
        width: u32,
        app: Box<dyn App>,
        options: &LayerBarOptions,
    ) -> Result<LayerWindow, Box<dyn Error>> {
        let wl = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            wl.clone(),
            Layer::Top,
            Some(options.namespace.clone()),
            Some(output),
        );
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_exclusive_zone(exclusive_zone_for_height(options.height));
        layer.set_margin(0, 0, 0, 0);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        // Width follows the output (configure confirms); height is fixed.
        layer.set_size(0, options.height);

        // Raw handles à la winit: display and surface pointers for wgpu.
        let display_ptr =
            NonNull::new(self.connection.display().id().as_ptr() as *mut c_void)
                .ok_or("null wayland display")?;
        let surface_ptr =
            NonNull::new(wl.id().as_ptr() as *mut c_void).ok_or("null surface")?;
        let handles = LayerHandles {
            display: display_ptr,
            surface: surface_ptr,
        };
        // `From` via wgpu's `DisplayAndWindowHandle` blanket impl.
        let target: wgpu::SurfaceTarget = handles.into();
        let scale = 1.0f32;
        let phys_w = physical_size(width, scale).max(1);
        let phys_h = physical_size(options.height, scale).max(1);
        let mut surface = futures::executor::block_on(self.context.create_surface(
            target,
            phys_w,
            phys_h,
            PresentMode::AutoVsync,
        ))?;
        // Same transparency story as the winit shell: keep alpha so
        // rounded elements blend instead of sitting on black.
        super::frame::ensure_transparent_alpha(&self.context, &mut surface);
        let renderer = Renderer::new(
            &self.context.devices[surface.dev_id].device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                ..Default::default()
            },
        )?;

        Ok(LayerWindow {
            wl,
            layer,
            output: output.clone(),
            output_name,
            surface,
            renderer,
            fonts: FontSystem::new(),
            images: ImageCache::new(),
            scene: Scene::new(),
            app,
            _handles: handles,
            logical_w: width,
            logical_h: options.height,
            scale,
            cursor: (0.0, 0.0),
            pending_size: None,
        })
    }

    /// Remove the window of a closed layer surface. Exits when none left.
    fn remove_window(&mut self, layer: &LayerSurface) {
        self.windows.retain(|w| w.layer != *layer);
        if self.windows.is_empty() {
            self.exit = true;
        }
    }

    /// Render one window: apply resizes, draw the app, present, commit and
    /// request the next frame (continuous redraw, like the winit shell).
    fn render(&mut self, qh: &QueueHandle<LayerState>, surface: &wl_surface::WlSurface) {
        let Some(index) = self.windows.iter().position(|w| w.wl == *surface) else {
            return;
        };
        // Window commands: only Close is honored (drops the surface).
        if self.windows[index].app.poll_window_command() == Some(WindowCommand::Close) {
            let layer = self.windows[index].layer.clone();
            self.remove_window(&layer);
            return;
        }
        let window = &mut self.windows[index];
        if let Some((w, h)) = window.pending_size.take() {
            if w > 0 && h > 0 {
                window.logical_w = w;
                window.logical_h = h;
                let phys_w = physical_size(w, window.scale).max(1);
                let phys_h = physical_size(h, window.scale).max(1);
                self.context
                    .resize_surface(&mut window.surface, phys_w, phys_h);
            }
        }
        let scale = window.scale;
        window.fonts.scale = scale;
        let elapsed = self.start.elapsed().as_secs_f64();
        let phys_w = physical_size(window.logical_w, scale).max(1);
        let phys_h = physical_size(window.logical_h, scale).max(1);

        let devices = &self.context.devices;
        let device_handle = &devices[window.surface.dev_id];
        let params = RenderParams {
            base_color: Color::TRANSPARENT,
            width: phys_w,
            height: phys_h,
            antialiasing_method: AaConfig::Msaa8,
        };
        let viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: window.logical_w as f32,
            height: window.logical_h as f32,
        };
        window.scene.reset();
        let background = if window.app.transparent_body() {
            None
        } else {
            Some(window.app.background())
        };
        if let Some(bg) = background {
            let rect = Rect::new(0.0, 0.0, phys_w as f64, phys_h as f64);
            window.scene.fill(
                Fill::NonZero,
                vello::kurbo::Affine::IDENTITY,
                &Brush::Solid(bg),
                None,
                &rect,
            );
        }
        {
            let mut loader = ImageLoader::new(
                &mut window.renderer,
                &device_handle.device,
                &device_handle.queue,
                &mut window.images,
            );
            window
                .app
                .draw(&mut window.scene, &mut window.fonts, &mut loader, viewport, elapsed);
        }
        if let Err(err) = window.renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            &window.scene,
            &window.surface.target_view,
            &params,
        ) {
            eprintln!("layer render error: {err:?}");
            return;
        }
        let frame = match window.surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.context
                    .resize_surface(&mut self.windows[index].surface, phys_w, phys_h);
                return;
            }
        };
        let device = &device_handle.device;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("layer-blit"),
        });
        window.surface.blitter.copy(
            device,
            &mut encoder,
            &window.surface.target_view,
            &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        device_handle.queue.submit(Some(encoder.finish()));
        frame.present();
        // Borrow dance for the borrow checker: re-borrow to commit.
        let window = &self.windows[index];
        window.wl.frame(qh, window.wl.clone());
        window.wl.commit();
    }
}

// --- Wayland dispatch -----------------------------------------------------

impl CompositorHandler for LayerState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        scale_factor: i32,
    ) {
        if let Some(window) = self.window_for_surface(surface) {
            window.scale = scale_factor.max(1) as f32;
        }
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        _serial: u32,
    ) {
        self.render(qh, surface);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
        let scale = self
            .output_state
            .info(output)
            .map(|info| info.scale_factor.max(1) as f32);
        if let (Some(scale), Some(window)) =
            (scale, self.windows.iter_mut().find(|w| w.wl == *surface))
        {
            window.scale = scale;
        }
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for LayerState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if let Some(info) = self.output_state.info(&output) {
            self.ensure_window(qh, &output, &info);
        }
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        // New outputs may have been skipped (no size yet); retry.
        if let Some(info) = self.output_state.info(&output) {
            self.ensure_window(qh, &output, &info);
            let name = info.name.clone();
            let scale = info.scale_factor.max(1) as f32;
            for window in self
                .windows
                .iter_mut()
                .filter(|w| w.output == output || name.as_deref() == Some(w.output_name.as_str()))
            {
                window.scale = scale;
            }
        }
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        self.windows.retain(|w| w.output != output);
        if self.windows.is_empty() {
            self.exit = true;
        }
    }
}

impl LayerShellHandler for LayerState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.remove_window(layer);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // The ack is sent by sctk before this runs.
        if let Some(window) = self.windows.iter_mut().find(|w| w.layer == *layer) {
            let (w, h) = configure.new_size;
            if w > 0 && h > 0 {
                window.pending_size = Some((w, h));
            }
        }
    }
}

impl SeatHandler for LayerState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: SeatCapability,
    ) {
        if capability == SeatCapability::Pointer {
            if let Ok(pointer) = self.seat_state.get_pointer(qh, &seat) {
                self.pointers.push(pointer);
            }
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: SeatCapability,
    ) {
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
    }
}

impl PointerHandler for LayerState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(window) = self.window_for_surface(&event.surface) else {
                continue;
            };
            let scale = window.scale.max(1.0) as f64;
            let x = event.position.0 / scale;
            let y = event.position.1 / scale;
            match &event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    window.cursor = (x, y);
                    window.app.mouse_move(x, y);
                }
                PointerEventKind::Press { button, .. } => {
                    if button_press(*button, true) == Some(true) {
                        window.cursor = (x, y);
                        window.app.mouse_down(x, y);
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    if button_press(*button, false) == Some(false) {
                        window.cursor = (x, y);
                        window.app.mouse_up(x, y);
                    }
                }
                _ => {}
            }
        }
    }
}

delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_seat!(LayerState);
delegate_pointer!(LayerState);
delegate_layer!(LayerState);
delegate_registry!(LayerState);

impl ProvidesRegistryState for LayerState {
    registry_handlers![OutputState, SeatState];

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_zone_matches_height() {
        assert_eq!(exclusive_zone_for_height(30), 30);
        assert_eq!(exclusive_zone_for_height(0), 1);
    }

    #[test]
    fn logical_size_divides_by_scale() {
        assert_eq!(logical_size(3840, 2), 1920);
        assert_eq!(logical_size(1920, 1), 1920);
        assert_eq!(logical_size(1920, 0), 1920);
    }

    #[test]
    fn physical_size_multiplies_by_scale() {
        assert_eq!(physical_size(1920, 2.0), 3840);
        assert_eq!(physical_size(340, 1.0), 340);
    }

    #[test]
    fn output_label_falls_back() {
        assert_eq!(output_label(Some("eDP-1"), 0), "eDP-1");
        assert_eq!(output_label(None, 2), "output-2");
        assert_eq!(output_label(Some(""), 1), "output-1");
    }

    #[test]
    fn only_left_button_clicks() {
        assert_eq!(button_press(LAYER_BUTTON_LEFT, true), Some(true));
        assert_eq!(button_press(LAYER_BUTTON_LEFT, false), Some(false));
        assert_eq!(button_press(0x111, true), None);
    }

    #[test]
    fn options_clamp_height() {
        assert_eq!(LayerBarOptions::new("menubar", 0).height, 1);
        assert_eq!(LayerBarOptions::new("menubar", 30).height, 30);
    }
}
