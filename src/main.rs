use iced::{Alignment, Length, Size, Task, application};
use iced_glass::Message;

#[tokio::main]
async fn main() -> iced::Result {
    application(Ui::boot, Ui::update, Ui::view)
        .antialiasing(true)
        .window_size(Size::new(2560.0, 1440.0))
        .run()
}

#[derive(Default)]
struct Ui {
    width: f32,
    height: f32,
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
    mouse_position: Option<iced::Point>,
    moving: bool,
    sub_blurs: [f32; 4],
    sub_lightnesses: [f32; 4],
}

impl Ui {
    fn boot() -> (Ui, Task<Message>) {
        let ui = Ui {
            width: 800.0,
            height: 400.0,
            blur_radius: 10.0,
            corner_radius: 10.0,
            saturation: 1.0,
            lightness: 0.0,
            ..Default::default()
        };
        (ui, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetScale(scale) => {
                self.width = 2.0 * scale;
                self.height = scale;
                Task::none()
            }
            Message::SetBlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
                Task::none()
            }
            Message::SetCornerRadius(corner_radius) => {
                self.corner_radius = corner_radius;
                Task::none()
            }
            Message::SetSaturation(saturation) => {
                self.saturation = saturation;
                Task::none()
            }
            Message::SetLightness(lightness) => {
                self.lightness = lightness;
                Task::none()
            }
            Message::MouseMove(point) => {
                if self.moving {
                    self.mouse_position = Some(point);
                }
                Task::none()
            }
            Message::MouseState(moving) => {
                self.moving = moving;
                Task::none()
            }
            Message::SetSubBlurRadius(radius, id) => {
                self.sub_blurs[id] = radius;
                Task::none()
            }
            Message::SetSubLightness(lightness, id) => {
                self.sub_lightnesses[id] = lightness;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::column![
            iced::widget::stack![self.image(), self.mouse_area(), self.glass(),],
            iced::widget::container(iced::widget::slider(
                0.0..=1000.0,
                self.height,
                Message::SetScale
            ))
            .padding(10.0),
            iced::widget::container(iced::widget::slider(
                0.0..=1000.0,
                self.blur_radius,
                Message::SetBlurRadius
            ))
            .padding(10.0),
            iced::widget::container(iced::widget::slider(
                0.0..=200.0,
                self.corner_radius,
                Message::SetCornerRadius
            ))
            .padding(10.0),
            iced::widget::container(
                iced::widget::slider(0.0..=2.0, self.saturation, Message::SetSaturation).step(0.01)
            )
            .padding(10.0),
            iced::widget::container(
                iced::widget::slider(-4.0..=2.0, self.lightness, Message::SetLightness).step(0.01)
            )
            .padding(10.0)
        ]
        .into()
    }

    fn image(&self) -> iced::Element<'_, Message> {
        iced::widget::image("assets/lilly_small.jpg")
            // iced::widget::image("assets/ferris.png")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn mouse_area(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced::widget::mouse_area(
                iced::widget::container(iced::widget::text("Liquid Glass").size(200.0))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_move(Message::MouseMove)
            .on_press(Message::MouseState(true))
            .on_release(Message::MouseState(false)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            border: iced::Border {
                color: iced::Color::from_rgb(1.0, 1.0, 1.0),
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn glass(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::container::glass_container(self.inner_content())
                .width(Length::from(self.width))
                .height(Length::from(self.height))
                .center_y(Length::from(self.height))
                .blur_radius(self.blur_radius)
                .saturation(self.saturation)
                .lightness(self.lightness)
                .style(|theme| self.style(theme)),
        )
        .align_left(Length::Fill)
        .align_top(Length::Fill)
        .padding(iced::Padding {
            top: self
                .mouse_position
                .map(|point| (point.y - self.height / 2.0).max(0.0))
                .unwrap_or(0.0),
            left: self
                .mouse_position
                .map(|point| (point.x - self.width / 2.0).max(0.0))
                .unwrap_or(0.0),
            bottom: 0.0,
            right: 0.0,
        })
        .into()
    }

    fn inner_content(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![
            iced::widget::text("Liquid Glass").size(30.0),
            iced::widget::container(
                iced::widget::row![
                    self.styled_text("Blur Radius: ", format!("{:.0}", self.blur_radius), 0),
                    self.styled_text("Corner Radius: ", format!("{:.0}", self.corner_radius), 1),
                    self.styled_text("Saturation: ", format!("{:.2}", self.saturation), 2),
                    self.styled_text("Lightness: ", format!("{:.2}", self.lightness), 3),
                ]
                .spacing(20.0)
                .padding(20.0)
            )
            .center_y(Length::Fill)
            .center_x(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(self.corner_radius / 2.0)
        .into()
    }

    fn styled_text(
        &self,
        text: &'static str,
        value: String,
        id: usize,
    ) -> iced::Element<'_, Message> {
        // iced::widget::container(
        iced_glass::container::glass_container(
            iced::widget::column![
                iced::widget::text(text).size(20.0).center(),
                iced::widget::text(value).size(30.0).center(),
                iced::widget::slider(0.0..=1000.0, self.sub_blurs[id], move |v| {
                    Message::SetSubBlurRadius(v, id)
                }),
                iced::widget::slider(-4.0..=2.0, self.sub_lightnesses[id], move |v| {
                    Message::SetSubLightness(v, id)
                })
                .step(0.01)
            ]
            .align_x(Alignment::Center),
        )
        .style(|theme| self.style_outline(theme))
        .center_x(Length::from(150.0))
        .center_y(Length::from(150.0))
        .padding(10.0)
        .blur_radius(self.sub_blurs[id])
        .saturation(self.saturation)
        .lightness(self.sub_lightnesses[id])
        .into()
    }

    fn style(&self, _theme: &iced::Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: iced::Border {
                radius: self.corner_radius.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn style_outline(&self, _theme: &iced::Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: iced::Border {
                radius: self.corner_radius.into(),
                width: 1.0,
                color: iced::Color::from_rgb(1.0, 1.0, 1.0),
            },
            ..Default::default()
        }
    }
}
