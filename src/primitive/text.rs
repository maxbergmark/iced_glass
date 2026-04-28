use crate::{
    font,
    pipeline::{AtlasData, AtlasPosition, Pipeline, TextInstance, round_up},
    primitive::{copy_background, downsample, horizontal_blur, upsample, vertical_blur},
    shader::text::TEXT_ATLAS_SIZE,
    uniforms::Uniforms,
    widget::text::GlyphData,
};

#[derive(Debug, Default, Clone)]
pub struct TextPrimitive {
    pub id: u64,
    pub text: String,
    pub font_size: f32,
    pub glyphs: Vec<GlyphData>,
    pub uniforms: Uniforms,
}

// TODO: make these configurable
pub const MSDF_FONT_SIZE: f32 = 64.0;
pub const MSDF_PADDING: u32 = 32;
pub const VERTICES_PER_GLYPH: u32 = 6;

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
        let size = iced::Size::new(
            (bounds.width * scale) as u32,
            (bounds.height * scale) as u32,
        );
        pipeline.prepare_text_instance(device, queue, self.id, size, scale, &self.uniforms);

        let instance = pipeline.text_instances.get_mut(&self.id).unwrap();
        let atlas_data = &mut pipeline.atlas_data;

        let vertices = self
            .create_vertex_buffer(atlas_data, queue, bounds)
            .unwrap_or_default();
        instance.num_glyphs = vertices.len() as u32 / VERTICES_PER_GLYPH;
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
        let text_instance = pipeline.text_instance(self.id);
        let instance = &text_instance.instance;
        let width_limit = texture.width() - bounds.x;
        let height_limit = texture.height() - bounds.y;
        let copy_size = wgpu::Extent3d {
            width: round_up(bounds.width, 256).min(width_limit),
            height: round_up(bounds.height, 256).min(height_limit),
            depth_or_array_layers: 1,
        };

        let mip_level = self.uniforms.mip_level();
        copy_background(encoder, &instance.tex_a, texture, bounds, &copy_size);
        downsample(encoder, pipeline, instance, mip_level);
        horizontal_blur(encoder, pipeline, instance, mip_level);
        vertical_blur(encoder, pipeline, instance, mip_level);
        upsample(encoder, pipeline, instance, mip_level);
        text_pass(
            encoder,
            pipeline,
            text_instance,
            target,
            bounds,
            text_instance.num_glyphs,
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
    pass.set_bind_group(1, &instance.instance.uniform_bg_h, &[]);
    pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
    pass.draw(0..(num_glyphs * 6), 0..1);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexData {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub sdf_scale: f32,
}

impl TextPrimitive {
    fn create_vertex_buffer(
        &self,
        atlas_data: &mut AtlasData,
        queue: &wgpu::Queue,
        bounds: &iced::Rectangle<f32>,
    ) -> Option<Vec<VertexData>> {
        let mut vertices = Vec::new();
        for glyph in self.glyphs.iter().filter(is_visible) {
            let ap = atlas_data
                .atlas_position
                .get(&glyph.glyph_id)
                .copied()
                .or_else(|| add_to_atlas(atlas_data, queue, glyph))?;

            vertices.extend(add_vertices(bounds, &ap, glyph, self.font_size));
        }
        Some(vertices)
    }
}

fn is_visible(glyph: &&GlyphData) -> bool {
    glyph.glyph_id != GlyphId(32) && glyph.glyph_id != GlyphId(3)
}

use etagere::size2;
use msdfgen::{Bitmap, FillRule, FontExt, Framing, MsdfGeneratorConfig, Range, Rgb};
use ttf_parser::{Face, GlyphId};
fn get_sdf_data(glyph: GlyphId) -> Option<(Vec<u8>, iced::Size<u32>, Framing<f64>)> {
    // let c = glyph;

    let font = Face::parse(font::FONT, 0).ok()?;
    // let glyph = font.glyph_index(glyph).unwrap();

    let mut shape = font.glyph_shape(glyph)?;

    let size = get_glyph_size(&font, glyph);
    // println!("Font size MSDF: {}x{} (ID: {:?})", width, height, glyph);

    let bound = shape.get_bound();
    let range = Range::Px(MSDF_PADDING as f64);
    let framing = bound.autoframe(size.width, size.height, range, None)?;

    let fill_rule = FillRule::default();
    let mut bitmap = Bitmap::<Rgb<f32>>::new(size.width, size.height);

    shape.edge_coloring_simple(3.0, 0);

    let config = MsdfGeneratorConfig::default();

    shape.generate_msdf(&mut bitmap, framing, config);

    // optionally
    shape.correct_sign(&mut bitmap, framing, fill_rule);
    shape.correct_msdf_error(&mut bitmap, framing, config);

    // bitmap.flip_y();
    let data = to_rgbau8(&bitmap);

    Some((data, size, framing))
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

fn get_glyph_size(font: &Face, glyph: GlyphId) -> iced::Size<u32> {
    let bbox = font.glyph_bounding_box(glyph).unwrap();
    let units_per_em = font.units_per_em() as f32;
    let scale = MSDF_FONT_SIZE / units_per_em;

    let x_min = bbox.x_min as f32 * scale;
    let y_min = bbox.y_min as f32 * scale;
    let x_max = bbox.x_max as f32 * scale;
    let y_max = bbox.y_max as f32 * scale;
    let width = x_max - x_min;
    let height = y_max - y_min;

    iced::Size::new(
        width.ceil() as u32 + MSDF_PADDING * 2,
        height.ceil() as u32 + MSDF_PADDING * 2,
    )
}

fn add_to_atlas(
    atlas_data: &mut AtlasData,
    queue: &wgpu::Queue,
    glyph: &GlyphData,
) -> Option<AtlasPosition> {
    let (data, size, framing) = get_sdf_data(glyph.glyph_id)?;
    let allocation = atlas_data
        .allocator
        .allocate(size2(size.width as i32, size.height as i32))
        .unwrap();

    let offset = allocation.rectangle.min;
    let position = iced::Point::new(offset.x as u32, offset.y as u32);

    let font = Face::parse(font::FONT, 0).unwrap();
    let bbox = font.glyph_bounding_box(glyph.glyph_id).unwrap();
    let units_per_em = font.units_per_em() as f32;

    let ap = AtlasPosition {
        position,
        size,
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
                x: position.x,
                y: position.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.width * 4),
            rows_per_image: Some(size.height),
        },
        wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
    );
    // offset_x += width + 2;
    // if offset_x > TEXT_ATLAS_SIZE {
    //     offset_x = 0;
    //     offset_y += 100;
    // }

    Some(ap)
}

struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

fn compute_quad(ap: &AtlasPosition, glyph: &GlyphData, bmp_to_screen: f32) -> Quad {
    // bitmap-to-screen scale: how many screen pixels per bitmap pixel
    let origin_bmp_x = (ap.framing.projection.translate.x * ap.framing.projection.scale.x) as f32;
    let origin_bmp_y = (ap.framing.projection.translate.y * ap.framing.projection.scale.y) as f32;

    // Origin in bitmap (after flip_y)
    let origin_bmp_y_flipped = ap.size.height as f32 - origin_bmp_y;

    // Full quad = full bitmap mapped to screen
    let quad_w = ap.size.width as f32 * bmp_to_screen;
    let quad_h = ap.size.height as f32 * bmp_to_screen;

    // Anchor the quad so the font origin maps to (glyph.x, run_line_y)
    let quad_x = glyph.x - origin_bmp_x * bmp_to_screen;
    let quad_y = glyph.run_line_y - origin_bmp_y_flipped * bmp_to_screen;

    Quad {
        x: quad_x,
        y: quad_y,
        w: quad_w,
        h: quad_h,
    }
}

fn compute_clip(
    ap: &AtlasPosition,
    bounds: &iced::Rectangle<f32>,
    glyph: &GlyphData,
    bmp_to_screen: f32,
) -> Bounds {
    let quad = compute_quad(ap, glyph, bmp_to_screen);

    let clip_x = (quad.x / bounds.width) * 2.0 - 1.0;
    let clip_y = 1.0 - (quad.y / bounds.height) * 2.0;
    let clip_w = (quad.w / bounds.width) * 2.0;
    let clip_h = -(quad.h / bounds.height) * 2.0; // negative because Y is flipped

    Bounds {
        left: clip_x,
        bottom: clip_y,
        right: clip_x + clip_w,
        top: clip_y + clip_h,
    }
}

struct Bounds {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

fn compute_uv(ap: &AtlasPosition) -> Bounds {
    let uv_left = ap.position.x as f32 / TEXT_ATLAS_SIZE as f32;
    let uv_bottom = ap.position.y as f32 / TEXT_ATLAS_SIZE as f32; // glyph.y / TEXT_ATLAS_SIZE as f32;
    let uv_right = (ap.position.x as f32 + ap.size.width as f32) / TEXT_ATLAS_SIZE as f32;
    let uv_top = (ap.position.y as f32 + ap.size.height as f32) / TEXT_ATLAS_SIZE as f32;

    Bounds {
        left: uv_left,
        bottom: uv_bottom,
        right: uv_right,
        top: uv_top,
    }
}

fn add_vertices(
    bounds: &iced::Rectangle<f32>,
    ap: &AtlasPosition,
    glyph: &GlyphData,
    font_size: f32,
) -> impl Iterator<Item = VertexData> {
    let bmp_to_screen = (font_size / ap.units_per_em) / ap.framing.projection.scale.x as f32;
    let clip = compute_clip(ap, bounds, glyph, bmp_to_screen);
    let uv = compute_uv(ap);
    let sdf_scale = MSDF_PADDING as f32 * bmp_to_screen;

    [
        VertexData {
            x: clip.left,
            y: clip.bottom,
            u: uv.left,
            v: uv.top,
            sdf_scale,
        },
        VertexData {
            x: clip.right,
            y: clip.bottom,
            u: uv.right,
            v: uv.top,
            sdf_scale,
        },
        VertexData {
            x: clip.left,
            y: clip.top,
            u: uv.left,
            v: uv.bottom,
            sdf_scale,
        },
        VertexData {
            x: clip.left,
            y: clip.top,
            u: uv.left,
            v: uv.bottom,
            sdf_scale,
        },
        VertexData {
            x: clip.right,
            y: clip.bottom,
            u: uv.right,
            v: uv.top,
            sdf_scale,
        },
        VertexData {
            x: clip.right,
            y: clip.top,
            u: uv.right,
            v: uv.bottom,
            sdf_scale,
        },
    ]
    .into_iter()
    // .flatten()
}
