//! Wayland layer-shell backend: shell bars and desktop overlays.
//!
//! Normal apps keep using [`super::window::run`]. Shell surfaces use
//! [`run_layer`] / [`run_layer_multi`]: every Wayland output gets its own
//! layer surface driving one [`App`] instance, so one process serves all
//! monitors. Rendering reuses the Vello/WGPU pipeline from `window.rs`
//! (offscreen texture, blit, present).
//!
//! Three placements cover the shell surfaces TontooOS needs
//! ([`LayerPlacement`]): a top bar (menubar), a bottom bar (dock) and a
//! fullscreen overlay (the LaunchPad grid, the widget layer). A bottom bar
//! is centered by the compositor and reserves no exclusive zone, so
//! maximized windows keep the whole output.
//!
//! An app floats above the desktop without swallowing input by returning
//! an [`App::input_region`] rect: everything outside it is click-through.
//! A surface may also ask for keyboard focus (`options.keyboard`), which
//! the compositor grants on map; keys then arrive through the same
//! [`App::key`] / [`App::raw_key`] / [`App::text`] hooks the winit shell
//! uses.
//!
//! One app can open further surfaces at runtime through
//! [`App::poll_overlay`]: the dock returns a LaunchPad app, which stays up
//! until its own app returns [`WindowCommand::Close`].
//!
//! Without a Wayland compositor `run_layer` returns an error instead of
//! hanging. There is deliberately no backdrop blur here: the compositor
//! backdrop stream only feeds space windows, so bars paint their own
//! translucent fills (see `BarMenu`) or a baked panel image.

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
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat,
    output::{OutputHandler, OutputInfo, OutputState},
    reexports::client::{
        globals::{registry_queue_init, GlobalList},
        protocol::{
            wl_compositor, wl_keyboard, wl_output, wl_pointer,
            wl_region::{Event as RegionEvent, WlRegion as Region},
            wl_seat, wl_surface,
        },
        Connection, Dispatch, Proxy, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers as SeatModifiers},
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
use super::window::{App, KeyPress, Modifiers, MouseButtonKind, RawKey, Viewport, WindowCommand};

/// Linux evdev code of the left mouse button.
pub const LAYER_BUTTON_LEFT: u32 = 0x110;

/// Where a layer surface sits on the output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerPlacement {
    /// Top edge, full width, reserving `height` as exclusive zone so
    /// maximized windows never overlap the bar. `Layer::Top`.
    TopBar,
    /// Bottom edge, centered, no exclusive zone: the surface floats over
    /// the desktop and over app windows. `Layer::Top`.
    BottomBar,
    /// The whole output on `Layer::Bottom`, above the wallpaper and below
    /// every app window. Takes no exclusive zone.
    Fullscreen,
}

/// Linux evdev code of the right mouse button.
pub const LAYER_BUTTON_RIGHT: u32 = 0x111;

/// Options of one layer surface.
///
/// The default is a top anchored bar, which is what shell surfaces need.
/// Use [`LayerBarOptions::bottom`] for a floating bottom bar (the dock) and
/// [`LayerBarOptions::fullscreen`] for a surface that covers the whole
/// output, like the desktop widget layer or the LaunchPad grid.
#[derive(Clone, Debug)]
pub struct LayerBarOptions {
    /// Layer-shell namespace, e.g. `"menubar"`.
    pub namespace: String,
    /// Surface height in logical px. For a [`LayerPlacement::TopBar`] this
    /// also becomes the exclusive zone; ignored when fullscreen.
    pub height: u32,
    /// Surface width in logical px. `0` follows the output width (top bar);
    /// a fixed width plus a vertical-only anchor makes the compositor center
    /// the surface, which is what the dock wants.
    pub width: u32,
    /// Where the surface sits on the output.
    pub placement: LayerPlacement,
    /// Ask the compositor for the pixels behind this surface so glass views
    /// can blur them. Requires a compositor that offers the backdrop stream;
    /// silently ignored elsewhere (and on every compositor for layer
    /// surfaces, which the stream does not cover).
    pub backdrop: bool,
    /// Take keyboard focus while the surface is mapped. Off for bars and the
    /// dock (pointer only), on for a modal overlay such as the LaunchPad
    /// grid, whose search field needs typing.
    pub keyboard: bool,
}

impl LayerBarOptions {
    /// Top anchored bar reserving `height` as exclusive zone.
    pub fn new(namespace: impl Into<String>, height: u32) -> Self {
        Self {
            namespace: namespace.into(),
            height: height.max(1),
            width: 0,
            placement: LayerPlacement::TopBar,
            backdrop: false,
            keyboard: false,
        }
    }

