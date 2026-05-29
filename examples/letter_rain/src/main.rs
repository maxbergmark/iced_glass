use std::{ops::RangeInclusive, time::Instant};

use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Padding, Point, Shadow, Size,
    Subscription, Task, Theme, Vector,
    font::{self, Family, Stretch, Weight},
    widget::{
        Stack, column, container, image, mouse_area, responsive, row, slider, space, stack, text,
    },
};
use iced_glass::widget::{EdgeType, container as glass_container, text as glass_text};

#[derive(Debug, Default, Clone)]
pub struct Ui {
    letters: Vec<AnimatedLetter>,
    window_size: Size,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Message {
    KeyPress(iced::keyboard::Key),
    Noop,
}

#[derive(Debug, Clone)]
struct AnimatedLetter {
    letter: char,
    spawn_time: Instant,
    position: Point<f32>,
    velocity: Vector<f32>,
}

const FONT_BOLD: Font = Font {
    family: Family::Name("Noto Sans"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .pretty() // multi-line, color-coded output with file:line info
        .with_env_filter("info,iced=info")
        .init();

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

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPress(key) => {
                tracing::info!("Adding key: {key:?}");
                if let iced::keyboard::Key::Character(c) = key {
                    self.letters.push(AnimatedLetter {
                        letter: c
                            .to_ascii_uppercase()
                            .chars()
                            .next()
                            .map(|c| match c {
                                'å' => 'Å',
                                'ä' => 'Ä',
                                'ö' => 'Ö',
                                _ => c,
                            })
                            .unwrap_or(' '),
                        spawn_time: Instant::now(),
                        position: Point {
                            x: fastrand::f32(),
                            y: fastrand::f32(),
                        },
                        velocity: Vector {
                            x: fastrand::f32() - 0.5,
                            y: fastrand::f32() - 0.5,
                        },
                    })
                }
            }
            Message::Noop => {
                for l in &mut self.letters {
                    l.position.x += 0.01 * l.velocity.x;
                    l.position.y += 0.01 * l.velocity.y;
                }
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_events = iced::window::resize_events();
        let animation =
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop);
        let keyboard = iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyPressed { key, .. } => Some(Message::KeyPress(key)),
            _ => None,
        });
        Subscription::batch(vec![animation, keyboard])
    }

    pub fn view(&self) -> Element<'_, Message> {
        responsive(move |size| stack![self.image(), self.glass(size),].into())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn image(&self) -> Element<'_, Message> {
        image("examples/letter_rain/assets/flowers.jpg")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn glass(&self, window_size: Size) -> Element<'_, Message> {
        Stack::from_vec(
            self.letters
                .iter()
                .map(|l| {
                    container(
                        glass_text(l.letter)
                            .glass_style(|theme| self.glass_style(theme))
                            .font(FONT_BOLD)
                            .size(200.0)
                            .width(200.0)
                            .height(300.0),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(
                        Padding::default()
                            .left(l.position.x * (window_size.width - 500.0))
                            .top(l.position.y * (window_size.height - 500.0)),
                    )
                    .into()
                })
                .collect(),
        )
        .into()
    }

    fn glass_style(&self, _theme: &Theme) -> iced_glass::Style {
        iced_glass::Style {
            blur_radius: 50.0,
            saturation: 1.1,
            lightness: 2.0,
            edge_radius: 5.0,
            edge_height: 50.0,
            refractive_index: 2.0,
            chromatic_aberration: 0.2,
            rim_width: 1.0,
            rim_angle: 1.0,
            opacity: 1.0,
            edge_type: EdgeType::GlassEdge,
        }
    }
}
