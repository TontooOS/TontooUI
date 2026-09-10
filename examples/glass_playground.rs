//! TontooUI Glass Playground — tune real glass material properties live.
//!
//! Left: a round, elongated glass menu bar with app buttons, composited over
//! a photo background (`examples/assets/glass_bg.jpg`, Unsplash photo Wocy2asXI7k; the built-in
//! procedural underwater scene is used as fallback when the asset is
//! missing). Right: 16 sliders and 8 toggles that control how the glass
//! itself behaves — transparency, tint, blur sigma, refraction, depth,
//! dispersion, saturation, brightness, contrast, specular light, rim light,
//! frost grain and backdrop dim. Every change re-composites the glass from
//! the backdrop instantly.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use gtk::prelude::*;
use image::{Rgb, RgbImage, RgbaImage};
use tontooui::prelude::*;
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

// ── Geometry ─────────────────────────────────────────────────────────

const BG_W: u32 = 620;
const BG_H: u32 = 760;
const GW: u32 = 320;
const GH: u32 = 68;

// ── Parameters: the library material, shared with the live controls ───
// (GlassParams is tontooui::GlassMaterial; same fields.)

thread_local! {
    static PARAMS: RefCell<GlassMaterial> = RefCell::new(GlassMaterial::default());
    static BG: RefCell<Option<RgbImage>> = const { RefCell::new(None) };
    static LIVE: RefCell<Option<LiveHandles>> = RefCell::new(None);
    static DIRTY: RefCell<bool> = const { RefCell::new(false) };
    static FLUSH_PENDING: RefCell<bool> = const { RefCell::new(false) };
    static PERF: RefCell<PerfStats> = RefCell::new(PerfStats::new());
    static PERF_UI: RefCell<Option<PerfUi>> = RefCell::new(None);
}

struct LiveHandles {
    pic: gtk::Picture,
}

struct PerfStats {
    history: VecDeque<f32>,
    last_total: f32,
    last_blur: f32,
    last_paint: f32,
    rate: f32,
    window_start: Instant,
    window_count: u32,
}

impl PerfStats {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(96),
            last_total: 0.0,
            last_blur: 0.0,
            last_paint: 0.0,
            rate: 0.0,
            window_start: Instant::now(),
            window_count: 0,
        }
    }
    fn record(&mut self, total_ms: f32, blur_ms: f32, paint_ms: f32) {
        self.last_total = total_ms;
        self.last_blur = blur_ms;
        self.last_paint = paint_ms;
        if self.history.len() >= 90 {
            self.history.pop_front();
        }
        self.history.push_back(total_ms);
        self.window_count += 1;
        let elapsed = self.window_start.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            self.rate = self.window_count as f32 / elapsed;
            self.window_count = 0;
            self.window_start = Instant::now();
        }
    }
}

struct PerfUi {
    total: gtk::Label,
    blur: gtk::Label,
    paint: gtk::Label,
    rate: gtk::Label,
    graph: gtk::DrawingArea,
}

fn bg_path() -> std::path::PathBuf {
    std::env::temp_dir().join("tontooui_glass_bg.png")
}

// (Compositing lives in the library: tontooui::render_glass.)

// ── Glass compositor (library) ───────────────────────────────────────
// Thin shim so the playground's flush path stays untouched.
fn composite(p: &GlassMaterial, bg: &RgbImage) -> (RgbaImage, f32) {
    render_glass(p, bg, GW, GH, GH as f32 / 2.0)
}

/// Mark the preview dirty and ensure a flush is scheduled. Coalesces fast
/// slider drags to at most one composite per 33ms instead of one PNG
/// encode + upload per tick.
fn request_refresh() {
    DIRTY.with(|d| *d.borrow_mut() = true);
    let schedule = FLUSH_PENDING.with(|p| {
        if *p.borrow() {
            false
        } else {
            *p.borrow_mut() = true;
            true
        }
    });
    if schedule {
        gtk::glib::timeout_add_local(Duration::from_millis(33), || {
            FLUSH_PENDING.with(|p| *p.borrow_mut() = false);
            flush_live();
            gtk::glib::ControlFlow::Break
        });
    }
}

