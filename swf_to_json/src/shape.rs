use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    path::Path,
};

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use swf::{CharacterId, DefineBitsLossless, GradientInterpolation, Tag};
use tracing::error;
use wgpu::util::DeviceExt;

use crate::render::{
    bitmap::{CompressedBitmap, decoder::decode_define_bits_jpeg_dimensions},
    create_render_pipelines, create_texture_and_view, get_device_and_queue,
    mesh::{GradientUniform, VertexColor, VertexPosition, ViewMatrix},
    tessellator::{DrawType, Gradient, ShapeTessellator},
};

/// 制作多大得渐变纹理，
const GRADIENT_SIZE: usize = 256;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Offset {
    x: f32,
    y: f32,
}

pub fn parse_shape_generate_img(
    tags: &Vec<Tag<'_>>,
    shape_offset: &mut BTreeMap<CharacterId, Offset>,
    scale: f32,
    special_scale: HashMap<CharacterId, f32>,
    path: &Path,
) -> anyhow::Result<()> {
    // 初始化设备和队列
    let (device, queue) = get_device_and_queue()?;
    // 准备渲染管线
    let (
        (color_render_pipeline, view_bind_group_layout),
        (gradient_render_pipeline, gradient_bind_group_layout),
        (bitmap_render_pipeline, bitmap_bind_group_layout),
    ) = create_render_pipelines(&device);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let mut bitmaps = HashMap::new();
    for tag in tags {
        match tag {
            swf::Tag::DefineBitsJpeg3(jpeg_data) => {
                let (width, height) = decode_define_bits_jpeg_dimensions(jpeg_data.data).unwrap();

                bitmaps.insert(
                    jpeg_data.id,
                    CompressedBitmap::Jpeg {
                        data: jpeg_data.data.to_vec(),
                        alpha: Some(jpeg_data.alpha_data.to_vec()),
                        width,
                        height,
                    },
                );
            }
            swf::Tag::DefineBitsLossless(bit_loss_less) => {
                bitmaps.insert(
                    bit_loss_less.id,
                    CompressedBitmap::Lossless(DefineBitsLossless {
                        version: bit_loss_less.version,
                        id: bit_loss_less.id,
                        format: bit_loss_less.format,
                        width: bit_loss_less.width,
                        height: bit_loss_less.height,
                        data: Cow::Owned(bit_loss_less.data.clone().into_owned()),
                    }),
                );
            }
            _ => {
                continue;
            }
        }
    }

    let shapes: Vec<_> = tags
        .iter()
        .filter(|tag| matches!(tag, swf::Tag::DefineShape(_)))
        .map(|tag| {
            if let swf::Tag::DefineShape(shape) = tag {
                shape
            } else {
                unreachable!()
            }
        })
        .collect();

    let pb = ProgressBar::new(shapes.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(format!("正在生成Shape纹理,缩放:{}", scale));

    let mut tessellator = ShapeTessellator::default();

    for shape in shapes {
        pb.inc(1);
        let lyon_mesh = tessellator.tessellate_shape(shape.into(), &bitmaps);
        let mut gradient_textures = Vec::new();
        let gradients = lyon_mesh.gradients;
        for gradient in gradients {
            generation_gradient(&mut gradient_textures, gradient, &device, &queue);
        }

        let scale = if let Some(special_scale) = special_scale.get(&shape.id) {
            scale * special_scale
        } else {
            scale
        };
        pb.set_message(format!("正在生成Shape:{}纹理,缩放:{}", shape.id, scale));

        // 因为SWF中的图形坐标和WebGPU空间中的坐标Y轴相反，且WebGPU中的原点在中心，
        // 而SWF中的坐标原点在左上角，所以需要将SWF的顶点坐标的左下角移动到WebGPU的坐标的原点，
        // 这样整个图形在为进行view变换时就会靠近紧贴X轴和Y轴位于第一象限的位置。
        // 在进行view变换时，Y坐标将会被反转，从而实现SWF中图形的Y轴方向与WebGPU中的Y轴方向一致。
        // 这样，SWF中的图形就会映射到WebGPU中的第四象限，且会进行其次单位像左上角进行平移，
        // 从而将SWF中图形的左上角移动到WebGPU中坐标的(-1, 1)位置，也就是WebGPU中的左上角。
        // 这样图形就会恰好处于WebGPU坐标中的正中心位置。
        let bound = &shape.shape_bounds;
        let x_min = bound.x_min.to_pixels() as f32;
        let y_min = bound.y_min.to_pixels() as f32;
        let x_max = bound.x_max.to_pixels() as f32;
        let y_max = bound.y_max.to_pixels() as f32;

        shape_offset.insert(
            shape.id,
            Offset {
                x: ((x_min + x_max) / 2.0),
                y: ((y_min + y_max) / 2.0),
            },
        );

        let draw_offset_x = 0.0 - x_min;
        let draw_offset_y = 0.0 - y_min;

        let width = bound.width().to_pixels() as f32;
        let height = bound.height().to_pixels() as f32;

        let size = wgpu::Extent3d {
            width: (width * scale) as u32 + 1,
            height: (height * scale) as u32 + 1,
            depth_or_array_layers: 1,
        };

        let (_, sample_view) = create_texture_and_view(
            &device,
            Some("多重采样抗锯齿纹理"),
            size,
            4,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let (main_texture, main_view) = create_texture_and_view(
            &device,
            Some("主纹理"),
            size,
            1,
            wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &sample_view,
                resolve_target: Some(&main_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        let view_bind_group =
            ViewMatrix::bind_group(&device, &view_bind_group_layout, width, height);

        for draw in lyon_mesh.draws {
            let indices = draw.indices;
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("索引缓冲区"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            match draw.draw_type {
                DrawType::Color => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexColor::new(
                            [vertex.x + draw_offset_x, vertex.y + draw_offset_y],
                            [
                                vertex.color.r as f32 / 255.0,
                                vertex.color.g as f32 / 255.0,
                                vertex.color.b as f32 / 255.0,
                                vertex.color.a as f32 / 255.0,
                            ],
                        );
                        meshes.push(mesh);
                    }

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(&color_render_pipeline);
                    render_pass.set_bind_group(0, &view_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                DrawType::Gradient { matrix, gradient } => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexPosition::new([vertex.x, vertex.y]);
                        meshes.push(mesh);
                    }
                    let transform = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [draw_offset_x, draw_offset_y, 0.0, 1.0],
                    ];

                    let transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("变换缓冲区"),
                            contents: bytemuck::cast_slice(&[transform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let gradient_texture = gradient_textures.get(gradient).unwrap();
                    let gradient_uniform = GradientUniform {
                        ..gradient_texture.1
                    };
                    // 将matrix 转为[f32; 4; 4]
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&matrix[2]);

                    let texture_transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("纹理变换缓冲区"),
                            contents: bytemuck::cast_slice(&[texture_transform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let gradient_uniform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("渐变缓冲区"),
                            contents: bytemuck::cast_slice(&[gradient_uniform]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &gradient_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&gradient_texture.0),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: texture_transform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: gradient_uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: transform_buffer.as_entire_binding(),
                            },
                        ],
                        label: Some("gradient texture bind group"),
                    });

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(&gradient_render_pipeline);
                    render_pass.set_bind_group(0, &view_bind_group, &[]);
                    render_pass.set_bind_group(1, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                DrawType::Bitmap(bitmap) => {
                    let mut meshes = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices {
                        let mesh = VertexPosition::new([vertex.x, vertex.y]);
                        meshes.push(mesh);
                    }

                    let transform = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [draw_offset_x, draw_offset_y, 0.0, 1.0],
                    ];

                    let transform_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("transform buffer"),
                            contents: bytemuck::cast_slice(&transform),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("顶点缓冲区"),
                            contents: bytemuck::cast_slice(&meshes),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&bitmap.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&bitmap.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&bitmap.matrix[2]);
                    if let Some(compressed_bitmap) = bitmaps.get(&bitmap.bitmap_id) {
                        let decoded = match compressed_bitmap.decode() {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                error!("Failed to decode bitmap: {:?}", e);
                                continue;
                            }
                        };
                        let bitmap = decoded.into_rgba();
                        let texture = device.create_texture_with_data(
                            &queue,
                            &wgpu::TextureDescriptor {
                                label: Some("Bitmap Texture"),
                                size: wgpu::Extent3d {
                                    width: bitmap.width(),
                                    height: bitmap.height(),
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                                view_formats: &[],
                            },
                            wgpu::util::TextureDataOrder::LayerMajor,
                            bitmap.data(),
                        );

                        let texture_transform_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Texture Transform Buffer"),
                                contents: bytemuck::cast_slice(&texture_transform),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });

                        let texture_view =
                            texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

                        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Bitmap Bind Group"),
                            layout: &bitmap_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: texture_transform_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: transform_buffer.as_entire_binding(),
                                },
                            ],
                        });
                        render_pass.set_pipeline(&bitmap_render_pipeline);
                        render_pass.set_bind_group(0, &view_bind_group, &[]);
                        render_pass.set_bind_group(1, &bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..indices.len() as _, 0, 0..1);
                    }
                }
            }
        }

        drop(render_pass);
        queue.submit(std::iter::once(encoder.finish()));
        let unpadded_byte_per_row = size.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_byte_per_row_padding = (align - unpadded_byte_per_row % align) % align;
        let padded_byte_per_row = unpadded_byte_per_row + padded_byte_per_row_padding;
        // 将输出纹理提取到缓冲区中
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (padded_byte_per_row * size.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &main_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_per_row),
                    rows_per_image: None,
                },
            },
            size,
        );
        // 从缓冲区数据到image
        queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        let _ = device.poll(wgpu::PollType::Wait);
        let _ = receiver.recv().expect("MPSC channel must not fail");
        let data = buffer_slice.get_mapped_range();

        let mut bytes = Vec::with_capacity(size.height as usize);
        for pixel in data.chunks_exact(padded_byte_per_row as usize) {
            bytes.extend_from_slice(&pixel[..unpadded_byte_per_row as usize]);
        }
        let image_buffer = image::RgbaImage::from_raw(size.width, size.height, bytes)
            .expect("Retrieved texture buffer must be a valid RgbaImage");
        image_buffer
            .save(path.join(format!("{}.png", shape.id)))
            .expect("Failed to save image");
    }

    Ok(())
}

