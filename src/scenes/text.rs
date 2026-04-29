use std::ops::RangeInclusive;

use iced::{Alignment, Length, Task, color};

#[allow(unused_imports)]
use crate::scenes::declaration;

#[derive(Debug, Clone)]
pub struct Ui {
    container_size: f32,
    blur_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
    rim_width: f32,
    opacity: f32,
    font_size: f32,
    line_height: f32,
    font_selection: Option<FontSelection>,
    text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    ContainerSize(f32),
    BlurRadius(f32),
    Saturation(f32),
    Lightness(f32),
    EdgeRadius(f32),
    EdgeHeight(f32),
    FontSize(f32),
    LineHeight(f32),
    RefractiveIndex(f32),
    RimWidth(f32),
    Opacity(f32),
    FontSelection(FontSelection),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSelection {
    NotoSans,
    ArialUnicodeMS,
    SongtiSC,
}

impl FontSelection {
    fn name(&self) -> &'static str {
        match self {
            FontSelection::NotoSans => "Noto Sans",
            FontSelection::ArialUnicodeMS => "Arial Unicode MS",
            FontSelection::SongtiSC => "Songti SC",
        }
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            container_size: 500.0,
            edge_radius: 1.5,
            edge_height: 100.0,
            font_size: 200.0,
            line_height: 1.2,
            blur_radius: 100.0,
            saturation: 1.0,
            lightness: 2.0,
            refractive_index: 1.5,
            rim_width: 0.5,
            opacity: 1.0,
            font_selection: Some(FontSelection::ArialUnicodeMS),
            text: String::new(),
        }
    }
}

impl Ui {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContainerSize(size) => {
                self.container_size = size;
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
            Message::LineHeight(line_height) => {
                self.line_height = line_height;
                Task::none()
            }
            Message::Saturation(saturation) => {
                self.saturation = saturation;
                Task::none()
            }
            Message::Lightness(lightness) => {
                self.lightness = lightness;
                Task::none()
            }
            Message::RefractiveIndex(refractive_index) => {
                self.refractive_index = refractive_index;
                Task::none()
            }
            Message::RimWidth(rim_width) => {
                self.rim_width = rim_width;
                Task::none()
            }
            Message::Opacity(opacity) => {
                self.opacity = opacity;
                Task::none()
            }
            Message::FontSelection(font_selection) => {
                self.font_selection = Some(font_selection);
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
                    self.styled_slider(
                        "Container Size: ",
                        self.container_size,
                        100.0..=1000.0,
                        Message::ContainerSize
                    ),
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
                    self.font_selector(),
                ]
                .align_y(Alignment::Center)
                .padding(20.0)
                .spacing(20.0),
                iced::widget::row![
                    self.styled_slider(
                        "Line Height: ",
                        self.line_height,
                        1.0..=4.0,
                        Message::LineHeight
                    ),
                    self.styled_slider(
                        "Refractive Index: ",
                        self.refractive_index,
                        1.0..=10.0,
                        Message::RefractiveIndex
                    ),
                    self.styled_slider("Rim Width: ", self.rim_width, 0.0..=1.0, Message::RimWidth),
                    self.styled_slider("Opacity: ", self.opacity, 0.0..=1.0, Message::Opacity),
                    self.styled_slider(
                        "Saturation: ",
                        self.saturation,
                        0.0..=1.5,
                        Message::Saturation
                    ),
                    self.styled_slider(
                        "Lightness: ",
                        self.lightness,
                        -3.0..=3.0,
                        Message::Lightness
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

    fn font_selector(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Font: "),
                iced::widget::row![
                    iced::widget::column![
                        iced::widget::radio(
                            "Noto",
                            FontSelection::NotoSans,
                            self.font_selection,
                            Message::FontSelection
                        ),
                        iced::widget::radio(
                            "Arial",
                            FontSelection::ArialUnicodeMS,
                            self.font_selection,
                            Message::FontSelection
                        ),
                    ],
                    iced::widget::column![iced::widget::radio(
                        "Songti",
                        FontSelection::SongtiSC,
                        self.font_selection,
                        Message::FontSelection
                    ),],
                ]
                .spacing(10.0),
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
        .center_x(200.0)
        .center_y(100.0)
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(1.0)
        .lightness(-2.0)
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

    fn text_input(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Text input: "),
                iced::widget::text_input("Text...", &self.text).on_input(Message::Text)
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
        .lightness(-2.0)
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
    fn styled_text<'a>(&self, s: &'a str) -> iced::Element<'a, Message> {
        let family = self.font_selection.map(|f| iced::Font::with_name(f.name()));
        iced::widget::container(
            iced_glass::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .blur_radius(self.blur_radius)
                .edge_radius(self.edge_radius)
                .edge_height(self.edge_height)
                .refractive_index(self.refractive_index)
                .rim_width(self.rim_width)
                .opacity(self.opacity)
                .saturation(self.saturation)
                .lightness(self.lightness)
                .size(self.font_size)
                .font_maybe(family)
                .line_height(self.line_height),
        )
        .width(self.container_size)
        .height(self.container_size)
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
        .width(self.container_size)
        .height(self.container_size)
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
        .lightness(-2.0)
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
