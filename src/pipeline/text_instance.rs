use tracing::info;

use crate::{
    pipeline::{AtlasData, Pipeline, SharedBindGroupData, create_textures, instance::Instance},
    shader::{create_sampler, text::TextShader, texture_bind_groups},
    uniforms::Uniforms,
};
#[derive(Debug)]
pub struct TextInstance {
    pub instance: Instance,
    pub vertex_buffer: wgpu::Buffer,
    pub texture_atlas_bg: wgpu::BindGroup,
    pub num_glyphs: u32,
}

impl TextInstance {
    #[must_use]
    pub fn new(
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bgl_textures: &wgpu::BindGroupLayout,
        size: iced::Size<u32>,
    ) -> Self {
        info!(
            "creating text instance with size: {:?}x{:?}",
            size.width, size.height
        );
        let instance = Instance::new(pipeline, device, bgl_textures, size);

        let vertex_buffer = TextShader::create_vertex_buffer(device);
        let texture_atlas_bg = TextShader::create_bind_group(
            device,
            &pipeline.shared_bind_group_data,
            &pipeline.atlas_data,
            &instance.tex_a,
        );

        Self {
            instance,
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
        info!(
            "updating text instance size from {:?}x{:?} to {:?}x{:?}",
            self.instance.size.width, self.instance.size.height, size.width, size.height
        );
        let (tex_a, tex_b) = create_textures(device, shared_bind_group_data.device_format, size);
        self.instance.tex_a = tex_a;
        self.instance.tex_b = tex_b;

        let sampler = create_sampler(device);
        self.instance.tex_a_bg = texture_bind_groups(
            device,
            &shared_bind_group_data.bgl_textures,
            &self.instance.tex_a,
            &sampler,
        );
        self.instance.tex_b_bg = texture_bind_groups(
            device,
            &shared_bind_group_data.bgl_textures,
            &self.instance.tex_b,
            &sampler,
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