fn flush_live() {
    let dirty = DIRTY.with(|d| d.replace(false));
    if !dirty {
        return;
    }
    let t0 = Instant::now();
    let p = PARAMS.with(|c| c.borrow().clone());
    // Borrow (no clone) the backdrop: composite never touches thread-locals.
    let (raw, blur_ms) = BG.with(|b| {
        let bg = b.borrow();
        let bg = bg.as_ref().expect("background built in main");
        let (img, blur_ms) = composite(&p, bg);
        (img.into_raw(), blur_ms)
    });
    let t1 = Instant::now();
    // Zero-copy upload: raw RGB straight into a GPU texture, no PNG
    // encode and no disk round-trip like the previous set_filename path.
    let bytes = gtk::glib::Bytes::from_owned(raw);
    let tex = gtk::gdk::MemoryTexture::new(
        GW as i32,
        GH as i32,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (GW * 4) as usize,
    );
    LIVE.with(|l| {
        if let Some(st) = l.borrow().as_ref() {
            st.pic.set_paintable(Some(&tex));
        }
    });
    let t2 = Instant::now();
    let total_ms = (t2 - t0).as_secs_f32() * 1000.0;
    let paint_ms = (t2 - t1).as_secs_f32() * 1000.0;
    PERF.with(|pf| pf.borrow_mut().record(total_ms, blur_ms, paint_ms));
    update_perf_ui();
}

fn update_perf_ui() {
    let (total, blur, paint, rate) = PERF.with(|p| {
        let s = p.borrow();
        (s.last_total, s.last_blur, s.last_paint, s.rate)
    });
    PERF_UI.with(|u| {
        if let Some(ui) = u.borrow().as_ref() {
            ui.total.set_text(&format!("{:.1} ms", total));
            ui.blur.set_text(&format!("blur {:.1}", blur));
            ui.paint.set_text(&format!("up {:.1}", paint));
            ui.rate.set_text(&format!("{:.0}/s", rate));
            ui.graph.queue_draw();
        }
    });
}

// ── Control setters (plain fns so closures stay Send + Sync) ─────────

fn set_tint_r(v: f32) { PARAMS.with(|c| c.borrow_mut().tint_r = v); request_refresh(); }
fn set_tint_g(v: f32) { PARAMS.with(|c| c.borrow_mut().tint_g = v); request_refresh(); }
fn set_tint_b(v: f32) { PARAMS.with(|c| c.borrow_mut().tint_b = v); request_refresh(); }
fn set_tint_a(v: f32) { PARAMS.with(|c| c.borrow_mut().tint_a = v); request_refresh(); }
fn set_sigma(v: f32) { PARAMS.with(|c| c.borrow_mut().sigma = v); request_refresh(); }
fn set_refraction(v: f32) { PARAMS.with(|c| c.borrow_mut().refraction = v); request_refresh(); }
fn set_depth(v: f32) { PARAMS.with(|c| c.borrow_mut().depth = v); request_refresh(); }
fn set_dispersion(v: f32) { PARAMS.with(|c| c.borrow_mut().dispersion = v); request_refresh(); }
fn set_saturation(v: f32) { PARAMS.with(|c| c.borrow_mut().saturation = v); request_refresh(); }
fn set_brightness(v: f32) { PARAMS.with(|c| c.borrow_mut().brightness = v); request_refresh(); }
fn set_contrast(v: f32) { PARAMS.with(|c| c.borrow_mut().contrast = v); request_refresh(); }
fn set_specular(v: f32) { PARAMS.with(|c| c.borrow_mut().specular = v); request_refresh(); }
fn set_spec_angle(v: f32) { PARAMS.with(|c| c.borrow_mut().spec_angle = v); request_refresh(); }
fn set_rim(v: f32) { PARAMS.with(|c| c.borrow_mut().rim = v); request_refresh(); }
fn set_grain(v: f32) { PARAMS.with(|c| c.borrow_mut().grain = v); request_refresh(); }
fn set_dim(v: f32) { PARAMS.with(|c| c.borrow_mut().dim = v); request_refresh(); }

