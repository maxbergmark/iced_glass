use crate::{
    pipeline::{Instance, Pipeline},
    uniforms::Uniforms,
};

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
        // read_timestamps(device, queue, pipeline.instance_mut(self.id));
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
        copy_background(encoder, instance, texture, bounds, &copy_size);
        downsample(encoder, pipeline, instance, &instance.tex_a, mip_level);
        horizontal_blur(encoder, pipeline, instance, mip_level);
        vertical_blur(encoder, pipeline, instance, mip_level);
        upsample(encoder, pipeline, instance, &instance.tex_a, mip_level);
        fragment_pass(encoder, pipeline, instance, target, bounds);
        // resolve_timestamps(encoder, instance);
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
    instance: &Instance,
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
        instance.tex_a.as_image_copy(),
        *copy_size,
    );
}

fn downsample(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    texture: &wgpu::Texture,
    mip_level: u32,
) {
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
            // timestamp_writes: if level == 1 {
            //     Some(wgpu::RenderPassTimestampWrites {
            //         query_set: &instance.query_set,
            //         beginning_of_pass_write_index: Some(0),
            //         end_of_pass_write_index: Some(1),
            //     })
            // } else {
            //     None
            // },
            ..Default::default()
        });
        pass.set_pipeline(&pipeline.downsample_pipeline);
        pass.set_bind_group(0, &instance.tex_a_bg[(level - 1) as usize], &[]);
        pass.draw(0..6, 0..1);
    }
}

fn upsample(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &Instance,
    texture: &wgpu::Texture,
    mip_level: u32,
) {
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
            // timestamp_writes: if level == 1 {
            //     Some(wgpu::RenderPassTimestampWrites {
            //         query_set: &instance.query_set,
            //         beginning_of_pass_write_index: Some(6),
            //         end_of_pass_write_index: Some(7),
            //     })
            // } else {
            //     None
            // },
            ..Default::default()
        });
        pass.set_pipeline(&pipeline.downsample_pipeline);
        pass.set_bind_group(0, &instance.tex_a_bg[level as usize], &[]);
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
        // timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
        //     query_set: &instance.query_set,
        //     beginning_of_pass_write_index: Some(2),
        //     end_of_pass_write_index: Some(3),
        // }),
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
                ..Default::default() // label: todo!(),
                                     // format: todo!(),
                                     // dimension: todo!(),
                                     // usage: todo!(),
                                     // aspect: todo!(),
                                     // mip_level_count: todo!(),
            }),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        // timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
        //     query_set: &instance.query_set,
        //     beginning_of_pass_write_index: Some(4),
        //     end_of_pass_write_index: Some(5),
        // }),
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
        // timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
        //     query_set: &instance.query_set,
        //     beginning_of_pass_write_index: Some(8),
        //     end_of_pass_write_index: Some(9),
        // }),
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

// #[allow(unused)]
// fn resolve_timestamps(encoder: &mut wgpu::CommandEncoder, instance: &Instance) {
//     encoder.resolve_query_set(&instance.query_set, 0..10, &instance.resolve_buffer, 0);
//     encoder.copy_buffer_to_buffer(
//         &instance.resolve_buffer,
//         0,
//         &instance.readback_buffer,
//         0,
//         (10 * std::mem::size_of::<u64>()) as u64,
//     );
// }

// #[allow(unused)]
// fn read_timestamps(device: &wgpu::Device, queue: &wgpu::Queue, instance: &mut Instance) {
//     let slice = instance.readback_buffer.slice(..);
//     slice.map_async(wgpu::MapMode::Read, |_| {});
//     device.poll(PollType::wait_indefinitely()).unwrap();

//     let data = slice.get_mapped_range();
//     let timestamps: &[u64] = bytemuck::cast_slice(&data);
//     // drop(data);

//     let period = queue.get_timestamp_period(); // nanoseconds per tick
//     let downsample_ns = (timestamps[1] - timestamps[0]) as f64 * period as f64;
//     let h_blur_ns = (timestamps[3] - timestamps[2]) as f64 * period as f64;
//     let v_blur_ns = (timestamps[5] - timestamps[4]) as f64 * period as f64;
//     let upsample_ns = (timestamps[7] - timestamps[6]) as f64 * period as f64;
//     let fragment_ns = (timestamps[9] - timestamps[8]) as f64 * period as f64;
//     let total = downsample_ns + h_blur_ns + v_blur_ns + upsample_ns + fragment_ns;

//     instance.downsample_ns += downsample_ns;
//     instance.h_blur_ns += h_blur_ns;
//     instance.v_blur_ns += v_blur_ns;
//     instance.upsample_ns += upsample_ns;
//     instance.fragment_ns += fragment_ns;
//     instance.total_ns += total;
//     instance.sample_count += 1.0;

//     drop(data);
//     instance.readback_buffer.unmap();
//     println!(
//         "downsample: {:.2}µs",
//         instance.downsample_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "h_blur: {:.2}µs",
//         instance.h_blur_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "v_blur: {:.2}µs",
//         instance.v_blur_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "upsample: {:.2}µs",
//         instance.upsample_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "fragment: {:.2}µs",
//         instance.fragment_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "total: {:.2}µs",
//         instance.total_ns / instance.sample_count / 1000.0
//     );
//     println!(
//         "fps: {:.2}",
//         1e9 / (instance.total_ns / instance.sample_count)
//     );
//     println!("sample_count: {:.0}", instance.sample_count);
//     println!();
// }