    /// Bottom anchored bar of `height`, centered by the compositor and
    /// floating above app windows without reserving screen space.
    pub fn bottom(namespace: impl Into<String>, height: u32) -> Self {
        Self {
            namespace: namespace.into(),
            height: height.max(1),
            width: 0,
            placement: LayerPlacement::BottomBar,
            backdrop: false,
            keyboard: false,
        }
    }

    /// A surface covering the whole output, used for desktop overlays that sit
    /// below app windows (`Layer::Bottom`) or, with
    /// [`with_keyboard`](Self::with_keyboard), for a modal grid above them.
    pub fn fullscreen(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            height: 0,
            width: 0,
            placement: LayerPlacement::Fullscreen,
            backdrop: false,
            keyboard: false,
        }
    }

    /// Request the compositor backdrop stream for this surface.
    pub fn with_backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// Fixed logical surface width; `0` follows the output.
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Ask the compositor for keyboard focus while mapped.
    pub fn with_keyboard(mut self, keyboard: bool) -> Self {
        self.keyboard = keyboard;
        self
    }

    /// True when the surface takes keyboard focus.
    pub fn wants_keyboard(&self) -> bool {
        self.keyboard
    }

    /// Layer-shell layer of this placement.
    pub fn layer_kind(&self) -> Layer {
        match self.placement {
            // `Bottom` sits above the wallpaper and below every app window,
            // which is what a desktop overlay wants. Bars want `Top` instead.
            LayerPlacement::Fullscreen => Layer::Bottom,
            LayerPlacement::TopBar | LayerPlacement::BottomBar => Layer::Top,
        }
    }

    /// Exclusive zone: a top bar reserves its own strip, a bottom bar and a
    /// fullscreen overlay reserve nothing so windows keep the whole output.
    pub fn exclusive_zone(&self) -> i32 {
        match self.placement {
            LayerPlacement::TopBar => exclusive_zone_for_height(self.height),
            LayerPlacement::BottomBar | LayerPlacement::Fullscreen => 0,
        }
    }

    /// Anchors for this placement.
    pub fn anchors(&self) -> Anchor {
        match self.placement {
            // All four anchors: the compositor sizes the surface to the output.
            LayerPlacement::Fullscreen => Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            LayerPlacement::TopBar => Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            // Vertical only: the compositor centers the surface horizontally,
            // which is exactly the bottom-center float a dock needs.
            LayerPlacement::BottomBar => Anchor::BOTTOM,
        }
    }

    /// Logical size requested from the compositor. A fullscreen surface is
    /// sized by the compositor, so the height it reports back is used
    /// instead of a fixed bar height.
    pub fn configured_size(&self, output_width: u32, output_height: u32) -> (u32, u32) {
        match self.placement {
            LayerPlacement::Fullscreen => (output_width.max(1), output_height.max(1)),
            LayerPlacement::TopBar => (0, self.height.max(1)),
            LayerPlacement::BottomBar => (self.width, self.height.max(1)),
        }
    }

    /// Logical height the surface is configured with.
    pub fn configured_height(&self, output_height: u32) -> u32 {
        self.configured_size(output_height, output_height).1
    }
}

/// An extra layer surface an app opens at runtime.
///
/// Returned from [`App::poll_overlay`]; the shell maps it on the same output
/// as the requesting surface. It closes again when its own app returns
/// [`WindowCommand::Close`], which leaves the requesting surface running.
pub struct OverlayRequest {
    /// The app driving the overlay surface.
    pub app: Box<dyn App>,
    /// Placement, size, backdrop and keyboard behavior of the overlay.
    pub options: LayerBarOptions,
}

