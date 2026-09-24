use vello::peniko::{Extend, ImageBrush, ImageData};
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

/// Owns the offscreen targets and the compute pipeline for one window.
pub struct BackdropBlur {
    pipeline: ComputePipeline,
    layout: BindGroupLayout,
    targets: Option<PassTargets>,
    image: Option<ImageData>,
    sigma: f32,
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
        Self {
            pipeline,
            layout,
            targets: None,
            image: None,
            sigma: BACKDROP_SIGMA,
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

    /// Unregister the previous output before the targets are replaced.
    pub fn take_image(&mut self, renderer: &mut Renderer) -> Option<ImageData> {
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
            width,
            height,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let temp = create_target(
            device,
            width,
            height,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let output = create_target(
            device,
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
    pub fn run(&self, device: &Device, queue: &Queue) {
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
        queue.submit(Some(encoder.finish()));
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

fn create_target(device: &Device, width: u32, height: u32, usage: TextureUsages) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("tontooui backdrop"),
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
