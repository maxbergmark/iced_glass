use iced::{Alignment, Background, Gradient, Length, Radians, Task, gradient::Linear};

#[derive(Debug, Clone)]
pub struct Ui {
    blur_radius: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    BlurRadius(f32),
}

impl Default for Ui {
    fn default() -> Self {
        Self { blur_radius: 100.0 }
    }
}

impl Ui {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::stack![
            iced::widget::image("assets/waterfall.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            iced::widget::column![
                iced::widget::Row::from_iter((0..100).map(|_i| {
                    iced_glass::widget::container(iced::widget::space())
                        .center(100.0)
                        .blur_radius(self.blur_radius)
                        .edge_height(200.0)
                        .refractive_index(2.5)
                        .edge_radius(30.0)
                        .lightness(-1.0)
                        .style(border_radius(30.0))
                        .into()
                }))
                .wrap(),
                iced::widget::slider(0.0..=1000.0, self.blur_radius, Message::BlurRadius)
                    .width(200.0)
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
