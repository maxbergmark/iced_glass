use crate::{
    pipeline::{
        AtlasData, Pipeline, SharedBindGroupData, create_textures, create_uniforms_buffer,
        instance::Instance,
    },
    shader::{text::TextShader, texture_bind_groups, uniforms_bind_group},
    uniforms::Uniforms,
};
pub struct TextInstance {
    pub instance: Instance,
    pub vertex_buffer: wgpu::Buffer,
    pub texture_atlas_bg: wgpu::BindGroup,
    pub num_glyphs: u32,
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
