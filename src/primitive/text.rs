use crate::{
    font,
    pipeline::{AtlasData, AtlasPosition, Pipeline, TextInstance, round_up},
    primitive::{copy_background, downsample, upsample},
    shader::text::TEXT_ATLAS_SIZE,
    uniforms::Uniforms,
    widget::text::GlyphData,
};

#[derive(Debug, Default, Clone)]
pub struct TextPrimitive {
    pub id: u64,
    pub text: String,
    pub font_size: f32,
    // pub line_height: f32,
    pub glyphs: Vec<GlyphData>,
    pub uniforms: Uniforms,
}

pub const MSDF_FONT_SIZE: f32 = 128.0;
pub const MSDF_PADDING: u32 = 32;

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
        // let now = std::time::Instant::now();
        let scale = viewport.scale_factor();
        let w = (bounds.width * scale) as u32;
        let h = (bounds.height * scale) as u32;
        pipeline.prepare_text_instance(device, queue, self.id, w, h, scale, &self.uniforms);

        let instance = pipeline.text_instances.get_mut(&self.id).unwrap();
        let atlas_data = &mut pipeline.atlas_data;

        instance.num_glyphs = 0;
        let vertices = create_vertex_buffer(
            atlas_data,
            instance,
            &self.glyphs,
            queue,
            bounds,
            self.font_size,
        );
        queue.write_buffer(&instance.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        // let elapsed = now.elapsed();
        // println!("Time taken to prepare text: {:?}", elapsed);
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        bounds: &iced::Rectangle<u32>,
    ) {
        // let now = std::time::Instant::now();
        let texture = target.texture();
        let instance = pipeline.text_instance(self.id);
        let width_limit = texture.width() - bounds.x;
        let height_limit = texture.height() - bounds.y;
        let copy_size = wgpu::Extent3d {
            width: round_up(bounds.width, 256).min(width_limit),
            height: round_up(bounds.height, 256).min(height_limit),
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
            instance.num_glyphs,
        );
        // let elapsed = now.elapsed();
        // println!("Time taken to render text: {:?}", elapsed);
    }
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

