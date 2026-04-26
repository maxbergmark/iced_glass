use std::collections::{HashMap, HashSet};

use crate::{
    shader::{
        MIP_LEVEL_COUNT, create_sampler,
        downsample::DownsampleShader,
        fragment::FragmentShader,
        gaussian::GaussianShader,
        text::{TEXT_ATLAS_SIZE, TextShader},
        texture_bind_groups, uniforms_bind_group, uniforms_bind_group_layout,
    },
    uniforms::Uniforms,
};

pub struct Pipeline {
    // Shared, created once:
    pub device_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub bgl_textures: wgpu::BindGroupLayout, // group 0 layout
    pub bgl_uniforms: wgpu::BindGroupLayout, // group 1 layout
    pub bgl_text: wgpu::BindGroupLayout,     // group 0 layout

    pub downsample_pipeline: wgpu::RenderPipeline,
    pub blur_pipeline: wgpu::RenderPipeline,
    pub fragment_pipeline: wgpu::RenderPipeline,
    pub text_pipeline: wgpu::RenderPipeline,

    // One entry per GlassContainer:
    instances: std::collections::HashMap<u64, Instance>,
    live_this_frame: std::collections::HashSet<u64>,

    pub text_instances: std::collections::HashMap<u64, TextInstance>,
    pub live_text_this_frame: std::collections::HashSet<u64>,
}
pub struct Instance {
    pub tex_a: wgpu::Texture,
    pub tex_b: wgpu::Texture,
    pub uniforms_h: wgpu::Buffer,
    pub uniforms_v: wgpu::Buffer,
    pub uniform_bg_h: wgpu::BindGroup,
    pub uniform_bg_v: wgpu::BindGroup,
    pub tex_a_bg: Vec<wgpu::BindGroup>,
    pub tex_b_bg: Vec<wgpu::BindGroup>,
    pub size: wgpu::Extent3d,
}

pub struct TextInstance {
    pub texture_atlas: wgpu::Texture,
    pub vertex_buffer: wgpu::Buffer,
    pub tex_a: wgpu::Texture,
    pub tex_b: wgpu::Texture,
    pub uniforms_h: wgpu::Buffer,
    pub uniforms_v: wgpu::Buffer,
    pub uniform_bg_h: wgpu::BindGroup,
    pub uniform_bg_v: wgpu::BindGroup,
    pub tex_a_bg: Vec<wgpu::BindGroup>,
    pub tex_b_bg: Vec<wgpu::BindGroup>,
    pub texture_atlas_bg: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}
impl iced::widget::shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        let downsample_pipeline = DownsampleShader::create_pipeline(device, format);
        let blur_pipeline = GaussianShader::create_pipeline(device, format);
        let fragment_pipeline = FragmentShader::create_pipeline(device, format);
        let text_pipeline = TextShader::create_pipeline(device, format);

        Self {
            device_format: format,
            sampler: create_sampler(device),
            bgl_textures: create_bgl_texture_layout(device),
            bgl_uniforms: uniforms_bind_group_layout(device),
            bgl_text: TextShader::create_bind_group_layout(device),
            downsample_pipeline,
            blur_pipeline,
            fragment_pipeline,
            text_pipeline,
            instances: HashMap::new(),
            live_this_frame: HashSet::new(),
            text_instances: HashMap::new(),
            live_text_this_frame: HashSet::new(),
        }
    }
}

pub fn create_bgl_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("gaussian.bind_group_layout"),
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
        ],
    })
}

pub fn create_textures(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::Texture) {
    let copy_to_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glass.copy"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: MIP_LEVEL_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let gaussian_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glass.gaussian"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: MIP_LEVEL_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    (copy_to_texture, gaussian_texture)
}

