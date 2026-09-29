use std::any::Any;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::Button;
use super::super::images::{ImageFit, fit_rect};
use super::super::layout::View;
use super::super::progress::Spinner;
use super::super::sliders::Slider;
use super::{VIDEO_BAR_H, VIDEO_BTN, VIDEO_GAP, VIDEO_RADIUS, VIDEO_SHADOW, VIDEO_SHADOW_BLUR,
    VIDEO_SHADOW_DY, VIDEO_TIME_SIZE, VIDEO_TIME_W};
use super::source::VideoSource;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::GlassAmount;

/// Load/playback state. Deliberately no next/previous: a player shows
/// one file or URL, period.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VideoState {
    /// URL download in flight (or file open pending).
    Loading,
    /// Opened, frames decode on demand.
    Ready,
    /// Unsupported container/codec, missing file or failed download.
    Failed(String),
}

/// File/URL video playback with transport controls (play/pause, stop,
/// scrubber, time, mute, fullscreen). No next/previous controls by
/// design. Frames decode through MediaKit (`frame_at`) and upload via
/// the streaming image cache, so no player binary is involved.
///
/// Use `VideoPlayer::file` for disk media and `VideoPlayer::url` for
/// progressive `http(s)` downloads (temporarily stored, deleted on
/// drop).
pub struct VideoPlayer {
    source: VideoSource,
    width: f32,
    height: f32,
    fit: ImageFit,
    radius: f32,
    dark: bool,
    focused: bool,
    accent: Color,
    autoplay: bool,
    player: mediakit::VideoPlayer,
    state: VideoState,
    rx: Option<Receiver<Result<PathBuf, String>>>,
    temp_file: Option<PathBuf>,
    duration: f64,
    speed: f32,
    fullscreen: bool,
    seq: u64,
    last_pts: Option<f64>,
    force_decode: bool,
    rgba: Vec<u8>,
    fw: u32,
    fh: u32,
    last_tick: Option<Instant>,
    play_btn: Button,
    pause_btn: Button,
    stop_btn: Button,
    mute_btn: Button,
    unmute_btn: Button,
    fs_btn: Button,
    toggle_play: Rc<Cell<bool>>,
    req_stop: Rc<Cell<bool>>,
    req_mute: Rc<Cell<bool>>,
    req_fs: Rc<Cell<bool>>,
    scrub: Slider,
    seek_req: Rc<Cell<Option<f64>>>,
    spinner: Spinner,
    text_color: Color,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl VideoPlayer {
    pub fn new(source: VideoSource, width: f32, height: f32) -> Self {
        let toggle_play = Rc::new(Cell::new(false));
        let req_stop = Rc::new(Cell::new(false));
        let req_mute = Rc::new(Cell::new(false));
        let req_fs = Rc::new(Cell::new(false));
        let seek_req: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));
        let mk_button = |icon: &str, flag: &Rc<Cell<bool>>| {
            let f = flag.clone();
            Button::new("")
                .icon(icon)
                .on_press(move || f.set(true))
        };
        let seek_clone = seek_req.clone();
        let scrub = Slider::new(0.0, 0.0, 1.0).on_change(move |v| seek_clone.set(Some(v)));
        let mut this = Self {
            source,
            width: width.max(0.0),
            height: height.max(0.0),
            fit: ImageFit::Cover,
            radius: VIDEO_RADIUS,
            dark: true,
            focused: true,
            accent: Color::from_rgb8(0x0a, 0x84, 0xff),
            autoplay: false,
            player: mediakit::VideoPlayer::new(),
            state: VideoState::Loading,
            rx: None,
            temp_file: None,
            duration: 0.0,
            speed: 1.0,
            fullscreen: false,
            seq: 0,
            last_pts: None,
            force_decode: true,
            rgba: Vec::new(),
            fw: 0,
            fh: 0,
            last_tick: None,
            play_btn: mk_button("play.fill", &toggle_play),
            pause_btn: mk_button("pause.fill", &toggle_play),
            stop_btn: mk_button("stop.fill", &req_stop),
            mute_btn: mk_button("speaker.fill", &req_mute),
            unmute_btn: mk_button("speaker.slash.fill", &req_mute),
            fs_btn: mk_button("arrow.up.left.and.arrow.down.right", &req_fs),
            toggle_play,
            req_stop,
            req_mute,
            req_fs,
            scrub,
            seek_req,
            spinner: Spinner::new(),
            text_color: Color::from_rgb8(0xd8, 0xd9, 0xd9),
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        };
        this.apply_theme();
        match &this.source {
            VideoSource::File(path) => this.open_file(path.clone()),
            VideoSource::Url(url) => this.start_download(url.clone()),
        }
        this
    }

    /// Disk file player.
    pub fn file(path: impl Into<PathBuf>, width: f32, height: f32) -> Self {
        Self::new(VideoSource::File(path.into()), width, height)
    }

    /// Progressive `http(s)` URL player (downloaded to temp first).
    pub fn url(url: impl Into<String>, width: f32, height: f32) -> Self {
        Self::new(VideoSource::Url(url.into()), width, height)
    }

    /// How the frame fills the viewport: cover (crop, default) or fit
    /// (letterbox).
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Corner radius in logical px (clamped to >= 0).
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = px.max(0.0);
        self
    }

    /// Start playing as soon as the media is ready.
    pub fn autoplay(mut self, autoplay: bool) -> Self {
        self.autoplay = autoplay;
        if autoplay && self.state == VideoState::Ready {
            let _ = self.player.play();
        }
        self
    }

    /// Playback speed multiplier (MediaKit range 0.5-2.0).
    pub fn set_speed(&mut self, speed: f32) -> Result<(), mediakit::MediaError> {
        self.player.set_speed(speed)?;
        self.speed = self.player.speed();
        Ok(())
    }

    /// Current speed multiplier.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// UI volume 0.0-1.0 (stored on the player for the app mixer).
    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
    }

    /// Mute flag (stored on the player for the app mixer).
    pub fn set_muted(&mut self, muted: bool) {
        self.player.set_muted(muted);
    }

    /// Live theme for placeholder, spinner and time label.
    pub fn set_theme(&mut self, dark: bool) {
        self.dark = dark;
        self.apply_theme();
    }

    /// Accent for the scrubber.
    pub fn set_accent(&mut self, accent: Color) {
        self.accent = accent;
        self.apply_theme();
    }

    pub fn set_focused_state(&mut self, focused: bool) {
        self.focused = focused;
        self.play_btn.set_focused(focused);
        self.pause_btn.set_focused(focused);
        self.stop_btn.set_focused(focused);
        self.mute_btn.set_focused(focused);
        self.unmute_btn.set_focused(focused);
        self.fs_btn.set_focused(focused);
        self.scrub.set_focused(focused);
    }

    fn apply_theme(&mut self) {
        self.text_color = if self.dark {
            Color::from_rgb8(0xd8, 0xd9, 0xd9)
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        };
        self.scrub.set_theme(self.accent, self.dark, GlassAmount::Glass);
    }

    fn open_file(&mut self, path: PathBuf) {
        match self.player.open(&path) {
            Ok(()) => {
                self.duration = self.player.duration_secs();
                self.rebuild_scrubber();
                self.state = VideoState::Ready;
                self.force_decode = true;
                if self.autoplay {
                    let _ = self.player.play();
                }
            }
            Err(e) => {
                self.state = VideoState::Failed(format!(
                    "{}: {e}",
                    path.display()
                ));
            }
        }
    }

    fn start_download(&mut self, url: String) {
        if let Some(problem) = VideoSource::url_problem(&url) {
            self.state = VideoState::Failed(problem);
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let result = match networkkit::http::HttpRequest::get(&url).send() {
                Ok(resp) if resp.is_success() => {
                    let ext = url
                        .split(['?', '#'])
                        .next()
                        .unwrap_or("")
                        .rsplit('.')
                        .next()
                        .unwrap_or("mov")
                        .to_ascii_lowercase();
                    let ext = if ["mov", "mp4", "m4v", "mkv", "webm", "avi"]
                        .contains(&ext.as_str())
                    {
                        ext
                    } else {
                        "mov".to_string()
                    };
                    let path = std::env::temp_dir().join(format!(
                        "tontooui-video-{}-{}.{}",
                        std::process::id(),
                        VideoPlayer::download_seq(),
                        ext
                    ));
                    match std::fs::write(&path, &resp.body) {
                        Ok(()) => Ok(path),
                        Err(e) => Err(format!("temp write failed: {e}")),
                    }
                }
                Ok(resp) => Err(format!("HTTP {}", resp.status)),
                Err(_) => Err("download failed".to_string()),
            };
            let _ = tx.send(result);
        });
    }

    fn download_seq() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_add(1, Ordering::SeqCst)
    }

    fn poll_download(&mut self) {
        let done = match &mut self.rx {
            Some(rx) => match rx.try_recv() {
                Ok(Ok(path)) => {
                    self.temp_file = Some(path.clone());
                    self.open_file(path);
                    true
                }
                Ok(Err(msg)) => {
                    self.state = VideoState::Failed(msg);
                    true
                }
                Err(_) => false,
            },
            None => false,
        };
        if done {
            self.rx = None;
        }
    }

    fn rebuild_scrubber(&mut self) {
        let max = self.duration.max(0.0);
        let seek_clone = self.seek_req.clone();
        self.scrub = Slider::new(0.0, 0.0, max.max(1.0))
            .on_change(move |v| seek_clone.set(Some(v)));
        self.scrub.set_theme(self.accent, self.dark, GlassAmount::Glass);
        self.scrub.set_focused(self.focused);
    }

    /// Advances the clock; injectable time keeps tests deterministic
    /// (mirrors `LinearProgress::advance`).
    pub fn advance(&mut self, now: Instant) {
        let dt = match self.last_tick {
            Some(last) => now.saturating_duration_since(last),
            None => std::time::Duration::ZERO,
        };
        self.last_tick = Some(now);
        if self.state == VideoState::Ready
            && self.player.state() == mediakit::PlaybackState::Playing
        {
            self.player.tick(dt.as_secs_f64());
        }
    }

    fn poll_requests(&mut self) {
        if self.toggle_play.take() {
            match self.player.state() {
                mediakit::PlaybackState::Playing => {
                    let _ = self.player.pause();
                }
                mediakit::PlaybackState::Finished => {
                    let _ = self.player.seek(0.0);
                    let _ = self.player.play();
                    self.force_decode = true;
                }
                _ => {
                    let _ = self.player.play();
                }
            }
        }
        if self.req_stop.take() {
            let _ = self.player.stop();
            self.force_decode = true;
        }
        if self.req_mute.take() {
            let muted = self.player.is_muted();
            self.player.set_muted(!muted);
        }
        if self.req_fs.take() {
            self.fullscreen = !self.fullscreen;
        }
        if let Some(pos) = self.seek_req.take() {
            if self.player.seek(pos).is_ok() {
                self.force_decode = true;
            }
        }
    }

    fn ensure_frame(&mut self) {
        if self.state != VideoState::Ready {
            return;
        }
        let pts = self.player.position_secs();
        let playing = self.player.state() == mediakit::PlaybackState::Playing;
        if !playing && !self.force_decode {
            if let Some(last) = self.last_pts {
                if (last - pts).abs() < f64::EPSILON {
                    return;
                }
            }
        }
        match self.player.frame_at(pts) {
            Ok(frame) => {
                self.rgba = rgba_from_rgb24(&frame.rgb);
                self.fw = frame.width;
                self.fh = frame.height;
                self.seq = self.seq.wrapping_add(1);
                self.last_pts = Some(pts);
                self.force_decode = false;
            }
            Err(e) => {
                let _ = self.player.pause();
                self.state = VideoState::Failed(format!("decode: {e}"));
            }
        }
    }

    /// Transport state for app chrome.
    pub fn position(&self) -> f64 {
        self.player.position_secs()
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn is_playing(&self) -> bool {
        self.player.state() == mediakit::PlaybackState::Playing
    }

    pub fn is_finished(&self) -> bool {
        self.player.state() == mediakit::PlaybackState::Finished
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn state(&self) -> &VideoState {
        &self.state
    }

    pub fn source(&self) -> &VideoSource {
        &self.source
    }

    pub fn play(&mut self) {
        let _ = self.player.play();
    }

    pub fn pause(&mut self) {
        let _ = self.player.pause();
    }

    pub fn stop(&mut self) {
        let _ = self.player.stop();
        self.force_decode = true;
    }

    pub fn seek(&mut self, position_secs: f64) {
        if self.player.seek(position_secs).is_ok() {
            self.force_decode = true;
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    /// App-level mouse motion (scrubber drag); forward from the shell.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.scrub.mouse_move(x, y);
        self.play_btn.set_hover(x as f32, y as f32);
        self.pause_btn.set_hover(x as f32, y as f32);
        self.stop_btn.set_hover(x as f32, y as f32);
        self.mute_btn.set_hover(x as f32, y as f32);
        self.unmute_btn.set_hover(x as f32, y as f32);
        self.fs_btn.set_hover(x as f32, y as f32);
    }

    fn bar_y(&self) -> f32 {
        self.y + self.placed_h - VIDEO_BAR_H
    }

    fn draw_time(&self, scene: &mut Scene, fonts: &mut FontSystem, x: f32, y: f32) {
        let text = format!("{} / {}", format_time(self.position()), format_time(self.duration));
        let layout = fonts.layout_text(&text, VIDEO_TIME_SIZE, self.text_color, None);
        draw_layout(scene, &layout, x, y, fonts.scale);
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        if let Some(path) = &self.temp_file {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl View for VideoPlayer {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.width, self.height)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        let bar_y = self.bar_y();
        let mut cx = x + VIDEO_GAP;
        let btn_y = bar_y + (VIDEO_BAR_H - VIDEO_BTN) / 2.0;
        self.play_btn.place(_fonts, cx, btn_y, VIDEO_BTN, VIDEO_BTN);
        self.pause_btn.place(_fonts, cx, btn_y, VIDEO_BTN, VIDEO_BTN);
        cx += VIDEO_BTN + VIDEO_GAP;
        self.stop_btn.place(_fonts, cx, btn_y, VIDEO_BTN, VIDEO_BTN);
        cx += VIDEO_BTN + VIDEO_GAP;
        // Time label slot, then the scrubber takes the rest.
        cx += VIDEO_TIME_W + VIDEO_GAP;
        let right = x + w - VIDEO_GAP;
        let fs_x = right - VIDEO_BTN;
        let mute_x = fs_x - VIDEO_GAP - VIDEO_BTN;
        self.fs_btn.place(_fonts, fs_x, btn_y, VIDEO_BTN, VIDEO_BTN);
        if self.player.is_muted() {
            self.unmute_btn.place(_fonts, mute_x, btn_y, VIDEO_BTN, VIDEO_BTN);
            self.mute_btn.place(_fonts, -100.0, -100.0, 0.0, 0.0);
        } else {
            self.mute_btn.place(_fonts, mute_x, btn_y, VIDEO_BTN, VIDEO_BTN);
            self.unmute_btn.place(_fonts, -100.0, -100.0, 0.0, 0.0);
        }
        let scrub_x = cx;
        let scrub_w = (mute_x - VIDEO_GAP - scrub_x).max(0.0);
        self.scrub
            .place(_fonts, scrub_x, bar_y, scrub_w, VIDEO_BAR_H);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        self.poll_download();
        self.poll_requests();
        self.advance(Instant::now());
        // Keep the scrubber in sync unless the user drags it.
        if !self.scrub.is_dragging() {
            if let Some(target) =
                should_resync_slider(self.scrub.value(), self.position(), self.duration)
            {
                self.scrub.set_value(target);
            }
        }
        self.ensure_frame();
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let video_h = (self.placed_h - VIDEO_BAR_H).max(0.0);
        let radius = self
            .radius
            .min(self.placed_w / 2.0)
            .min(video_h / 2.0)
            .max(0.0);
        let frame = RoundedRect::new(px(self.x), px(self.y), px(self.x + self.placed_w), px(self.y + video_h), px(radius));
        // Small drop shadow under the frame.
        scene.draw_blurred_rounded_rect(
            Affine::translate((0.0, VIDEO_SHADOW_DY as f64 * scale)),
            Rect::new(px(self.x), px(self.y), px(self.x + self.placed_w), px(self.y + video_h)),
            VIDEO_SHADOW,
            px(radius),
            VIDEO_SHADOW_BLUR as f64 * scale,
        );
        match &self.state {
            VideoState::Loading => {
                let fill = placeholder_fill(self.dark);
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fill), None, &frame);
                let s = 28.0f32;
                self.spinner.place(
                    fonts,
                    self.x + (self.placed_w - s) / 2.0,
                    self.y + (video_h - s) / 2.0,
                    s,
                    s,
                );
                self.spinner.draw(scene, fonts, images);
            }
            VideoState::Failed(msg) => {
                let fill = placeholder_fill(self.dark);
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fill), None, &frame);
                let msg = msg.clone();
                let layout = fonts.layout_text(&msg, 13.0, self.text_color, Some(self.placed_w - 32.0));
                let (tw, th) = FontSystem::layout_size(&layout);
                draw_layout(
                    scene,
                    &layout,
                    self.x + (self.placed_w - tw / fonts.scale) / 2.0,
                    self.y + (video_h - th / fonts.scale) / 2.0,
                    fonts.scale,
                );
            }
            VideoState::Ready => {
                if self.fw > 0 && self.fh > 0 && !self.rgba.is_empty() {
                    if let Some((image, iw, ih)) =
                        images.upload_frame(self.seq, &self.rgba, self.fw, self.fh)
                    {
                        let (dw, _dh, dx, dy) = fit_rect(
                            iw as f32,
                            ih as f32,
                            self.placed_w,
                            video_h,
                            self.fit,
                        );
                        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &frame);
                        let s = (dw / iw as f32) as f64 * scale;
                        let base = Affine::translate((
                            (self.x + dx) as f64 * scale,
                            (self.y + dy) as f64 * scale,
                        )) * Affine::scale(s);
                        scene.draw_image(&image, base);
                        scene.pop_layer();
                    }
                } else {
                    let fill = placeholder_fill(self.dark);
                    scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fill), None, &frame);
                }
            }
        }
        // Transport bar: play/pause, stop, time, scrubber, mute, fullscreen.
        let playing = self.is_playing();
        if playing {
            self.pause_btn.draw(scene, fonts, images);
        } else {
            self.play_btn.draw(scene, fonts, images);
        }
        self.stop_btn.draw(scene, fonts, images);
        self.draw_time(
            scene,
            fonts,
            self.x + VIDEO_GAP * 2.0 + VIDEO_BTN * 2.0,
            self.bar_y() + (VIDEO_BAR_H - 14.0) / 2.0,
        );
        self.scrub.draw(scene, fonts, images);
        if self.player.is_muted() {
            self.unmute_btn.draw(scene, fonts, images);
        } else {
            self.mute_btn.draw(scene, fonts, images);
        }
        self.fs_btn.draw(scene, fonts, images);
        let _ = self.focused;
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        if self.is_playing() {
            self.pause_btn.mouse_down(x, y);
        } else {
            self.play_btn.mouse_down(x, y);
        }
        self.stop_btn.mouse_down(x, y);
        if self.player.is_muted() {
            self.unmute_btn.mouse_down(x, y);
        } else {
            self.mute_btn.mouse_down(x, y);
        }
        self.fs_btn.mouse_down(x, y);
        self.scrub.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.is_playing() {
            self.pause_btn.mouse_up(x, y);
        } else {
            self.play_btn.mouse_up(x, y);
        }
        self.stop_btn.mouse_up(x, y);
        if self.player.is_muted() {
            self.unmute_btn.mouse_up(x, y);
        } else {
            self.mute_btn.mouse_up(x, y);
        }
        self.fs_btn.mouse_up(x, y);
        self.scrub.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.mouse_move(x as f64, y as f64);
    }

    fn set_focused(&mut self, focused: bool) {
        self.set_focused_state(focused);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn placeholder_fill(dark: bool) -> Color {
    if dark {
        Color::from_rgb8(0x2c, 0x2c, 0x2e)
    } else {
        Color::from_rgb8(0xe5, 0xe5, 0xe5)
    }
}

/// `m:ss` timecode.
pub fn format_time(secs: f64) -> String {
    let total = secs.max(0.0).floor() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

/// RGB24 to opaque RGBA8 for the streaming texture cache.
pub fn rgba_from_rgb24(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for px in rgb.chunks_exact(3) {
        out.extend_from_slice(&[px[0], px[1], px[2], 255]);
    }
    out
}

/// Scrubber resync target: while the user drags, hands off;
/// otherwise follow the transport position.
pub fn should_resync_slider(value: f64, position: f64, duration: f64) -> Option<f64> {
    let _ = duration;
    if (value - position).abs() > 1e-6 {
        Some(position)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn timecode_formats() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(65.7), "1:05");
        assert_eq!(format_time(-3.0), "0:00");
        assert_eq!(format_time(600.0), "10:00");
    }

    #[test]
    fn rgba_expands_opaque() {
        assert_eq!(
            rgba_from_rgb24(&[10, 20, 30, 40, 50, 60]),
            vec![10, 20, 30, 255, 40, 50, 60, 255]
        );
        assert!(rgba_from_rgb24(&[]).is_empty());
    }

    #[test]
    fn resync_follows_position() {
        assert_eq!(should_resync_slider(0.0, 5.0, 60.0), Some(5.0));
        assert_eq!(should_resync_slider(5.0, 5.0, 60.0), None);
    }

    #[test]
    fn sources_validate() {
        assert!(VideoSource::url_problem("https://cdn/x/clip.mp4").is_none());
        assert!(VideoSource::url_problem("ftp://x/y").is_some());
        assert!(VideoSource::url_problem("https://cdn/x/s.m3u8").is_some());
        assert_eq!(
            VideoSource::Url("https://cdn/x/clip.mp4".into()).display_name(),
            "clip.mp4"
        );
    }

    #[test]
    fn missing_file_fails_cleanly() {
        let player = VideoPlayer::file("/definitely/missing/clip.mov", 320.0, 240.0);
        assert!(matches!(player.state(), VideoState::Failed(_)));
        assert_eq!(player.duration(), 0.0);
        assert!(!player.is_playing());
    }

    #[test]
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut player =
            VideoPlayer::file("/definitely/missing/clip.mov", 320.0, 240.0);
        assert_eq!(player.measure(&mut fonts), (320.0, 240.0));
        player.place(&mut fonts, 10.0, 20.0, 320.0, 240.0);
        assert_eq!(player.rect(), (10.0, 20.0, 320.0, 240.0));
    }

    #[test]
    fn transport_api_without_media() {
        let mut player = VideoPlayer::file("/definitely/missing/clip.mov", 320.0, 240.0);
        player.play();
        assert!(!player.is_playing());
        player.pause();
        player.stop();
        player.seek(5.0);
        assert!(player.set_speed(1.5).is_ok());
        assert!((player.speed() - 1.5).abs() < 1e-6);
        assert!(player.set_speed(9.0).is_err());
        player.set_volume(0.5);
        player.set_muted(true);
        assert!(!player.is_fullscreen());
        player.toggle_fullscreen();
        assert!(player.is_fullscreen());
    }

    #[test]
    fn real_file_plays_frames() {
        use mediakit::{RawMovParams, build_raw_mov};
        let params = RawMovParams {
            width: 8,
            height: 4,
            fps: 10,
            audio: None,
        };
        let body: Vec<Vec<u8>> = (0..10).map(|f| vec![f as u8; 8 * 4 * 3]).collect();
        let file = build_raw_mov(&params, &body).unwrap();
        let dir = std::env::temp_dir();
        let path = dir.join(format!("tontooui-video-{}.mov", std::process::id()));
        std::fs::write(&path, &file).unwrap();
        let mut player = VideoPlayer::file(&path, 160.0, 120.0);
        assert!(matches!(player.state(), VideoState::Ready));
        assert!((player.duration() - 1.0).abs() < 0.001);
        player.play();
        assert!(player.is_playing());
        let t0 = Instant::now();
        player.advance(t0);
        player.advance(t0 + std::time::Duration::from_millis(250));
        assert!((player.position() - 0.25).abs() < 0.02);
        player.pause();
        assert!(!player.is_playing());
        let _ = std::fs::remove_file(&path);
    }
}