fn set_blur_on(on: bool) { PARAMS.with(|c| c.borrow_mut().blur_on = on); request_refresh(); }
fn set_refraction_on(on: bool) { PARAMS.with(|c| c.borrow_mut().refraction_on = on); request_refresh(); }
fn set_dispersion_on(on: bool) { PARAMS.with(|c| c.borrow_mut().dispersion_on = on); request_refresh(); }
fn set_grain_on(on: bool) { PARAMS.with(|c| c.borrow_mut().grain_on = on); request_refresh(); }
fn set_specular_on(on: bool) { PARAMS.with(|c| c.borrow_mut().specular_on = on); request_refresh(); }
fn set_rim_on(on: bool) { PARAMS.with(|c| c.borrow_mut().rim_on = on); request_refresh(); }
fn set_tint_on(on: bool) { PARAMS.with(|c| c.borrow_mut().tint_on = on); request_refresh(); }
fn set_dim_on(on: bool) { PARAMS.with(|c| c.borrow_mut().dim_on = on); request_refresh(); }

fn preset_frost() {
    PARAMS.with(|c| *c.borrow_mut() = GlassMaterial::default());
    request_refresh();
}

fn preset_clear() {
    PARAMS.with(|c| {
        let mut p = GlassMaterial::default();
        p.tint_a = 8.0;
        p.sigma = 2.5;
        p.refraction = 18.0;
        p.saturation = 105.0;
        p.brightness = 102.0;
        p.specular = 70.0;
        p.grain = 6.0;
        p.dispersion = 8.0;
        p.rim = 45.0;
        *c.borrow_mut() = p;
    });
    request_refresh();
}

fn preset_sunset() {
    PARAMS.with(|c| {
        let mut p = GlassMaterial::default();
        p.tint_r = 255.0;
        p.tint_g = 150.0;
        p.tint_b = 80.0;
        p.tint_a = 38.0;
        p.sigma = 12.0;
        p.refraction = 60.0;
        p.depth = 65.0;
        p.saturation = 135.0;
        p.brightness = 110.0;
        p.specular = 65.0;
        p.spec_angle = 120.0;
        p.rim = 70.0;
        p.grain = 26.0;
        p.dispersion = 26.0;
        p.dim = 25.0;
        p.dim_on = true;
        *c.borrow_mut() = p;
    });
    request_refresh();
}

// ── Image background (flat underwater scene, no gradients) ───────────

struct Rng(u64);
impl Rng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo).max(1) as u32;
        lo + (self.next_u32() % span) as i32
    }
}

fn put(img: &mut RgbImage, x: i32, y: i32, c: Rgb<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, c);
    }
}

fn blend(img: &mut RgbImage, x: i32, y: i32, src: [u8; 3], a: f32) {
    if x < 0 || y < 0 || (x as u32) >= img.width() || (y as u32) >= img.height() {
        return;
    }
    let dst = img.get_pixel(x as u32, y as u32).0;
    let mix = |d: u8, s: u8| (d as f32 * (1.0 - a) + s as f32 * a).round() as u8;
    img.put_pixel(x as u32, y as u32, Rgb([mix(dst[0], src[0]), mix(dst[1], src[1]), mix(dst[2], src[2])]));
}

fn fill_rect(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb<u8>) {
    for y in y0..y1 {
        for x in x0..x1 {
            put(img, x, y, c);
        }
    }
}

fn fill_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, c: Rgb<u8>) {
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                put(img, x, y, c);
            }
        }
    }
}

fn ring_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, src: [u8; 3], a: f32) {
    for y in (cy - r - 1)..=(cy + r + 1) {
        for x in (cx - r - 1)..=(cx + r + 1) {
            let d = (((x - cx).pow(2) + ((y - cy).pow(2))) as f32).sqrt();
            if (d - r as f32).abs() <= 1.0 {
                blend(img, x, y, src, a);
            }
        }
    }
}

