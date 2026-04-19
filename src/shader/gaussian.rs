use std::borrow::Cow;

use crate::shader::{RenderShaderData, uniforms_bind_group_layout};

pub struct GaussianShader;

impl GaussianShader {
    pub fn compile(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniforms: &wgpu::Buffer,
        input_texture: &wgpu::Texture,
        output_texture: &wgpu::Texture,
    ) -> (RenderShaderData, RenderShaderData) {
        let (horizontal_pipeline, vertical_pipeline) = Self::create_pipeline(device, format);
        // let (bind_group, uniform_bind_group) =
        //     Self::create_bind_group(device, &pipeline, uniforms, output_texture);
        let (horizontal_bind_group, horizontal_uniform_bind_group) =
            Self::create_bind_group(device, &horizontal_pipeline, uniforms, input_texture);
        let (vertical_bind_group, vertical_uniform_bind_group) =
            Self::create_bind_group(device, &vertical_pipeline, uniforms, output_texture);

        // let horizontal_uniform_bind_group =
        //     Self::create_uniforms_bind_group(device, &horizontal_pipeline, uniforms);
        // let vertical_uniform_bind_group =
        //     Self::create_uniforms_bind_group(device, &vertical_pipeline, uniforms);

        let horizontal_shader = RenderShaderData {
            pipeline: horizontal_pipeline,
            bind_group: horizontal_bind_group,
            uniform_bind_group: horizontal_uniform_bind_group,
        };
        let vertical_shader = RenderShaderData {
            pipeline: vertical_pipeline,
            bind_group: vertical_bind_group,
            uniform_bind_group: vertical_uniform_bind_group,
        };
        (horizontal_shader, vertical_shader)
    }
    pub fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> (wgpu::RenderPipeline, wgpu::RenderPipeline) {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gaussian.create_pipeline.layout"),
            bind_group_layouts: &[
                &Self::create_bind_group_layout(device),
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

        let horizontal_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                entry_point: Some("horizontal_pass"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let vertical_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                entry_point: Some("vertical_pass"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        (horizontal_pipeline, vertical_pipeline)
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
        pipeline: &wgpu::RenderPipeline,
        uniforms: &wgpu::Buffer,
        input_texture: &wgpu::Texture,
        // output_texture: &wgpu::Texture,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let sampler = create_sampler(device);
        let input_texture_view = to_texture_view(input_texture);
        // let output_texture_view = to_texture_view(output_texture);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gaussian.bind_group"),
            layout: &pipeline.get_bind_group_layout(0),
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
        });

        let uniform_bind_group_layout = pipeline.get_bind_group_layout(1);
        let uniform_bind_group =
            crate::shader::uniforms_bind_group(device, &uniform_bind_group_layout, uniforms);
        (bind_group, uniform_bind_group)
        // bind_group
    }
}

pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("gaussian.sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

pub fn to_texture_view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("gaussian.texture_view"),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::D2),
        // usage: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
        usage: None,
    })
}
