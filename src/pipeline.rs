use std::collections::{HashMap, HashSet};

use cosmic_text::fontdb;
use etagere::{AtlasAllocator, size2};
use ttf_parser::GlyphId;

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
    pub shared_bind_group_data: SharedBindGroupData,
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub blur_pipeline: wgpu::RenderPipeline,
    pub fragment_pipeline: wgpu::RenderPipeline,
    pub text_pipeline: wgpu::RenderPipeline,

    instances: HashMap<u64, Instance>,
    live_this_frame: HashSet<u64>,

    pub text_instances: HashMap<u64, TextInstance>,
    live_text_this_frame: HashSet<u64>,

    pub atlas_data: AtlasData,
}

pub struct SharedBindGroupData {
    pub device_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub bgl_textures: wgpu::BindGroupLayout, // group 0 layout
    pub bgl_uniforms: wgpu::BindGroupLayout, // group 1 layout
    pub bgl_text: wgpu::BindGroupLayout,     // group 0 layout
}

pub struct AtlasData {
    pub texture_atlas: wgpu::Texture,
    pub atlas_position: HashMap<(fontdb::ID, GlyphId), AtlasPosition>,
    pub allocator: AtlasAllocator,
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
    pub instance: Instance,
    pub vertex_buffer: wgpu::Buffer,
    pub texture_atlas_bg: wgpu::BindGroup,
    pub num_glyphs: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasPosition {
    pub position: iced::Point<u32>,
    pub size: iced::Size<u32>,
    pub bbox: ttf_parser::Rect,
    pub units_per_em: f32,
    pub framing: msdfgen::Framing<f64>,
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

        Self {
            shared_bind_group_data: SharedBindGroupData {
                device_format: format,
                sampler: create_sampler(device),
                bgl_textures: create_bgl_texture_layout(device),
                bgl_uniforms: uniforms_bind_group_layout(device),
                bgl_text: TextShader::create_bind_group_layout(device),
            },
            downsample_pipeline,
            blur_pipeline,
            fragment_pipeline,
            text_pipeline,
            instances: HashMap::new(),
            live_this_frame: HashSet::new(),
            text_instances: HashMap::new(),
            live_text_this_frame: HashSet::new(),

            atlas_data: AtlasData {
                texture_atlas,
                atlas_position: HashMap::new(),
                allocator: AtlasAllocator::new(size2(
                    TEXT_ATLAS_SIZE as i32,
                    TEXT_ATLAS_SIZE as i32,
                )),
            },
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
                Instance::new(
                    self,
                    device,
                    &self.shared_bind_group_data.bgl_textures,
                    width,
                    height,
                ),
            );
        }
        let inst = self.instances.get_mut(&id).unwrap();
        inst.copy_uniforms_to_device(queue, uniforms, scale);
        self.live_this_frame.insert(id);
    }

    pub fn prepare_text_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        size: iced::Size<u32>,
        scale: f32,
        uniforms: &Uniforms,
    ) {
        let needs_new = self.text_instances.get(&id).is_none_or(|inst| {
            inst.instance.size.width < size.width || inst.instance.size.height < size.height
        });
        if needs_new {
            let alloc_size = iced::Size::new(round_up(size.width, 256), round_up(size.height, 256));

            match self.text_instances.get_mut(&id) {
                Some(inst) => {
                    inst.update_size(
                        &self.shared_bind_group_data,
                        &self.atlas_data,
                        device,
                        alloc_size,
                    );
                }
                None => {
                    self.text_instances.insert(
                        id,
                        TextInstance::new(
                            self,
                            device,
                            &self.shared_bind_group_data.bgl_textures,
                            alloc_size,
                        ),
                    );
                }
            }
        }
        let inst = self.text_instances.get_mut(&id).unwrap();

        // TODO: find a cleaner way to do this
        let mut uniforms = *uniforms;
        uniforms.content_scale = (
            size.width as f32 / inst.instance.size.width as f32,
            size.height as f32 / inst.instance.size.height as f32,
        );

        inst.copy_uniforms_to_device(queue, &uniforms, scale);
        self.live_text_this_frame.insert(id);
    }

    pub fn instance(&self, id: u64) -> &Instance {
        &self.instances[&id]
    }

    pub fn text_instance(&self, id: u64) -> &TextInstance {
        &self.text_instances[&id]
    }

    pub fn text_instance_mut(&mut self, id: u64) -> &mut TextInstance {
        self.text_instances.get_mut(&id).unwrap()
    }

    /// Call at the end of rendering each frame.
    pub fn gc(&mut self) {
        self.instances
            .retain(|id, _| self.live_this_frame.contains(id));
        self.text_instances
            .retain(|id, _| self.live_text_this_frame.contains(id));
        self.live_this_frame.clear();
        self.live_text_this_frame.clear();
    }
}