fn fill_ellipse(img: &mut RgbImage, cx: i32, cy: i32, rx: i32, ry: i32, c: Rgb<u8>) {
    for y in (cy - ry)..=(cy + ry) {
        for x in (cx - rx)..=(cx + rx) {
            let dx = (x - cx) as f32 / rx.max(1) as f32;
            let dy = (y - cy) as f32 / ry.max(1) as f32;
            if dx * dx + dy * dy <= 1.0 {
                put(img, x, y, c);
            }
        }
    }
}

fn fill_triangle(img: &mut RgbImage, ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32, c: Rgb<u8>) {
    let min_x = ax.min(bx).min(cx);
    let max_x = ax.max(bx).max(cx);
    let min_y = ay.min(by).min(cy);
    let max_y = ay.max(by).max(cy);
    let sign = |x: i32, y: i32, px: i32, py: i32, qx: i32, qy: i32| (x - qx) * (py - qy) - (px - qx) * (y - qy);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let d1 = sign(x, y, ax, ay, bx, by);
            let d2 = sign(x, y, bx, by, cx, cy);
            let d3 = sign(x, y, cx, cy, ax, ay);
            let neg = d1 < 0 || d2 < 0 || d3 < 0;
            let pos = d1 > 0 || d2 > 0 || d3 > 0;
            if !(neg && pos) {
                put(img, x, y, c);
            }
        }
    }
}

fn fill_polygon(img: &mut RgbImage, pts: &[(i32, i32)], c: Rgb<u8>) {
    if pts.len() < 3 {
        return;
    }
    for i in 1..(pts.len() - 1) {
        fill_triangle(img, pts[0].0, pts[0].1, pts[i].0, pts[i].1, pts[i + 1].0, pts[i + 1].1, c);
    }
}

fn build_background() -> RgbImage {
    if let Some(photo) = load_photo_background() {
        let _ = photo.save(bg_path());
        return photo;
    }
    let img = procedural_background();
    let _ = img.save(bg_path());
    img
}

/// Load `examples/assets/glass_bg.jpg` (Unsplash photo Wocy2asXI7k) with a
/// cover fit onto the stage size. Returns `None` when the asset is missing
/// or unreadable so the caller can fall back to the procedural scene.
fn load_photo_background() -> Option<RgbImage> {
    let path = format!("{}/examples/assets/glass_bg.jpg", env!("CARGO_MANIFEST_DIR"));
    let img = image::open(&path).ok()?.to_rgb8();
    // Cover: scale so the stage rect is filled, then center-crop. A fast
    // thumbnail pass shrinks the multi-megapixel photo first so the exact
    // Triangle pass only runs on ~2x stage pixels.
    let thumb = image::imageops::thumbnail(&img, BG_W * 2, BG_H * 2);
    let (tw, th) = (thumb.width() as f32, thumb.height() as f32);
    let scale = (BG_W as f32 / tw).max(BG_H as f32 / th);
    let sw = (tw * scale).ceil() as u32;
    let sh = (th * scale).ceil() as u32;
    let scaled = image::imageops::resize(&thumb, sw, sh, image::imageops::FilterType::Triangle);
    let x = (sw - BG_W) / 2;
    let y = (sh - BG_H) / 2;
    Some(image::imageops::crop_imm(&scaled, x, y, BG_W, BG_H).to_image())
}

