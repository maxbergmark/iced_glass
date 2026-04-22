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
}

impl Uniforms {
    pub fn to_raw(self, direction: [f32; 2], scale: f32) -> Raw {
        Raw {
            radius: self.blur_radius * scale,
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
        }
    }
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub tint: [f32; 4],
    pub direction: [f32; 2],
    pub radius: f32,
    pub corner_radius: f32,

    pub saturation: f32,
    pub lightness: f32,
    pub edge_radius: f32,
    pub height: f32,

    pub refractive_index: f32,
    pub rim_width: f32,
    pub opacity: f32,
    pub _pad: f32,
}
