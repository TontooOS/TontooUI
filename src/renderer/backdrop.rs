use vello::kurbo::{Affine, Point, Rect, RoundedRect, Stroke};
use vello::peniko::{Extend, Fill, ImageBrush, ImageData, ImageQuality};
use vello::Renderer;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, Queue,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension,
};

/// Default gaussian sigma in physical px for the in-app backdrop blur.
pub const BACKDROP_SIGMA: f32 = 8.0;

/// Busyness tile size in physical px for the variance map.
pub const BUSY_TILE: u32 = 16;
/// Variance sampling cadence: every Nth blur run (~2 Hz at 60 fps).
pub const BUSY_EVERY_N_FRAMES: u64 = 30;
/// Busyness below this variance reads as plain background.
pub const BUSY_LO: f32 = 0.004;
/// Busyness at/above this variance reads as fully busy media.
pub const BUSY_HI: f32 = 0.03;
/// Veil ceiling for automatic frost (stays below full frost).
pub const AUTO_FROST_VEIL: f32 = 0.65;

const WORKGROUP: u32 = 8;

/// Separable gaussian blur over the captured in-app backdrop.
///
/// Vello renders the content pass into `content`; two compute dispatches
/// (horizontal then vertical) write the blurred result into `output`, which
/// is registered with the Vello renderer so views can sample it as an image
/// brush. The window body is part of the capture, so glass sees the same
/// pixels an opaque widget would paint over.
const SHADER: &str = r#"
struct Params {
  width: u32,
  height: u32,
  dir_x: u32,
  dir_y: u32,
  sigma: f32,
  pad0: f32,
  pad1: f32,
  pad2: f32,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: Params;

fn gauss(x: f32, sigma: f32) -> f32 {
  return exp(-(x * x) / (2.0 * sigma * sigma));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if gid.x >= params.width || gid.y >= params.height {
    return;
  }
  let sigma = params.sigma;
  let src_px = textureLoad(src, vec2<i32>(gid.xy), 0);
  if sigma <= 0.0 {
    textureStore(dst, vec2<i32>(gid.xy), src_px);
    return;
  }
  let radius = i32(ceil(sigma * 3.0));
  let dir = vec2<i32>(i32(params.dir_x), i32(params.dir_y));
  let center = vec2<i32>(gid.xy);
  let max_x = i32(params.width) - 1;
  let max_y = i32(params.height) - 1;
  var sum = vec4<f32>(0.0);
  var wsum = 0.0;
  for (var i = -radius; i <= radius; i = i + 1) {
    let w = gauss(f32(i), sigma);
    var c = center + dir * i;
    c.x = clamp(c.x, 0, max_x);
    c.y = clamp(c.y, 0, max_y);
    sum = sum + textureLoad(src, c, 0) * w;
    wsum = wsum + w;
  }
  textureStore(dst, center, sum / wsum);
}
"#;

/// Low-res luma variance of the sharp capture, one float per
/// `BUSY_TILE` square. Glass samples it to frost busy backdrops
/// (video, photos) while staying clear over plain content.
/// Row-major, `tiles_x` per row.
#[derive(Clone, Default)]
pub struct BusyGrid {
    /// Capture size in physical px (for scale sanity checks).
    pub width: u32,
    pub height: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub tiles: Vec<f32>,
}

impl BusyGrid {
    /// Average variance over the physical-px rect. Returns `None`
    /// while no sample arrived yet.
    pub fn amount_at(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Option<f32> {
        if self.tiles.is_empty() || self.tiles_x == 0 || self.tiles_y == 0 {
            return None;
        }
        let tile = BUSY_TILE as f32;
        let tx0 = (x0.div_euclid(tile) as u32).min(self.tiles_x.saturating_sub(1));
        let tx1 = ((x1 - 1.0).div_euclid(tile) as u32).min(self.tiles_x.saturating_sub(1));
        let ty0 = (y0.div_euclid(tile) as f32).max(0.0) as u32;
        let ty0 = ty0.min(self.tiles_y.saturating_sub(1));
        let ty1 = ((y1 - 1.0).div_euclid(tile) as f32).max(0.0) as u32;
        let ty1 = ty1.min(self.tiles_y.saturating_sub(1));
        if tx1 < tx0 || ty1 < ty0 {
            return None;
        }
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if let Some(v) = self
                    .tiles
                    .get(ty as usize * self.tiles_x as usize + tx as usize)
                {
                    sum += *v;
                    n += 1;
                }
            }
        }
        if n == 0 { None } else { Some(sum / n as f32) }
    }
}

