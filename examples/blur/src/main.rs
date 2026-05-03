use std::ops::RangeInclusive;

use iced::{Alignment, Color, Length, Size, Task, color};

#[derive(Debug, Clone)]
pub struct Ui {
    blur_radius: f32,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Message {
    SetBlurRadius(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

fn main() -> iced::Result {
    iced::application(Ui::boot, Ui::update, Ui::view)
        .subscription(Ui::subscription)
        .antialiasing(true)
        .window_size(Size::new(2560.0, 1440.0))
        .title("Liquid Glass Demo")
        .run()
}

impl Default for Ui {
    fn default() -> Self {
        Self { blur_radius: 500.0 }
    }
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetBlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::stack![self.background(), self.glass(),]).into()
    }

    fn background(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![
            iced::widget::row![
                iced::widget::container(iced::widget::space())
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .style(bg_color(color!(0xff0000))),
                iced::widget::container(iced::widget::space())
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .style(bg_color(color!(0x00ff00))),
            ],
            iced::widget::row![
                iced::widget::container(iced::widget::space())
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .style(bg_color(color!(0x0000ff))),
                iced::widget::container(iced::widget::space())
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .style(bg_color(color!(0xff00ff))),
            ]
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn glass(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::widget::container(self.styled_text(
                "Blur Radius: ",
                self.blur_radius,
                0.0..=100.0,
                Message::SetBlurRadius,
            ))
            .center_x(1000.00)
            .height(1000.0)
            .padding(20.0)
            .glass_style(|_theme| iced_glass::Style {
                blur_radius: self.blur_radius,
                ..Default::default()
            }),
        )
        .center(Length::Fill)
        .into()
    }

    fn styled_text(
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
                    iced::widget::text(format!("{value:.2}"))
                        .size(15.0)
                        .center(),
                ]
                .spacing(5.0),
                iced_glass::widget::slider(range, value.powf(1.0 / 3.0), move |v| message(
                    v.powf(3.0)
                ))
                .step(0.01_f32)
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
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 50.0,
            lightness: -2.0,
            rim_width: 1.0,
            ..Default::default()
        })
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

fn bg_color(color: Color) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(color)),
        ..Default::default()
    }
}
