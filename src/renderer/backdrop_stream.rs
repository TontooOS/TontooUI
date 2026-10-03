//! Desktop backdrop stream client.
//!
//! TontooCompositor owns the desktop pixels: a client can never read what is
//! behind its own window. The compositor therefore captures the elements
//! below a window once, writes them into a memory file this client
//! allocated, and announces the rect. TontooUI blurs that image itself with
//! the existing [`BackdropBlur`](super::backdrop::BackdropBlur) compute pass,
//! so the compositor never runs a Gaussian blur.
//!
//! Protocol: `protocol/tontoo_ui.xml` (see `wiki/BackdropStream.md`).
//! Everything here is optional: without a compositor that offers the stream
//! the client silently keeps its own in-app capture pass.

use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::io::{AsRawFd, FromRawFd};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::backend::{Backend, ObjectId};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};

/// Generated `tontoo_ui` client protocol.
///
/// `generate_client_code!` emits one module per XML interface, so the
/// manager and surface live directly under this module.
pub mod protocol {
    pub use wayland_client;
    // The generated code refers to `wayland_backend::protocol` from every
    // module it emits into, which only resolves through an extern crate
    // declaration.
    pub extern crate wayland_backend;

    pub mod __interfaces {
        wayland_scanner::generate_interfaces!("protocol/tontoo_ui.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("protocol/tontoo_ui.xml");
}

use protocol::tontoo_ui_manager::TontooUiManager;
use protocol::tontoo_ui_surface::TontooUiSurface;

/// Downscale divisor requested from the compositor.
///
/// One means full resolution. That is the right choice because the compositor
/// only captures when the content behind the window actually changed, so a
/// panel opening over a still desktop pays for exactly one readback no matter
/// how long it stays open. A divisor above one only helps when something
/// behind the window animates continuously.
pub const BACKDROP_SCALE: u32 = 1;


// ---------------------------------------------------------------------------
// Shared memory
// ---------------------------------------------------------------------------

/// Read-write mapping of the file both sides share.
struct Mapping {
    /// Kept alive so the descriptor stays valid; the compositor holds a
    /// duplicate of it.
    _file: std::fs::File,
    map: *mut u8,
    len: usize,
    width: i32,
    height: i32,
    stride: i32,
}

impl Mapping {
    /// Create the shared file and map it read-write.
    fn create(width: i32, height: i32, stride: i32) -> Option<Self> {
        if width < 1 || height < 1 || stride < width * 4 {
            return None;
        }
        let len = (stride as usize).checked_mul(height as usize)?;
        // SAFETY: a fixed, NUL terminated name.
        let raw = unsafe {
            libc::memfd_create(
                b"tontooui-backdrop\0".as_ptr() as *const libc::c_char,
                libc::MFD_CLOEXEC,
            )
        };
        if raw < 0 {
            return None;
        }
        // SAFETY: `raw` is a fresh descriptor nothing else owns.
        let file = unsafe { std::fs::File::from_raw_fd(raw) };
        file.set_len(len as u64).ok()?;
        // SAFETY: plain shared file mapping, no pointers are dereferenced.
        let map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if map == libc::MAP_FAILED {
            return None;
        }
        Some(Self {
            _file: file,
            map: map as *mut u8,
            len,
            width,
            height,
            stride,
        })
    }

    /// Duplicate the descriptor for the compositor.
    fn duplicate_fd(&self) -> Option<OwnedFd> {
        self._file.try_clone().ok().map(OwnedFd::from)
    }