/// One Wayland output offered to the factory.
#[derive(Clone, Debug)]
pub struct LayerOutput {
    /// Output name (`"eDP-1"`, `"HDMI-A-1"`, ...), or `"output-N"`.
    pub name: String,
    /// Logical width in px.
    pub width: u32,
    /// Logical height in px. A fullscreen overlay needs it to size its
    /// content; a top anchored bar can ignore it.
    pub height: u32,
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

/// Linux evdev code of the middle mouse button.
pub const LAYER_BUTTON_MIDDLE: u32 = 0x112;

/// Map a Linux evdev button code to a [`MouseButtonKind`].
///
/// Codes outside the three mouse buttons return `None`, so a layer surface
/// ignores scroll wheel pseudo-buttons and any extra device buttons.
pub fn button_kind(code: u32) -> Option<MouseButtonKind> {
    match code {
        LAYER_BUTTON_LEFT => Some(MouseButtonKind::Left),
        LAYER_BUTTON_RIGHT => Some(MouseButtonKind::Right),
        LAYER_BUTTON_MIDDLE => Some(MouseButtonKind::Middle),
        _ => None,
    }
}

/// Surface set one output should get, built by the [`run_layer_multi`]
/// factory. Empty or `None` skips the output.
pub type LayerSurfaces = Vec<(Box<dyn App>, LayerBarOptions)>;

/// Run one layer surface per Wayland output.
///
/// The factory is called for every output (and for hotplugged ones) and
/// returns the bar app plus its options, or `None` to skip the output.
/// Surfaces without keyboard focus drive the app with pointer events only.
/// Returns when the last surface closes, or an error without a Wayland
/// compositor (no `WAYLAND_DISPLAY`).
///
/// Use [`run_layer_multi`] for a process that needs more than one surface
/// per output (a bar plus a modal overlay).
pub fn run_layer(
    mut make: impl FnMut(LayerOutput) -> Option<(Box<dyn App>, LayerBarOptions)> + 'static,
) -> Result<(), Box<dyn Error>> {
    run_layer_multi(move |output| make(output).map(|surface| vec![surface]))
}

/// Run any number of layer surfaces per Wayland output.
///
/// The factory returns every surface this output should get, in stacking
/// order: later entries sit above earlier ones, so a dock can return its bar
/// first and a modal grid second. Returns when the last surface closes, or
/// an error without a Wayland compositor (no `WAYLAND_DISPLAY`).
///
/// Further surfaces can be opened while running through
/// [`App::poll_overlay`].
pub fn run_layer_multi(
    make: impl FnMut(LayerOutput) -> Option<LayerSurfaces> + 'static,
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
        // Raw proxy for `create_region` (input regions); sctk's helper only
        // keeps the surface factory.
        compositor_proxy: globals.bind(&qh, 1..=6, ()).ok(),
        layer_shell: LayerShell::bind(&globals, &qh)
            .map_err(|e| format!("no wlr-layer-shell (not a layer-shell compositor?): {e:?}"))?,
        context: RenderContext::new(),
        windows: Vec::new(),
        factory: Some(Box::new(make) as Box<dyn FnMut(LayerOutput) -> Option<LayerSurfaces>>),
        pointers: Vec::new(),
        keyboards: Vec::new(),
        modifiers: Modifiers::default(),
        focused: None,
        start: Instant::now(),
        exit: false,
        connection: conn,
        queue_handle: qh.clone(),
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
    /// Input region currently installed, so a repeat of the same rect does
    /// not re-send the request every frame. `None` means "not synced yet";
    /// the inner `None` means "whole surface clickable".
    input_region: Option<Option<(f32, f32, f32, f32)>>,
    /// True while this surface asked for keyboard focus.
    wants_keyboard: bool,
    /// True while the compositor has keyboard focus here.
    focused: bool,
}

struct LayerState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor: CompositorState,
    /// Raw `wl_compositor`, kept next to sctk's helper because input regions
    /// need `create_region` and the helper does not expose the proxy.
    compositor_proxy: Option<wl_compositor::WlCompositor>,
    layer_shell: LayerShell,
    context: RenderContext,
    windows: Vec<LayerWindow>,
    factory: Option<Box<dyn FnMut(LayerOutput) -> Option<LayerSurfaces>>>,
    pointers: Vec<wl_pointer::WlPointer>,
    keyboards: Vec<wl_keyboard::WlKeyboard>,
    /// Modifier state of the focused keyboard, forwarded to every app so
    /// pointer events carry the same snapshot as in the winit shell.
    modifiers: Modifiers,
    /// Index of the surface holding keyboard focus, if any.
    focused: Option<usize>,
    start: Instant,
    exit: bool,
    connection: Connection,
    queue_handle: QueueHandle<LayerState>,
    _globals: GlobalList,
}

impl LayerState {
    fn window_for_surface(&mut self, surface: &wl_surface::WlSurface) -> Option<&mut LayerWindow> {
        self.windows.iter_mut().find(|w| w.wl == *surface)
    }

    fn index_for_surface(&self, surface: &wl_surface::WlSurface) -> Option<usize> {
        self.windows.iter().position(|w| w.wl == *surface)
    }