fn create_vertex_buffer(
    atlas_data: &mut AtlasData,
    instance: &mut TextInstance,
    glyphs: &[GlyphData],
    queue: &wgpu::Queue,
    bounds: &iced::Rectangle<f32>,
    font_size: f32,
) -> Vec<f32> {
    let mut vertices = Vec::new();
    for glyph in glyphs.iter() {
        if glyph.glyph_id == GlyphId(32) || glyph.glyph_id == GlyphId(3) {
            continue;
        }
        // println!("Glyph: {:?} (ID: {:?})", glyph.glyph_id, glyph.glyph_id);
        // println!("Data length: {}", data.len());
        // let instance = pipeline.text_instance(self.id);
        let ap = if let Some(ap) = atlas_data.atlas_position.get(&glyph.glyph_id) {
            // println!(
            //     "Glyph already in atlas: {:?} (ID: {:?})",
            //     glyph.glyph_id, glyph.glyph_id
            // );
            *ap
        } else {
            let (data, width, height, framing) = get_sdf_data(glyph.glyph_id);
            let allocation = atlas_data
                .allocator
                .allocate(size2(width as i32, height as i32))
                .unwrap();

            let (offset_x, offset_y) = allocation.rectangle.min.to_tuple();
            let offset_x = offset_x as u32;
            let offset_y = offset_y as u32;

            let font = Face::parse(font::FONT, 0).unwrap();
            // println!(
            //     "Adding glyph to atlas: {:?} at offset: ({}, {})",
            //     glyph.glyph_id, offset_x, offset_y
            // );
            // println!(
            //     "Getting glyph bounding box for glyph: (ID: {:?})",
            //     glyph.glyph_id
            // );
            let bbox = font.glyph_bounding_box(glyph.glyph_id).unwrap();
            let units_per_em = font.units_per_em() as f32;

            let ap = AtlasPosition {
                x: offset_x,
                y: offset_y,
                width,
                height,
                bbox,
                units_per_em,
                framing,
            };
            atlas_data.atlas_position.insert(glyph.glyph_id, ap);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas_data.texture_atlas,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: offset_x,
                        y: offset_y,
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
            // offset_x += width + 2;
            // if offset_x > TEXT_ATLAS_SIZE {
            //     offset_x = 0;
            //     offset_y += 100;
            // }

            ap
        };

        // let bbox = ap.bbox;
        // let scale = self.font_size / ap.units_per_em;
        // let glyph_width = (bbox.x_max - bbox.x_min) as f32 * scale;
        // let glyph_height = (bbox.y_max - bbox.y_min) as f32 * scale;
        // let bearing_x = bbox.x_min as f32 * scale;
        // let bearing_y = bbox.y_max as f32 * scale; // top of glyph relative to baseline

        // println!(
        //     "glyph.y_offset ({:?}): {:.1}",
        //     glyph.glyph_id, glyph.y_offset
        // );
        // let screen_scale = self.font_size / MSDF_FONT_SIZE;
        // let top = glyph.run_line_y + glyph.y_offset - bearing_y;
        // let bottom = glyph.run_line_y - bbox.y_min as f32 * scale;
        // let pad = 8.0 * screen_scale; // uniform padding in screen pixels

        // bitmap-to-screen scale: how many screen pixels per bitmap pixel
        let origin_bmp_x =
            (ap.framing.projection.translate.x * ap.framing.projection.scale.x) as f32;
        let origin_bmp_y =
            (ap.framing.projection.translate.y * ap.framing.projection.scale.y) as f32;
        let bmp_to_screen = (font_size / ap.units_per_em) / ap.framing.projection.scale.x as f32;

        // Full quad = full bitmap mapped to screen
        let quad_w = ap.width as f32 * bmp_to_screen;
        let quad_h = ap.height as f32 * bmp_to_screen;

        // Origin in bitmap (after flip_y)
        let origin_bmp_y_flipped = ap.height as f32 - origin_bmp_y;

        // Anchor the quad so the font origin maps to (glyph.x, run_line_y)
        let quad_x = glyph.x - origin_bmp_x * bmp_to_screen;
        let quad_y = glyph.run_line_y - origin_bmp_y_flipped * bmp_to_screen;

        // println!(
        //     "quad_x: {:.1}, quad_y: {:.1}, quad_w: {:.1}, quad_h: {:.1}",
        //     quad_x, quad_y, quad_w, quad_h
        // );

        let uv_left = ap.x as f32 / TEXT_ATLAS_SIZE as f32;
        let uv_bottom = ap.y as f32 / TEXT_ATLAS_SIZE as f32; // glyph.y / TEXT_ATLAS_SIZE as f32;
        let uv_right = (ap.x as f32 + ap.width as f32) / TEXT_ATLAS_SIZE as f32;
        let uv_top = (ap.y as f32 + ap.height as f32) / TEXT_ATLAS_SIZE as f32;

        let clip_x = (quad_x / bounds.width) * 2.0 - 1.0;
        let clip_y = 1.0 - (quad_y / bounds.height) * 2.0;
        let clip_w = (quad_w / bounds.width) * 2.0;
        let clip_h = -(quad_h / bounds.height) * 2.0; // negative because Y is flipped
        // println!("Clip: {} {} {} {}", clip_x, clip_y, clip_w, clip_h);
        // println!(
        //     "bbox: x_min={}, y_min={}, x_max={}, y_max={}",
        //     bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max
        // );
        // println!("glyph pixel size: {:.1}x{:.1}", glyph_width, glyph_height);
        // println!("bounds: {:.1}x{:.1}", bounds.width, bounds.height);
        let sdf_scale = 16.0 * bmp_to_screen; // range_px * bmp_to_screen
        vertices.extend_from_slice(&[
            clip_x, clip_y, uv_left, uv_top, // TL
            sdf_scale,
        ]);
        vertices.extend_from_slice(&[
            clip_x + clip_w,
            clip_y,
            uv_right,
            uv_top, // TR
            sdf_scale,
        ]);
        vertices.extend_from_slice(&[
            clip_x,
            clip_y + clip_h,
            uv_left,
            uv_bottom, // BL
            sdf_scale,
        ]);
        vertices.extend_from_slice(&[
            clip_x,
            clip_y + clip_h,
            uv_left,
            uv_bottom, // BL
            sdf_scale,
        ]);
        vertices.extend_from_slice(&[
            clip_x + clip_w,
            clip_y,
            uv_right,
            uv_top, // TR
            sdf_scale,
        ]);
        vertices.extend_from_slice(&[
            clip_x + clip_w,
            clip_y + clip_h,
            uv_right,
            uv_bottom, // BR
            sdf_scale,
        ]);
        instance.num_glyphs += 1;
        // println!();
    }
    vertices
}