    /// Copy the announced rect out as RGBA8, sampling every `scale`-th
    /// pixel. Returns the sampled pixels plus their dimensions.
    fn sample(&self, x: i32, y: i32, width: i32, height: i32, scale: u32) -> Option<Sampled> {
        let scale = scale.max(1) as i32;
        if width < 1 || height < 1 {
            return None;
        }
        let columns = (width - 1) / scale + 1;
        let rows = (height - 1) / scale + 1;
        if x < 0
            || y < 0
            || x + (columns - 1) * scale >= self.width
            || y + (rows - 1) * scale >= self.height
        {
            return None;
        }
        let mut pixels = vec![0u8; (columns as usize) * (rows as usize) * 4];
        let stride = self.stride as usize;
        // SAFETY: the rect and the row length were bounds checked above, so
        // every read stays inside the mapping.
        unsafe {
            let base = self.map.add(y as usize * stride + x as usize * 4);
            for row in 0..rows {
                let from = base.add(row as usize * stride);
                for col in 0..columns {
                    let src = from.add(col as usize * scale as usize * 4);
                    let dst = pixels.as_mut_ptr().add((row * columns + col) as usize * 4);
                    std::ptr::copy_nonoverlapping(src, dst, 4);
                }
            }
        }
        Some(Sampled {
            width: columns as u32,
            height: rows as u32,
            pixels,
        })
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: the mapping came from the matching `mmap` in `create`.
        unsafe {
            libc::munmap(self.map as *mut libc::c_void, self.len);
        }
    }
}

/// One sampled desktop rect, still at the compositor's reduced resolution.
struct Sampled {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Upsample
// ---------------------------------------------------------------------------

/// Nearest-neighbour upscale of a sampled frame to `width` x `height`.
///
/// The compositor samples the desktop at `1 / scale`; glass blurs the result
/// with a sigma far larger than `scale`, so blocky samples disappear
/// afterwards while every glass view keeps assuming a full window sized
/// backdrop texture.
pub fn upsample(src: &[u8], src_w: u32, src_h: u32, width: u32, height: u32) -> Option<Vec<u8>> {
    if src_w == 0 || src_h == 0 || width == 0 || height == 0 {
        return None;
    }
    if src.len() < src_w as usize * src_h as usize * 4 {
        return None;
    }
    if src_w == width && src_h == height {
        return Some(src.to_vec());
    }
    let mut out = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        let sy = (y * src_h as usize / height as usize).min(src_h as usize - 1);
        for x in 0..width as usize {
            let sx = (x * src_w as usize / width as usize).min(src_w as usize - 1);
            let from = (sy * src_w as usize + sx) * 4;
            let to = (y * width as usize + x) * 4;
            out[to..to + 4].copy_from_slice(&src[from..from + 4]);
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Protocol state
// ---------------------------------------------------------------------------

/// One captured desktop frame, window sized.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Event enums of the generated interfaces.
type ManagerEvent = <TontooUiManager as Proxy>::Event;
type SurfaceEvent = <TontooUiSurface as Proxy>::Event;

/// Dispatch state for the `tontoo_ui` objects.
#[derive(Default)]
struct BackdropState {
    surface: Option<TontooUiSurface>,
    map: Option<Mapping>,
    /// Serial of the frame currently waiting to be copied out.
    serial: u32,
    frame: Option<Frame>,
    /// Window physical size every sampled frame is upscaled to.
    target: (u32, u32),
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BackdropState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _queue: &QueueHandle<Self>,
    ) {
        // `registry_queue_init` already collected the globals; nothing to do
        // for late announcements because the client binds them eagerly.
    }
}

impl Dispatch<TontooUiManager, ()> for BackdropState {
    fn event(
        _state: &mut Self,
        manager: &TontooUiManager,
        event: ManagerEvent,
        _data: &(),
        _conn: &Connection,
        _queue: &QueueHandle<Self>,
    ) {
        match event {
            ManagerEvent::Ping { serial } => manager.pong(serial),
        }
    }
}

impl Dispatch<TontooUiSurface, ()> for BackdropState {
    fn event(
        state: &mut Self,
        _surface: &TontooUiSurface,
        event: SurfaceEvent,
        _data: &(),
        _conn: &Connection,
        _queue: &QueueHandle<Self>,
    ) {
        let SurfaceEvent::Backdrop {
            serial,
            x,
            y,
            width,
            height,
            scale,
        } = event
        else {
            return;
        };
        // Copy before acknowledging: the compositor may overwrite the buffer
        // as soon as the ack arrives.
        let sampled = state
            .map
            .as_ref()
            .and_then(|map| map.sample(x, y, width, height, scale));
        let (tw, th) = state.target;
        let Some(sampled) = sampled else {
            eprintln!("tontooui: backdrop rect {width}x{height}+{x}+{y} rejected");
            return;
        };
        match upsample(
            &sampled.pixels,
            sampled.width,
            sampled.height,
            tw,
            th,
        ) {
            Some(pixels) => {
                state.serial = serial;
                state.frame = Some(Frame {
                    width: tw,
                    height: th,
                    pixels,
                });
            }
            None => eprintln!("tontooui: backdrop upsample failed"),
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Live subscription to the compositor backdrop of one window.
pub struct CompositorBackdrop {
    connection: Connection,
    queue: EventQueue<BackdropState>,
    state: BackdropState,
    /// Protocol id of the watched window, `0` when unknown.
    surface_id: u32,
    enabled: bool,
    alive: bool,
}

impl CompositorBackdrop {
    /// Attach to the compositor for `window`, if it offers the stream.
    ///
    /// `buffer_width` / `buffer_height` must cover the whole output in
    /// physical pixels so a moving window never needs a new allocation.
    /// Returns `None` on any other Wayland compositor, which is not an
    /// error: the shell then falls back to its own in-app capture pass.
    pub fn attach<R>(window: &R, buffer_width: u32, buffer_height: u32) -> Option<Self>
    where
        R: HasDisplayHandle + HasWindowHandle,
    {
        let display = window.display_handle().ok()?;
        let RawDisplayHandle::Wayland(display) = display.as_raw() else {
            return None;
        };
        let handle = window.window_handle().ok()?;
        let RawWindowHandle::Wayland(handle) = handle.as_raw() else {
            return None;
        };
        let display_ptr = display.display.as_ptr();
        let surface_ptr = handle.surface.as_ptr();
        // SAFETY: winit owns both objects and keeps them alive for as long
        // as the window exists. The client is dropped with the window, so
        // the guest backend never outlives them.
        let backend = unsafe { Backend::from_foreign_display(display_ptr.cast()) };
        let connection = Connection::from_backend(backend);
        let (globals, queue) = registry_queue_init::<BackdropState>(&connection).ok()?;

        let mut state = BackdropState::default();
        let mut queue = queue;
        // SAFETY: the surface pointer is the live `wl_surface` winit created
        // on the display we just wrapped, so both share one id space.
        let surface_id = unsafe {
            ObjectId::from_ptr(WlSurface::interface(), surface_ptr.cast())
                .ok()?
                .protocol_id()
        };
        if surface_id == 0 {
            return None;
        }

        let width = buffer_width as i32;
        let height = buffer_height as i32;
        let stride = width * 4;
        let map = Mapping::create(width, height, stride)?;
        let fd = map.duplicate_fd()?;
        state.map = Some(map);

        let manager: TontooUiManager = globals.bind(&queue.handle(), 1..=1, ()).ok()?;
        let qh = queue.handle();
        manager.get_tontoo_ui_surface(&qh, ());
        queue.roundtrip(&mut state).ok()?;
        let surface = state.surface.clone()?;
        surface.create_backdrop_buffer(fd.as_fd(), width, height, stride);
        queue.roundtrip(&mut state).ok()?;

        Some(Self {
            connection,
            queue,
            state,
            surface_id,
            enabled: false,
            alive: true,
        })
    }

    /// True while the subscription is usable.
    pub fn is_alive(&self) -> bool {
        self.alive
    }

    /// Enable or disable the stream for a window of `width` x `height`
    /// physical pixels.
    ///
    /// The target size is restated on every call so a resize while enabled
    /// keeps the frames aligned; the request itself only goes out on a state
    /// change.
    pub fn set_enabled(&mut self, enabled: bool, width: u32, height: u32) {
        if !self.alive {
            return;
        }
        if enabled {
            self.state.target = (width.max(1), height.max(1));
        }
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        let Some(surface) = self.state.surface.as_ref() else {
            return;
        };
        surface.set_backdrop(
            u32::from(enabled),
            BACKDROP_SCALE,
            self.surface_id,
        );
        let _ = self.connection.flush();
    }

    /// Drain pending protocol events. A frame that arrived is acknowledged
    /// and returned by the next [`take_frame`](Self::take_frame).
    pub fn poll(&mut self) {
        if !self.alive {
            return;
        }
        loop {
            if self.connection.flush().is_err() {
                self.alive = false;
                return;
            }
            match self.connection.prepare_read() {
                Some(guard) => {
                    let failed = self.queue.dispatch_pending(&mut self.state).is_err();
                    // `read` consumes the guard.
                    let failed = failed || guard.read().is_err();
                    if failed {
                        self.alive = false;
                        return;
                    }
                }
                None => {
                    let _ = self.queue.dispatch_pending(&mut self.state);
                    break;
                }
            }
        }
        // The frame is ours: release the compositor to overwrite it.
        if self.state.frame.is_some() {
            if let Some(surface) = self.state.surface.as_ref() {
                surface.ack_backdrop(self.state.serial);
                let _ = self.connection.flush();
            }
        }
    }

    /// Take the newest frame, if one arrived since the last call.
    pub fn take_frame(&mut self) -> Option<Frame> {
        self.state.frame.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsample_replicates_pixels() {
        let src = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let out = upsample(&src, 2, 1, 4, 2).expect("upsample");
        assert_eq!(out.len(), 4 * 2 * 4);
        assert_eq!(&out[0..4], &[1, 2, 3, 4]);
        assert_eq!(&out[4..8], &[1, 2, 3, 4]);
        assert_eq!(&out[8..12], &[5, 6, 7, 8]);
        assert_eq!(&out[12..16], &[5, 6, 7, 8]);
        assert_eq!(&out[16..20], &[1, 2, 3, 4]);
    }

    #[test]
    fn upsample_identity_copies() {
        let src = vec![9u8; 2 * 2 * 4];
        let out = upsample(&src, 2, 2, 2, 2).expect("upsample");
        assert_eq!(out, src);
    }

    #[test]
    fn upsample_rejects_bad_input() {
        assert!(upsample(&[], 0, 0, 4, 4).is_none());
        assert!(upsample(&[0; 4], 4, 4, 2, 2).is_none());
        assert!(upsample(&[0; 16], 2, 2, 0, 2).is_none());
    }

    #[test]
    fn mapping_samples_every_scale_pixel() {
        let map = Mapping::create(8, 8, 8 * 4).expect("map");
        let fd = map.duplicate_fd().expect("dup");
        assert!(fd.as_fd().as_raw_fd() >= 0);
        let sampled = map.sample(0, 0, 4, 4, 2).expect("sample");
        assert_eq!((sampled.width, sampled.height), (2, 2));
        assert_eq!(sampled.pixels.len(), 2 * 2 * 4);
        // A rect that does not fit the buffer is rejected.
        assert!(map.sample(4, 4, 8, 8, 1).is_none());
    }
}
