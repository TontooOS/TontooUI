# BackdropStream

The backdrop stream is how a TontooUI window blurs the *desktop*. Wayland
never lets a client read the pixels behind its own window, so the compositor
has to hand them over: TontooCompositor captures the elements below the
window once, writes them into a memory file TontooUI allocated, and
announces the rect. TontooUI then runs its own WGSL blur over those pixels.

The division of labour is what makes this cheap on the compositor side:

| Step | Owner |
|---|---|
| Deciding which pixels sit behind the window | Compositor |
| Copying them out of the GPU | Compositor |
| Blurring, lensing and frosting them | TontooUI |

Without the stream the shell falls back to its own second render pass
(`App::wants_backdrop`), which blurs only what the app itself draws. Glass
still works, but the desktop behind a transparent body stays sharp.

Everything in this page is optional. On any compositor that does not offer
`tontoo_ui_manager` (or on a non-Wayland backend) `attach` returns `None`
and the shell behaves exactly as before.

## Attach

### CompositorBackdrop::attach

```rust
pub fn attach<R>(window: &R, buffer_width: u32, buffer_height: u32) -> Option<Self>
where
    R: HasDisplayHandle + HasWindowHandle,
```

Wraps winit's `wl_display` in a guest Wayland backend, binds
`tontoo_ui_manager`, creates a `tontoo_ui_surface` and sends
`create_backdrop_buffer` with a fresh `memfd` of `buffer_width` x
`buffer_height` x 4 bytes.

`buffer_width` and `buffer_height` must cover the whole output in physical
pixels: the compositor writes the window rect at its position inside that
buffer, so a small buffer would only work for a window in the top left
corner. The shell passes the monitor size from
`ActiveEventLoop::primary_monitor`, falling back to the window size.

| Case | Result |
|---|---|
| Not a Wayland display or window handle | `None` |
| `wl_display` or `wl_surface` handle is missing | `None` |
| Compositor has no `tontoo_ui_manager` | `None` |
| `memfd_create`, `ftruncate` or `mmap` fails | `None` |
| Surface protocol id is `0` | `None` |
| Success | `Some(CompositorBackdrop)` |

The window surface protocol id is read from winit's raw handle with
`ObjectId::from_ptr`. Both processes share one `wl_display`, so the id space
matches and the compositor can resolve it to the mapped window.

## Driving the stream

### CompositorBackdrop::set_enabled

```rust
pub fn set_enabled(&mut self, enabled: bool, width: u32, height: u32)
```

Enables or disables the capture for the attached window. `width` and
`height` are the window's physical size and are restated on every call, so a
resize while enabled keeps the frames aligned. The request itself only goes
out on a state change.

The shell follows `App::wants_backdrop`, so a window that never shows glass
never makes the compositor capture anything.

### CompositorBackdrop::poll

```rust
pub fn poll(&mut self)
```

Flushes the connection, drains every readable event without blocking and
acknowledges a frame that arrived. Called once per rendered frame.

| Case | Effect |
|---|---|
| Socket not readable | Dispatches whatever is buffered and returns |
| Flush, dispatch or read fails | Marks the client dead |
| A frame is waiting | Sends `ack_backdrop`, which unblocks the next capture |

### CompositorBackdrop::take_frame

```rust
pub fn take_frame(&mut self) -> Option<Frame>
```

Returns the newest window-sized frame, or `None` when no new one arrived.

### CompositorBackdrop::is_alive

```rust
pub fn is_alive(&self) -> bool
```

`false` once the connection failed. The shell drops the client and keeps the
in-app capture pass.

## Frame

```rust
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
```

RGBA8, `width` x `height` physical pixels, already upsampled to the window
size. `pixels.len() == width * height * 4`.

## Upsample

### upsample

```rust
pub fn upsample(src: &[u8], src_w: u32, src_h: u32, width: u32, height: u32) -> Option<Vec<u8>>
```

Nearest-neighbour upscale of the compositor's reduced-resolution samples to
the window size.

| Case | Result |
|---|---|
| Any dimension is `0` | `None` |
| `src` shorter than `src_w * src_h * 4` | `None` |
| Same size requested | Copy of `src` |
| Success | Upscaled RGBA8 |

The compositor samples the desktop at `1 / BACKDROP_SCALE`. The glass blur
uses a sigma far larger than that, so blocky samples disappear afterwards.
Upsampling on the CPU keeps every glass view (`fill_lens_glass`,
`fill_backdrop_veil`, `stroke_backdrop_edge`, ...) unchanged: they all
assume a full window sized backdrop texture with image pixel `(0, 0)` at
scene `(0, 0)`.

## Constants

### BACKDROP_SCALE

```rust
pub const BACKDROP_SCALE: u32 = 2;
```

Downscale divisor sent with `set_backdrop`. Two halves the compositor's
readback and is visually indistinguishable from full resolution once blurred.

## Renderer integration

`src/renderer/window.rs` owns one `CompositorBackdrop` per window and one
flag, `backdrop_external`:

| Situation | Backdrop source | Vello passes per frame |
|---|---|---|
| Stream alive, frame received | Compositor pixels | 1 |
| Stream alive, no new frame yet | Last compositor frame | 1 |
| No stream, `wants_backdrop()` | Own capture pass | 2 |
| No stream, no `wants_backdrop()` | None | 1 |

A received frame is uploaded with
`BackdropBlur::upload_content`, which writes straight into the `content`
target the blur reads, so the existing compute pipeline needs no changes.

## Errors

`attach` swallows every failure and returns `None`; nothing about the stream
is fatal. `poll` marks the client dead on a broken connection, after which
the shell drops it. A `backdrop` event whose rect does not fit the mapping is
logged and dropped, and the client falls back to its last good frame.

## Cross References

- [Renderer.md](Renderer.md) -- `App::wants_backdrop`, `BackdropBlur`, the
  two-pass frame loop
- [Glass.md](Glass.md) -- what consumes the blurred backdrop
- [Theme.md](Theme.md) -- glass stage and frost tints