    /// Build the layer surfaces for an output unless the factory skips it
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
        let (logical_w, logical_h) = info.logical_size.unwrap_or((0, 0));
        if logical_w <= 0 {
            return;
        }
        let summary = LayerOutput {
            name: name.clone(),
            width: logical_w.max(1) as u32,
            height: logical_h.max(1) as u32,
            scale: info.scale_factor.max(1),
        };
        let Some(surfaces) = self.factory.as_mut().and_then(|f| f(summary)) else {
            return;
        };
        let width = logical_w.max(1) as u32;
        let height = logical_h.max(1) as u32;
        let mut first: Option<LayerWindow> = None;
        // Created in factory order, so stacking follows the list.
        for (app, options) in surfaces {
            match self.create_window(qh, output, name.clone(), width, height, app, &options) {
                Ok(window) => {
                    window.wl.frame(qh, window.wl.clone());
                    window.wl.commit();
                    match first.as_mut() {
                        None => first = Some(window),
                        Some(_) => self.windows.push(window),
                    }
                }
                Err(err) => eprintln!("layer surface error: {err:?}"),
            }
        }
        // The first surface of an output goes in front so the list order
        // matches the z-order the compositor builds from creation order.
        if let Some(window) = first {
            self.windows.insert(0, window);
        }
    }

    /// Open every overlay an app asked for through [`App::poll_overlay`].
    ///
    /// Called once per frame before rendering. A new surface is mapped on
    /// the same output as the requesting one and sits above every surface
    /// that existed when it was created.
    fn open_overlays(&mut self, qh: &QueueHandle<LayerState>) {
        loop {
            let mut request: Option<(usize, OverlayRequest)> = None;
            for (index, window) in self.windows.iter_mut().enumerate() {
                if let Some(requested) = window.app.poll_overlay() {
                    request = Some((index, requested));
                    break;
                }
            }
            let Some((index, requested)) = request else {
                return;
            };
            let (output, name, width, height) = {
                let parent = &self.windows[index];
                (
                    parent.output.clone(),
                    parent.output_name.clone(),
                    parent.logical_w,
                    parent.logical_h,
                )
            };
            match self.create_window(
                qh,
                &output,
                name,
                width,
                height,
                requested.app,
                &requested.options,
            ) {
                Ok(window) => {
                    window.wl.frame(qh, window.wl.clone());
                    window.wl.commit();
                    self.windows.push(window);
                }
                Err(err) => eprintln!("layer overlay error: {err:?}"),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_window(
        &mut self,
        qh: &QueueHandle<LayerState>,
        output: &wl_output::WlOutput,
        output_name: String,
        output_width: u32,
        output_height: u32,
        app: Box<dyn App>,
        options: &LayerBarOptions,
    ) -> Result<LayerWindow, Box<dyn Error>> {
        let wl = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            wl.clone(),
            options.layer_kind(),
            Some(options.namespace.clone()),
            Some(output),
        );
        // `Neutral` exclusive zones keep the surface from reserving screen
        // space, so app windows are unaffected; only a top bar reserves its
        // own strip.
        layer.set_anchor(options.anchors());
        layer.set_exclusive_zone(options.exclusive_zone());
        let (request_w, request_h) = options.configured_size(output_width, output_height);
        layer.set_size(request_w, request_h);
        layer.set_margin(0, 0, 0, 0);
        // A keyboard surface is a modal overlay: it takes focus while it is
        // mapped and gives it back when it closes. Bars and the dock stay
        // pointer only.
        layer.set_keyboard_interactivity(if options.wants_keyboard() {
            KeyboardInteractivity::Exclusive
        } else {
            KeyboardInteractivity::None
        });

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
        // A fullscreen surface is sized by the compositor, so the output size
        // is what to configure against; a bar uses its own height.
        let logical_w = if request_w == 0 { output_width } else { request_w };
        let logical_h = request_h;
        let phys_w = physical_size(logical_w, scale).max(1);
        let phys_h = physical_size(logical_h, scale).max(1);
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
            logical_w,
            logical_h,
            scale,
            cursor: (0.0, 0.0),
            pending_size: None,
            input_region: None,
            wants_keyboard: options.wants_keyboard(),
            focused: false,
        })
    }

    /// Install the clickable rect of a surface, if it changed since the last
    /// frame. An app that returns `None` from [`App::input_region`] keeps the
    /// whole surface clickable; a negative width or height makes the surface
    /// click-through entirely.
    fn sync_input_region(&mut self, index: usize) {
        let requested = self.windows[index].app.input_region();
        if self.windows[index].input_region.as_ref() == Some(&requested) {
            return;
        }
        self.windows[index].input_region = Some(requested);
        // An empty region (no rectangles) never receives a pointer press,
        // which is how a surface goes fully click-through.
        let Some((x, y, w, h)) = requested else {
            self.windows[index].wl.set_input_region(None);
            let _ = self.connection.flush();
            return;
        };
        let Some(compositor) = self.compositor_proxy.clone() else {
            return;
        };
        let scale = self.windows[index].scale.max(1.0);
        let (pw, ph) = ((w * scale).round(), (h * scale).round());
        let region = compositor.create_region(&self.queue_handle, ());
        if pw >= 0.0 && ph >= 0.0 {
            region.add(
                (x * scale).round().max(0.0) as i32,
                (y * scale).round().max(0.0) as i32,
                pw.max(1.0) as i32,
                ph.max(1.0) as i32,
            );
        }
        self.windows[index].wl.set_input_region(Some(&region));
        let _ = self.connection.flush();
    }

    /// Remove the window of a closed layer surface. Exits when none left.
    fn remove_window(&mut self, layer: &LayerSurface) {
        let index = self.windows.iter().position(|w| w.layer == *layer);
        self.windows.retain(|w| w.layer != *layer);
        // Indices shift, so a focused window above the removed one moves.
        if let Some(focused) = self.focused {
            let still_there = index
                .map(|removed| focused >= removed)
                .unwrap_or(false);
            if !still_there {
                self.focused = None;
            } else if let Some(index) = index {
                self.focused = focused.checked_sub(if focused > index { 1 } else { 0 });
            }
        }
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
        // Overlays an app asked for last frame are mapped first, so they
        // also get a frame callback and start rendering right away.
        self.open_overlays(qh);
        self.sync_input_region(index);
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
        match capability {
            SeatCapability::Pointer => {
                if let Ok(pointer) = self.seat_state.get_pointer(qh, &seat) {
                    self.pointers.push(pointer);
                }
            }
            SeatCapability::Keyboard => {
                // `None` RMLVO: the compositor already picked the layout, so
                // the client only needs the keymap it sends on enter.
                if let Ok(keyboard) = self.seat_state.get_keyboard(qh, &seat, None) {
                    self.keyboards.push(keyboard);
                }
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: SeatCapability,
    ) {
        match capability {
            SeatCapability::Pointer => self.pointers.clear(),
            SeatCapability::Keyboard => {
                self.keyboards.clear();
                self.focused = None;
            }
            _ => {}
        }
        let _ = seat;
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        self.pointers.clear();
        self.keyboards.clear();
        self.focused = None;
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
            let Some(index) = self.index_for_surface(&event.surface) else {
                continue;
            };
            let modifiers = self.modifiers;
            let Some(window) = self.windows.get_mut(index) else {
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
                    let Some(kind) = button_kind(*button) else {
                        continue;
                    };
                    let pressed = true;
                    window.cursor = (x, y);
                    // Every button reports its raw transition first, so an app
                    // can act on right click and middle click, not just left.
                    window.app.mouse_button(kind, pressed, x, y, modifiers);
                    match kind {
                        MouseButtonKind::Left => window.app.mouse_down(x, y),
                        MouseButtonKind::Right => window.app.context_click(x, y),
                        // Middle click and any other device button are
                        // reported through `mouse_button` only.
                        MouseButtonKind::Middle | MouseButtonKind::Other(_) => {}
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    let Some(kind) = button_kind(*button) else {
                        continue;
                    };
                    let pressed = false;
                    window.cursor = (x, y);
                    window.app.mouse_button(kind, pressed, x, y, modifiers);
                    if kind == MouseButtonKind::Left {
                        window.app.mouse_up(x, y);
                    }
                }
                _ => {}
            }
        }
    }
}

