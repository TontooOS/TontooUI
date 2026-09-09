//! Glass material — parameters and CPU compositor for liquid glass.
//!
//! [`GlassMaterial`] describes how the glass itself behaves (tint, blur
//! sigma, refraction, depth, dispersion, vibrancy, specular, rim, grain,
//! dim) and [`render_glass`] composites it over any backdrop image behind a
//! rounded-rect mask. Pure image math, no GTK involved, so it is unit
//! testable and reusable from widgets and examples alike.

use std::time::Instant;

use image::RgbImage;

// ── Parameters ───────────────────────────────────────────────────────

/// How the glass itself behaves. All sliders are plain `f32` values in the
/// ranges documented per field; the matching toggles gate whole stages.
#[derive(Debug, Clone)]
pub struct GlassMaterial {
    /// Tint color red channel, `0..=255`.
    pub tint_r: f32,
    /// Tint color green channel, `0..=255`.
    pub tint_g: f32,
    /// Tint color blue channel, `0..=255`.
    pub tint_b: f32,
    /// Tint layer opacity, `0..=100`.
    pub tint_a: f32,
    /// Backdrop gaussian blur, `0..=25`.
    pub sigma: f32,
    /// Lens refraction strength, `0..=100`.
    pub refraction: f32,
    /// Glass thickness (bend + edge darkening), `0..=100`.
    pub depth: f32,
    /// Chromatic aberration, `0..=100`.
    pub dispersion: f32,
    /// Vibrancy, `0..=200` (`100` is neutral).
    pub saturation: f32,
    /// Brightness, `20..=180` (`100` is neutral).
    pub brightness: f32,
    /// Contrast, `0..=200` (`100` is neutral).
    pub contrast: f32,
    /// Specular sheen intensity, `0..=100`.
    pub specular: f32,
    /// Specular light angle in degrees, `0..=360`.
    pub spec_angle: f32,
    /// Rim light intensity, `0..=100`.
    pub rim: f32,
    /// Frost grain amount, `0..=100`.
    pub grain: f32,
    /// Backdrop dim before light enters the glass, `0..=80`.
    pub dim: f32,
    /// Gate for the blur stage.
    pub blur_on: bool,
    /// Gate for the refraction stage.
    pub refraction_on: bool,
    /// Gate for the dispersion stage.
    pub dispersion_on: bool,
    /// Gate for the frost grain stage.
    pub grain_on: bool,
    /// Gate for the specular stage.
    pub specular_on: bool,
    /// Gate for the rim light stage.
    pub rim_on: bool,
    /// Gate for the tint layer.
    pub tint_on: bool,
    /// Gate for the backdrop dim stage.
    pub dim_on: bool,
}

impl Default for GlassMaterial {
    fn default() -> Self {
        Self {
            tint_r: 168.0,
            tint_g: 213.0,
            tint_b: 255.0,
            tint_a: 17.0,
            sigma: 1.5,
            refraction: 60.0,
            depth: 100.0,
            dispersion: 100.0,
            saturation: 100.0,
            brightness: 140.0,
            contrast: 150.0,
            specular: 69.0,
            spec_angle: 175.0,
            rim: 61.0,
            grain: 22.0,
            dim: 0.0,
            blur_on: true,
            refraction_on: true,
            dispersion_on: true,
            grain_on: true,
            specular_on: true,
            rim_on: true,
            tint_on: true,
            dim_on: false,
        }
    }
}

// ── Math helpers ─────────────────────────────────────────────────────

fn clamp01(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash01(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) % 10000) as u32 as f32 / 10000.0
}

/// Signed distance to a rounded rect (negative inside).
fn rounded_rect_sdf(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let r = r.clamp(0.0, h / 2.0);
    let cx = w / 2.0;
    let cy = h / 2.0;
    let qx = (x - cx).abs() - (cx - r);
    let qy = (y - cy).abs() - (cy - r);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - r
}