fn procedural_background() -> RgbImage {
    let w = BG_W;
    let h = BG_H;
    let mut img = RgbImage::new(w, h);
    let mut rng = Rng(0x9E3779B97F4A7C15);

    // Water bands (flat fills, lighter at the top).
    let top = [26u8, 112u8, 138u8];
    let bottom = [5u8, 17u8, 30u8];
    let bands = 12;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let jitter = rng.range(-3, 4) as f32;
        let mix = |a: u8, b: u8| ((a as f32 * (1.0 - t) + b as f32 * t + jitter).round().clamp(0.0, 255.0)) as u8;
        let c = Rgb([mix(top[0], bottom[0]), mix(top[1], bottom[1]), mix(top[2], bottom[2])]);
        fill_rect(&mut img, 0, (i * h / bands) as i32, w as i32, ((i + 1) * h / bands) as i32, c);
    }

    // Moon disc with thin ring.
    fill_circle(&mut img, 492, 108, 46, Rgb([244, 236, 216]));
    ring_circle(&mut img, 492, 108, 58, [240, 232, 210], 0.5);

    // Far + near rock silhouettes.
    fill_polygon(
        &mut img,
        &[(0, 566), (90, 502), (200, 548), (330, 496), (470, 552), (620, 508), (620, 760), (0, 760)],
        Rgb([10, 27, 39]),
    );
    fill_polygon(
        &mut img,
        &[(0, 660), (140, 606), (300, 652), (450, 610), (620, 656), (620, 760), (0, 760)],
        Rgb([6, 14, 23]),
    );

    // Seaweed blades (overlapping flat discs form each blade).
    for _ in 0..13 {
        let base_x = rng.range(10, w as i32 - 10);
        let height = rng.range(120, 260);
        let lean = rng.range(-26, 27) as f32 / height as f32;
        let green = if rng.range(0, 2) == 0 { [29u8, 107u8, 79u8] } else { [39u8, 147u8, 106u8] };
        let steps = 26;
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            let y = h as i32 - (t * height as f32) as i32;
            let x = base_x + (lean * t * height as f32) as i32 + ((t * 5.0).sin() * 8.0) as i32;
            let r = (7.0 * (1.0 - t) + 1.5) as i32;
            fill_circle(&mut img, x, y, r, Rgb(green));
        }
    }

    // Fish (flat ellipse body + tail triangle).
    let fish_cols = [[255u8, 140u8, 66u8], [255, 209, 102], [232, 241, 242]];
    for i in 0..9 {
        let cx = rng.range(50, w as i32 - 50);
        let cy = rng.range(90, 480);
        let rx = rng.range(12, 27);
        let ry = (rx as f32 * 0.38) as i32 + 2;
        let dir = if i % 2 == 0 { 1 } else { -1 };
        let c = Rgb(fish_cols[rng.range(0, 3) as usize]);
        fill_ellipse(&mut img, cx, cy, rx, ry, c);
        fill_triangle(&mut img, cx - dir * rx, cy, cx - dir * (rx + 12), cy - 8, cx - dir * (rx + 12), cy + 8, c);
        put(&mut img, cx + dir * (rx / 2), cy - 1, Rgb([10, 20, 28]));
    }

    // Bubbles + drifting motes.
    for _ in 0..34 {
        ring_circle(&mut img, rng.range(0, w as i32), rng.range(0, h as i32), rng.range(2, 9), [200, 230, 235], 0.55);
    }
    for _ in 0..220 {
        blend(&mut img, rng.range(0, w as i32), rng.range(0, h as i32), [190, 220, 226], 0.5);
    }

    // Film grain over everything.
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let n = rng.range(-7, 8);
            let dst = img.get_pixel(x as u32, y as u32).0;
            let add = |d: u8| ((d as i32 + n).clamp(0, 255)) as u8;
            img.put_pixel(x as u32, y as u32, Rgb([add(dst[0]), add(dst[1]), add(dst[2])]));
        }
    }

    img
}

// ── Stage: live-composited glass over the image ──────────────────────

struct GlassStage {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl GlassStage {
    fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
}

impl ViewContent for GlassStage {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(BG_W as i32, BG_H as i32);

        let bg_pic = gtk::Picture::for_filename(&bg_path());
        bg_pic.set_content_fit(gtk::ContentFit::Cover);
        bg_pic.set_can_shrink(true);
        bg_pic.set_hexpand(true);
        bg_pic.set_vexpand(true);
        overlay.set_child(Some(&bg_pic));

        // Live glass layer: a small picture exactly over the composited
        // rect, refreshed from `refresh()` on every control change.
        let glass_pic = gtk::Picture::new();
        glass_pic.set_size_request(GW as i32, GH as i32);
        glass_pic.set_can_shrink(false);
        glass_pic.set_halign(gtk::Align::Center);
        glass_pic.set_valign(gtk::Align::Center);
        overlay.add_overlay(&glass_pic);

