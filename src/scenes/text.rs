use std::ops::RangeInclusive;

use iced::{Alignment, Length, Task, color};

#[allow(unused_imports)]
use crate::scenes::declaration;

#[derive(Debug, Clone)]
pub struct Ui {
    size: f32,
    edge_radius: f32,
    edge_height: f32,
    font_size: f32,
    blur_radius: f32,
    text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Size(f32),
    EdgeRadius(f32),
    EdgeHeight(f32),
    FontSize(f32),
    BlurRadius(f32),
    Text(String),
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            size: 500.0,
            edge_radius: 1.5,
            edge_height: 100.0,
            font_size: 200.0,
            blur_radius: 100.0,
            text: String::new(),
        }
    }
}

impl Ui {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Size(size) => {
                self.size = size;
                Task::none()
            }
            Message::EdgeRadius(edge_radius) => {
                self.edge_radius = edge_radius;
                Task::none()
            }
            Message::EdgeHeight(edge_height) => {
                self.edge_height = edge_height;
                Task::none()
            }
            Message::FontSize(font_size) => {
                self.font_size = font_size;
                Task::none()
            }
            Message::BlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
                Task::none()
            }
            Message::Text(text) => {
                self.text = text;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::stack![
            iced::widget::image("assets/waterfall.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            iced::widget::column![
                iced::widget::row![
                    self.styled_slider("Size: ", self.size, 100.0..=1000.0, Message::Size),
                    self.styled_slider(
                        "Blur Radius: ",
                        self.blur_radius,
                        0.0..=200.0,
                        Message::BlurRadius
                    ),
                    self.styled_slider(
                        "Edge Radius: ",
                        self.edge_radius,
                        0.0..=20.0,
                        Message::EdgeRadius
                    ),
                    self.styled_slider(
                        "Edge Height: ",
                        self.edge_height,
                        0.0..=400.0,
                        Message::EdgeHeight
                    ),
                    self.styled_slider(
                        "Font Size: ",
                        self.font_size,
                        1.0..=400.0,
                        Message::FontSize
                    ),
                ]
                .align_y(Alignment::Center)
                .padding(20.0)
                .spacing(20.0),
                self.text_input(),
                iced::widget::space().height(100.0),
                iced::widget::container(
                    iced::widget::row![
                        self.styled_text(declaration::DECLARATION),
                        self.styled_text(&self.text),
                        self.styled_text("Hello\nHallå\n你好\nสวัสดี"),
                    ]
                    .spacing(20.0)
                )
                .center_x(Length::Fill)
                .style(|_theme: &iced::Theme| {
                    iced::widget::container::Style {
                        border: iced::Border {
                            color: iced::Color::WHITE,
                            width: 0.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    }
                })
            ]
            .align_x(Alignment::Center),
        ]
        .into()
    }

    fn text_input(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Text: "),
                iced::widget::text_input("Text: ", &self.text).on_input(Message::Text)
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
        .center_x(5.0 * 200.0 + 4.0 * 20.0)
        .center_y(100.0)
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(1.0)
        .lightness(0.0)
        .rim_width(1.0)
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

    #[allow(dead_code)]
    fn styled_text(&self, s: &str) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .blur_radius(self.blur_radius)
                .edge_radius(self.edge_radius)
                .edge_height(self.edge_height)
                .refractive_index(1.5)
                .rim_width(0.5)
                .opacity(1.0)
                .lightness(1.0)
                .font_size(self.font_size)
                .line_height(self.font_size * 1.2),
        )
        .width(self.size)
        .height(self.size)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            border: iced::Border {
                color: color!(0xFFFFFF),
                width: 0.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    #[allow(dead_code)]
    fn normal_text(&self, s: &'static str) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .size(self.font_size),
        )
        .width(self.size)
        .height(self.size)
        .into()
    }

    fn styled_slider(
        &self,
        text: &'static str,
        value: f32,
        range: RangeInclusive<f32>,
        message: impl Fn(f32) -> Message + 'static,
    ) -> iced::Element<'_, Message> {
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
        .center_x(Length::from(200.0))
        .center_y(Length::from(100.0))
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(1.0)
        .lightness(0.0)
        .rim_width(1.0)
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
                    radius: 20.0.into(),
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