/// Busyness 0..1 from a mean tile variance: plain below `BUSY_LO`,
/// fully busy at `BUSY_HI`, smooth between.
pub fn frost_for_busy(busy: f32) -> f32 {
    if busy <= BUSY_LO {
        0.0
    } else if busy >= BUSY_HI {
        1.0
    } else {
        let t = (busy - BUSY_LO) / (BUSY_HI - BUSY_LO);
        t * t * (3.0 - 2.0 * t)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Params {
    width: u32,
    height: u32,
    dir_x: u32,
    dir_y: u32,
    sigma: f32,
    pad0: f32,
    pad1: f32,
    pad2: f32,
}

impl Params {
    fn to_bytes(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[0..4].copy_from_slice(&self.width.to_le_bytes());
        out[4..8].copy_from_slice(&self.height.to_le_bytes());
        out[8..12].copy_from_slice(&self.dir_x.to_le_bytes());
        out[12..16].copy_from_slice(&self.dir_y.to_le_bytes());
        out[16..20].copy_from_slice(&self.sigma.to_le_bytes());
        out
    }
}

struct PassTargets {
    content: Texture,
    content_view: TextureView,
    temp_view: TextureView,
    output: Texture,
    output_view: TextureView,
}

/// Luma variance of the sharp capture, one float per `BUSY_TILE`
/// square: feeds the `BusyGrid` glass reads for automatic frost.
const VARIANCE_SHADER: &str = r#"
struct VParams {
  width: u32,
  height: u32,
  tiles_x: u32,
  tiles_y: u32,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var<uniform> params: VParams;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if gid.x >= params.tiles_x || gid.y >= params.tiles_y {
    return;
  }
  let tile = 16u;
  var sum = 0.0;
  var sum2 = 0.0;
  var n = 0u;
  for (var ty = 0u; ty < tile; ty = ty + 1u) {
    for (var tx = 0u; tx < tile; tx = tx + 1u) {
      let px = vec2<i32>(vec2<u32>(gid.x * tile + tx, gid.y * tile + ty));
      if px.x >= i32(params.width) || px.y >= i32(params.height) {
        continue;
      }
      let c = textureLoad(src, px, 0);
      let luma = dot(c.rgb, vec3<f32>(0.299, 0.587, 0.114));
      sum = sum + luma;
      sum2 = sum2 + luma * luma;
      n = n + 1u;
    }
  }
  var v = 0.0;
  if n > 0u {
    let m = sum / f32(n);
    v = max(sum2 / f32(n) - m * m, 0.0);
  }
  textureStore(dst, vec2<i32>(gid.xy), vec4<f32>(v, 0.0, 0.0, 0.0));
}
"#;

#[repr(C)]
#[derive(Clone, Copy)]
struct VParams {
    width: u32,
    height: u32,
    tiles_x: u32,
    tiles_y: u32,
}

impl VParams {
    fn to_bytes(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[0..4].copy_from_slice(&self.width.to_le_bytes());
        out[4..8].copy_from_slice(&self.height.to_le_bytes());
        out[8..12].copy_from_slice(&self.tiles_x.to_le_bytes());
        out[12..16].copy_from_slice(&self.tiles_y.to_le_bytes());
        out
    }
}

struct VarTargets {
    tex: Texture,
    view: TextureView,
    tiles_x: u32,
    tiles_y: u32,
    staging: Buffer,
    /// Floats per staging row (256-byte aligned).
    row_stride: usize,
}

/// Owns the offscreen targets and the compute pipeline for one window.
pub struct BackdropBlur {
    pipeline: ComputePipeline,
    layout: BindGroupLayout,
    targets: Option<PassTargets>,
    image: Option<ImageData>,
    sharp: Option<ImageData>,
    sigma: f32,
    var_pipeline: ComputePipeline,
    var_layout: BindGroupLayout,
    var_targets: Option<VarTargets>,
    busy: std::sync::Arc<std::sync::Mutex<BusyGrid>>,
    frames: u64,
}

impl BackdropBlur {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("tontooui backdrop blur"),
            source: ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tontooui backdrop blur"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("tontooui backdrop blur"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("tontooui backdrop blur"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        let var_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("tontooui backdrop variance"),
            source: ShaderSource::Wgsl(VARIANCE_SHADER.into()),
        });
        let var_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tontooui backdrop variance"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::R32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let var_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("tontooui backdrop variance"),
            bind_group_layouts: &[Some(&var_layout)],
            immediate_size: 0,
        });
        let var_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("tontooui backdrop variance"),
            layout: Some(&var_pipeline_layout),
            module: &var_shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            pipeline,
            layout,
            targets: None,
            image: None,
            sharp: None,
            sigma: BACKDROP_SIGMA,
            var_pipeline,
            var_layout,
            var_targets: None,
            busy: std::sync::Arc::new(std::sync::Mutex::new(BusyGrid::default())),
            frames: 0,
        }
    }

    pub fn set_sigma(&mut self, sigma: f32) {
        self.sigma = sigma.max(0.0);
    }

    /// Current capture target size, or `None` before the first
    /// `ensure_size`.
    pub fn size(&self) -> Option<(u32, u32)> {
        self.targets
            .as_ref()
            .map(|t| (t.content.width(), t.content.height()))
    }

    /// Unregister the previous outputs before the targets are replaced.
    pub fn take_image(&mut self, renderer: &mut Renderer) -> Option<ImageData> {
        if let Some(sharp) = self.sharp.take() {
            renderer.unregister_texture(sharp);
        }
        let img = self.image.take()?;
        renderer.unregister_texture(img.clone());
        Some(img)
    }

    pub fn ensure_size(&mut self, device: &Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let needs = match &self.targets {
            Some(t) => t.content.width() != width || t.content.height() != height,
            None => true,
        };
        if !needs {
            return;
        }
        let content = create_target(
            device,
            "tontooui backdrop content",
            width,
            height,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
        );
        let temp = create_target(
            device,
            "tontooui backdrop temp",
            width,
            height,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let output = create_target(
            device,
            "tontooui backdrop blurred",
            width,
            height,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
        );
        let content_view = content.create_view(&Default::default());
        let temp_view = temp.create_view(&Default::default());
        let output_view = output.create_view(&Default::default());
        // TextureView keeps the temp texture alive after this move.
        drop(temp);
        self.targets = Some(PassTargets {
            content,
            content_view,
            temp_view,
            output,
            output_view,
        });
        self.image = None;
        self.sharp = None;
        // Variance map plus readback staging for automatic frost.
        let tiles_x = width.div_ceil(BUSY_TILE);
        let tiles_y = height.div_ceil(BUSY_TILE);
        let var_tex = device.create_texture(&TextureDescriptor {
            label: Some("tontooui backdrop variance"),
            size: wgpu::Extent3d {
                width: tiles_x,
                height: tiles_y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        // Rows pad to 256 bytes for texture-to-buffer copies.
        let row_stride = (tiles_x as usize).div_ceil(64) * 64;
        let staging = device.create_buffer(&BufferDescriptor {
            label: Some("tontooui backdrop variance staging"),
            size: (row_stride * tiles_y as usize * 4) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.var_targets = Some(VarTargets {
            view: var_tex.create_view(&Default::default()),
            tex: var_tex,
            tiles_x,
            tiles_y,
            staging,
            row_stride,
        });
        if let Ok(mut busy) = self.busy.lock() {
            *busy = BusyGrid {
                width,
                height,
                tiles_x,
                tiles_y,
                tiles: vec![0.0; tiles_x as usize * tiles_y as usize],
            };
        }
    }

    /// Shared busyness grid handle for `ImageLoader` (glass queries
    /// it per rect; refreshed about twice per second).
    pub fn busy_handle(&self) -> std::sync::Arc<std::sync::Mutex<BusyGrid>> {
        self.busy.clone()
    }

    /// View Vello renders the backdrop capture into.
    pub fn content_view(&self) -> &TextureView {
        &self
            .targets
            .as_ref()
            .expect("backdrop size ensured")
            .content_view
    }

    /// Horizontal then vertical gaussian over `content` into `output`.
    /// Every `BUSY_EVERY_N_FRAMES`-th run also refreshes the variance
    /// map (blocking briefly on a tiny readback, about twice a second).
    pub fn run(&mut self, device: &Device, queue: &Queue) {
        let Some(t) = self.targets.as_ref() else {
            return;
        };
        let width = t.content.width();
        let height = t.content.height();
        let sigma = self.sigma;
        let h = bind_pass(
            device,
            &self.layout,
            &t.content_view,
            &t.temp_view,
            width,
            height,
            1,
            0,
            sigma,
        );
        let v = bind_pass(
            device,
            &self.layout,
            &t.temp_view,
            &t.output_view,
            width,
            height,
            0,
            1,
            sigma,
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("tontooui backdrop blur h"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &h.0, &[]);
            pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("tontooui backdrop blur v"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &v.0, &[]);
            pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
        }
        self.frames = self.frames.wrapping_add(1);
        let sample = self.frames % BUSY_EVERY_N_FRAMES == 0;
        if sample {
            self.dispatch_variance(device, &mut encoder, width, height);
        }
        queue.submit(Some(encoder.finish()));
        if sample {
            self.readback_variance(device);
        }
    }

    /// Queue one variance dispatch plus the staging copy.
    fn dispatch_variance(
        &self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
    ) {
        let (Some(t), Some(vt)) = (self.targets.as_ref(), self.var_targets.as_ref()) else {
            return;
        };
        let params = VParams {
            width,
            height,
            tiles_x: vt.tiles_x,
            tiles_y: vt.tiles_y,
        };
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("tontooui backdrop variance params"),
            size: std::mem::size_of::<VParams>() as u64,
            usage: BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(&params.to_bytes());
        buffer.unmap();
        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("tontooui backdrop variance"),
            layout: &self.var_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&t.content_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&vt.view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("tontooui backdrop variance"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.var_pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(vt.tiles_x, vt.tiles_y, 1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &vt.tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &vt.staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(vt.row_stride as u32 * 4),
                    rows_per_image: Some(vt.tiles_y),
                },
            },
            wgpu::Extent3d {
                width: vt.tiles_x,
                height: vt.tiles_y,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Block briefly on the tiny staging readback and publish the
    /// grid. Skips silently when mapping fails so a frame never dies
    /// on measurement.
    fn readback_variance(&mut self, device: &Device) {
        let Some(vt) = self.var_targets.as_ref() else {
            return;
        };
        let (tiles_x, tiles_y, row_stride) = (vt.tiles_x, vt.tiles_y, vt.row_stride);
        let (width, height) = (tiles_x, tiles_y);
        let slice = vt.staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result.is_ok());
        });
        // Bounded wait: a late map just skips this sample.
        let wait = wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_millis(100)),
        };
        if device.poll(wait).is_err() {
            return;
        }
        if rx.recv().unwrap_or(false) {
            let data = slice.get_mapped_range();
            let mut tiles = vec![0.0f32; tiles_x as usize * tiles_y as usize];
            for ty in 0..tiles_y as usize {
                for tx in 0..tiles_x as usize {
                    let off = (ty * row_stride + tx) * 4;
                    let bytes: [u8; 4] = data[off..off + 4].try_into().unwrap_or([0; 4]);
                    tiles[ty * tiles_x as usize + tx] = f32::from_le_bytes(bytes).max(0.0);
                }
            }
            drop(data);
            vt.staging.unmap();
            if let Ok(mut busy) = self.busy.lock() {
                busy.width = width;
                busy.height = height;
                busy.tiles_x = tiles_x;
                busy.tiles_y = tiles_y;
                busy.tiles = tiles;
            }
        }
    }

    /// Register (or refresh) the blurred output so views can sample it.
    pub fn sync_image(&mut self, renderer: &mut Renderer) -> Option<ImageData> {
        let t = self.targets.as_ref()?;
        let tex = t.output.clone();
        match &self.image {
            Some(img) => {
                renderer.mark_override_image_dirty(img);
                Some(img.clone())
            }
            None => {
                let img = renderer.register_texture(tex);
                self.image = Some(img.clone());
                Some(img)
            }
        }
    }

    pub fn image(&self) -> Option<&ImageData> {
        self.image.as_ref()
    }

    /// Sharp (unblurred) capture for the magnified glass center. Registered
    /// from the same `content` texture the blur reads, so it stays in sync
    /// frame by frame. `None` before `ensure_size`.
    pub fn sync_sharp_image(&mut self, renderer: &mut Renderer) -> Option<ImageData> {
        let t = self.targets.as_ref()?;
        let tex = t.content.clone();
        match &self.sharp {
            Some(img) => {
                renderer.mark_override_image_dirty(img);
                Some(img.clone())
            }
            None => {
                let img = renderer.register_texture(tex);
                self.sharp = Some(img.clone());
                Some(img)
            }
        }
    }

    pub fn sharp_image(&self) -> Option<&ImageData> {
        self.sharp.as_ref()
    }
}

/// Fill `shape` with the blurred in-app backdrop when available.
/// Scene coordinates are physical px and the backdrop texture is the
/// full window physical size, so identity maps image pixel (0, 0) to
/// scene (0, 0).
pub fn fill_backdrop(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    shape: &impl vello::kurbo::Shape,
) {
    let Some(bd) = images.backdrop() else {
        return;
    };
    scene.fill(
        vello::peniko::Fill::NonZero,
        vello::kurbo::Affine::IDENTITY,
        &vello::peniko::Brush::Image(
            ImageBrush::new(bd.clone()).with_extend(Extend::Pad),
        ),
        None,
        shape,
    );
}

/// Liquid glass lens center: fills `shape` with the sharp (unblurred)
/// capture, zoomed by `zoom` around `center` (both in physical px).
/// The brush transform maps brush pixels onto the surface, so a `zoom`
/// above 1.0 samples a smaller backdrop region stretched over the shape
/// (content looks enlarged) while a `zoom` below 1.0 samples a larger
/// region compressed into the shape (content looks shrunk). Falls back to
/// nothing when no sharp capture is available.
pub fn fill_backdrop_lens(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    shape: &impl vello::kurbo::Shape,
    center: Point,
    zoom: f64,
) {
    let Some(sharp) = images.backdrop_sharp() else {
        return;
    };
    let z = if zoom <= 0.0 { 1.0 } else { zoom };
    // Brush -> surface: scale around the glass center so each surface
    // point samples away from (z < 1, minify) or toward (z > 1, magnify)
    // the center.
    let brush_transform = Affine::translate((center.x, center.y))
        * Affine::scale(z)
        * Affine::translate((-center.x, -center.y));
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &vello::peniko::Brush::Image(
            ImageBrush::new(sharp.clone())
                .with_extend(Extend::Pad)
                .with_quality(ImageQuality::High),
        ),
        Some(brush_transform),
        shape,
    );
}

/// Shared liquid glass body: clear zoomed center with only a thin
/// blurred rim band inside the outline. `rect`/`radius` are in physical px,
/// `zoom` is the lens zoom (below 1.0 minifies, above 1.0 magnifies) and
/// `edge_width` the rim band width in physical px. Bodies smaller than twice
/// the band fall back to a full blur fill.
pub fn fill_lens_glass(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    rect: &Rect,
    radius: f64,
    zoom: f64,
    edge_width: f64,
) {
    let body = RoundedRect::from_rect(*rect, radius);
    let min_side = (rect.x1 - rect.x0).min(rect.y1 - rect.y0);
    if edge_width <= 0.0 || min_side <= edge_width * 2.0 {
        fill_backdrop(scene, images, &body);
        return;
    }
    let center = Point::new((rect.x0 + rect.x1) * 0.5, (rect.y0 + rect.y1) * 0.5);
    fill_backdrop_lens(scene, images, &body, center, zoom);
    let inset = edge_width * 0.5;
    let ring = RoundedRect::new(
        rect.x0 + inset,
        rect.y0 + inset,
        rect.x1 - inset,
        rect.y1 - inset,
        (radius - inset).max(0.0),
    );
    stroke_backdrop_edge(scene, images, &ring, edge_width);
}

/// Light blur veil over the whole `shape`: the blurred capture at `alpha`
/// (0.0 transparent, 1.0 full frost). Layered under a full-strength edge
/// band it reads as frost everywhere with the strongest frost at the rim.
pub fn fill_backdrop_veil(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    shape: &impl vello::kurbo::Shape,
    alpha: f32,
) {
    let Some(bd) = images.backdrop() else {
        return;
    };
    if alpha <= 0.0 {
        return;
    }
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &vello::peniko::Brush::Image(
            ImageBrush::new(bd.clone())
                .with_extend(Extend::Pad)
                .with_alpha(alpha.clamp(0.0, 1.0)),
        ),
        None,
        shape,
    );
}

