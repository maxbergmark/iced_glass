use crate::{Direction, shader::MIP_LEVEL_COUNT, widget::EdgeType};

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
    pub fill_level: f32,
    pub fill_color: iced::Color,
    pub fill_direction: Direction,
}

impl Uniforms {
    #[must_use]
    pub fn to_raw(self, direction: [f32; 2], scale: f32) -> Raw {
        Raw {
            blur_radius: self.blur_radius * scale / self.mip_factor(scale),
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
            tint: to_array(self.tint),
            scrim: to_array(self.scrim),
            edge_type: match self.edge_type {
                EdgeType::GlassEdge => 0,
                EdgeType::SoftEdge => 1,
            },
            content_scale: [self.content_scale.0, self.content_scale.1],
            num_children: self.num_children,
            blending_factor: self.blending_factor,
            fill_level: self.fill_level,
            fill_color: to_array(self.fill_color),
            fill_direction: match self.fill_direction {
                Direction::Horizontal => 0,
                Direction::Vertical => 1,
            },
        }
    }

    #[must_use]
    pub fn mip_level(&self, scale: f32) -> u32 {
        match self.blur_radius * scale {
            r if r < 10.0 => 0,
            r if r < 50.0 => 1,
            r if r < 100.0 => 2,
            r if r < 200.0 => 3,
            r if r < 800.0 => 4,
            r if r < 1600.0 => 5,
            _ => MIP_LEVEL_COUNT - 1, // 6 is the highest mip level for now
        }
    }

    fn mip_factor(&self, scale: f32) -> f32 {
        4.0_f32.powi(self.mip_level(scale) as i32)
    }
}

const fn to_array(color: iced::Color) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub tint: [f32; 4],
    pub scrim: [f32; 4],
    pub fill_color: [f32; 4],
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
    pub fill_level: f32,
    pub fill_direction: i32,
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ChildRaw {
    pub center: [f32; 2], // in pixels, relative to the group's bounds
    pub half_size: [f32; 2],
    // pub corner_radius: f32,
    // pub _pad: [f32; 3], // pad to 32 bytes (multiple of 16)
}
