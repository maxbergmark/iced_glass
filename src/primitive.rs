use crate::{pipeline::Pipeline, uniforms::Uniforms};

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
        // let width = bounds.width.min(bounds.width - bounds.x).max(0.0);
        let width = (bounds.width * scale) as u32;
        // let height = bounds.height.min(bounds.height - bounds.y).max(0.0);
        let height = (bounds.height * scale) as u32;
        // let instance = pipeline.instance(self.id);
        pipeline.prepare_instance(device, queue, self.id, width, height, &self.uniforms);
        // pipeline.resize_if_needed(device, width, height, self.id);
        // pipeline.copy_uniforms_to_device(queue, &self.uniforms, self.id);
    }

    fn draw(&self, _pipeline: &Self::Pipeline, _render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        false
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        texture: &wgpu::Texture,
        bounds: &iced::Rectangle<u32>,
    ) {
        let instance = pipeline.instance(self.id);
        let src_size = texture.size();
        let dst_size = instance.copy_texture.size();
        let copy_width = bounds
            .width
            .min(dst_size.width)
            .min(src_size.width.saturating_sub(bounds.x));
        let copy_height = bounds
            .height
            .min(dst_size.height)
            .min(src_size.height.saturating_sub(bounds.y));

        if copy_width == 0 || copy_height == 0 {
            return;
        }

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
            instance.copy_texture.as_image_copy(),
            wgpu::Extent3d {
                width: copy_width,
                height: copy_height,
                depth_or_array_layers: 1,
            },
        );

        {
            let mut horizontal_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("primitive.horizontal_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &instance
                        .gaussian_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
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
            horizontal_pass.set_scissor_rect(0, 0, copy_width, copy_height);
            horizontal_pass.set_viewport(0.0, 0.0, copy_width as f32, copy_height as f32, 0.0, 1.0);
            horizontal_pass.set_pipeline(&pipeline.horizontal_blur_pipeline);
            horizontal_pass.set_bind_group(0, &instance.horizontal_bg, &[]);
            horizontal_pass.set_bind_group(1, &instance.uniform_bg, &[]);
            horizontal_pass.draw(0..6, 0..1);
        }

        {
            let mut vertical_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("primitive.vertical_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &instance
                        .copy_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
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
            vertical_pass.set_scissor_rect(0, 0, copy_width, copy_height);
            vertical_pass.set_viewport(0.0, 0.0, copy_width as f32, copy_height as f32, 0.0, 1.0);
            vertical_pass.set_pipeline(&pipeline.vertical_blur_pipeline);
            vertical_pass.set_bind_group(0, &instance.vertical_bg, &[]);
            vertical_pass.set_bind_group(1, &instance.uniform_bg, &[]);
            vertical_pass.draw(0..6, 0..1);
        }

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
        pass.set_bind_group(0, &instance.fragment_bg, &[]);
        pass.set_bind_group(1, &instance.uniform_bg, &[]);
        pass.draw(0..6, 0..1);
    }
}

// impl iced_wgpu::primitive::Primitive for Primitive {
//     type Pipeline = Pipeline;

//     fn prepare(
//         &self,
//         pipeline: &mut Self::Pipeline,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         bounds: &iced::Rectangle,
//         viewport: &iced_wgpu::graphics::Viewport,
//     ) {
//         self.prepare(pipeline, device, queue, bounds, viewport);
//     }

//     fn draw(&self, _pipeline: &Self::Pipeline, _render_pass: &mut wgpu::RenderPass<'_>) -> bool {
//         false
//     }

//     fn render(
//         &self,
//         _pipeline: &Self::Pipeline,
//         _encoder: &mut wgpu::CommandEncoder,
//         _target: &wgpu::TextureView,
//         _texture: &wgpu::Texture,
//         _clip_bounds: &iced::Rectangle<u32>,
//     ) {
//     }
// }