        // Menu-bar content: icon buttons floating above the glass.
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.set_size_request(GW as i32, GH as i32);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Center);
        let spacer_top = gtk::Box::new(gtk::Orientation::Vertical, 0);
        spacer_top.set_vexpand(true);
        content.append(&spacer_top);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        let blue = Color::from_rgb(10, 132, 255);
        let back = Button::new("")
            .icon("chevron.left")
            .style(ButtonStyle::Plain)
            .tint(blue)
            .height(40.0)
            .on_click(|| println!("menu back"))
            .to_gtk();
        let fwd = Button::new("")
            .icon("chevron.right")
            .style(ButtonStyle::Plain)
            .tint(blue)
            .height(40.0)
            .on_click(|| println!("menu forward"))
            .to_gtk();
        let search = Button::new("")
            .icon("magnifyingglass")
            .style(ButtonStyle::Plain)
            .tint(blue)
            .height(40.0)
            .on_click(|| println!("menu search"))
            .to_gtk();
        let add = Button::new("")
            .icon("plus")
            .style(ButtonStyle::Plain)
            .tint(blue)
            .height(40.0)
            .on_click(|| println!("menu add"))
            .to_gtk();
        let settings = Button::new("")
            .icon("gearshape")
            .style(ButtonStyle::Plain)
            .tint(blue)
            .height(40.0)
            .on_click(|| println!("menu settings"))
            .to_gtk();
        row.append(&back);
        row.append(&fwd);
        row.append(&search);
        row.append(&add);
        row.append(&settings);
        content.append(&row);
        let spacer_bottom = gtk::Box::new(gtk::Orientation::Vertical, 0);
        spacer_bottom.set_vexpand(true);
        content.append(&spacer_bottom);
        overlay.add_overlay(&content);

        LIVE.with(|l| {
            *l.borrow_mut() = Some(LiveHandles { pic: glass_pic });
        });
        flush_live();

        overlay.upcast()
    }

    fn size_that_fits(&self, _: Size) -> Size {
        Size::new(BG_W as f32, BG_H as f32)
    }
}

impl Widget for GlassStage {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }
    fn position(&self) -> Position {
        self.position
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, BG_W as f32, BG_H as f32))
    }
    fn is_interactive(&self) -> bool {
        false
    }
    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

// ── Performance panel: live timings + frame-time graph ──────────────

struct PerfPanel {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl PerfPanel {
    fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
}

fn perf_value() -> gtk::Label {
    let l = gtk::Label::new(Some("--"));
    l.set_halign(gtk::Align::Start);
    uikit::widget::apply_css(
        &l,
        "label { color: white; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; }",
    );
    l
}

fn perf_caption(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_halign(gtk::Align::Start);
    uikit::widget::apply_css(
        &l,
        "label { color: rgba(255,255,255,0.55); font-family: 'SF Pro Display'; font-size: 9px; }",
    );
    l
}

impl ViewContent for PerfPanel {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
        root.set_size_request(580, 118);

        let stats = gtk::Box::new(gtk::Orientation::Horizontal, 18);
        let total = perf_value();
        let blur = perf_value();
        let paint = perf_value();
        let rate = perf_value();
        for (cap, val) in [("TOTAL", &total), ("BLUR", &blur), ("UPLOAD", &paint), ("RATE", &rate)] {
            let cell = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cell.append(&perf_caption(cap));
            cell.append(val);
            stats.append(&cell);
        }
        root.append(&stats);

