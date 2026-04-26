use crate::{
    font,
    pipeline::{Instance, Pipeline, TextInstance},
    shader::text::TEXT_ATLAS_SIZE,
    uniforms::Uniforms,
    widget::text::GlyphData,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Primitive {
    pub id: u64,
    pub uniforms: Uniforms,
}

#[derive(Debug, Default, Clone)]
pub struct TextPrimitive {
    pub id: u64,
    pub text: String,
    pub font_size: f32,
    // pub line_height: f32,
    pub glyphs: Vec<GlyphData>,
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
        downsample(
            encoder,
            pipeline,
            &instance.tex_a_bg,
            &instance.tex_a,
            mip_level,
        );
        horizontal_blur(encoder, pipeline, instance, mip_level);
        vertical_blur(encoder, pipeline, instance, mip_level);
        upsample(
            encoder,
            pipeline,
            &instance.tex_a_bg,
            &instance.tex_a,
            mip_level,
        );
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
    tex_a_bg: &[wgpu::BindGroup],
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
    tex_a_bg: &[wgpu::BindGroup],
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
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    vertical_pass.set_pipeline(&pipeline.blur_pipeline);
    vertical_pass.set_bind_group(0, &instance.tex_b_bg[mip_level as usize], &[]);
    vertical_pass.set_bind_group(1, &instance.uniform_bg_v, &[]);
    vertical_pass.draw(0..6, 0..1);
}

fn text_horizontal_blur(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &TextInstance,
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

fn text_vertical_blur(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &TextInstance,
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

impl iced::widget::shader::Primitive for TextPrimitive {
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
        pipeline.prepare_text_instance(
            device,
            queue,
            self.id,
            width,
            height,
            scale,
            &self.uniforms,
        );

        let instance = pipeline.text_instance(self.id);
        let mut vertices = Vec::new();
        let mut offset = 0;

        for glyph in self.glyphs.iter() {
            println!("Glyph: {:?} (ID: {:?})", glyph.glyph, glyph.glyph_id);
            let (data, width, height, baseline_from_top, origin_from_left) =
                get_sdf_data(glyph.glyph);
            // println!("Data length: {}", data.len());
            let instance = pipeline.text_instance(self.id);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &instance.texture_atlas,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: offset,
                        y: 0,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            // write quad vertices to vertices vector
            // let f = include_bytes!("/System/Library/Fonts/Supplemental/Arial Unicode.ttf");
            // let f = font::FONT;
            let font = Face::parse(font::FONT, 0).unwrap();
            let glyph_id = ttf_parser::GlyphId(glyph.glyph_id);
            let bbox = font.glyph_bounding_box(glyph_id).unwrap();
            let units_per_em = font.units_per_em() as f32;
            let scale = self.font_size / units_per_em;
            let glyph_width = (bbox.x_max - bbox.x_min) as f32 * scale;
            let glyph_height = (bbox.y_max - bbox.y_min) as f32 * scale;
            let bearing_x = bbox.x_min as f32 * scale;
            let bearing_y = bbox.y_max as f32 * scale; // top of glyph relative to baseline

            // let top = glyph.run_line_y - bbox.y_max as f32 * scale;
            let top = glyph.run_line_y - bearing_y;
            let bottom = glyph.run_line_y - bbox.y_min as f32 * scale;

            println!("top: {:.1}, bottom: {:.1}", top, bottom);

            let pad_x = 8.0 * (glyph_width / (width as f32 - 16.0));
            let pad_y = 8.0 * (glyph_height / (height as f32 - 16.0));

            let quad_x = glyph.x + bearing_x - pad_x;
            let quad_y = top - pad_y;
            let quad_h = bottom - top + 2.0 * pad_y;
            // let quad_x = glyph.x + bearing_x;
            // let quad_y = glyph.run_line_y - bearing_y; // baseline minus top bearing (Y flipped)
            let quad_w = glyph_width + 2.0 * pad_x;
            // let quad_h = glyph_height;

            // let screen_scale_x = self.font_size / (get_font_size_scale() * units_per_em);

            // ... or more simply, use the ratio of screen glyph size to bitmap content size:
            let screen_scale = self.font_size / 64.0; // 64.0 = the font_size used in get_font_size()
            let top = glyph.run_line_y - bearing_y;
            let bottom = glyph.run_line_y - bbox.y_min as f32 * scale;
            let pad = 8.0 * screen_scale; // uniform padding in screen pixels
            let quad_x = glyph.x + bearing_x - pad;
            let quad_y = top - pad;
            let quad_w = glyph_width + 2.0 * pad;
            let quad_h = (bottom - top) + 2.0 * pad;

            // let quad_y = glyph.run_line_y - baseline_from_top * screen_scale;
            // let quad_h = height as f32 * screen_scale;

            // let quad_x = glyph.x - origin_from_left * screen_scale;
            // let quad_w = width as f32 * screen_scale;

            let uv_left = offset as f32 / TEXT_ATLAS_SIZE as f32;
            let uv_top = 0.0; // glyph.y / TEXT_ATLAS_SIZE as f32;
            let uv_right = (offset as f32 + width as f32) / TEXT_ATLAS_SIZE as f32;
            let uv_bottom = (height as f32) / TEXT_ATLAS_SIZE as f32;

            let clip_x = (quad_x / bounds.width) * 2.0 - 1.0;
            let clip_y = 1.0 - (quad_y / bounds.height) * 2.0;
            let clip_w = (quad_w / bounds.width) * 2.0;
            let clip_h = -(quad_h / bounds.height) * 2.0; // negative because Y is flipped
            println!("Clip: {} {} {} {}", clip_x, clip_y, clip_w, clip_h);
            println!(
                "bbox: x_min={}, y_min={}, x_max={}, y_max={}",
                bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max
            );
            println!("glyph pixel size: {:.1}x{:.1}", glyph_width, glyph_height);
            println!("bounds: {:.1}x{:.1}", bounds.width, bounds.height);
            vertices.extend_from_slice(&[
                clip_x, clip_y, uv_left, uv_top, // TL
            ]);
            vertices.extend_from_slice(&[
                clip_x + clip_w,
                clip_y,
                uv_right,
                uv_top, // TR
            ]);
            vertices.extend_from_slice(&[
                clip_x,
                clip_y + clip_h,
                uv_left,
                uv_bottom, // BL
            ]);
            vertices.extend_from_slice(&[
                clip_x,
                clip_y + clip_h,
                uv_left,
                uv_bottom, // BL
            ]);
            vertices.extend_from_slice(&[
                clip_x + clip_w,
                clip_y,
                uv_right,
                uv_top, // TR
            ]);
            vertices.extend_from_slice(&[
                clip_x + clip_w,
                clip_y + clip_h,
                uv_right,
                uv_bottom, // BR
            ]);
            offset += width + 2;
            println!();
        }
        println!();
        println!("vertices: {:?}", vertices.len());
        queue.write_buffer(&instance.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        bounds: &iced::Rectangle<u32>,
    ) {
        let texture = target.texture();
        let instance = pipeline.text_instance(self.id);
        let copy_size = wgpu::Extent3d {
            width: bounds.width,
            height: bounds.height,
            depth_or_array_layers: 1,
        };
        // let copy_size = match calculate_copy_size(texture, instance, bounds) {
        //     Some(size) => size,
        //     None => return,
        // };

        let mip_level = self.uniforms.mip_level();
        copy_background(encoder, &instance.tex_a, texture, bounds, &copy_size);
        downsample(
            encoder,
            pipeline,
            &instance.tex_a_bg,
            &instance.tex_a,
            mip_level,
        );
        text_horizontal_blur(encoder, pipeline, instance, mip_level);
        text_vertical_blur(encoder, pipeline, instance, mip_level);
        upsample(
            encoder,
            pipeline,
            &instance.tex_a_bg,
            &instance.tex_a,
            mip_level,
        );
        text_pass(
            encoder,
            pipeline,
            instance,
            target,
            bounds,
            self.glyphs.len() as u32,
        );
    }
}

fn text_pass(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &Pipeline,
    instance: &TextInstance,
    target: &wgpu::TextureView,
    bounds: &iced::Rectangle<u32>,
    num_glyphs: u32,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("text.render_pass"),
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

    pass.set_pipeline(&pipeline.text_pipeline);
    pass.set_bind_group(0, &instance.texture_atlas_bg, &[]);
    pass.set_bind_group(1, &instance.uniform_bg_h, &[]);
    pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
    // pass.set_index_buffer(instance.index_buffer.slice(..));
    // pass.draw(0..(num_glyphs * 6), 0..1);
    pass.draw(0..(num_glyphs * 6), 0..1);
}

use ttf_parser::{Face, GlyphId};
fn get_sdf_data(glyph: char) -> (Vec<u8>, u32, u32, f32, f32) {
    let c = glyph;
    use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig, Range};

    // let f = include_bytes!("/System/Library/Fonts/Supplemental/Arial Unicode.ttf");
    // let font = Face::parse(f, 0).unwrap();
    let font = Face::parse(font::FONT, 0).unwrap();
    let glyph = font.glyph_index(glyph).unwrap();

    let mut shape = font.glyph_shape(glyph).unwrap();
    // let mut shape = font.(glyph).unwrap();

    // let width = TEXT_ATLAS_SIZE;
    // let height = TEXT_ATLAS_SIZE;
    let (width, height) = get_font_size(&font, glyph);
    println!(
        "Font size MSDF '{}': {}x{} (ID: {:?})",
        c, width, height, glyph
    );

    let bound = shape.get_bound();
    let mut framing = bound
        .autoframe(width, height, Range::Px(8.0), None)
        .unwrap();
    println!("framing: {:?}", framing);
    println!("framing.scale: {:?}", framing.scale);
    println!("framing.translate: {:?}", framing.translate);
    framing.translate.x = 100.0;
    framing.translate.y = 100.0;
    // println!("framing: {:?}", framing.translate);
    // framing.translate.y = 0.0;
    // framing.projection.scale; // Vec2<f64
    // framing.projection.translate; // Vec2<f64
    // framing.range; // f64
    // framing.scale; // Vec2<f64>
    // framing.translate; // Vec2<f64>
    let fill_rule = FillRule::default();

    let mut bitmap = Bitmap::new(width, height);

    shape.edge_coloring_simple(3.0, 0);

    let config = MsdfGeneratorConfig::default();

    shape.generate_msdf(&mut bitmap, framing, config);

    // optionally
    shape.correct_sign(&mut bitmap, framing, fill_rule);
    shape.correct_msdf_error(&mut bitmap, framing, config);

    // let error = shape.estimate_error(&mut bitmap, framing, 5, Default::default());

    // println!("Estimated error: {}", error);

    bitmap.flip_y();

    let data = bitmap
        .pixels()
        .iter()
        .flat_map(|p| {
            [
                (p.r.clamp(0.0, 1.0) * 255.0) as u8,
                (p.g.clamp(0.0, 1.0) * 255.0) as u8,
                (p.b.clamp(0.0, 1.0) * 255.0) as u8,
                255u8,
            ]
        })
        .collect();

    let baseline_from_bottom = (framing.scale.y * framing.translate.y) as f32;
    let baseline_from_top = height as f32 - baseline_from_bottom;
    let origin_from_left = (framing.scale.x * framing.translate.x) as f32;
    (data, width, height, baseline_from_top, origin_from_left)
}

fn get_font_size(font: &Face, glyph: GlyphId) -> (u32, u32) {
    let bbox = font.glyph_bounding_box(glyph).unwrap();
    let units_per_em = font.units_per_em() as f32;
    let font_size = 64.0;
    let scale = font_size / units_per_em;

    let x_min = bbox.x_min as f32 * scale;
    let y_min = bbox.y_min as f32 * scale;
    let x_max = bbox.x_max as f32 * scale;
    let y_max = bbox.y_max as f32 * scale;
    let width = x_max - x_min;
    let height = y_max - y_min;

    (width.ceil() as u32 + 16, height.ceil() as u32 + 16)
}