impl KeyboardHandler for LayerState {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[Keysym],
    ) {
        // Only a surface that asked for keyboard focus takes input; a bar or
        // the dock is pointer only and ignores whatever the compositor sends.
        let Some(index) = self
            .index_for_surface(surface)
            .filter(|index| self.windows[*index].wants_keyboard)
        else {
            return;
        };
        if let Some(previous) = self.focused {
            if previous != index {
                if let Some(window) = self.windows.get_mut(previous) {
                    window.focused = false;
                    window.app.set_focused(false);
                }
            }
        }
        self.focused = Some(index);
        let window = &mut self.windows[index];
        window.focused = true;
        window.app.set_focused(true);
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _serial: u32,
    ) {
        let Some(index) = self.index_for_surface(surface) else {
            return;
        };
        if let Some(window) = self.windows.get_mut(index) {
            window.focused = false;
            window.app.set_focused(false);
        }
        if self.focused == Some(index) {
            self.focused = None;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        self.forward_key(event, true);
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        self.forward_key(event, false);
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: SeatModifiers,
        _layout: u32,
    ) {
        self.modifiers = Modifiers {
            shift: modifiers.shift,
            ctrl: modifiers.ctrl,
            alt: modifiers.alt,
            super_key: modifiers.logo,
        };
        let (ctrl, shift) = (self.modifiers.ctrl, self.modifiers.shift);
        // Every app learns about the change, not just the focused one: the
        // modifier state is shared with pointer events on any surface.
        for window in &mut self.windows {
            window.app.set_modifiers(ctrl, shift);
        }
    }
}

