use std::collections::{HashMap, HashSet};

#[cfg(not(target_arch = "wasm32"))]
use etagere::{AtlasAllocator, size2};
use tracing::info;

pub mod instance;

#[cfg(not(target_arch = "wasm32"))]
pub mod text_atlas;
#[cfg(not(target_arch = "wasm32"))]
pub mod text_instance;

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    pipeline::{text_atlas::AtlasData, text_instance::TextInstance},
    shader::text::{TEXT_ATLAS_SIZE, TextShader},
};

use crate::{
    pipeline::instance::Instance,
    shader::{
        MIP_LEVEL_COUNT, create_sampler, downsample::DownsampleShader, fragment::FragmentShader,
        gaussian::GaussianShader, uniforms_bind_group_layout,
    },
    uniforms::Uniforms,
};

pub struct Pipeline {
    pub shared_bind_group_data: SharedBindGroupData,
    pub downsample: wgpu::RenderPipeline,
    pub blur: wgpu::RenderPipeline,
    pub fragment: wgpu::RenderPipeline,
    #[cfg(not(target_arch = "wasm32"))]
    pub text: wgpu::RenderPipeline,

    instances: HashMap<u64, Instance>,
    live_this_frame: HashSet<u64>,

    #[cfg(not(target_arch = "wasm32"))]
    pub text_instances: HashMap<u64, TextInstance>,
    #[cfg(not(target_arch = "wasm32"))]
    live_text_this_frame: HashSet<u64>,

    #[cfg(not(target_arch = "wasm32"))]
    pub atlas_data: AtlasData,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("instances", &self.instances.len())
            // .field("text_instances", &self.text_instances.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct SharedBindGroupData {
    pub device_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub bgl_textures: wgpu::BindGroupLayout, // group 0 layout
    pub bgl_uniforms: wgpu::BindGroupLayout, // group 1 layout
    #[cfg(not(target_arch = "wasm32"))]
    pub bgl_text: wgpu::BindGroupLayout, // group 0 layout
}

impl iced::widget::shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        info!("creating pipeline with format: {:?}", format);
        let downsample_pipeline = DownsampleShader::create_pipeline(device, format);
        let blur_pipeline = GaussianShader::create_pipeline(device, format);
        let fragment_pipeline = FragmentShader::create_pipeline(device, format);
        #[cfg(not(target_arch = "wasm32"))]
        let text_pipeline = TextShader::create_pipeline(device, format);

        #[cfg(not(target_arch = "wasm32"))]
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
            format,
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
                #[cfg(not(target_arch = "wasm32"))]
                bgl_text: TextShader::create_bind_group_layout(device),
            },
            downsample: downsample_pipeline,
            blur: blur_pipeline,
            fragment: fragment_pipeline,
            #[cfg(not(target_arch = "wasm32"))]
            text: text_pipeline,
            instances: HashMap::new(),
            live_this_frame: HashSet::new(),
            #[cfg(not(target_arch = "wasm32"))]
            text_instances: HashMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            live_text_this_frame: HashSet::new(),

            #[cfg(not(target_arch = "wasm32"))]
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

#[must_use]
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

#[must_use]
pub fn create_textures(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    size: iced::Size<u32>,
) -> (wgpu::Texture, wgpu::Texture) {
    let tex_a = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glass.copy"),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
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

    let tex_b = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glass.gaussian"),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: MIP_LEVEL_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    (tex_a, tex_b)
}

impl Pipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        size: iced::Size<u32>,
        scale: f32,
        uniforms: &Uniforms,
    ) {
        let needs_new = self
            .instances
            .get(&id)
            .is_none_or(|inst| inst.size.width != size.width || inst.size.height != size.height);
        if needs_new {
            self.instances.insert(
                id,
                Instance::new(
                    self,
                    device,
                    &self.shared_bind_group_data.bgl_textures,
                    size,
                    &self.shared_bind_group_data.sampler,
                ),
            );
        }
        #[allow(clippy::expect_used)]
        let inst = self.instances.get_mut(&id).expect("Instance not found");
        inst.copy_uniforms_to_device(queue, uniforms, scale);
        self.live_this_frame.insert(id);
    }

    #[cfg(not(target_arch = "wasm32"))]
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
        #[allow(clippy::expect_used)]
        let inst = self
            .text_instances
            .get_mut(&id)
            .expect("Text instance not found");

        // TODO: find a cleaner way to do this
        let mut uniforms = *uniforms;
        uniforms.content_scale = (
            size.width as f32 / inst.instance.size.width as f32,
            size.height as f32 / inst.instance.size.height as f32,
        );

        inst.copy_uniforms_to_device(queue, &uniforms, scale);
        self.live_text_this_frame.insert(id);
    }

    #[must_use]
    pub fn instance(&self, id: u64) -> &Instance {
        &self.instances[&id]
    }

    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn text_instance(&self, id: u64) -> &TextInstance {
        &self.text_instances[&id]
    }

    // Call at the end of rendering each frame.
    // #[allow(unused)] // TODO: use this function to clean up instances and text instances
    // pub fn gc(&mut self) {
    //     self.instances
    //         .retain(|id, _| self.live_this_frame.contains(id));
    //     self.text_instances
    //         .retain(|id, _| self.live_text_this_frame.contains(id));
    //     self.live_this_frame.clear();
    //     self.live_text_this_frame.clear();
    // }
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn round_up(size: u32, granularity: u32) -> u32 {
    size.div_ceil(granularity) * granularity
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn content_scale(size: iced::Size<f32>) -> (f32, f32) {
    (
        size.width / round_up(size.width as u32, 256) as f32,
        size.height / round_up(size.height as u32, 256) as f32,
    )
}