#[inline]
fn samp(src: &[u8], w: i32, h: i32, x: f32, y: f32) -> [f32; 3] {
    let xc = x.clamp(0.0, (w - 1) as f32);
    let yc = y.clamp(0.0, (h - 1) as f32);
    let x0 = xc as i32;
    let y0 = yc as i32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = xc - x0 as f32;
    let fy = yc - y0 as f32;
    let at = |x: i32, y: i32| {
        let o = ((y * w + x) * 3) as usize;
        [src[o] as f32, src[o + 1] as f32, src[o + 2] as f32]
    };
    let a = at(x0, y0);
    let b = at(x1, y0);
    let c = at(x0, y1);
    let d = at(x1, y1);
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[i] = a[i] * (1.0 - fx) * (1.0 - fy) + b[i] * fx * (1.0 - fy) + c[i] * (1.0 - fx) * fy + d[i] * fx * fy;
    }
    out
}

// ── Compositor ───────────────────────────────────────────────────────

/// Composite liquid glass over `backdrop`.
///
/// Renders a `w`×`h` image: the glass inside a rounded rect with corner
/// `radius`, the pure backdrop outside the mask. The backdrop is
/// center-cropped when larger and cover-scaled when smaller. Returns the
/// image plus the blur stage time in milliseconds (for perf HUDs).
pub fn render_glass(mat: &GlassMaterial, backdrop: &RgbImage, w: u32, h: u32, radius: f32) -> (RgbImage, f32) {
    if w == 0 || h == 0 {
        return (RgbImage::new(w, h), 0.0);
    }
    // Cover-scale small backdrops up so there is always enough material.
    let fitted: RgbImage;
    let bg: &RgbImage = if backdrop.width() < w || backdrop.height() < h {
        let scale = (w as f32 / backdrop.width().max(1) as f32)
            .max(h as f32 / backdrop.height().max(1) as f32);
        fitted = image::imageops::resize(
            backdrop,
            (backdrop.width() as f32 * scale).ceil() as u32,
            (backdrop.height() as f32 * scale).ceil() as u32,
            image::imageops::FilterType::Triangle,
        );
        &fitted
    } else {
        backdrop
    };

    // 1 — center-crop the backdrop behind the glass. `orig` stays sharp so
    // pixels outside the mask show the pure backdrop.
    let gx0 = (bg.width().saturating_sub(w)) / 2;
    let gy0 = (bg.height().saturating_sub(h)) / 2;
    let mut crop = RgbImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            crop.put_pixel(x, y, *bg.get_pixel(gx0 + x, gy0 + y));
        }
    }
    let orig = crop.clone().into_raw();

    // 2 — backdrop dim (light absorbed before it enters the glass).
    if mat.dim_on && mat.dim > 0.5 {
        let k = 1.0 - (mat.dim / 100.0) * 0.85;
        for px in crop.pixels_mut() {
            let v = px.0;
            px.0 = [
                (v[0] as f32 * k).round() as u8,
                (v[1] as f32 * k).round() as u8,
                (v[2] as f32 * k).round() as u8,
            ];
        }
    }

    // 3 — blur at reduced resolution, sampled back with bilinear upscale.
    // Frosted glass hides all downscale artifacts; far fewer pixels go
    // through the blur. True gaussian on the small buffer: a box-blur
    // approximation paints square halos around bright edges.
    let ds_w = (w / 3).max(32);
    let ds_h = (h / 3).max(16);
    let t_blur = Instant::now();
    let (src, kw, kh) = if mat.blur_on && mat.sigma > 0.5 {
        let small = image::imageops::resize(&crop, ds_w, ds_h, image::imageops::FilterType::Triangle);
        (image::imageops::blur(&small, mat.sigma / 3.0).into_raw(), ds_w as i32, ds_h as i32)
    } else if mat.blur_on {
        let small = image::imageops::resize(&crop, ds_w, ds_h, image::imageops::FilterType::Triangle);
        (small.into_raw(), ds_w as i32, ds_h as i32)
    } else {
        (crop.into_raw(), w as i32, h as i32)
    };
    let blur_ms = t_blur.elapsed().as_secs_f32() * 1000.0;
    let kx = kw as f32 / w as f32;
    let ky = kh as f32 / h as f32;

    // 4 — per-pixel refraction, grade, tint, grain, specular, rim, mask.
    let sw = w as i32;
    let mut out = vec![0u8; (w * h * 3) as usize];
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let refr = if mat.refraction_on { mat.refraction / 100.0 } else { 0.0 };
    let depth = mat.depth / 100.0;
    let disp = if mat.dispersion_on { mat.dispersion / 100.0 } else { 0.0 };
    let zoom_base = 1.0 + refr * 0.30;
    let sat = mat.saturation / 100.0;
    let bri = mat.brightness / 100.0;
    let con = mat.contrast / 100.0;
    let spec = if mat.specular_on { mat.specular / 100.0 } else { 0.0 };
    let ang = mat.spec_angle.to_radians();
    let (dx, dy) = (ang.cos(), ang.sin());
    let rim = if mat.rim_on { mat.rim / 100.0 } else { 0.0 };
    let grain = if mat.grain_on { mat.grain / 100.0 } else { 0.0 };
    let tint_a = if mat.tint_on { mat.tint_a / 100.0 } else { 0.0 };

    for y in 0..h as i32 {
        for x in 0..sw {
            let fx = x as f32;
            let fy = y as f32;
            let nx = (fx - cx) / cx;
            let ny = (fy - cy) / cy;
            // Mask: negative deep inside, ~0 at the rim.
            let d = rounded_rect_sdf(fx, fy, w as f32, h as f32, radius);

            // Refraction: lens magnification toward the center, stronger
            // near the rim, plus an outward bend scaled by depth.
            let mut col = if refr > 0.001 {
                let r2 = (nx * nx + ny * ny).min(1.0);
                let zoom = zoom_base * (0.55 + 0.45 * r2);
                let bend = (1.0 - smoothstep(-40.0, 0.0, d)) * depth * 10.0;
                let len = (nx * nx + ny * ny).sqrt().max(1e-4);
                let ex = nx / len * bend;
                let ey = ny / len * bend;
                if disp > 0.001 {
                    // Dispersion: each channel refracts slightly differently.
                    // Kept subtle on purpose: at 100% it reads as glassy
                    // fringe, not rainbow soup.
                    let zr = zoom * (1.0 + disp * 0.012);
                    let zb = zoom * (1.0 - disp * 0.012);
                    let r = samp(&src, kw, kh, (cx + (fx - cx) / zr + ex) * kx, (cy + (fy - cy) / zr + ey) * ky)[0];
                    let g = samp(&src, kw, kh, (cx + (fx - cx) / zoom + ex) * kx, (cy + (fy - cy) / zoom + ey) * ky)[1];
                    let b = samp(&src, kw, kh, (cx + (fx - cx) / zb + ex) * kx, (cy + (fy - cy) / zb + ey) * ky)[2];
                    [r, g, b]
                } else {
                    samp(&src, kw, kh, (cx + (fx - cx) / zoom + ex) * kx, (cy + (fy - cy) / zoom + ey) * ky)
                }
            } else {
                // No refraction: read through at the matching position.
                // Must scale into source coords — the blurred source is
                // smaller than the card (direct raw indexing with card
                // coords overruns it).
                samp(&src, kw, kh, fx * kx, fy * ky)
            };

            // Vibrancy grade: saturation, brightness, contrast.
            let luma = 0.299 * col[0] + 0.587 * col[1] + 0.114 * col[2];
            for c in col.iter_mut() {
                let mut v = luma + (*c - luma) * sat;
                v *= bri;
                v = (v - 128.0) * con + 128.0;
                *c = v;
            }

            // Tint layer.
            if tint_a > 0.001 {
                col[0] = mat.tint_r * tint_a + col[0] * (1.0 - tint_a);
                col[1] = mat.tint_g * tint_a + col[1] * (1.0 - tint_a);
                col[2] = mat.tint_b * tint_a + col[2] * (1.0 - tint_a);
            }

            // Frost grain.
            if grain > 0.001 {
                let n = (hash01(x as u32, y as u32) - 0.5) * 90.0 * grain;
                for c in col.iter_mut() {
                    *c += n;
                }
            }

            // Specular sheen: diagonal light band across the card.
            if spec > 0.001 {
                let proj = (fx * dx + fy * dy) / (w + h) as f32 + 0.5;
                let band = (-(proj - 0.30).powi(2) / (2.0 * 0.10 * 0.10)).exp();
                let add = band * 70.0 * spec;
                for c in col.iter_mut() {
                    *c += add;
                }
            }

            // Rim light plus thickness darkening hugging the rim.
            if rim > 0.001 {
                let e = 1.0 - smoothstep(-7.0, 0.5, d);
                let add = e * 90.0 * rim;
                for c in col.iter_mut() {
                    *c += add;
                }
            }
            if depth > 0.001 && refr > 0.001 {
                let k = 1.0 - (1.0 - smoothstep(-16.0, 0.0, d)) * 0.18 * depth;
                for c in col.iter_mut() {
                    *c *= k;
                }
            }

            // Mask: crisp glass inside, pure backdrop outside.
            let a = 1.0 - smoothstep(-1.2, 1.2, d);
            let o = ((y * sw + x) * 3) as usize;
            out[o] = clamp01(orig[o] as f32 * (1.0 - a) + col[0] * a);
            out[o + 1] = clamp01(orig[o + 1] as f32 * (1.0 - a) + col[1] * a);
            out[o + 2] = clamp01(orig[o + 2] as f32 * (1.0 - a) + col[2] * a);
        }
    }
    (RgbImage::from_raw(w, h, out).unwrap(), blur_ms)
}

