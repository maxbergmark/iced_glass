use std::ops::RangeInclusive;

use iced::{Alignment, Length, Task};

#[derive(Debug, Clone)]
pub struct Ui {
    width: f32,
    height: f32,
    mouse_position: Option<iced::Point>,
    moving: bool,
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
    rim_width: f32,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Message {
    SetScale(f32),
    SetBlurRadius(f32),
    SetCornerRadius(f32),
    SetSaturation(f32),
    SetLightness(f32),
    SetEdgeRadius(f32),
    SetEdgeHeight(f32),
    SetRefractiveIndex(f32),
    SetRimWidth(f32),
    MouseMove(iced::Point),
    MouseState(bool),
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 500.0,
            blur_radius: 100.0,
            corner_radius: 100.0,
            saturation: 1.1,
            lightness: -1.5,
            mouse_position: Some(iced::Point::new(1400.0, 800.0)),
            moving: false,
            edge_radius: 30.0,
            edge_height: 300.0,
            refractive_index: 2.5,
            rim_width: 2.0,
        }
    }
}

impl Ui {
    pub fn update(&mut self, message: Message) -> Task<Message> {
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
            Message::SetEdgeRadius(edge_radius) => {
                self.edge_radius = edge_radius;
                Task::none()
            }
            Message::SetEdgeHeight(edge_height) => {
                self.edge_height = edge_height;
                Task::none()
            }
            Message::SetRefractiveIndex(refractive_index) => {
                self.refractive_index = refractive_index;
                Task::none()
            }
            Message::SetRimWidth(rim_width) => {
                self.rim_width = rim_width;
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
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::responsive(move |size| {
            iced::widget::stack![self.image(), self.mouse_area(), self.glass(size),].into()
        })
        .into()
    }

    fn image(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![
            iced::widget::row![
                iced::widget::image("assets/waterfall.jpg")
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1)),
                iced::widget::image("assets/eclipse.jpg")
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
            ],
            iced::widget::row![
                iced::widget::image("assets/tree.jpg")
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1)),
                iced::widget::image("assets/tulips.jpg")
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
            ]
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn mouse_area(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced::widget::mouse_area(
                iced::widget::container(iced::widget::text("Liquid Glass Demo").size(300.0))
                    .width(Length::Fill)
                    .center_y(Length::Fill),
            )
            .on_move(Message::MouseMove)
            .on_press(Message::MouseState(true))
            .on_release(Message::MouseState(false)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        // .style(|_| iced::widget::container::Style {
        //     border: iced::Border {
        //         color: iced::Color::from_rgb(1.0, 1.0, 1.0),
        //         width: 1.0,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // })
        .into()
    }

    fn glass(&self, window_size: iced::Size) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::widget::container(self.inner_content())
                .width(Length::from(self.width))
                .height(Length::from(self.height))
                .center_y(Length::from(self.height))
                .blur_radius(self.blur_radius)
                .saturation(self.saturation)
                .lightness(self.lightness)
                .edge_radius(self.edge_radius)
                .edge_height(self.edge_height)
                .refractive_index(self.refractive_index)
                .rim_width(self.rim_width)
                .opacity(1.0)
                .style(|theme| self.style(theme)),
        )
        .align_left(Length::Fill)
        .align_top(Length::Fill)
        .padding(iced::Padding {
            top: self
                .mouse_position
                .map(|point| {
                    (point.y - self.height / 2.0)
                        .max(0.0)
                        .min(window_size.height - self.height)
                })
                .unwrap_or(0.0),
            left: self
                .mouse_position
                .map(|point| {
                    (point.x - self.width / 2.0)
                        .max(0.0)
                        .min(window_size.width - self.width)
                })
                .unwrap_or(0.0),
            bottom: 0.0,
            right: 0.0,
        })
        .into()
    }

    fn inner_content(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![
            iced::widget::text("Liquid Glass").size(30.0),
            iced::widget::container(iced::widget::column![
                iced::widget::row![
                    self.styled_text(
                        "Rim Width: ",
                        self.rim_width,
                        0.0..=5.0,
                        Message::SetRimWidth
                    ),
                    self.styled_text(
                        "Blur Radius: ",
                        self.blur_radius,
                        0.0..=1000.0,
                        Message::SetBlurRadius
                    ),
                    self.styled_text(
                        "Corner Radius: ",
                        self.corner_radius,
                        0.0..=150.0,
                        Message::SetCornerRadius
                    ),
                    self.styled_text(
                        "Saturation: ",
                        self.saturation,
                        0.0..=2.0,
                        Message::SetSaturation
                    ),
                ]
                .spacing(20.0)
                .padding(20.0),
                iced::widget::row![
                    self.styled_text(
                        "Lightness: ",
                        self.lightness,
                        -4.0..=2.0,
                        Message::SetLightness
                    ),
                    self.styled_text(
                        "Edge Radius: ",
                        self.edge_radius,
                        0.0..=100.0,
                        Message::SetEdgeRadius
                    ),
                    self.styled_text(
                        "Edge Height: ",
                        self.edge_height,
                        0.0..=1000.0,
                        Message::SetEdgeHeight
                    ),
                    self.styled_text(
                        "Refractive Index: ",
                        self.refractive_index,
                        1.0..=10.0,
                        Message::SetRefractiveIndex
                    ),
                ]
                .spacing(20.0)
                .padding(20.0)
            ])
            .center_y(Length::Fill)
            .center_x(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(50.0)
        .into()
    }

    fn styled_text(
        &self,
        text: &'static str,
        value: f32,
        range: RangeInclusive<f32>,
        message: impl Fn(f32) -> Message + 'static,
    ) -> iced::Element<'_, Message> {
        // iced::widget::container(
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::row![
                    iced::widget::text(text).size(15.0).center(),
                    iced::widget::text(format!("{:.2}", value))
                        .size(15.0)
                        .center(),
                ],
                iced_glass::widget::slider(range, value, message)
                    .step(0.01)
                    .style(|theme, status| self.slider_style(theme, status)),
            ]
            .align_x(Alignment::Center)
            .spacing(5.0)
            .padding(iced::Padding {
                top: 0.0,
                right: 15.0,
                bottom: 0.0,
                left: 15.0,
            }),
        )
        // .style(|theme| self.style(theme))
        .center_x(Length::from(200.0))
        .center_y(Length::from(100.0))
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(self.saturation)
        .lightness(0.0)
        // .edge_radius(self.edge_radius)
        // .edge_height(self.edge_height)
        .style(|_theme| iced::widget::container::Style {
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: iced::Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
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

    fn slider_style(
        &self,
        _theme: &iced::Theme,
        _status: iced::widget::slider::Status,
    ) -> iced::widget::slider::Style {
        iced::widget::slider::Style {
            rail: iced::widget::slider::Rail {
                backgrounds: (
                    iced::Background::Color(iced::Color::from_rgba(0.3, 0.3, 1.0, 1.0)),
                    iced::Background::Color(iced::Color::WHITE),
                ),
                width: 5.0,
                border: iced::Border {
                    radius: self.corner_radius.into(),
                    ..Default::default()
                },
            },
            handle: iced::widget::slider::Handle {
                shape: iced::widget::slider::HandleShape::Rectangle {
                    width: 30,
                    border_radius: 10.0.into(),
                },
                background: iced::Background::Color(iced::Color::from_rgba(0.3, 0.3, 1.0, 1.0)),
                border_width: 1.0,
                border_color: iced::Color::from_rgba(0.3, 0.3, 1.0, 1.0),
            },
        }
    }
}
