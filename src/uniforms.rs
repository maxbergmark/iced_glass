#[derive(Debug, Default, Clone, Copy)]
pub struct Uniforms {
    pub blur_radius: f32,
    pub corner_radius: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl Uniforms {
    pub fn to_raw(self) -> Raw {
        Raw {
            radius: self.blur_radius,
            corner_radius: self.corner_radius,
            saturation: self.saturation,
            lightness: self.lightness,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub radius: f32,
    pub corner_radius: f32,
    pub saturation: f32,
    pub lightness: f32,
}