/// Blur veil alpha for the frosted glass type: heavy frost over the whole
/// body on top of the lens, just below full blur.
pub const GLASS_FROST_VEIL: f32 = 0.95;

/// Frosted liquid glass body: the lens (clear zoomed center, strong rim)
/// plus a heavy blur veil over everything, so behind shows through but
/// stays unrecognizable, with the strongest frost at the edge.
pub fn fill_frosted_glass(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    rect: &Rect,
    radius: f64,
    zoom: f64,
    edge_width: f64,
) {
    fill_lens_glass(scene, images, rect, radius, zoom, edge_width);
    let body = RoundedRect::from_rect(*rect, radius);
    fill_backdrop_veil(scene, images, &body, GLASS_FROST_VEIL);
}

/// Liquid glass edge: strokes `ring` with the blurred capture. Callers pass
/// a rounded rect inset by half the band width with a stroke width equal to
/// the band, so the blur sits fully inside the glass body (outer stroke
/// edge aligns with the body outline). Only the rim stays frosted while
/// the lens center above stays clear.
pub fn stroke_backdrop_edge(
    scene: &mut vello::Scene,
    images: &crate::renderer::images::ImageLoader<'_>,
    ring: &RoundedRect,
    width: f64,
) {
    let Some(bd) = images.backdrop() else {
        return;
    };
    if width <= 0.0 {
        return;
    }
    scene.stroke(
        &Stroke::new(width),
        Affine::IDENTITY,
        &vello::peniko::Brush::Image(
            ImageBrush::new(bd.clone()).with_extend(Extend::Pad),
        ),
        None,
        ring,
    );
}

