use crate::{
    shader::{fragment, gaussian, uniforms_bind_group, uniforms_bind_group_layout},
    uniforms::Uniforms,
};

pub struct Pipeline {
    // Shared, created once:
    pub device_format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
    pub bgl_textures: wgpu::BindGroupLayout, // group 0 layout
    pub bgl_uniforms: wgpu::BindGroupLayout, // group 1 layout
    pub horizontal_blur_pipeline: wgpu::RenderPipeline,
    pub vertical_blur_pipeline: wgpu::RenderPipeline,
    pub fragment_pipeline: wgpu::RenderPipeline,

    // One entry per GlassContainer:
    instances: std::collections::HashMap<u64, Instance>,
    live_this_frame: std::collections::HashSet<u64>,
}
pub struct Instance {
    pub copy_texture: wgpu::Texture,
    pub gaussian_texture: wgpu::Texture,
    pub uniforms: wgpu::Buffer,
    pub horizontal_bg: wgpu::BindGroup, // sampling copy_texture
    pub vertical_bg: wgpu::BindGroup,   // sampling gaussian_texture
    pub fragment_bg: wgpu::BindGroup,   // sampling copy_texture
    pub uniform_bg: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

impl iced::widget::shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<crate::uniforms::Raw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (copy_texture, gaussian_texture) = create_copy_texture(device, format, 1, 1);
        let (horizontal_shader, vertical_shader) = gaussian::GaussianShader::compile(
            device,
            format,
            &uniforms,
            &copy_texture,
            &gaussian_texture,
        );

        let fragment_pipeline =
            fragment::FragmentShader::compile(device, format, &uniforms, &copy_texture);

        Self {
            device_format: format,
            sampler: create_sampler(device),
            bgl_textures: create_bgl_texture_layout(device),
            bgl_uniforms: uniforms_bind_group_layout(device),
            horizontal_blur_pipeline: horizontal_shader.pipeline,
            vertical_blur_pipeline: vertical_shader.pipeline,
            fragment_pipeline: fragment_pipeline.pipeline,
            instances: std::collections::HashMap::new(),
            live_this_frame: std::collections::HashSet::new(),
        }
    }
}

pub fn create_bgl_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bgl_textures"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    })
}

pub fn create_copy_texture(
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
        mip_level_count: 1,
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
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    (copy_to_texture, gaussian_texture)
}

impl Pipeline {
    pub fn resize_if_needed(&mut self, device: &wgpu::Device, width: u32, height: u32, id: u64) {
        let instance = self.instance(id);
        if (instance.size.width, instance.size.height) == (width, height) {
            return;
        }
        let (copy_texture, gaussian_texture) =
            create_copy_texture(device, instance.copy_texture.format(), width, height);
        // instance.copy_texture = copy_texture;
        // instance.gaussian_texture = gaussian_texture;
        // self.fragment_shader = fragment::FragmentShader::compile(
        //     device,
        //     self.format,
        //     &self.uniforms,
        //     &self.copy_texture,
        // );
        let (horizontal_blur, vertical_blur) = gaussian::GaussianShader::compile(
            device,
            instance.copy_texture.format(),
            &instance.uniforms,
            &instance.copy_texture,
            &instance.gaussian_texture,
        );

        let fragment_pipeline = fragment::FragmentShader::compile(
            device,
            instance.copy_texture.format(),
            &instance.uniforms,
            &instance.copy_texture,
        );
        // instance.horizontal_bg = horizontal_blur.bind_group;
        // instance.vertical_bg = vertical_blur.bind_group;
        // instance.size = wgpu::Extent3d {
        //     width: width.max(1),
        //     height: height.max(1),
        //     depth_or_array_layers: 1,
        // };
        let new_instance = Instance {
            copy_texture,
            gaussian_texture,
            uniforms: instance.uniforms.clone(),
            horizontal_bg: horizontal_blur.bind_group,
            vertical_bg: vertical_blur.bind_group,
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            fragment_bg: fragment_pipeline.bind_group,
            uniform_bg: instance.uniform_bg.clone(),
        };
        self.instances.insert(id, new_instance);
    }

    pub fn copy_uniforms_to_device(&self, queue: &wgpu::Queue, uniforms: &Uniforms, id: u64) {
        let instance = self.instance(id);
        queue.write_buffer(
            &instance.uniforms,
            0,
            bytemuck::bytes_of(&uniforms.to_raw()),
        );
    }

    pub fn prepare_instance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        width: u32,
        height: u32,
        uniforms: &Uniforms,
    ) {
        let needs_new = match self.instances.get(&id) {
            Some(inst) => inst.size.width != width || inst.size.height != height,
            None => true,
        };
        if needs_new {
            self.instances
                .insert(id, Instance::new(self, device, width, height));
        }
        let inst = self.instances.get_mut(&id).unwrap();
        queue.write_buffer(&inst.uniforms, 0, bytemuck::bytes_of(&uniforms.to_raw()));
        self.live_this_frame.insert(id);
    }
    pub fn instance(&self, id: u64) -> &Instance {
        &self.instances[&id]
    }
    /// Call at the end of rendering each frame.
    pub fn gc(&mut self) {
        self.instances
            .retain(|id, _| self.live_this_frame.contains(id));
        self.live_this_frame.clear();
    }
}

impl Instance {
    pub fn new(pipeline: &Pipeline, device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (copy_texture, gaussian_texture) =
            create_copy_texture(device, pipeline.device_format, width, height);

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<crate::uniforms::Raw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (horizontal_blur, vertical_blur) = gaussian::GaussianShader::compile(
            device,
            pipeline.device_format,
            &uniforms,
            &copy_texture,
            &gaussian_texture,
        );
        let fragment_pipeline = fragment::FragmentShader::compile(
            device,
            pipeline.device_format,
            &uniforms,
            &copy_texture,
        );
        let uniform_bg = uniforms_bind_group(device, &pipeline.bgl_uniforms, &uniforms);
        Self {
            copy_texture,
            gaussian_texture,
            uniforms,
            horizontal_bg: horizontal_blur.bind_group,
            vertical_bg: vertical_blur.bind_group,
            fragment_bg: fragment_pipeline.bind_group,
            uniform_bg,
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        }
    }
}

pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("my_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