impl LayerState {
    /// Translate one key transition into the same `App` calls the winit shell
    /// makes: `raw_key` first with the full snapshot, then the intent hooks.
    ///
    /// Mirrors the winit order exactly: a Ctrl or Shift chord resolves to one
    /// intent and stops, a named key resolves to its intent, and everything
    /// else arrives as printable `text`.
    fn forward_key(&mut self, event: KeyEvent, pressed: bool) {
        let Some(index) = self.focused else {
            return;
        };
        let modifiers = self.modifiers;
        let text = event.utf8.clone();
        // A bare modifier press has no identity of its own; it reaches the
        // app through `Modifiers` alone, like in the winit shell.
        let Some(key) = raw_key_from_keysym(event.keysym.raw(), &modifiers) else {
            return;
        };
        let press = KeyPress {
            key,
            modifiers,
            text: if pressed { text.clone() } else { None },
            pressed,
            // The compositor sends one press per physical transition; an
            // auto-repeat arrives as further press events.
            repeat: false,
        };
        let Some(window) = self.windows.get_mut(index) else {
            return;
        };
        window.app.raw_key(&press);
        if !pressed {
            return;
        }
        if let Some(key) = chord_key(&press) {
            window.app.key(key);
            return;
        }
        match key {
            RawKey::Backspace => window.app.key(super::window::Key::Backspace),
            RawKey::Left => window.app.key(super::window::Key::Left),
            RawKey::Right => window.app.key(super::window::Key::Right),
            RawKey::Enter | RawKey::KeypadEnter => window.app.key(super::window::Key::Enter),
            RawKey::Escape => window.app.key(super::window::Key::Escape),
            _ => {
                if let Some(text) = text {
                    if !text.is_empty() {
                        window.app.text(&text);
                    }
                }
            }
        }
    }
}

/// X11 keysyms (keysymdef.h) of the keys the intent hooks care about. Kept as
/// literals so the mapping needs no further xkb dependency: the compositor
/// already hands out a translated keysym per press.
pub const KS_BACKSPACE: u32 = 0xff08;
pub const KS_TAB: u32 = 0xff09;
pub const KS_RETURN: u32 = 0xff0d;
pub const KS_PAUSE: u32 = 0xff13;
pub const KS_SCROLL_LOCK: u32 = 0xff14;
pub const KS_ESCAPE: u32 = 0xff1b;
pub const KS_HOME: u32 = 0xff50;
pub const KS_LEFT: u32 = 0xff51;
pub const KS_UP: u32 = 0xff52;
pub const KS_RIGHT: u32 = 0xff53;
pub const KS_DOWN: u32 = 0xff54;
pub const KS_PRIOR: u32 = 0xff55;
pub const KS_NEXT: u32 = 0xff56;
pub const KS_END: u32 = 0xff57;
pub const KS_MENU: u32 = 0xff67;
pub const KS_INSERT: u32 = 0xff63;
pub const KS_ISO_LEFT_TAB: u32 = 0xfe20;
pub const KS_NUM_LOCK: u32 = 0xff7f;
pub const KS_KP_SPACE: u32 = 0xff80;
pub const KS_KP_ENTER: u32 = 0xff8d;
pub const KS_KP_ASTERISK: u32 = 0xffaa;
pub const KS_KP_PLUS: u32 = 0xffab;
pub const KS_KP_MINUS: u32 = 0xffad;
pub const KS_KP_PERIOD: u32 = 0xffae;
pub const KS_KP_SLASH: u32 = 0xffaf;
pub const KS_KP_0: u32 = 0xffb0;
pub const KS_KP_9: u32 = 0xffb9;
pub const KS_F1: u32 = 0xffbe;
pub const KS_F12: u32 = 0xffc9;
pub const KS_CAPS_LOCK: u32 = 0xffe5;