pub fn round_up(size: u32, granularity: u32) -> u32 {
    size.div_ceil(granularity) * granularity
}

pub fn content_scale(size: iced::Size<f32>) -> (f32, f32) {
    (
        size.width / round_up(size.width as u32, 256) as f32,
        size.height / round_up(size.height as u32, 256) as f32,
    )
}

impl Instance {
    pub fn new(
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bgl_textures: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let (copy_texture, gaussian_texture) = create_textures(
            device,
            pipeline.shared_bind_group_data.device_format,
            width,
            height,
        );

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);

        let copy_texture_bg = texture_bind_groups(device, bgl_textures, &copy_texture);
        let gaussian_texture_bg = texture_bind_groups(device, bgl_textures, &gaussian_texture);

        let uniform_bg_h = uniforms_bind_group(
            device,
            &pipeline.shared_bind_group_data.bgl_uniforms,
            &uniforms_h,
        );
        let uniform_bg_v = uniforms_bind_group(
            device,
            &pipeline.shared_bind_group_data.bgl_uniforms,
            &uniforms_v,
        );

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
        size: iced::Size<u32>,
    ) -> Self {
        let (copy_texture, gaussian_texture) = create_textures(
            device,
            pipeline.shared_bind_group_data.device_format,
            size.width,
            size.height,
        );

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);

        let copy_texture_bg = texture_bind_groups(device, bgl_textures, &copy_texture);
        let gaussian_texture_bg = texture_bind_groups(device, bgl_textures, &gaussian_texture);

        let uniform_bg_h = uniforms_bind_group(
            device,
            &pipeline.shared_bind_group_data.bgl_uniforms,
            &uniforms_h,
        );
        let uniform_bg_v = uniforms_bind_group(
            device,
            &pipeline.shared_bind_group_data.bgl_uniforms,
            &uniforms_v,
        );

        let vertex_buffer = TextShader::create_vertex_buffer(device);
        let texture_atlas_bg = TextShader::create_bind_group(
            device,
            &pipeline.shared_bind_group_data,
            &pipeline.atlas_data,
            &copy_texture,
        );

        Self {
            instance: Instance {
                tex_a: copy_texture,
                tex_b: gaussian_texture,
                uniforms_h,
                uniforms_v,
                uniform_bg_h,
                uniform_bg_v,
                tex_a_bg: copy_texture_bg,
                tex_b_bg: gaussian_texture_bg,
                size: wgpu::Extent3d {
                    width: size.width.max(1),
                    height: size.height.max(1),
                    depth_or_array_layers: 1,
                },
            },

            vertex_buffer,
            texture_atlas_bg,
            num_glyphs: 0,
        }
    }

    pub fn update_size(
        &mut self,
        shared_bind_group_data: &SharedBindGroupData,
        atlas_data: &AtlasData,
        device: &wgpu::Device,
        size: iced::Size<u32>,
    ) {
        let (copy_texture, gaussian_texture) = create_textures(
            device,
            shared_bind_group_data.device_format,
            size.width,
            size.height,
        );
        self.instance.tex_a = copy_texture;
        self.instance.tex_b = gaussian_texture;

        self.instance.tex_a_bg = texture_bind_groups(
            device,
            &shared_bind_group_data.bgl_textures,
            &self.instance.tex_a,
        );
        self.instance.tex_b_bg = texture_bind_groups(
            device,
            &shared_bind_group_data.bgl_textures,
            &self.instance.tex_b,
        );

        self.texture_atlas_bg = TextShader::create_bind_group(
            device,
            shared_bind_group_data,
            atlas_data,
            &self.instance.tex_a,
        );

        self.instance.size = wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        };
    }

    pub fn copy_uniforms_to_device(&self, queue: &wgpu::Queue, uniforms: &Uniforms, scale: f32) {
        queue.write_buffer(
            &self.instance.uniforms_h,
            0,
            bytemuck::bytes_of(&uniforms.to_raw([1.0, 0.0], scale)),
        );
        queue.write_buffer(
            &self.instance.uniforms_v,
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
