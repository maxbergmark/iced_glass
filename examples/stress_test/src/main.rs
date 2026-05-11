use std::time::{Duration, Instant};

use iced::{
    Alignment, Background, Border, Color, Element, Gradient, Length, Radians, Size, Subscription,
    Task, Theme,
    gradient::Linear,
    widget::{Row, column, container, image, row, slider, space, stack, text},
};
use iced_glass::widget::container as glass_container;

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
        Self {
            blur_radius: 100.0,
            num_containers: 100,
            last_update: Instant::now(),
            elapsed: Instant::now().elapsed(),
        }
    }
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

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

    pub fn subscription(&self) -> Subscription<Message> {
        // Only request frames while the animation is running
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(stack![
            image("examples/stress_test/assets/mountain.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            column![
                row![
                    slider(0.0..=1000.0, self.blur_radius, Message::BlurRadius).width(200.0),
                    text(format!("{:.0}", self.blur_radius)).width(50.0),
                ],
                row![
                    slider(0.0..=1000.0, self.num_containers as f32, |v| {
                        Message::NumContainers(v as usize)
                    })
                    .width(200.0),
                    text(format!("{}", self.num_containers)).width(50.0),
                ],
                text(format!(
                    "Frame time: {:.1}ms",
                    self.elapsed.as_secs_f32() * 1e3
                )),
                Row::from_iter((0..self.num_containers).map(|_i| {
                    glass_container(space())
                        .center(50.0)
                        .glass_style(|_theme| iced_glass::Style {
                            blur_radius: 50.0,
                            edge_radius: 5.0,
                            edge_height: 200.0,
                            refractive_index: 2.5,
                            ..Default::default()
                        })
                        .style(border_radius(10.0))
                        .into()
                }))
                .wrap(),
            ]
            .align_x(Alignment::Center)
        ])
        .style(|_theme| container::Style {
            background: Some(Background::Gradient(Gradient::Linear(
                Linear::new(Radians::from(3.0))
                    .add_stop(0.0, Color::from_rgba(0.6, 0.6, 0.6, 1.0))
                    .add_stop(1.0, Color::from_rgba(0.4, 0.4, 0.4, 1.0)),
            ))),
            ..Default::default()
        })
        .center(Length::Fill)
        .into()
    }
}

fn border_radius(radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        border: Border {
            radius: radius.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
