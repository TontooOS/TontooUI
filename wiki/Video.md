# Video

Video category in `src/elements/video/`: real file/URL playback
through MediaKit (`frame_at`), uploaded via the streaming image
cache. Transport controls are play/pause, stop, scrubber, time,
mute and fullscreen — deliberately no next/previous controls.
`source.rs` holds `VideoSource`, `player.rs` the `VideoPlayer`
element; shared geometry lives in `mod.rs`.

## VideoSource

```rust
pub enum VideoSource { File(PathBuf), Url(String) }
```

- Only containers/codecs MediaKit decodes natively play.
- URLs must point to a directly downloadable file; HLS playlists
  (`.m3u8`) are rejected via `url_problem` (segments use delivery
  codecs outside the native decoders).
- `display_name()` returns the file name or URL tail.

## VideoPlayer

```rust
VideoPlayer::file(path, width, height)
VideoPlayer::url(url, width, height)
```

- File opens synchronously (`Ready` or `Failed`); URL downloads on
  a background thread to temp (deleted on drop) with a `Spinner`
  while `Loading`.
- Clock advances in `draw` via `advance(Instant)`; frames decode on
  demand and upload with a sequence number, so still frames cost
  nothing.
- Transport API: `play`, `pause`, `stop`, `seek`, `set_speed`
  (0.5-2.0), `set_volume`, `set_muted`, `toggle_fullscreen`, plus
  `position`, `duration`, `is_playing`, `is_finished`,
  `is_fullscreen`, `state`, `source`.
- App-level `mouse_move(x, y)` forwards scrubber drags; the shell
  forwards it like `mouse_down`/`mouse_up`.

## Geometry

| Token | Value |
|---|---|
| `VIDEO_BAR_H` | 48 px transport bar |
| `VIDEO_BTN` | 32 px transport button box |
| `VIDEO_GAP` | 8 px transport gaps |
| `VIDEO_TIME_W` | 110 px time label slot |
| `VIDEO_TIME_SIZE` | 12 px time label |
| `VIDEO_RADIUS` | 12 px frame corner radius |
| `VIDEO_SHADOW` / `VIDEO_SHADOW_BLUR` / `VIDEO_SHADOW_DY` | Small drop shadow under the frame |

## Cross References

- [Images.md](Images.md) – `ImageFit`, `fit_rect`, placeholder fills
- [Slider.md](Slider.md) – scrubber behavior
- [Renderer.md](Renderer.md) – streaming frame cache upload
