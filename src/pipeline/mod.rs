use std::collections::{HashMap, HashSet, hash_map::Entry};

use tracing::info;

pub mod instance;

#[cfg(feature = "text")]
pub mod text;
#[cfg(feature = "text")]
pub mod text_atlas;
#[cfg(feature = "text")]
pub mod text_instance;

#[cfg(feature = "text")]
use crate::{pipeline::text::TextPipeline, shader::text::TextShader};

use crate::{
    pipeline::instance::Instance,
    shader::{
        MIP_LEVEL_COUNT, create_sampler, downsample::DownsampleShader, fragment::FragmentShader,
        gaussian::GaussianShader, mip_level_count, uniforms_bind_group_layout,
    },
    uniforms::Uniforms,
};

pub struct Pipeline {
    pub scale: f32,
    pub shared_bind_group_data: SharedBindGroupData,
    pub downsample: wgpu::RenderPipeline,
    pub blur: wgpu::RenderPipeline,
    pub fragment: wgpu::RenderPipeline,
    #[cfg(feature = "text")]
    pub text: TextPipeline,

    instances: HashMap<u64, Instance>,
    live_this_frame: HashSet<u64>,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Pipeline");
        d.field("instances", &self.instances.len());
        #[cfg(feature = "text")]
        d.field("text", &self.text);
        d.finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct SharedBindGroupData {
    pub device_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub bgl_textures: wgpu::BindGroupLayout, // group 0 layout
    pub bgl_uniforms: wgpu::BindGroupLayout, // group 1 layout
    #[cfg(feature = "text")]
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

        device.on_uncaptured_error(std::sync::Arc::new(move |error| {
            tracing::error!("Uncaptured error: {:?}", error);
        }));
        device.set_device_lost_callback(move |reason, message| {
            tracing::error!("Device lost: {:?}, {}", reason, message);
        });

        Self {
            scale: 1.0,
            shared_bind_group_data: SharedBindGroupData {
                device_format: format,
                sampler: create_sampler(device),
                bgl_textures: create_bgl_texture_layout(device),
                bgl_uniforms: uniforms_bind_group_layout(device),
                #[cfg(feature = "text")]
                bgl_text: TextShader::create_bind_group_layout(device),
            },
            downsample: downsample_pipeline,
            blur: blur_pipeline,
            fragment: fragment_pipeline,
            #[cfg(feature = "text")]
            text: TextPipeline::new(device, format),
            instances: HashMap::new(),
            live_this_frame: HashSet::new(),
        }
    }

    fn trim(&mut self) {
        self.instances
            .retain(|id, _| self.live_this_frame.contains(id));
        self.live_this_frame.clear();

        #[cfg(feature = "text")]
        self.text.trim();
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
    let width = size.width.max(1);
    let height = size.height.max(1);
    let mip_level_count = mip_level_count(size);
    let tex_a = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glass.copy"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count,
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
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count,
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
        let inst = match self.instances.entry(id) {
            Entry::Occupied(mut occ) => {
                let same_size =
                    occ.get().size.width == size.width && occ.get().size.height == size.height;
                if !same_size {
                    *occ.get_mut() = Instance::new(&self.shared_bind_group_data, device, size);
                }
                occ.into_mut()
            }
            Entry::Vacant(vac) => {
                vac.insert(Instance::new(&self.shared_bind_group_data, device, size))
            }
        };
        inst.copy_uniforms_to_device(queue, uniforms, scale);
        self.live_this_frame.insert(id);
    }

    #[must_use]
    pub fn instance(&self, id: u64) -> &Instance {
        &self.instances[&id]
    }
}
