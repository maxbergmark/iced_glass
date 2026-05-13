use iced::time::Instant;
use std::{ops::RangeInclusive, time::Duration};

use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Point, Shadow, Size,
    Subscription, Task, Theme, Vector,
    widget::{column, container, image, mouse_area, responsive, row, slider, space, stack, text},
};
use iced_glass::{
    glass_stack,
    widget::{
        EdgeType, InnerContent, StackOffset, container as glass_container, slider as glass_slider,
    },
};

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
    chromatic_aberration: f32,
    rim_width: f32,
    rim_angle: f32,
    opacity: f32,
    tint: Color,
    ship_handle: image::Handle,
    bw_handle: image::Handle,
    nat_handle: image::Handle,
    flower_handle: image::Handle,
    start_time: Instant,
    blending_factor: f32,
    number_of_boxes: usize,
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
    SetChromaticAberration(f32),
    SetRimWidth(f32),
    SetRimAngle(f32),
    MouseMove(iced::Point),
    MouseState(bool),
    SetTint(ColorChannel, f32),
    SetOpacity(f32),
    SetBlendingFactor(f32),
    SetNumberOfBoxes(usize),
    Noop,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Warn);
    }

    iced::application(Ui::boot, Ui::update, Ui::view)
        .subscription(Ui::subscription)
        .theme(theme)
        .antialiasing(true)
        .title("Liquid Glass In The Browser")
        .run()
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            blur_radius: 500.0,
            corner_radius: 100.0,
            saturation: 1.1,
            lightness: 0.0,
            mouse_position: Some(Point::new(400.0, 300.0)),
            moving: false,
            edge_radius: 30.0,
            edge_height: 300.0,
            refractive_index: 2.5,
            chromatic_aberration: 0.0,
            rim_width: 2.0,
            rim_angle: 0.0,
            opacity: 1.0,
            tint: Color::WHITE,
            ship_handle: image_handle(SHIP),
            bw_handle: image_handle(BW),
            nat_handle: image_handle(NAT),
            flower_handle: image_handle(FLW),
            start_time: Instant::now(),
            blending_factor: 100.0,
            number_of_boxes: 10,
        }
    }
}

fn theme(_ui: &Ui) -> Theme {
    Theme::Dark
}

fn image_handle(bytes: &'static [u8]) -> image::Handle {
    image::Handle::from_bytes(bytes)
}

