use std::time::{Duration, Instant};

use iced::{Alignment, Background, Gradient, Length, Radians, Task, gradient::Linear};

#[derive(Debug, Clone)]
pub struct Ui {
    blur_radius: f32,
    num_containers: usize,
    last_update: Instant,
    elapsed: Duration,
}

#[derive(Debug, Clone)]
pub enum Message {
    BlurRadius(f32),
    NumContainers(usize),
    Noop,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            blur_radius: 100.0,
            num_containers: 100,
            last_update: Instant::now(),
            elapsed: Instant::now().elapsed(),
        }
    }
}

impl Ui {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.elapsed = self.last_update.elapsed();
        self.last_update = Instant::now();

        match message {
            Message::BlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
                Task::none()
            }
            Message::NumContainers(num_containers) => {
                self.num_containers = num_containers;
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Only request frames while the animation is running
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::stack![
            iced::widget::image("assets/waterfall.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            iced::widget::column![
                iced::widget::row![
                    iced::widget::slider(0.0..=1000.0, self.blur_radius, Message::BlurRadius)
                        .width(200.0),
                    iced::widget::text(format!("{:.0}", self.blur_radius)),
                ],
                iced::widget::row![
                    iced::widget::slider(0.0..=1000.0, self.num_containers as f32, |v| {
                        Message::NumContainers(v as usize)
                    })
                    .width(200.0),
                    iced::widget::text(format!("{}", self.num_containers)),
                ],
                iced::widget::text(format!(
                    "Frame time: {:.1}ms",
                    self.elapsed.as_secs_f32() * 1e3
                )),
                iced::widget::Row::from_iter((0..self.num_containers).map(|_i| {
                    iced_glass::widget::container(iced::widget::space())
                        .center(50.0)
                        .blur_radius(self.blur_radius)
                        .edge_height(200.0)
                        .refractive_index(2.5)
                        .edge_radius(10.0)
                        .lightness(-1.0)
                        .style(border_radius(10.0))
                        .into()
                }))
                .wrap(),
            ]
            .align_x(Alignment::Center)
        ])
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

fn border_radius(radius: f32) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        border: iced::Border {
            radius: radius.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