        // Frame-time history: last 90 composites, green < 17ms,
        // yellow < 33ms, red above, white line marks the 33ms budget.
        let graph = gtk::DrawingArea::new();
        graph.set_size_request(580, 64);
        graph.set_draw_func(|_area, cr, w, h| {
            PERF.with(|p| {
                let pf = p.borrow();
                cr.set_source_rgb(0.07, 0.07, 0.08);
                let _ = cr.paint();
                let w = w as f64;
                let h = h as f64;
                let max_ms = 50.0;
                let n = pf.history.len().min(90);
                if n > 0 {
                    let bw = w / 90.0;
                    for (i, v) in pf.history.iter().skip(pf.history.len() - n).enumerate() {
                        let frac = (*v as f64 / max_ms).clamp(0.02, 1.0);
                        let bh = frac * h;
                        if *v < 17.0 {
                            cr.set_source_rgb(0.20, 0.85, 0.40);
                        } else if *v < 33.0 {
                            cr.set_source_rgb(1.00, 0.80, 0.20);
                        } else {
                            cr.set_source_rgb(1.00, 0.30, 0.25);
                        }
                        cr.rectangle(i as f64 * bw, h - bh, (bw - 1.0).max(1.0), bh);
                        let _ = cr.fill();
                    }
                }
                let line_y = h - (33.0 / max_ms) * h;
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.45);
                cr.set_line_width(1.0);
                cr.move_to(0.0, line_y);
                cr.line_to(w, line_y);
                let _ = cr.stroke();
            });
        });
        root.append(&graph);

        PERF_UI.with(|u| {
            *u.borrow_mut() = Some(PerfUi { total, blur, paint, rate, graph });
        });
        update_perf_ui();

        root.upcast()
    }

    fn size_that_fits(&self, _: Size) -> Size {
        Size::new(580.0, 118.0)
    }
}

impl Widget for PerfPanel {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }
    fn position(&self) -> Position {
        self.position
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, 580.0, 118.0))
    }
    fn is_interactive(&self) -> bool {
        false
    }
    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

// ── Controls ─────────────────────────────────────────────────────────

fn slider_row(label: &str, min: f32, max: f32, val: f32, step: f32, f: fn(f32)) -> Slider {
    Slider::new(min, max).value(val).step(step).label(label).width(232.0).on_change(move |v| f(v))
}

fn toggle_row(label: &str, val: bool, f: fn(bool)) -> Toggle {
    Toggle::new(label).value(val).width(232.0).on_change(move |on| f(on))
}

fn group_title(text: &str) -> Text {
    Text::new(text).font_size(13.0).bold()
}

