use std::ops::RangeInclusive;

use iced::{Alignment, Length, Size, Task, color, widget::container};
use iced_glass::widget::EdgeType;

mod declaration;

#[derive(Debug, Clone)]
pub struct Ui {
    container_size: f32,
    blur_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
    chromatic_aberration: f32,
    rim_width: f32,
    opacity: f32,
    font_size: f32,
    line_height: f32,
    font_selection: Option<FontSelection>,
    style: iced::font::Style,
    weight: iced::font::Weight,
    stretch: iced::font::Stretch,
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
    ChromaticAberration(f32),
    RimWidth(f32),
    Opacity(f32),
    FontSelection(FontSelection),
    Style(iced::font::Style),
    Weight(iced::font::Weight),
    Stretch(iced::font::Stretch),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSelection {
    NotoSans,
    ArialUnicodeMS,
    SongtiSC,
    System,
}

impl FontSelection {
    fn name(&self) -> &'static str {
        match self {
            FontSelection::NotoSans => "Noto Sans",
            FontSelection::ArialUnicodeMS => "Arial Unicode MS",
            FontSelection::SongtiSC => "Songti SC",
            FontSelection::System => "System",
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
            chromatic_aberration: 0.0,
            rim_width: 0.5,
            opacity: 1.0,
            font_selection: None,
            style: iced::font::Style::Normal,
            weight: iced::font::Weight::Normal,
            stretch: iced::font::Stretch::Normal,
            text: String::new(),
        }
    }
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
            Message::ChromaticAberration(chromatic_aberration) => {
                self.chromatic_aberration = chromatic_aberration;
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
                if font_selection == FontSelection::System {
                    self.font_selection = None;
                } else {
                    self.font_selection = Some(font_selection);
                }
                Task::none()
            }
            Message::Style(style) => {
                self.style = style;
                Task::none()
            }
            Message::Weight(weight) => {
                self.weight = weight;
                Task::none()
            }
            Message::Stretch(stretch) => {
                self.stretch = stretch;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::stack![
            iced::widget::image("examples/text/assets/flowers.jpg")
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
                    self.styled_slider(
                        "Chromatic Aberration: ",
                        self.chromatic_aberration,
                        0.0..=1.0,
                        Message::ChromaticAberration
                    ),
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
                iced::widget::row![
                    self.text_input(),
                    self.font_selector(),
                    self.style_selector(),
                    self.weight_selector(),
                    self.stretch_selector()
                ]
                .spacing(20.0),
                iced::widget::space().height(100.0),
                iced::widget::container(
                    iced::widget::row![
                        self.styled_text(declaration::DECLARATION),
                        self.styled_text(&self.text),
                        self.styled_text("Hello\nHallå\n你好\nสวัสดี"),
                        // self.normal_text("Hello\nHallå\n你好\nสวัสดี"),
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
                    iced::widget::column![
                        iced::widget::radio(
                            "Songti",
                            FontSelection::SongtiSC,
                            self.font_selection,
                            Message::FontSelection
                        ),
                        iced::widget::radio(
                            "System",
                            FontSelection::System,
                            self.font_selection,
                            Message::FontSelection
                        ),
                    ],
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
        .glass_style(container_glass_style)
        .style(container_style)
        .into()
    }

    fn style_selector(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Style: "),
                iced::widget::row![
                    iced::widget::column![
                        iced::widget::radio(
                            "Normal",
                            iced::font::Style::Normal,
                            Some(self.style),
                            Message::Style
                        ),
                        iced::widget::radio(
                            "Italic",
                            iced::font::Style::Italic,
                            Some(self.style),
                            Message::Style
                        ),
                    ],
                    iced::widget::column![iced::widget::radio(
                        "Oblique",
                        iced::font::Style::Oblique,
                        Some(self.style),
                        Message::Style
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
        .glass_style(container_glass_style)
        .style(container_style)
        .into()
    }

    fn weight_selector(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Weight: "),
                iced::widget::row![
                    iced::widget::column![
                        iced::widget::radio(
                            "ExtraLight",
                            iced::font::Weight::ExtraLight,
                            Some(self.weight),
                            Message::Weight
                        ),
                        iced::widget::radio(
                            "Normal",
                            iced::font::Weight::Normal,
                            Some(self.weight),
                            Message::Weight
                        ),
                    ],
                    iced::widget::column![iced::widget::radio(
                        "ExtraBold",
                        iced::font::Weight::ExtraBold,
                        Some(self.weight),
                        Message::Weight
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
        .glass_style(container_glass_style)
        .style(container_style)
        .into()
    }

    fn stretch_selector(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::column![
                iced::widget::text("Stretch: "),
                iced::widget::row![
                    iced::widget::column![
                        iced::widget::radio(
                            "Condensed",
                            iced::font::Stretch::Condensed,
                            Some(self.stretch),
                            Message::Stretch
                        ),
                        iced::widget::radio(
                            "Normal",
                            iced::font::Stretch::Normal,
                            Some(self.stretch),
                            Message::Stretch
                        ),
                    ],
                    iced::widget::column![iced::widget::radio(
                        "Expanded",
                        iced::font::Stretch::Expanded,
                        Some(self.stretch),
                        Message::Stretch
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
        .glass_style(container_glass_style)
        .style(container_style)
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
        .center_x(2.0 * 200.0 + 1.0 * 20.0)
        .center_y(100.0)
        .padding(10.0)
        .glass_style(container_glass_style)
        .style(container_style)
        .into()
    }

    #[allow(dead_code)]
    fn styled_text<'a>(&'a self, s: &'a str) -> iced::Element<'a, Message> {
        let font = self.font_selection.map(|f| iced::Font {
            family: iced::font::Family::Name(f.name()),
            weight: self.weight,
            stretch: self.stretch,
            style: self.style,
        });
        iced::widget::container(
            iced_glass::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .glass_style(|theme| self.glass_style(theme))
                .size(self.font_size)
                .font_maybe(font)
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
        let font = self.font_selection.map(|f| iced::Font {
            family: iced::font::Family::Name(f.name()),
            weight: self.weight,
            stretch: self.stretch,
            style: self.style,
        });
        iced::widget::container(
            iced::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .size(self.font_size)
                .shaping(iced::widget::text::Shaping::Advanced)
                .font_maybe(font)
                .line_height(self.line_height),
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
                    iced::widget::text(format!("{value:.2}"))
                        .size(15.0)
                        .center(),
                ],
                iced_glass::widget::slider(range, value, message)
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
        .glass_style(container_glass_style)
        .style(container_style)
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

    fn glass_style(&self, _theme: &iced::Theme) -> iced_glass::Style {
        iced_glass::Style {
            blur_radius: self.blur_radius,
            saturation: self.saturation,
            lightness: self.lightness,
            edge_radius: self.edge_radius,
            edge_height: self.edge_height,
            refractive_index: self.refractive_index,
            chromatic_aberration: self.chromatic_aberration,
            rim_width: self.rim_width,
            opacity: self.opacity,
            edge_type: EdgeType::GlassEdge,
        }
    }
}

fn container_glass_style(_theme: &iced::Theme) -> iced_glass::Style {
    iced_glass::Style {
        blur_radius: 50.0,
        lightness: -2.0,
        edge_radius: 10.0,
        edge_height: 100.0,
        refractive_index: 1.5,
        rim_width: 1.0,
        ..Default::default()
    }
}

fn container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
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
    }
}