const SHIP: &[u8] = include_bytes!("../assets/ship.jpg");
const BW: &[u8] = include_bytes!("../assets/black_white.jpg");
const NAT: &[u8] = include_bytes!("../assets/nature.jpg");
const FLW: &[u8] = include_bytes!("../assets/flowers.jpg");

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetScale(scale) => {
                self.width = 2.0 * scale;
                self.height = scale;
            }
            Message::SetBlurRadius(blur_radius) => {
                self.blur_radius = blur_radius;
            }
            Message::SetCornerRadius(corner_radius) => {
                self.corner_radius = corner_radius;
            }
            Message::SetSaturation(saturation) => {
                self.saturation = saturation;
            }
            Message::SetLightness(lightness) => {
                self.lightness = lightness;
            }
            Message::SetEdgeRadius(edge_radius) => {
                self.edge_radius = edge_radius;
            }
            Message::SetEdgeHeight(edge_height) => {
                self.edge_height = edge_height;
            }
            Message::SetRefractiveIndex(refractive_index) => {
                self.refractive_index = refractive_index;
            }
            Message::SetChromaticAberration(chromatic_aberration) => {
                self.chromatic_aberration = chromatic_aberration;
            }
            Message::SetRimWidth(rim_width) => {
                self.rim_width = rim_width;
            }
            Message::SetRimAngle(rim_angle) => {
                self.rim_angle = rim_angle;
            }
            Message::MouseMove(point) => {
                if self.moving {
                    self.mouse_position = Some(point);
                }
            }
            Message::MouseState(moving) => {
                self.moving = moving;
            }
            Message::SetOpacity(opacity) => {
                self.opacity = opacity;
            }
            Message::SetTint(channel, value) => match channel {
                ColorChannel::Red => self.tint.r = value,
                ColorChannel::Green => self.tint.g = value,
                ColorChannel::Blue => self.tint.b = value,
            },
            Message::SetBlendingFactor(blending_factor) => {
                self.blending_factor = blending_factor;
            }
            Message::SetNumberOfBoxes(number_of_boxes) => {
                self.number_of_boxes = number_of_boxes;
            }
            Message::Noop => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(10)).map(|_| Message::Noop)
    }

    pub fn view(&self) -> Element<'_, Message> {
        responsive(move |size| stack![self.image(), self.mouse_area(), self.glass(size),].into())
            .into()
    }

    fn image(&self) -> Element<'_, Message> {
        container(column![
            row![
                image(&self.ship_handle)
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1)),
                image(&self.bw_handle)
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
            ],
            row![
                image(&self.nat_handle)
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1)),
                image(&self.flower_handle)
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
            ]
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn mouse_area(&self) -> Element<'_, Message> {
        container(
            mouse_area(
                container(text("Liquid Glass Demo").size(300.0).color(Color::WHITE))
                    .width(Length::Fill)
                    .center_y(Length::Fill),
            )
            .on_move(Message::MouseMove)
            .on_press(Message::MouseState(true))
            .on_release(Message::MouseState(false)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn glass(&self, window_size: Size) -> Element<'_, Message> {
        let x = self
            .mouse_position
            .map(|point| {
                (point.x - self.width / 2.0)
                    .max(0.0)
                    .min(window_size.width - self.width)
            })
            .unwrap_or(0.0);

        let y = self
            .mouse_position
            .map(|point| {
                (point.y - self.height / 2.0)
                    .max(0.0)
                    .min(window_size.height - self.height)
            })
            .unwrap_or(0.0);

        fastrand::seed(123);

        container(
            glass_stack![
                container(self.inner_content())
                    .width(Length::from(self.width))
                    .height(Length::from(self.height))
                    .center_y(Length::from(self.height))
                    .style(|theme| self.style(theme))
                    .with_offset(x, y),
            ]
            .extend((0..self.number_of_boxes).map(|_| self.random_space(window_size)))
            .width(window_size.width)
            .height(window_size.height)
            .glass_style(|theme| self.glass_style(theme))
            .tint(self.tint)
            .corner_radius(self.corner_radius)
            .blending_factor(self.blending_factor),
        )
        // .center(Length::Fill)
        .align_left(Length::Fill)
        .align_top(Length::Fill)
        .into()
    }

    fn random_space(&self, window_size: Size) -> InnerContent<'static, Message> {
        let size = 100.0 * (1.0 + fastrand::f32());
        let t = self.start_time.elapsed().as_secs_f32();
        let f = 1.0 + fastrand::f32();
        let x = 50.0 + (window_size.width - size - 100.0) * fastrand::f32() + 50.0 * (f * t).sin();
        let y = 50.0 + (window_size.height - size - 100.0) * fastrand::f32() + 50.0 * (f * t).cos();

        space().width(size).height(size).with_offset(x, y)
    }

    fn inner_content(&self) -> Element<'_, Message> {
        container(column![
            // text("Liquid Glass").size(30.0),
            container(column![
                row![
                    self.styled_text("Blur Radius: ", self.blur_radius.sqrt(), 0.0..=100.0, |v| {
                        Message::SetBlurRadius(v * v)
                    }),
                    self.styled_text(
                        "Corner Radius: ",
                        self.corner_radius,
                        0.0..=150.0,
                        Message::SetCornerRadius
                    ),
                    self.styled_text(
                        "Edge Radius: ",
                        self.edge_radius,
                        0.0..=100.0,
                        Message::SetEdgeRadius
                    ),
                    self.color_picker("Tint: ", self.tint),
                ]
                .spacing(10.0)
                .padding(10.0),
                row![
                    self.styled_text(
                        "Saturation: ",
                        self.saturation,
                        0.0..=2.0,
                        Message::SetSaturation
                    ),
                    self.styled_text(
                        "Lightness: ",
                        self.lightness,
                        -4.0..=2.0,
                        Message::SetLightness
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
                    self.styled_text("Opacity: ", self.opacity, 0.0..=1.0, Message::SetOpacity),
                ]
                .spacing(10.0)
                .padding(10.0),
                row![
                    self.styled_text(
                        "Rim Width: ",
                        self.rim_width,
                        0.0..=5.0,
                        Message::SetRimWidth
                    ),
                    self.styled_text(
                        "Rim Angle: ",
                        self.rim_angle,
                        0.0..=std::f32::consts::PI,
                        Message::SetRimAngle
                    ),
                    self.styled_text(
                        "Aberration: ",
                        self.chromatic_aberration,
                        0.0..=1.0,
                        Message::SetChromaticAberration
                    ),
                    self.styled_text(
                        "Blending: ",
                        self.blending_factor,
                        0.0..=200.0,
                        Message::SetBlendingFactor
                    ),
                    self.styled_text("#Boxes: ", self.number_of_boxes as f32, 1.0..=20.0, |v| {
                        Message::SetNumberOfBoxes(v.round() as usize)
                    }),
                ]
                .spacing(10.0)
                .padding(10.0)
            ])
            .center_y(Length::Fill)
            .center_x(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(25.0)
        .into()
    }

    fn styled_text(
        &self,
        s: &'static str,
        value: f32,
        range: RangeInclusive<f32>,
        message: impl Fn(f32) -> Message + 'static,
    ) -> Element<'_, Message> {
        // container(
        glass_container(
            column![
                row![
                    text(s).size(8.0).center(),
                    text(format!("{value:.2}")).size(8.0).center(),
                ],
                glass_slider(range, value, message)
                    .step(0.01_f32)
                    .height(5.0)
                    .style(|theme, status| self.slider_style(theme, status)),
            ]
            .align_x(Alignment::Center)
            .spacing(5.0)
            .padding(Padding::default().horizontal(15.0)),
        )
        .center_x(Length::from(120.0))
        .center_y(Length::from(60.0))
        .padding(5.0)
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 50.0,
            saturation: self.saturation,
            lightness: -2.0,
            rim_width: 1.0,
            ..Default::default()
        })
        // .edge_radius(self.edge_radius)
        // .edge_height(self.edge_height)
        .style(|_theme| container::Style {
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn color_picker(&self, s: &'static str, value: Color) -> Element<'_, Message> {
        // container(
        glass_container(
            column![
                row![
                    text(s).size(8.0).center(),
                    container(space())
                        .center(Length::from(8.0))
                        .style(move |_theme| container::Style {
                            background: Some(Background::Color(value)),
                            ..Default::default()
                        }),
                ]
                .align_y(Alignment::Center),
                row![
                    glass_slider(0.0..=1.0, value.r, |value| Message::SetTint(
                        ColorChannel::Red,
                        value
                    ))
                    .step(0.01_f32)
                    .height(5.0)
                    .style(|theme, status| self.colored_slider_style(
                        theme,
                        status,
                        Color::from_rgb(1.0, 0.0, 0.0)
                    )),
                    glass_slider(0.0..=1.0, value.g, |value| Message::SetTint(
                        ColorChannel::Green,
                        value
                    ))
                    .step(0.01_f32)
                    .height(5.0)
                    .style(|theme, status| self.colored_slider_style(
                        theme,
                        status,
                        Color::from_rgb(0.0, 1.0, 0.0)
                    )),
                    glass_slider(0.0..=1.0, value.b, |value| Message::SetTint(
                        ColorChannel::Blue,
                        value
                    ))
                    .step(0.01_f32)
                    .height(5.0)
                    .style(|theme, status| self.colored_slider_style(
                        theme,
                        status,
                        Color::from_rgb(0.0, 0.0, 1.0)
                    )),
                ]
                .spacing(5.0),
            ]
            .align_x(Alignment::Center)
            .spacing(5.0)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 0.0,
                left: 10.0,
            }),
        )
        // .style(|theme| self.style(theme))
        .center_x(Length::from(250.0))
        .center_y(Length::from(60.0))
        .padding(5.0)
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 50.0,
            saturation: self.saturation,
            lightness: -2.0,
            ..Default::default()
        })
        .style(|_theme| container::Style {
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25 * 0.0),
                offset: Vector::new(0.0, 12.0),
                blur_radius: 40.0 * 0.0,
            },
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn style(&self, _theme: &Theme) -> container::Style {
        container::Style {
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25 * 0.0),
                offset: Vector::new(0.0, 12.0),
                blur_radius: 40.0 * 0.0,
            },
            border: Border {
                radius: self.corner_radius.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn glass_style(&self, _theme: &Theme) -> iced_glass::Style {
        iced_glass::Style {
            blur_radius: self.blur_radius,
            saturation: self.saturation,
            lightness: self.lightness,
            edge_radius: self.edge_radius,
            edge_height: self.edge_height,
            refractive_index: self.refractive_index,
            chromatic_aberration: self.chromatic_aberration,
            rim_width: self.rim_width,
            rim_angle: self.rim_angle,
            opacity: self.opacity,
            edge_type: EdgeType::GlassEdge,
        }
    }

    fn slider_style(&self, _theme: &Theme, _status: slider::Status) -> slider::Style {
        slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(Color::from_rgba(0.3, 0.3, 1.0, 1.0)),
                    Background::Color(Color::WHITE),
                ),
                width: 2.5,
                border: Border {
                    radius: self.corner_radius.into(),
                    ..Default::default()
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 15,
                    border_radius: 5.0.into(),
                },
                background: Background::Color(Color::from_rgba(0.3, 0.3, 1.0, 1.0)),
                border_width: 1.0,
                border_color: Color::from_rgba(0.3, 0.3, 1.0, 1.0),
            },
        }
    }

    fn colored_slider_style(
        &self,
        _theme: &Theme,
        _status: slider::Status,
        color: Color,
    ) -> slider::Style {
        slider::Style {
            rail: slider::Rail {
                backgrounds: (Background::Color(color), Background::Color(Color::WHITE)),
                width: 2.5,
                border: Border {
                    radius: self.corner_radius.into(),
                    ..Default::default()
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 15,
                    border_radius: 5.0.into(),
                },
                background: Background::Color(color),
                border_width: 1.0,
                border_color: Color::from_rgba(0.3, 0.3, 1.0, 1.0),
            },
        }
    }
}
