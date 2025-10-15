pub(crate) mod bitmap;
pub mod filter;
mod matrix;
pub mod mesh;
pub mod shape_utils;
pub(crate) mod tessellator;

use anyhow::anyhow;
use wgpu::VertexBufferLayout;

use crate::render::mesh::{VertexColor, VertexPosition};

fn create_wgpu_instance() -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in wgpu::Backends::all() {
        if let Some(instance) = try_wgpu_backend(backend) {
            return Ok((instance, backend));
        }
    }
    Err(anyhow!("没有找到可用渲染后端"))
}

fn try_wgpu_backend(backends: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backends).is_empty() {
        None
    } else {
        Some(instance)
    }
}

pub(crate) fn get_device_and_queue() -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
    let (instance, _backend) = create_wgpu_instance()?;

    let (_adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
        &instance,
        None,
        wgpu::PowerPreference::HighPerformance,
    ))
    .map_err(|e| anyhow!(e.to_string()))?;

    Ok((device, queue))
}

type Error = Box<dyn std::error::Error>;

pub(crate) async fn request_adapter_and_device(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'static>>,
    power_preference: wgpu::PowerPreference,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await
        .inspect_err(|e| {
            eprintln!("请求适配器失败: {:?}", e);
        })?;

    let mut features = Default::default();

    let try_features = [
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
        wgpu::Features::TEXTURE_COMPRESSION_BC,
        wgpu::Features::FLOAT32_FILTERABLE,
    ];

    for feature in try_features {
        if adapter.features().contains(feature) {
            features |= feature;
        }
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("设备"),
            required_features: features,
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        })
        .await?;
    Ok((adapter, device, queue))
}

pub(crate) fn create_texture_and_view(
    device: &wgpu::Device,
    label: Option<&str>,
    size: wgpu::Extent3d,
    sample_count: u32,
    usage: wgpu::TextureUsages,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

pub(crate) fn create_render_pipelines(
    device: &wgpu::Device,
) -> (
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
    (wgpu::RenderPipeline, wgpu::BindGroupLayout),
) {
    let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Color Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/color.wgsl").into()),
    });

    let view_matrix_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Color Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout],
        push_constant_ranges: &[],
    });

    let color_render_pipeline = create_render_pipeline(
        device,
        Some("Color Render Pipeline"),
        &color_shader,
        &render_pipeline_layout,
        VertexColor::desc(),
    );

    let gradient_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Gradient Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/gradient.wgsl").into()),
    });

    let gradient_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gradient Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gradient Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout, &gradient_bind_group_layout],
        push_constant_ranges: &[],
    });

    let gradient_render_pipeline = create_render_pipeline(
        device,
        Some("Gradient Render Pipeline"),
        &gradient_shader,
        &render_pipeline_layout,
        VertexPosition::desc(),
    );

    let bitmap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bitmap Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render/shaders/bitmap.wgsl").into()),
    });

    let bitmap_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bitmap Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bitmap Render Pipeline Layout"),
        bind_group_layouts: &[&view_matrix_bind_group_layout, &bitmap_bind_group_layout],
        push_constant_ranges: &[],
    });
    let bitmap_render_pipeline = create_render_pipeline(
        device,
        Some("Bitmap Render Pipeline"),
        &bitmap_shader,
        &render_pipeline_layout,
        VertexPosition::desc(),
    );

    (
        (color_render_pipeline, view_matrix_bind_group_layout),
        (gradient_render_pipeline, gradient_bind_group_layout),
        (bitmap_render_pipeline, bitmap_bind_group_layout),
    )
}

fn create_render_pipeline(
    device: &wgpu::Device,
    label: Option<&str>,
    shader: &wgpu::ShaderModule,
    render_pipeline_layout: &wgpu::PipelineLayout,
    buffer: VertexBufferLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[buffer],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}