impl Pipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        width: u32,
        height: u32,
        scale: f32,
        uniforms: &Uniforms,
    ) {
        let needs_new = match self.instances.get(&id) {
            Some(inst) => inst.size.width != width || inst.size.height != height,
            None => true,
        };
        if needs_new {
            self.instances.insert(
                id,
                Instance::new(self, device, &self.bgl_textures, width, height),
            );
        }
        let inst = self.instances.get_mut(&id).unwrap();
        inst.copy_uniforms_to_device(queue, uniforms, scale);
        self.live_this_frame.insert(id);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_text_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        width: u32,
        height: u32,
        scale: f32,
        uniforms: &Uniforms,
    ) {
        let needs_new = match self.instances.get(&id) {
            Some(inst) => inst.size.width != width || inst.size.height != height,
            None => true,
        };
        if needs_new {
            self.text_instances.insert(
                id,
                TextInstance::new(self, device, &self.bgl_textures, width, height),
            );
        }
        let inst = self.text_instances.get_mut(&id).unwrap();
        inst.copy_uniforms_to_device(queue, uniforms, scale);
        self.live_this_frame.insert(id);
    }

    pub fn instance(&self, id: u64) -> &Instance {
        &self.instances[&id]
    }

    pub fn text_instance(&self, id: u64) -> &TextInstance {
        &self.text_instances[&id]
    }

    /// Call at the end of rendering each frame.
    pub fn gc(&mut self) {
        self.instances
            .retain(|id, _| self.live_this_frame.contains(id));
        self.live_this_frame.clear();
    }
}

impl Instance {
    pub fn new(
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bgl_textures: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let (copy_texture, gaussian_texture) =
            create_textures(device, pipeline.device_format, width, height);

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);

        let copy_texture_bg = texture_bind_groups(device, bgl_textures, &copy_texture);
        let gaussian_texture_bg = texture_bind_groups(device, bgl_textures, &gaussian_texture);

        let uniform_bg_h = uniforms_bind_group(device, &pipeline.bgl_uniforms, &uniforms_h);
        let uniform_bg_v = uniforms_bind_group(device, &pipeline.bgl_uniforms, &uniforms_v);

        Self {
            tex_a: copy_texture,
            tex_b: gaussian_texture,
            uniforms_h,
            uniforms_v,
            uniform_bg_h,
            uniform_bg_v,
            tex_a_bg: copy_texture_bg,
            tex_b_bg: gaussian_texture_bg,
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        }
    }

    pub fn copy_uniforms_to_device(&self, queue: &wgpu::Queue, uniforms: &Uniforms, scale: f32) {
        queue.write_buffer(
            &self.uniforms_h,
            0,
            bytemuck::bytes_of(&uniforms.to_raw([1.0, 0.0], scale)),
        );
        queue.write_buffer(
            &self.uniforms_v,
            0,
            bytemuck::bytes_of(&uniforms.to_raw([0.0, 1.0], scale)),
        );
    }
}

impl TextInstance {
    pub fn new(
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bgl_textures: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let (copy_texture, gaussian_texture) =
            create_textures(device, pipeline.device_format, width, height);

        let texture_atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text.texture_atlas"),
            size: wgpu::Extent3d {
                width: TEXT_ATLAS_SIZE,
                height: TEXT_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);

        let copy_texture_bg = texture_bind_groups(device, bgl_textures, &copy_texture);
        let gaussian_texture_bg = texture_bind_groups(device, bgl_textures, &gaussian_texture);

        let uniform_bg_h = uniforms_bind_group(device, &pipeline.bgl_uniforms, &uniforms_h);
        let uniform_bg_v = uniforms_bind_group(device, &pipeline.bgl_uniforms, &uniforms_v);

        let texture_atlas_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text.texture_atlas_bg"),
            layout: &pipeline.bgl_text,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.create_view(
                        &wgpu::TextureViewDescriptor {
                            base_mip_level: 0,
                            mip_level_count: Some(1),
                            ..Default::default()
                        },
                    )),
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
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text.vertex_buffer"),
            size: std::mem::size_of::<f32>() as u64 * 6 * 400, // TODO: increase this limit
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            texture_atlas,
            vertex_buffer,
            tex_a: copy_texture,
            tex_b: gaussian_texture,
            uniforms_h,
            uniforms_v,
            uniform_bg_h,
            uniform_bg_v,
            tex_a_bg: copy_texture_bg,
            tex_b_bg: gaussian_texture_bg,
            texture_atlas_bg,
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        }
    }

    pub fn copy_uniforms_to_device(&self, queue: &wgpu::Queue, uniforms: &Uniforms, scale: f32) {
        queue.write_buffer(
            &self.uniforms_h,
            0,
            bytemuck::bytes_of(&uniforms.to_raw([1.0, 0.0], scale)),
        );
        queue.write_buffer(
            &self.uniforms_v,
            0,
            bytemuck::bytes_of(&uniforms.to_raw([0.0, 1.0], scale)),
        );
    }
}

fn create_uniforms_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniforms"),
        size: std::mem::size_of::<crate::uniforms::Raw>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