fn main() {
    BG.with(|b| *b.borrow_mut() = Some(build_background()));

    // Headless benchmark: composite timing without opening a window.
    if std::env::args().any(|a| a == "--bench") {
        let bg = BG.with(|b| b.borrow().clone().unwrap());
        let p = GlassMaterial::default();
        let _ = composite(&p, &bg);
        let reps = 20;
        let t = Instant::now();
        let mut blur_sum = 0.0;
        for _ in 0..reps {
            let (_, b) = composite(&p, &bg);
            blur_sum += b;
        }
        println!(
            "default composite avg {:.1} ms (blur avg {:.1} ms) over {} runs",
            t.elapsed().as_secs_f32() * 1000.0 / reps as f32,
            blur_sum / reps as f32,
            reps
        );
        let mut worst = GlassMaterial::default();
        worst.sigma = 25.0;
        let t = Instant::now();
        for _ in 0..10 {
            let _ = composite(&worst, &bg);
        }
        println!("sigma=25 composite avg {:.1} ms over 10 runs", t.elapsed().as_secs_f32() * 100.0);
        // Exercise the no-refraction direct sampling path (previously crashed
        // on blurred sources) with blur on and off.
        let mut no_refr = GlassMaterial::default();
        no_refr.refraction_on = false;
        let _ = composite(&no_refr, &bg);
        no_refr.blur_on = false;
        let _ = composite(&no_refr, &bg);
        println!("refraction-off paths ok");
        let snap = std::env::temp_dir().join("tontooui_glass_snapshot.png");
        let (snap_img, _) = composite(&p, &bg);
        let _ = snap_img.save(&snap);
        println!("snapshot: {}", snap.display());
        return;
    }

    let system = ColorScheme::detect_system();
    let is_dark = system == ColorScheme::Dark;
    let mut app = App::new("TontooUI Glass Playground", 1440, 1200);
    app.set_color_scheme(system);

    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    let title = Text::new("Glass Playground").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());
    let subtitle = Text::new("16 sliders + 8 toggles for the glass material itself — blur, refraction, light, grain")
        .font_size(11.0)
        .color(desc_c);

    // Left: live glass stage over the image background.
    let stage_panel = VStack::new()
        .spacing(10.0)
        .child(GlassStage::new())
        .child(
            HStack::new()
                .spacing(8.0)
                .child(Button::new("Frost").style(ButtonStyle::Glass).on_click(preset_frost))
                .child(Button::new("Clear").style(ButtonStyle::Glass).on_click(preset_clear))
                .child(Button::new("Sunset").style(ButtonStyle::Glass).on_click(preset_sunset)),
        )
        .child(
            Text::new("Presets update the preview instantly, sliders stay manual")
                .font_size(9.0)
                .color(desc_c),
        )
        .child(PerfPanel::new());

    // Right: 24 material controls.
    let col_a = VStack::new()
        .spacing(10.0)
        .child(group_title("Tint & Light"))
        .child(slider_row("Tint red", 0.0, 255.0, 168.0, 1.0, set_tint_r))
        .child(slider_row("Tint green", 0.0, 255.0, 213.0, 1.0, set_tint_g))
        .child(slider_row("Tint blue", 0.0, 255.0, 255.0, 1.0, set_tint_b))
        .child(slider_row("Tint alpha %", 0.0, 100.0, 17.0, 1.0, set_tint_a))
        .child(slider_row("Specular %", 0.0, 100.0, 69.0, 1.0, set_specular))
        .child(slider_row("Specular angle", 0.0, 360.0, 175.0, 1.0, set_spec_angle))
        .child(slider_row("Rim light %", 0.0, 100.0, 61.0, 1.0, set_rim))
        .child(slider_row("Backdrop dim %", 0.0, 80.0, 0.0, 1.0, set_dim));

    let col_b = VStack::new()
        .spacing(10.0)
        .child(group_title("Blur & Body"))
        .child(slider_row("Blur sigma", 0.0, 25.0, 1.5, 0.5, set_sigma))
        .child(slider_row("Refraction %", 0.0, 100.0, 60.0, 1.0, set_refraction))
        .child(slider_row("Depth %", 0.0, 100.0, 100.0, 1.0, set_depth))
        .child(slider_row("Dispersion %", 0.0, 100.0, 100.0, 1.0, set_dispersion))
        .child(slider_row("Saturation %", 0.0, 200.0, 100.0, 1.0, set_saturation))
        .child(slider_row("Brightness %", 20.0, 180.0, 140.0, 1.0, set_brightness))
        .child(slider_row("Contrast %", 0.0, 200.0, 150.0, 1.0, set_contrast))
        .child(slider_row("Frost grain %", 0.0, 100.0, 22.0, 1.0, set_grain));

    let toggles = HStack::new()
        .spacing(24.0)
        .child(
            VStack::new()
                .spacing(8.0)
                .child(toggle_row("Blur", true, set_blur_on))
                .child(toggle_row("Refraction", true, set_refraction_on))
                .child(toggle_row("Dispersion", true, set_dispersion_on))
                .child(toggle_row("Frost grain", true, set_grain_on)),
        )
        .child(
            VStack::new()
                .spacing(8.0)
                .child(toggle_row("Specular", true, set_specular_on))
                .child(toggle_row("Rim light", true, set_rim_on))
                .child(toggle_row("Tint", true, set_tint_on))
                .child(toggle_row("Dim backdrop", false, set_dim_on)),
        );

    let controls = VStack::new()
        .spacing(12.0)
        .child(HStack::new().spacing(24.0).child(col_a).child(col_b))
        .child(group_title("Stages"))
        .child(toggles);

    let root = VStack::new()
        .spacing(14.0)
        .child(title)
        .child(subtitle)
        .child(HStack::new().spacing(28.0).child(stage_panel).child(controls));
    app.set_root(root);
    app.run();
}
