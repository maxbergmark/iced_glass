use iced::{
    Color, Theme,
    widget::{svg, text},
};

#[derive(Debug, Clone, Copy)]
pub struct Skin {
    pub opacity: f32,
    pub hovered: Option<usize>,
    pub edge_radius: f32,
    pub hover_t: f32,
}

impl Skin {
    pub fn glass_style(self, index: usize) -> iced_glass::Style {
        let t = if self.hovered == Some(index) {
            self.hover_t
        } else {
            0.0
        };
        iced_glass::Style {
            blur_radius: 0.0,
            saturation: 1.1,
            lightness: -0.25 + 0.25 * t,
            edge_radius: self.edge_radius,
            edge_height: 200.0 + 100.0 * t,
            rim_width: 2.0,
            rim_angle: 0.5,
            opacity: self.opacity,
            ..Default::default()
        }
    }

    pub fn glass_style_opacity(self, index: usize, opacity: f32) -> iced_glass::Style {
        let t = if self.hovered == Some(index) {
            self.hover_t
        } else {
            0.0
        };
        iced_glass::Style {
            blur_radius: 50.0 + 50.0 * t,
            saturation: 1.1,
            lightness: -0.25 + 0.25 * t,
            edge_radius: self.edge_radius,
            edge_height: 200.0 + 100.0 * t,
            rim_width: 2.0,
            rim_angle: 0.5,
            opacity,
            ..Default::default()
        }
    }

    pub fn svg_white(self) -> impl Fn(&Theme, svg::Status) -> svg::Style {
        let color = color_opacity(Color::WHITE, self.opacity);
        move |_, _| svg::Style { color: Some(color) }
    }

    pub fn svg_blue(self) -> impl Fn(&Theme, svg::Status) -> svg::Style {
        let color = color_opacity(Color::from_rgb(0.3, 0.3, 1.0), self.opacity);
        move |_, _| svg::Style { color: Some(color) }
    }

    pub fn text_white(self) -> impl Fn(&Theme) -> text::Style {
        let color = color_opacity(Color::WHITE, self.opacity);
        move |_| text::Style { color: Some(color) }
    }
}

fn color_opacity(base: Color, opacity: f32) -> Color {
    Color::from_rgba(base.r, base.g, base.b, opacity)
}
