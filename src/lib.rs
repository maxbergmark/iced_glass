pub mod container;
pub mod pipeline;
pub mod primitive;
// pub mod program;
pub mod shader;
pub mod uniforms;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SetScale(f32),
    SetBlurRadius(f32),
    SetCornerRadius(f32),
    SetSaturation(f32),
    SetLightness(f32),
    MouseMove(iced::Point),
    MouseState(bool),
    SetSubBlurRadius(f32, usize),
    SetSubLightness(f32, usize),
}
