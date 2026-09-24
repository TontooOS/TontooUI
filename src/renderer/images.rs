use std::collections::HashMap;
use std::fs::File;

use vello::Renderer;
use vello::peniko::{Color, ImageData};
use wgpu::{Device, Queue};

/// GPU-uploaded icon with natural size.
#[derive(Clone, Debug)]
struct CachedImage {
    image: ImageData,
    width: u32,
    height: u32,
}

/// Owns uploaded SF Symbol textures across frames. Lives in the shell;
/// views borrow it per frame through `ImageLoader`.
#[derive(Default)]
pub struct ImageCache {
    map: HashMap<String, CachedImage>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Per-frame image access for views. Resolves CoreIcon SF Symbols by name,
/// recolors glyphs to `tint` (assets are black with alpha) and uploads
/// them once; later frames hit the cache.
///
/// During a backdrop capture pass (`set_capture_pass(true)`) glass views
/// skip their body so the blur sees what is behind them. On the final
/// pass `backdrop` is the blurred capture that glass samples.
pub struct ImageLoader<'a> {
    renderer: &'a mut Renderer,
    device: &'a Device,
    queue: &'a Queue,
    cache: &'a mut ImageCache,
    backdrop: Option<ImageData>,
    backdrop_sharp: Option<ImageData>,
    capture_pass: bool,
}

impl<'a> ImageLoader<'a> {
    pub fn new(
        renderer: &'a mut Renderer,
        device: &'a Device,
        queue: &'a Queue,
        cache: &'a mut ImageCache,
    ) -> Self {
        Self {
            renderer,
            device,
            queue,
            cache,
            backdrop: None,
            backdrop_sharp: None,
            capture_pass: false,
        }
    }

    /// Blurred in-app backdrop for glass fills, or `None` on the capture
    /// pass or when the shell did not run the backdrop pass.
    pub fn backdrop(&self) -> Option<&ImageData> {
        self.backdrop.as_ref()
    }

    /// Sharp (unblurred) capture for the magnified glass center.
    pub fn backdrop_sharp(&self) -> Option<&ImageData> {
        self.backdrop_sharp.as_ref()
    }

    /// True while the shell is recording the pre-blur capture pass.
    /// Glass bodies (and their shadows) must skip drawing so the blur
    /// sees the content behind them.
    pub fn is_capture_pass(&self) -> bool {
        self.capture_pass
    }

    pub fn set_backdrop(&mut self, backdrop: Option<ImageData>) {
        self.backdrop = backdrop;
    }

    pub fn set_backdrop_sharp(&mut self, sharp: Option<ImageData>) {
        self.backdrop_sharp = sharp;
    }

    pub fn set_capture_pass(&mut self, capture_pass: bool) {
        self.capture_pass = capture_pass;
    }

    /// Get `(image, width, height)` for an SF Symbol name, downscaled with
    /// a high-quality filter to `target_px` (max dimension in physical px,
    /// pass ~2x the display size for crisp supersampling). Returns `None`
    /// when the asset is missing or undecodable; callers skip the icon.
    pub fn get(
        &mut self,
        name: &str,
        tint: Color,
        target_px: u32,
    ) -> Option<(ImageData, u32, u32)> {
        let rgba = tint.to_rgba8();
        let key = format!(
            "{name}#{:02x}{:02x}{:02x}{:02x}@{target_px}px",
            rgba.r, rgba.g, rgba.b, rgba.a
        );
        if let Some(cached) = self.cache.map.get(&key) {
            return Some((cached.image.clone(), cached.width, cached.height));
        }
        let (image, width, height) = self.upload(name, tint, target_px.max(1))?;
        self.cache.map.insert(
            key,
            CachedImage {
                image: image.clone(),
                width,
                height,
            },
        );
        Some((image, width, height))
    }

    fn upload(
        &mut self,
        name: &str,
        tint: Color,
        target_px: u32,
    ) -> Option<(ImageData, u32, u32)> {
        use std::io::BufReader;

        let path = coreicon::resolve_icon_path(name);
        let file = File::open(path).ok()?;
        let decoder = png::Decoder::new(BufReader::new(file));
        let mut reader = decoder.read_info().ok()?;
        // RGBA8 upper bound; interlaced assets are not supported.
        let capacity = {
            let info = reader.info();
            info.width as usize * info.height as usize * 4
        };
        let mut raw = vec![0; capacity];
        let info = reader.next_frame(&mut raw).ok()?;
        let (width, height) = (info.width, info.height);
        if width == 0 || height == 0 {
            return None;
        }
        let gray: Vec<u8> = match info.color_type {
            png::ColorType::Rgba => raw
                .chunks_exact(4)
                .flat_map(|px| [px[0], px[1], px[2], px[3]])
                .collect(),
            png::ColorType::Rgb => raw
                .chunks_exact(3)
                .flat_map(|px| [px[0], px[1], px[2], 255])
                .collect(),
            _ => return None,
        };
        // Downscale once with Lanczos3: 1024 px assets minified 50x by the
        // GPU sampler turn to mush without mipmaps, so the CPU bakes a
        // crisp ~2x texture instead.
        let long_side = width.max(height);
        let (pixels, width, height) = if long_side > target_px {
            let scale = target_px as f32 / long_side as f32;
            let nw = ((width as f32 * scale).round() as u32).max(1);
            let nh = ((height as f32 * scale).round() as u32).max(1);
            let src = image::RgbaImage::from_raw(width, height, gray)?;
            let small =
                image::imageops::resize(&src, nw, nh, image::imageops::FilterType::Lanczos3);
            (small.into_raw(), nw, nh)
        } else {
            (gray, width, height)
        };
        // Glyphs are black with alpha: paint the tint, keep the alpha.
        let rgba = tint.to_rgba8();
        let pixels: Vec<u8> = pixels
            .chunks_exact(4)
            .flat_map(|px| [rgba.r, rgba.g, rgba.b, px[3]])
            .collect();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tontooui icon"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        // Rows must align to COPY_BYTES_PER_ROW_ALIGNMENT.
        let row_bytes = width as usize * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded = row_bytes.div_ceil(align) * align;
        let mut upload = vec![0u8; padded * height as usize];
        for (dst, src) in upload.chunks_mut(padded).zip(pixels.chunks(row_bytes)) {
            dst[..row_bytes].copy_from_slice(src);
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &upload,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded as u32),
                rows_per_image: Some(height),
            },
            size,
        );
        let image = self.renderer.register_texture(texture);
        Some((image, width, height))
    }
}