use etagere::size2;
use msdfgen::{Bitmap, FillRule, FontExt, Framing, MsdfGeneratorConfig, Range, Rgb};
use ttf_parser::{Face, GlyphId};
fn get_sdf_data(glyph: GlyphId) -> (Vec<u8>, u32, u32, Framing<f64>) {
    // let c = glyph;

    let font = Face::parse(font::FONT, 0).unwrap();
    // let glyph = font.glyph_index(glyph).unwrap();

    let mut shape = font.glyph_shape(glyph).unwrap();

    let (width, height) = get_font_size(&font, glyph);
    // println!("Font size MSDF: {}x{} (ID: {:?})", width, height, glyph);

    let bound = shape.get_bound();
    let framing = bound
        .autoframe(width, height, Range::Px(MSDF_PADDING as f64), None)
        .unwrap();

    // framing.scale = msdfgen::Vector2 { x: 0.04, y: 0.04 };
    // println!("framing: {:?}", framing);

    // This helps with glyph positioning, but could affect SDF accuracy
    // framing.translate.x = 100.0;
    // framing.translate.y = 100.0;
    let fill_rule = FillRule::default();

    let mut bitmap = Bitmap::<Rgb<f32>>::new(width, height);

    shape.edge_coloring_simple(3.0, 0);

    let config = MsdfGeneratorConfig::default();

    shape.generate_msdf(&mut bitmap, framing, config);

    // optionally
    shape.correct_sign(&mut bitmap, framing, fill_rule);
    shape.correct_msdf_error(&mut bitmap, framing, config);

    // bitmap.flip_y();
    let data = to_rgbau8(&bitmap);

    (data, width, height, framing)
}

fn to_rgbau8(bitmap: &Bitmap<Rgb<f32>>) -> Vec<u8> {
    bitmap
        .pixels()
        .iter()
        .flat_map(|p| {
            [
                (p.r.clamp(0.0, 1.0) * 255.0) as u8,
                (p.g.clamp(0.0, 1.0) * 255.0) as u8,
                (p.b.clamp(0.0, 1.0) * 255.0) as u8,
                // (p.a.clamp(0.0, 1.0) * 255.0) as u8,
                255u8,
            ]
        })
        .collect()
}

fn get_font_size(font: &Face, glyph: GlyphId) -> (u32, u32) {
    let bbox = font.glyph_bounding_box(glyph).unwrap();
    let units_per_em = font.units_per_em() as f32;
    let scale = MSDF_FONT_SIZE / units_per_em;

    let x_min = bbox.x_min as f32 * scale;
    let y_min = bbox.y_min as f32 * scale;
    let x_max = bbox.x_max as f32 * scale;
    let y_max = bbox.y_max as f32 * scale;
    let width = x_max - x_min;
    let height = y_max - y_min;

    (
        width.ceil() as u32 + MSDF_PADDING * 2,
        height.ceil() as u32 + MSDF_PADDING * 2,
    )
}
