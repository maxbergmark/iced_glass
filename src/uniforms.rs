use crate::shader::MIP_LEVEL_COUNT;

#[derive(Debug, Default, Clone, Copy)]
pub struct Uniforms {
    pub blur_radius: f32,
    pub corner_radius: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub edge_radius: f32,
    pub height: f32,
    pub refractive_index: f32,
    pub rim_width: f32,
    pub opacity: f32,
    pub tint: iced::Color,
    pub content_scale: (f32, f32),
}

impl Uniforms {
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
            rim_width: self.rim_width * scale,
            opacity: self.opacity,
            tint: [self.tint.r, self.tint.g, self.tint.b, self.tint.a],
            _pad: 0.0,
            _pad2: [0.0, 0.0],
            content_scale: [self.content_scale.0, self.content_scale.1],
        }
    }

    pub fn mip_level(self) -> u32 {
        match self.blur_radius {
            r if r < 50.0 => 0,
            r if r < 100.0 => 1,
            r if r < 200.0 => 2,
            r if r < 400.0 => 3,
            r if r < 800.0 => 4,
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
    _pad: f32,
    _pad2: [f32; 2],
}
