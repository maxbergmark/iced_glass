use std::collections::HashMap;

use cosmic_text::fontdb;
use etagere::AtlasAllocator;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct GlyphId(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct AtlasPosition {
    pub position: iced::Point<u32>,
    pub size: iced::Size<u32>,
    // pub bbox: Rect,
    pub units_per_em: f32,
    pub framing: msdfgen::Framing<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

pub struct AtlasData {
    pub texture_atlas: wgpu::Texture,
    pub atlas_position: HashMap<(fontdb::ID, GlyphId), AtlasPosition>,
    pub allocator: AtlasAllocator,
}

impl std::fmt::Debug for AtlasData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlasData")
            .field("texture_atlas", &self.texture_atlas)
            .field("atlas_position", &self.atlas_position)
            .finish_non_exhaustive()
    }
}