fn create_target(
    device: &Device,
    label: &str,
    width: u32,
    height: u32,
    usage: TextureUsages,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}

fn bind_pass(
    device: &Device,
    layout: &BindGroupLayout,
    src: &TextureView,
    dst: &TextureView,
    width: u32,
    height: u32,
    dir_x: u32,
    dir_y: u32,
    sigma: f32,
) -> (BindGroup, Buffer) {
    let params = Params {
        width,
        height,
        dir_x,
        dir_y,
        sigma,
        pad0: 0.0,
        pad1: 0.0,
        pad2: 0.0,
    };
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("tontooui backdrop params"),
        size: std::mem::size_of::<Params>() as u64,
        usage: BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    buffer
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(&params.to_bytes());
    buffer.unmap();
    let bind = device.create_bind_group(&BindGroupDescriptor {
        label: Some("tontooui backdrop blur"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(src),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(dst),
            },
            BindGroupEntry {
                binding: 2,
                resource: buffer.as_entire_binding(),
            },
        ],
    });
    (bind, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frost_curve_plain_busy_smooth() {
        assert_eq!(frost_for_busy(0.0), 0.0);
        assert_eq!(frost_for_busy(BUSY_LO), 0.0);
        assert_eq!(frost_for_busy(BUSY_HI), 1.0);
        assert_eq!(frost_for_busy(BUSY_HI * 10.0), 1.0);
        let mid = frost_for_busy((BUSY_LO + BUSY_HI) / 2.0);
        assert!((mid - 0.5).abs() < 1e-6, "mid was {mid}");
        // Smoothstep rushes the middle, eases the ends.
        let q = frost_for_busy(BUSY_LO + (BUSY_HI - BUSY_LO) * 0.25);
        assert!(q > 0.1 && q < 0.25, "q was {q}");
    }

    #[test]
    fn grid_averages_overlapped_tiles() {
        let grid = BusyGrid {
            width: 64,
            height: 64,
            tiles_x: 4,
            tiles_y: 4,
            tiles: (0..16).map(|i| i as f32 / 100.0).collect(),
        };
        // Empty grid reports nothing (glass stays clear).
        assert_eq!(BusyGrid::default().amount_at(0.0, 0.0, 64.0, 64.0), None);
        // Single tile (tile (1,1) holds 0.05).
        assert_eq!(grid.amount_at(16.0, 16.0, 32.0, 32.0), Some(0.05));
        // Whole grid: mean of 0.00..0.15.
        let all = grid.amount_at(0.0, 0.0, 64.0, 64.0).expect("mean");
        assert!((all - 0.075).abs() < 1e-6, "mean was {all}");
        // Out-of-range clamps instead of failing.
        assert!(grid.amount_at(-100.0, -100.0, 1000.0, 1000.0).is_some());
    }

    #[test]
    fn variance_tiles_cover_content() {
        // 900x620 at tile 16: ceil math for partial edge tiles.
        assert_eq!(900u32.div_ceil(BUSY_TILE), 57);
        assert_eq!(620u32.div_ceil(BUSY_TILE), 39);
    }
}