/// Translate an X11 keysym into the layout-independent [`RawKey`] the `App`
/// hooks expect.
///
/// Latin-1 keysyms become `Character`, so a German layout reports the umlaut
/// it actually produced; the named function keys map to their variants.
/// Modifier keysyms (`Shift_L`, `Control_R`, ...) return `None`: they carry
/// no identity of their own and reach the app through `Modifiers`, exactly
/// like a bare modifier press in the winit shell.
pub fn raw_key_from_keysym(keysym: u32, modifiers: &Modifiers) -> Option<RawKey> {
    let named = match keysym {
        KS_BACKSPACE => RawKey::Backspace,
        KS_TAB => RawKey::Tab,
        KS_ISO_LEFT_TAB => RawKey::BackTab,
        KS_RETURN => RawKey::Enter,
        KS_KP_ENTER => RawKey::KeypadEnter,
        KS_ESCAPE => RawKey::Escape,
        KS_INSERT => RawKey::Insert,
        KS_HOME => RawKey::Home,
        KS_END => RawKey::End,
        KS_PRIOR => RawKey::PageUp,
        KS_NEXT => RawKey::PageDown,
        KS_LEFT => RawKey::Left,
        KS_UP => RawKey::Up,
        KS_RIGHT => RawKey::Right,
        KS_DOWN => RawKey::Down,
        KS_PAUSE => RawKey::Pause,
        KS_SCROLL_LOCK => RawKey::ScrollLock,
        KS_NUM_LOCK => RawKey::NumLock,
        KS_CAPS_LOCK => RawKey::CapsLock,
        KS_MENU => RawKey::ContextMenu,
        KS_KP_SPACE => RawKey::Character(' '),
        KS_KP_PERIOD => RawKey::KeypadDot,
        KS_KP_PLUS => RawKey::KeypadPlus,
        KS_KP_MINUS => RawKey::KeypadMinus,
        KS_KP_ASTERISK => RawKey::KeypadStar,
        KS_KP_SLASH => RawKey::KeypadSlash,
        // Function keys are one contiguous run, F1 at KS_F1.
        KS_F1..=KS_F12 => RawKey::Function((keysym - KS_F1 + 1) as u8),
        // Keypad digits are one contiguous run, 0 at KS_KP_0.
        KS_KP_0..=KS_KP_9 => RawKey::KeypadDigit((keysym - KS_KP_0) as u8),
        _ => {
            // Latin-1 is the whole keysym space for printable characters.
            if !(0x20..0x100).contains(&keysym) {
                return None;
            }
            let mut code = keysym;
            // With Ctrl or Alt held the platform produces no text and the
            // keysym stays the unshifted base, so letters arrive lower case.
            if modifiers.ctrl || modifiers.alt {
                code = (code as u8 as char).to_ascii_lowercase() as u32;
            }
            RawKey::Character(char::from_u32(code)?)
        }
    };
    Some(named)
}

/// Intent key of a Ctrl or Shift chord, or `None` for plain text input.
fn chord_key(press: &KeyPress) -> Option<super::window::Key> {
    use super::window::Key;
    if press.modifiers.ctrl {
        return match press.key {
            RawKey::Character('a') => Some(Key::SelectAll),
            RawKey::Character('c') => Some(Key::Copy),
            RawKey::Character('x') => Some(Key::Cut),
            RawKey::Character('v') => Some(Key::Paste),
            RawKey::Character('z') if press.modifiers.shift => Some(Key::Redo),
            RawKey::Character('z') => Some(Key::Undo),
            RawKey::Character('y') => Some(Key::Redo),
            _ => None,
        };
    }
    if press.modifiers.shift {
        return match press.key {
            RawKey::Left => Some(Key::SelectLeft),
            RawKey::Right => Some(Key::SelectRight),
            RawKey::Up => Some(Key::SelectUp),
            RawKey::Down => Some(Key::SelectDown),
            _ => None,
        };
    }
    None
}

delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_seat!(LayerState);
delegate_pointer!(LayerState);
delegate_keyboard!(LayerState);
delegate_layer!(LayerState);
delegate_registry!(LayerState);

