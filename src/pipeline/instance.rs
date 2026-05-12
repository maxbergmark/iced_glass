use tracing::debug;

use crate::{
    pipeline::{SharedBindGroupData, create_textures},
    shader::{create_uniforms_buffer, texture_bind_groups, uniforms_bind_group},
    uniforms::Uniforms,
};

pub const CHILDREN_CAPACITY: usize = 100;

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

    pub children: wgpu::Buffer,
    pub children_bg: wgpu::BindGroup,
}

impl Instance {
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn new(
        bg_data: &SharedBindGroupData,
        device: &wgpu::Device,
        size: iced::Size<u32>,
    ) -> Self {
        debug!(
            "creating instance with dimensions: {:?}x{:?}",
            size.width, size.height
        );
        let (tex_a, tex_b) = create_textures(device, bg_data.device_format, size);

        let uniforms_h = create_uniforms_buffer(device);
        let uniforms_v = create_uniforms_buffer(device);
        let sampler = &bg_data.sampler;
        let bgl_textures = &bg_data.bgl_textures;

        let tex_a_bg = texture_bind_groups(device, bgl_textures, &tex_a, sampler);
        let tex_b_bg = texture_bind_groups(device, bgl_textures, &tex_b, sampler);

        let uniform_bg_h = uniforms_bind_group(device, &bg_data.bgl_uniforms, &uniforms_h);
        let uniform_bg_v = uniforms_bind_group(device, &bg_data.bgl_uniforms, &uniforms_v);
        let (children, children_bg) = child_data(device);

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
            children,
            children_bg,
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

fn child_data(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroup) {
    let children = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("children"),
        // This is hard-coded for now, but we could make it dynamic in the future.
        size: std::mem::size_of::<crate::uniforms::ChildRaw>() as u64 * CHILDREN_CAPACITY as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let children_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("children_bg"),
        layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("children_bg_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(children.as_entire_buffer_binding()),
        }],
    });
    (children, children_bg)
}
