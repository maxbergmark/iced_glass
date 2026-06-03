use iced::{
    Background, Border, Color, Element, Theme,
    widget::{
        container, mouse_area,
        slider::{self, Handle, Rail},
        stack, svg,
    },
};
use iced_glass::{Direction, SliderType, widget::slider as glass_slider};

use crate::{Message, Skin, icons, spacing};

pub struct SliderWithIcon {
    index: usize,
    icon: &'static str,
    size: iced::Size,
    value: f32,
    message: fn(f32) -> Message,
}

impl SliderWithIcon {
    pub fn new(
        index: usize,
        icon: &'static str,
        size: iced::Size,
        value: f32,
        message: fn(f32) -> Message,
    ) -> Self {
        Self {
            index,
            icon,
            size,
            value,
            message,
        }
    }

    pub fn view<'a>(
        self,
        // size: iced::Size,
        // message: impl Fn(f32) -> Message + 'static,
        skin: Skin,
    ) -> Element<'a, Message> {
        let (direction, height, width) = if self.size.width < self.size.height {
            (
                Direction::Vertical,
                spacing::n_rows(self.size, 2),
                spacing::n_cols(self.size, 1),
            )
        } else {
            (
                Direction::Horizontal,
                spacing::n_cols(self.size, 1),
                spacing::n_rows(self.size, 2),
            )
        };
        // let height = spacing::n_rows(self.size, 2);
        // let width = spacing::n_cols(self.size, 1);
        mouse_area(stack![
            glass_slider(0.0..=1.0, self.value, self.message)
                .slider_type(SliderType::Filled(direction))
                .step(0.001_f32)
                .width(width)
                .height(height)
                .style(Self::style(skin.opacity))
                .glass_style(move |_theme| skin.glass_style(self.index)),
            match direction {
                Direction::Vertical => {
                    container(Self::icon(self.icon, width, skin))
                        .align_bottom(height)
                        .center_x(width)
                        .padding(0.25 * width)
                }
                Direction::Horizontal => {
                    container(Self::icon(self.icon, height, skin))
                        .align_left(width)
                        .center_y(height)
                        .padding(0.25 * height)
                }
            }
        ])
        .on_enter(Message::Hovered(self.index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn icon(icon: &'static str, width: f32, skin: Skin) -> Element<'static, Message> {
        svg(icons::svg_handle(icon))
            .style(skin.svg_blue())
            .opacity(skin.opacity)
            .width(0.5 * width)
            .height(0.5 * width)
            .into()
    }

    fn style(opacity: f32) -> impl Fn(&Theme, slider::Status) -> slider::Style {
        let color = Color::from_rgba(1.0, 1.0, 1.0, opacity);
        let background_color = Color::TRANSPARENT;
        move |_, status| {
            let handle_color = match status {
                slider::Status::Active => Color::TRANSPARENT,
                _ => color,
            };
            slider::Style {
                rail: Rail {
                    backgrounds: (
                        Background::Color(color),
                        Background::Color(background_color),
                    ),
                    width: 10.0,
                    border: Border {
                        color,
                        width: 0.0,
                        radius: 150.0.into(),
                    },
                },
                handle: Handle {
                    shape: slider::HandleShape::Circle { radius: 15.0 },
                    background: Background::Color(handle_color),
                    border_width: 0.0,
                    border_color: Color::WHITE,
                },
            }
        }
    }
}
