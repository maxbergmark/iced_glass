use crate::{shader::MIP_LEVEL_COUNT, widget::EdgeType};

#[derive(Debug, Clone, Copy)]
pub struct Uniforms {
    pub blur_radius: f32,
    pub corner_radius: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub edge_radius: f32,
    pub height: f32,
    pub refractive_index: f32,
    pub chromatic_aberration: f32,
    pub rim_width: f32,
    pub rim_angle: f32,
    pub opacity: f32,
    pub tint: iced::Color,
    pub scrim: iced::Color,
    pub content_scale: (f32, f32),
    pub edge_type: EdgeType,
    pub num_children: u32,
    pub blending_factor: f32,
}

impl Uniforms {
    #[must_use]
    pub fn to_raw(self, direction: [f32; 2], scale: f32) -> Raw {
        Raw {
            blur_radius: self.blur_radius * scale / self.mip_factor(),
            corner_radius: self.corner_radius * scale,
            saturation: self.saturation,
            lightness: self.lightness,
            direction,
            edge_radius: self.edge_radius * scale,
            height: self.height * scale,
            refractive_index: self.refractive_index,
            chromatic_aberration: self.chromatic_aberration,
            rim_width: self.rim_width * scale,
            rim_angle: self.rim_angle,
            opacity: self.opacity,
            tint: [self.tint.r, self.tint.g, self.tint.b, self.tint.a],
            scrim: [self.scrim.r, self.scrim.g, self.scrim.b, self.scrim.a],
            edge_type: match self.edge_type {
                EdgeType::GlassEdge => 0,
                EdgeType::SoftEdge => 1,
            },
            content_scale: [self.content_scale.0, self.content_scale.1],
            num_children: self.num_children,
            blending_factor: self.blending_factor,
            _pad: 0.0,
            _pad2: 0.0,
        }
    }

    #[must_use]
    pub fn mip_level(self) -> u32 {
        match self.blur_radius {
            r if r < 10.0 => 0,
            r if r < 50.0 => 1,
            r if r < 100.0 => 2,
            r if r < 200.0 => 3,
            // r if r < 800.0 => 4,
            _ => MIP_LEVEL_COUNT - 1, // 4 is the highest mip level for now
        }
    }

    fn mip_factor(self) -> f32 {
        4.0_f32.powi(self.mip_level() as i32)
    }
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub tint: [f32; 4],
    pub scrim: [f32; 4],
    pub direction: [f32; 2],
    pub content_scale: [f32; 2],

    pub blur_radius: f32,
    pub corner_radius: f32,
    pub saturation: f32,
    pub lightness: f32,

    pub edge_radius: f32,
    pub height: f32,
    pub refractive_index: f32,
    pub rim_width: f32,

    pub opacity: f32,
    pub edge_type: i32,
    pub chromatic_aberration: f32,
    pub rim_angle: f32,

    pub num_children: u32,
    pub blending_factor: f32,
    pub _pad: f32,
    pub _pad2: f32,
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ChildRaw {
    pub center: [f32; 2], // in pixels, relative to the group's bounds
    pub half_size: [f32; 2],
    // pub corner_radius: f32,
    // pub _pad: [f32; 3], // pad to 32 bytes (multiple of 16)
}