// Bound only to reach `wl_compositor.create_region` for input regions; every
// request on it goes through `CompositorState` above. Both objects carry no
// events, so `event` is never called.
impl Dispatch<wl_compositor::WlCompositor, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<Region, ()> for LayerState {
    fn event(
        _state: &mut Self,
        _proxy: &Region,
        _event: RegionEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

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

    #[test]
    fn top_bar_reserves_its_strip() {
        let options = LayerBarOptions::new("menubar", 36);
        assert_eq!(options.placement, LayerPlacement::TopBar);
        assert_eq!(options.exclusive_zone(), 36);
        assert_eq!(options.anchors(), Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        // Width follows the output, height is fixed.
        assert_eq!(options.configured_size(1920, 1080), (0, 36));
        assert_eq!(options.layer_kind(), Layer::Top);
    }

    #[test]
    fn bottom_bar_is_centered_and_reserves_nothing() {
        let options = LayerBarOptions::bottom("dock", 177).with_width(500);
        assert_eq!(options.placement, LayerPlacement::BottomBar);
        // No exclusive zone: maximized windows keep the whole output.
        assert_eq!(options.exclusive_zone(), 0);
        // A vertical-only anchor makes the compositor center the surface.
        assert_eq!(options.anchors(), Anchor::BOTTOM);
        assert_eq!(options.configured_size(1920, 1080), (500, 177));
        // The dock floats above app windows, like a top bar.
        assert_eq!(options.layer_kind(), Layer::Top);
    }

    #[test]
    fn fullscreen_sits_below_app_windows() {
        let options = LayerBarOptions::fullscreen("widgets");
        assert_eq!(options.placement, LayerPlacement::Fullscreen);
        assert_eq!(options.layer_kind(), Layer::Bottom);
        assert_eq!(options.exclusive_zone(), 0);
        assert_eq!(
            options.anchors(),
            Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT
        );
        // The compositor sizes it, so the output dimensions are the request.
        assert_eq!(options.configured_size(1920, 1080), (1920, 1080));
    }

    #[test]
    fn keyboard_is_opt_in() {
        assert!(!LayerBarOptions::new("menubar", 30).wants_keyboard());
        assert!(!LayerBarOptions::bottom("dock", 177).wants_keyboard());
        assert!(LayerBarOptions::fullscreen("launchpad").with_keyboard(true).wants_keyboard());
    }

    #[test]
    fn keysyms_map_to_raw_keys() {
        let plain = Modifiers::default();
        assert_eq!(raw_key_from_keysym(KS_ESCAPE, &plain), Some(RawKey::Escape));
        assert_eq!(raw_key_from_keysym(KS_RETURN, &plain), Some(RawKey::Enter));
        assert_eq!(raw_key_from_keysym(KS_BACKSPACE, &plain), Some(RawKey::Backspace));
        // F-keys and keypad digits are contiguous runs.
        assert_eq!(raw_key_from_keysym(KS_F1, &plain), Some(RawKey::Function(1)));
        assert_eq!(raw_key_from_keysym(KS_F1 + 11, &plain), Some(RawKey::Function(12)));
        assert_eq!(raw_key_from_keysym(0xffb0, &plain), Some(RawKey::KeypadDigit(0)));
        assert_eq!(raw_key_from_keysym(0xffb9, &plain), Some(RawKey::KeypadDigit(9)));
    }

    #[test]
    fn printable_keysyms_become_characters() {
        let plain = Modifiers::default();
        assert_eq!(raw_key_from_keysym(b'a' as u32, &plain), Some(RawKey::Character('a')));
        assert_eq!(raw_key_from_keysym(b'A' as u32, &plain), Some(RawKey::Character('A')));
        // A German layout produces the umlaut it actually typed.
        assert_eq!(raw_key_from_keysym(0xE4, &plain), Some(RawKey::Character('ä')));
        // Ctrl arrives as the unshifted base, like the winit shell reports it.
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        assert_eq!(raw_key_from_keysym(b'C' as u32, &ctrl), Some(RawKey::Character('c')));
    }

    #[test]
    fn modifier_keysyms_have_no_identity() {
        let plain = Modifiers::default();
        // Shift_L, Control_L, Alt_L, Super_L carry no RawKey of their own.
        assert_eq!(raw_key_from_keysym(0xffe1, &plain), None);
        assert_eq!(raw_key_from_keysym(0xffe3, &plain), None);
        assert_eq!(raw_key_from_keysym(0xffe9, &plain), None);
        assert_eq!(raw_key_from_keysym(0xffeb, &plain), None);
        // Nor does anything outside Latin-1 and the named keys.
        assert_eq!(raw_key_from_keysym(0x0100_0001, &plain), None);
    }

    #[test]
    fn chords_resolve_to_intent_keys() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        let press = KeyPress {
            key: RawKey::Character('c'),
            modifiers: ctrl,
            text: None,
            pressed: true,
            repeat: false,
        };
        assert_eq!(chord_key(&press), Some(crate::renderer::window::Key::Copy));
        let shift = Modifiers {
            shift: true,
            ..Modifiers::default()
        };
        let press = KeyPress {
            key: RawKey::Left,
            modifiers: shift,
            text: None,
            pressed: true,
            repeat: false,
        };
        assert_eq!(chord_key(&press), Some(crate::renderer::window::Key::SelectLeft));
        // Plain text resolves to no chord.
        let press = KeyPress {
            key: RawKey::Character('c'),
            modifiers: Modifiers::default(),
            text: Some("c".into()),
            pressed: true,
            repeat: false,
        };
        assert_eq!(chord_key(&press), None);
    }
}
