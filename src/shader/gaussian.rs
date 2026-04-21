use std::borrow::Cow;

use crate::shader::{create_sampler, to_texture_view, uniforms_bind_group_layout};

pub struct GaussianShader;

impl GaussianShader {
    pub fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gaussian.create_pipeline.layout"),
            bind_group_layouts: &[
                &Self::create_bind_group_layout_old(device),
                &uniforms_bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gaussian.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                concat!(include_str!("gaussian.wgsl"),),
            )),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gaussian.create_pipeline.render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("gaussian_blur"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        })
    }

    fn create_bind_group_layout_old(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gaussian.bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        input_texture: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        let sampler = create_sampler(device);
        let input_texture_view = to_texture_view(input_texture);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gaussian.bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }
}
