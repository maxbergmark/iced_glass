use iced::{Alignment, Background, Gradient, Length, Radians, Size, Task, gradient::Linear};

#[derive(Debug, Clone)]
pub struct Ui {
    value: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Value(f32),
    EdgeRadius(f32),
    EdgeHeight(f32),
    RefractiveIndex(f32),
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            value: 0.0,
            edge_radius: 10.0,
            edge_height: 60.0,
            refractive_index: 2.5,
        }
    }
}

fn main() -> iced::Result {
    iced::application(Ui::boot, Ui::update, Ui::view)
        .subscription(Ui::subscription)
        .antialiasing(true)
        .window_size(Size::new(2560.0, 1440.0))
        .title("Liquid Glass Demo")
        .run()
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Value(value) => {
                self.value = value;
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
            Message::RefractiveIndex(refractive_index) => {
                self.refractive_index = refractive_index;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::widget::container(
                iced::widget::column![
                    iced_glass::widget::slider(0.0..=1.0, self.value, Message::Value)
                        .step(0.01_f32)
                        .width(1000.0)
                        .height(100.0)
                        .style(slider_style)
                        .edge_radius(self.edge_radius)
                        .edge_height(self.edge_height)
                        .refractive_index(self.refractive_index),
                    iced::widget::row![
                        iced_glass::widget::container(iced::widget::column![
                            iced::widget::text("Edge Radius: "),
                            iced::widget::slider(
                                0.0..=100.0,
                                self.edge_radius,
                                Message::EdgeRadius
                            )
                            .step(1.0_f32)
                            .width(100.0)
                            .height(100.0)
                        ])
                        .padding(20.0)
                        .style(border_radius(20.0))
                        .lightness(-1.0)
                        .center_x(200.0)
                        .center_y(100.0),
                        iced_glass::widget::container(iced::widget::column![
                            iced::widget::text("Edge Height: "),
                            iced::widget::slider(
                                0.0..=100.0,
                                self.edge_height,
                                Message::EdgeHeight
                            )
                            .step(1.0_f32)
                            .width(100.0)
                            .height(100.0)
                        ])
                        .padding(20.0)
                        .style(border_radius(20.0))
                        .lightness(-1.0)
                        .center_x(200.0)
                        .center_y(100.0),
                        iced_glass::widget::container(iced::widget::column![
                            iced::widget::text("Refractive Index: "),
                            iced::widget::slider(
                                1.0..=2.0,
                                self.refractive_index,
                                Message::RefractiveIndex
                            )
                            .step(0.01_f32)
                            .width(100.0)
                            .height(100.0)
                        ])
                        .padding(20.0)
                        .style(border_radius(20.0))
                        .lightness(-1.0)
                        .center_x(200.0)
                        .center_y(100.0)
                    ]
                    .spacing(20.0)
                    .align_y(Alignment::Center)
                ]
                .spacing(50.0)
                .align_x(Alignment::Center),
            )
            .center_x(1500.0)
            .center_y(700.0)
            .blur_radius(50.0)
            .edge_height(200.0)
            .refractive_index(2.5)
            .edge_radius(30.0)
            .lightness(-1.0)
            .style(|_theme| iced::widget::container::Style {
                border: iced::Border {
                    radius: 50.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Gradient(Gradient::Linear(
                Linear::new(Radians::from(3.0))
                    .add_stop(0.0, iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0))
                    .add_stop(1.0, iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0)),
            ))),
            ..Default::default()
        })
        .center(Length::Fill)
        .into()
    }
}

fn slider_style(
    _theme: &iced::Theme,
    _status: iced::widget::slider::Status,
) -> iced::widget::slider::Style {
    let fill_color = iced::Color::from_rgba(0.3, 0.3, 1.0, 1.0);
    iced::widget::slider::Style {
        rail: iced::widget::slider::Rail {
            backgrounds: (
                iced::Background::Color(fill_color),
                iced::Background::Color(iced::Color::WHITE),
            ),
            width: 60.0,
            border: iced::Border {
                radius: 30.0.into(),
                ..Default::default()
            },
        },
        handle: iced::widget::slider::Handle {
            shape: iced::widget::slider::HandleShape::Rectangle {
                width: 200,
                border_radius: 50.0.into(),
            },
            background: iced::Background::Color(fill_color),
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
        },
    }
}

fn border_radius(radius: f32) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        border: iced::Border {
            radius: radius.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