// ── Clear glass ────────────────────────────────────────────────────

/// The simple glass: same color as what's behind, a micro lift brighter,
/// light top/bottom edges, darker left/right edges. Full pass-through —
/// no blur, no refraction, everything behind stays sharp.
#[derive(Debug, Clone)]
pub struct ClearGlass {
    /// Brightness lift added inside, `0..=30` (default `14`).
    pub lift: f32,
    /// Top/bottom edge light, `0..=100` (default `70`).
    pub edge_light: f32,
    /// Left/right edge darkening, `0..=100` (default `40`).
    pub edge_dark: f32,
}

impl Default for ClearGlass {
    fn default() -> Self {
        Self { lift: 14.0, edge_light: 70.0, edge_dark: 40.0 }
    }
}

/// Composite clear glass over `backdrop`: sharp pass-through with a lift
/// and directional edge light inside a rounded rect (`radius`, capsule
/// when `radius >= h / 2`). Outside the mask is the pure backdrop.
pub fn render_clear_glass(clear: &ClearGlass, backdrop: &RgbImage, w: u32, h: u32, radius: f32) -> RgbImage {
    if w == 0 || h == 0 {
        return RgbImage::new(w, h);
    }
    let fitted: RgbImage;
    let bg: &RgbImage = if backdrop.width() < w || backdrop.height() < h {
        let scale = (w as f32 / backdrop.width().max(1) as f32)
            .max(h as f32 / backdrop.height().max(1) as f32);
        fitted = image::imageops::resize(
            backdrop,
            (backdrop.width() as f32 * scale).ceil() as u32,
            (backdrop.height() as f32 * scale).ceil() as u32,
            image::imageops::FilterType::Triangle,
        );
        &fitted
    } else {
        backdrop
    };
    let gx0 = (bg.width().saturating_sub(w)) / 2;
    let gy0 = (bg.height().saturating_sub(h)) / 2;

    let mut out = vec![0u8; (w * h * 3) as usize];
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let band = (h as f32 * 0.16).clamp(6.0, 20.0);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let fx = x as f32;
            let fy = y as f32;
            let d = rounded_rect_sdf(fx, fy, w as f32, h as f32, radius);
            let o = ((y * w as i32 + x) * 3) as usize;
            let b = bg.get_pixel(gx0 + x as u32, gy0 + y as u32).0;
            let mut col = [b[0] as f32, b[1] as f32, b[2] as f32];

            // Micro lift.
            for c in col.iter_mut() {
                *c += clear.lift;
            }

            // Directional edge light: whiteness follows the vertical
            // share of the edge normal, darkness the horizontal share.
            // The band hugs the rim (0 deep inside, 1 at the edge).
            let e = smoothstep(-band, 0.0, d);
            if e > 0.001 {
                let nx = ((fx - cx) / cx).abs();
                let ny = ((fy - cy) / cy).abs();
                let vert = ny / (nx + ny + 1e-4);
                let add = e * vert * (clear.edge_light / 100.0) * 70.0;
                let dark = 1.0 - e * (1.0 - vert) * (clear.edge_dark / 100.0) * 0.30;
                for c in col.iter_mut() {
                    *c = (*c + add) * dark;
                }
            }

            // Mask: glass inside, pure backdrop outside.
            let a = 1.0 - smoothstep(-1.2, 1.2, d);
            out[o] = clamp01(b[0] as f32 * (1.0 - a) + col[0] * a);
            out[o + 1] = clamp01(b[1] as f32 * (1.0 - a) + col[1] * a);
            out[o + 2] = clamp01(b[2] as f32 * (1.0 - a) + col[2] * a);
        }
    }
    RgbImage::from_raw(w, h, out).unwrap()
}
#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn solid(w: u32, h: u32, c: [u8; 3]) -> RgbImage {
        RgbImage::from_pixel(w, h, Rgb(c))
    }

    #[test]
    fn render_size_matches() {
        let (img, _) = render_glass(&GlassMaterial::default(), &solid(400, 300, [10, 20, 30]), 160, 90, 20.0);
        assert_eq!((img.width(), img.height()), (160, 90));
    }

    #[test]
    fn corners_outside_mask_show_pure_backdrop() {
        let bg = solid(200, 200, [11, 22, 33]);
        let (img, _) = render_glass(&GlassMaterial::default(), &bg, 100, 60, 30.0);
        assert_eq!(img.get_pixel(0, 0).0, [11, 22, 33]);
        assert_eq!(img.get_pixel(99, 59).0, [11, 22, 33]);
    }

    #[test]
    fn oversized_radius_clamps_without_panic() {
        let (img, _) = render_glass(&GlassMaterial::default(), &solid(200, 200, [9, 9, 9]), 100, 60, 9999.0);
        assert_eq!((img.width(), img.height()), (100, 60));
    }

    #[test]
    fn empty_size_returns_empty() {
        let (img, _) = render_glass(&GlassMaterial::default(), &solid(50, 50, [1, 2, 3]), 0, 0, 0.0);
        assert_eq!((img.width(), img.height()), (0, 0));
    }

    #[test]
    fn small_backdrop_cover_scales_up() {
        let (img, _) = render_glass(&GlassMaterial::default(), &solid(20, 20, [7, 8, 9]), 100, 60, 10.0);
        assert_eq!((img.width(), img.height()), (100, 60));
    }

    #[test]
    fn material_defaults() {
        let m = GlassMaterial::default();
        assert_eq!((m.tint_r, m.tint_g, m.tint_b, m.tint_a), (168.0, 213.0, 255.0, 17.0));
        assert!(m.blur_on && m.refraction_on && !m.dim_on);
    }

    #[test]
    fn clear_glass_lifts_center() {
        let bg = solid(100, 100, [100, 100, 100]);
        let img = render_clear_glass(&ClearGlass::default(), &bg, 60, 40, 20.0);
        assert_eq!((img.width(), img.height()), (60, 40));
        // Center is far from any edge: pure lift, no edge light.
        assert_eq!(img.get_pixel(30, 20).0, [114, 114, 114]);
    }

    #[test]
    fn clear_glass_outside_is_backdrop() {
        let bg = solid(100, 100, [50, 60, 70]);
        let img = render_clear_glass(&ClearGlass::default(), &bg, 60, 40, 20.0);
        assert_eq!(img.get_pixel(0, 0).0, [50, 60, 70]);
    }

    #[test]
    fn clear_glass_top_edge_brighter_than_side() {
        let bg = solid(120, 120, [100, 100, 100]);
        let img = render_clear_glass(&ClearGlass::default(), &bg, 80, 60, 30.0);
        // Just inside the top edge vs just inside the left edge.
        let top = img.get_pixel(40, 1).0[0] as i32;
        let side = img.get_pixel(1, 30).0[0] as i32;
        assert!(top > side, "top {} should beat side {}", top, side);
    }
}
