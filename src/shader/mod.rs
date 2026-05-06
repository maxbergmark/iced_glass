pub mod downsample;
pub mod fragment;
pub mod gaussian;
pub mod text;

pub const MIP_LEVEL_COUNT: u32 = 4;

// #[derive(Debug)]
// pub struct RenderShaderData {
//     pub pipeline: wgpu::RenderPipeline,
//     pub bind_group: wgpu::BindGroup,
//     pub uniform_bind_group: wgpu::BindGroup,
// }

#[must_use]
pub fn uniforms_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniforms_bind_group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniforms.as_entire_buffer_binding()),
        }],
    })
}

#[must_use]
pub fn uniforms_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniforms_bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

#[must_use]
pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fragment.sampler"),
        address_mode_u: wgpu::AddressMode::MirrorRepeat,
        address_mode_v: wgpu::AddressMode::MirrorRepeat,
        address_mode_w: wgpu::AddressMode::MirrorRepeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[must_use]
pub fn texture_bind_groups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> Vec<wgpu::BindGroup> {
    (0..MIP_LEVEL_COUNT)
        .map(|level| {
            let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("mipmap.bind_group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            })
        })
        .collect()
}

pub fn create_uniforms_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniforms"),
        size: std::mem::size_of::<crate::uniforms::Raw>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
