use std::collections::{HashMap, HashSet, hash_map::Entry};

use etagere::{AtlasAllocator, size2};

use crate::{
    pipeline::{Pipeline, text_atlas::AtlasData, text_instance::TextInstance},
    shader::text::{TEXT_ATLAS_SIZE, TextShader},
    uniforms::Uniforms,
};

pub struct TextPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) atlas_data: AtlasData,
    pub(crate) text_instances: HashMap<u64, TextInstance>,
    live_text_this_frame: HashSet<u64>,
}

impl std::fmt::Debug for TextPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("TextPipeline");
        d.field("text_instances", &self.text_instances.len());
        d.finish_non_exhaustive()
    }
}

impl TextPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
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
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            pipeline: text_pipeline,
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

    pub fn trim(&mut self) {
        self.text_instances
            .retain(|id, _| self.live_text_this_frame.contains(id));
        self.live_text_this_frame.clear();
    }
}

impl Pipeline {
    pub fn prepare_text_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        size: iced::Size<u32>,
        scale: f32,
        uniforms: &Uniforms,
    ) {
        let alloc_size = iced::Size::new(round_up(size.width, 256), round_up(size.height, 256));
        let bg_data = &self.shared_bind_group_data;
        let atlas_data = &self.text.atlas_data;
        let inst = match self.text.text_instances.entry(id) {
            Entry::Occupied(mut occ) => {
                let cur = occ.get().instance.size;
                if cur.width < size.width || cur.height < size.height {
                    occ.get_mut()
                        .update_size(bg_data, atlas_data, device, alloc_size);
                }
                occ.into_mut()
            }
            Entry::Vacant(vac) => {
                vac.insert(TextInstance::new(bg_data, atlas_data, device, alloc_size))
            }
        };
        let scaled = Uniforms {
            content_scale: (
                size.width as f32 / inst.instance.size.width as f32,
                size.height as f32 / inst.instance.size.height as f32,
            ),
            ..*uniforms
        };
        inst.copy_uniforms_to_device(queue, &scaled, scale);
        self.text.live_text_this_frame.insert(id);
    }

    #[must_use]
    pub fn text_instance(&self, id: u64) -> &TextInstance {
        &self.text.text_instances[&id]
    }
}

#[must_use]
pub const fn round_up(size: u32, granularity: u32) -> u32 {
    size.div_ceil(granularity) * granularity
}

#[must_use]
pub fn content_scale(size: iced::Size<f32>) -> (f32, f32) {
    (
        size.width / round_up(size.width as u32, 256) as f32,
        size.height / round_up(size.height as u32, 256) as f32,
    )
}