fn generation_gradient(
    gradient_textures: &mut Vec<(wgpu::TextureView, GradientUniform)>,
    gradient: Gradient,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    let colors = if gradient.records.is_empty() {
        vec![0; GRADIENT_SIZE * 4]
    } else {
        let mut colors = vec![0; GRADIENT_SIZE * 4];
        let convert = if gradient.interpolation == GradientInterpolation::LinearRgb {
            |color| srgb_to_linear(color / 255.0) * 255.0
        } else {
            |color| color
        };

        for t in 0..GRADIENT_SIZE {
            let mut last = 0;
            let mut next = 0;
            for (i, record) in gradient.records.iter().enumerate().rev() {
                if (record.ratio as usize) < t {
                    last = i;
                    next = (i + 1).min(gradient.records.len() - 1);
                    break;
                }
            }
            assert!(last == next || last + 1 == next);
            let last_record = &gradient.records[last];
            let next_record = &gradient.records[next];
            let factor = if next == last {
                0.0
            } else {
                (t as f32 - last_record.ratio as f32)
                    / (next_record.ratio as f32 - last_record.ratio as f32)
            };

            colors[t * 4] = lerp(
                convert(last_record.color.r as f32),
                convert(next_record.color.r as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 1] = lerp(
                convert(last_record.color.g as f32),
                convert(next_record.color.g as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 2] = lerp(
                convert(last_record.color.b as f32),
                convert(next_record.color.b as f32),
                factor,
            ) as u8;
            colors[(t * 4) + 3] = lerp(
                last_record.color.a as f32,
                next_record.color.a as f32,
                factor,
            ) as u8;
        }
        colors
    };

    let size = wgpu::Extent3d {
        width: GRADIENT_SIZE as u32,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("Shape"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &colors[..],
    );
    let gradient_uniform = GradientUniform::from(gradient);
    gradient_textures.push((
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
        gradient_uniform,
    ));
}

/// Converts an RGBA color from sRGB space to linear color space.
fn srgb_to_linear(color: f32) -> f32 {
    if color <= 0.04045 {
        color / 12.92
    } else {
        f32::powf((color + 0.055) / 1.055, 2.4)
    }
}

/// 线性插值
fn lerp(a: f32, b: f32, factor: f32) -> f32 {
    a + (b - a) * factor
}
