use crate::{
    pipeline::{Pipeline, instance::Instance},
    uniforms::Uniforms,
};

pub mod text;

#[derive(Debug, Default, Clone, Copy)]
pub struct Primitive {
    pub id: u64,
    pub uniforms: Uniforms,
}

impl iced::widget::shader::Primitive for Primitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        let scale = viewport.scale_factor();
        let width = (bounds.width * scale) as u32;
        let height = (bounds.height * scale) as u32;
        pipeline.prepare_instance(device, queue, self.id, width, height, scale, &self.uniforms);
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        bounds: &iced::Rectangle<u32>,
    ) {
        let texture = target.texture();
        let instance = pipeline.instance(self.id);
        let copy_size = match calculate_copy_size(texture, instance, bounds) {
            Some(size) => size,
            None => return,
        };

        let mip_level = self.uniforms.mip_level();
        copy_background(encoder, &instance.tex_a, texture, bounds, &copy_size);
        downsample(encoder, pipeline, instance, mip_level);
        horizontal_blur(encoder, pipeline, instance, mip_level);
        vertical_blur(encoder, pipeline, instance, mip_level);
        upsample(encoder, pipeline, instance, mip_level);
        fragment_pass(encoder, pipeline, instance, target, bounds);
    }
}

fn calculate_copy_size(
    texture: &wgpu::Texture,
    instance: &Instance,
    bounds: &iced::Rectangle<u32>,
) -> Option<wgpu::Extent3d> {
    let src_size = texture.size();
    let dst_size = instance.tex_a.size();
    let copy_width = bounds
        .width
        .min(dst_size.width)
        .min(src_size.width.saturating_sub(bounds.x));
    let copy_height = bounds
        .height
        .min(dst_size.height)
        .min(src_size.height.saturating_sub(bounds.y));

    if copy_width == 0 || copy_height == 0 {
        None
    } else {
        Some(wgpu::Extent3d {
            width: copy_width,
            height: copy_height,
            depth_or_array_layers: 1,
        })
    }
}

fn copy_background(
    encoder: &mut wgpu::CommandEncoder,
    tex_a: &wgpu::Texture,
    texture: &wgpu::Texture,
    bounds: &iced::Rectangle<u32>,
    copy_size: &wgpu::Extent3d,
) {
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: bounds.x,
                y: bounds.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        tex_a.as_image_copy(),
        *copy_size,
    );
}

fn downsample(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    mip_level: u32,
) {
    let texture = &instance.tex_a;
    let tex_a_bg = &instance.tex_a_bg;
    #[allow(clippy::reversed_empty_ranges)]
    for level in 1..=mip_level {
        let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("downsample.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline.downsample_pipeline);
        pass.set_bind_group(0, &tex_a_bg[(level - 1) as usize], &[]);
        pass.draw(0..6, 0..1);
    }
}

fn upsample(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    mip_level: u32,
) {
    let tex_a_bg = &instance.tex_a_bg;
    let texture = &instance.tex_a;
    #[allow(clippy::reversed_empty_ranges)]
    for level in (1..=mip_level).rev() {
        let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: level - 1,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("upsample.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline.downsample_pipeline);
        pass.set_bind_group(0, &tex_a_bg[level as usize], &[]);
        pass.draw(0..6, 0..1);
    }
}

fn horizontal_blur(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    mip_level: u32,
) {
    let mut horizontal_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("primitive.horizontal_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &instance.tex_b.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            }),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    horizontal_pass.set_pipeline(&pipeline.blur_pipeline);
    horizontal_pass.set_bind_group(0, &instance.tex_a_bg[mip_level as usize], &[]);
    horizontal_pass.set_bind_group(1, &instance.uniform_bg_h, &[]);
    horizontal_pass.draw(0..6, 0..1);
}

fn vertical_blur(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    mip_level: u32,
) {
    let mut vertical_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("primitive.vertical_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &instance.tex_a.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            }),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    vertical_pass.set_pipeline(&pipeline.blur_pipeline);
    vertical_pass.set_bind_group(0, &instance.tex_b_bg[mip_level as usize], &[]);
    vertical_pass.set_bind_group(1, &instance.uniform_bg_v, &[]);
    vertical_pass.draw(0..6, 0..1);
}

fn fragment_pass(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    target: &wgpu::TextureView,
    bounds: &iced::Rectangle<u32>,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("primitive.render_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_scissor_rect(bounds.x, bounds.y, bounds.width, bounds.height);
    pass.set_viewport(
        bounds.x as f32,
        bounds.y as f32,
        bounds.width as f32,
        bounds.height as f32,
        0.0,
        1.0,
    );

    pass.set_pipeline(&pipeline.fragment_pipeline);
    pass.set_bind_group(0, &instance.tex_a_bg[0], &[]);
    pass.set_bind_group(1, &instance.uniform_bg_h, &[]);
    pass.draw(0..6, 0..1);
}
