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
pub struct ImageLoader<'a> {
    renderer: &'a mut Renderer,
    device: &'a Device,
    queue: &'a Queue,
    cache: &'a mut ImageCache,
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
        }
    }

    /// Get `(image, width, height)` for an SF Symbol name. Returns `None`
    /// when the asset is missing or undecodable; callers skip the icon.
    pub fn get(&mut self, name: &str, tint: Color) -> Option<(ImageData, u32, u32)> {
        let rgba = tint.to_rgba8();
        let key = format!(
            "{name}#{:02x}{:02x}{:02x}{:02x}",
            rgba.r, rgba.g, rgba.b, rgba.a
        );
        if let Some(cached) = self.cache.map.get(&key) {
            return Some((cached.image.clone(), cached.width, cached.height));
        }
        let (image, width, height) = self.upload(name, tint)?;
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

    fn upload(&mut self, name: &str, tint: Color) -> Option<(ImageData, u32, u32)> {
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
        let rgba = tint.to_rgba8();
        let pixels: Vec<u8> = match info.color_type {
            png::ColorType::Rgba => raw
                .chunks_exact(4)
                .flat_map(|px| [rgba.r, rgba.g, rgba.b, px[3]])
                .collect(),
            png::ColorType::Rgb => raw
                .chunks_exact(3)
                .flat_map(|_| [rgba.r, rgba.g, rgba.b, 255])
                .collect(),
            _ => return None,
        };

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
