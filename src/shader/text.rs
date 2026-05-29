use std::borrow::Cow;

use crate::{
    pipeline::{SharedBindGroupData, text_atlas::AtlasData},
    primitive::text::{VERTICES_PER_GLYPH, VertexData},
    shader::uniforms_bind_group_layout,
};

// TODO: make these dynamic based on the texture atlas size and the number of glyphs
pub const TEXT_ATLAS_SIZE: u32 = 2048;
pub const MAX_GLYPHS: u32 = 80000;

#[derive(Debug, Clone, Copy)]
pub struct TextShader;

impl TextShader {
    #[must_use]
    pub fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text.create_pipeline.layout"),
            bind_group_layouts: &[
                Some(&Self::create_bind_group_layout(device)),
                Some(&uniforms_bind_group_layout(device)),
            ],
            immediate_size: 0,
        });

        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(concat!(include_str!("text.wgsl"),))),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text.create_pipeline.render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (5 * std::mem::size_of::<f32>()) as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 2 * std::mem::size_of::<f32>() as u64,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 4 * std::mem::size_of::<f32>() as u64,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    }

    #[must_use]
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text.bind_group_layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    #[must_use]
    pub fn create_bind_group(
        device: &wgpu::Device,
        shared_bind_group_data: &SharedBindGroupData,
        atlas_data: &AtlasData,
        copy_texture: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text.texture_atlas_bg"),
            layout: &shared_bind_group_data.bgl_text,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &atlas_data
                            .texture_atlas
                            .create_view(&wgpu::TextureViewDescriptor {
                                base_mip_level: 0,
                                mip_level_count: Some(1),
                                ..Default::default()
                            }),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&copy_texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            base_mip_level: 0,
                            mip_level_count: Some(1),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shared_bind_group_data.sampler),
                },
            ],
        })
    }

    // TODO: make this dynamic based on the number of glyphs
    #[must_use]
    pub fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text.vertex_buffer"),
            size: std::mem::size_of::<f32>() as u64
                * std::mem::size_of::<VertexData>() as u64
                * u64::from(VERTICES_PER_GLYPH)
                * u64::from(MAX_GLYPHS),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
