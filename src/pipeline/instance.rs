use tracing::info;

use crate::{
    pipeline::{Pipeline, create_textures},
    shader::{create_sampler, create_uniforms_buffer, texture_bind_groups, uniforms_bind_group},
    uniforms::Uniforms,
};

#[derive(Debug)]
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

impl Instance {
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn new(
        pipeline: &Pipeline,
        device: &wgpu::Device,
        bgl_textures: &wgpu::BindGroupLayout,
        size: iced::Size<u32>,
    ) -> Self {
        info!(
            "creating instance with dimensions: {:?}x{:?}",
            size.width, size.height
        );
        let (tex_a, tex_b) =
            create_textures(device, pipeline.shared_bind_group_data.device_format, size);

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);

        let sampler = create_sampler(device);
        let tex_a_bg = texture_bind_groups(device, bgl_textures, &tex_a, &sampler);
        let tex_b_bg = texture_bind_groups(device, bgl_textures, &tex_b, &sampler);

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
            tex_a,
            tex_b,
            uniforms_h,
            uniforms_v,
            uniform_bg_h,
            uniform_bg_v,
            tex_a_bg,
            tex_b_bg,
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
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
