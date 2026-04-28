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
            size: 1000.0,
            edge_radius: 5.0,
            edge_height: 400.0,
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
            iced::widget::image("assets/tulips.jpg")
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
                // iced::widget::slider(0.0..=1000.0, self.size, Message::Size),
                // iced::widget::slider(0.0..=20.0, self.edge_radius, Message::EdgeRadius).step(0.01),
                // iced::widget::slider(0.0..=1000.0, self.edge_height, Message::EdgeHeight),
                // iced::widget::slider(0.0..=400.0, self.font_size, Message::FontSize),
                // iced::widget::slider(0.0..=1000.0, self.blur_radius, Message::BlurRadius),
                self.text_input(),
                iced::widget::space().height(100.0),
                iced::widget::container(iced::widget::row![
                    // self.styled_text("Hiello"),
                    // self.styled_text("你好齉蘭"),
                    // self.styled_text(declaration::DECLARATION),
                    // self.styled_text("Hello\nHallå\n你好\nสวัสดี"),
                    self.styled_text(&self.text),
                    // self.normal_text("Hello\nHallå\n你好\nสวัสดี"),
                    // self.styled_text("สวัสดี_"),
                    // self.styled_text("Héllö"),
                    // biang
                    // self.styled_text("𰻞"),
                    // self.styled_text("abcdefghijklmnopqrstuvwxyzåäö"),
                    // self.styled_text("wxy"),
                    // self.styled_text("However, I have to use a custom fork of iced due to not being able to copy the underlying background texture into my own texture for blurring and sampling. The diff in iced, is literally one line of code (https://github.com/iced-rs/iced/compare/latest...maxbergmark:iced:latest), which enables support for these types of effects. Both the master branch and the latest branch could be updated in the same way.\n\nIs this the best way to enable these types of effects? Is there anything I'm not considering that makes this a bad approach? I'm definitely not an expert when it comes to iced or wgpu, so I'm interested in hearing your input."),
                    // self.styled_text("วั"),
                    // self.styled_text('好'),
                    // iced::widget::row![
                    //     iced::widget::row![self.styled_text("1"), self.styled_text("2"),]
                    //         .spacing(-self.size / 3.0),
                    //     self.styled_text(":"),
                    //     iced::widget::row![self.styled_text("3"), self.styled_text("4"),]
                    //         .spacing(-self.size / 3.0),
                    // ]
                    // .spacing(-self.size / 2.0)
                ])
                // .width(self.size)
                // .height(self.size)
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
        // iced::widget::container(
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
        // .style(|theme| self.style(theme))
        .center_x(Length::from(800.0))
        .center_y(Length::from(100.0))
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(1.0)
        .lightness(0.0)
        .rim_width(1.0)
        // .edge_radius(self.edge_radius)
        // .edge_height(self.edge_height)
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
                .lightness(2.0)
                .font_size(self.font_size)
                .line_height(self.font_size * 1.2),
        )
        .width(self.size)
        .height(self.size)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            border: iced::Border {
                color: color!(0xFF0000),
                width: 0.0,
                radius: 0.0.into(),
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
                // .blur_radius(self.blur_radius)
                // .edge_radius(self.edge_radius)
                // .edge_height(self.edge_height)
                // .refractive_index(1.5)
                // .rim_width(0.5)
                // .opacity(1.0)
                // .lightness(2.0)
                .size(self.font_size), // .line_height(self.font_size * 1.2),
        )
        .width(self.size)
        .height(self.size)
        // .style(|_theme: &iced::Theme| iced::widget::container::Style {
        //     border: iced::Border {
        //         color: color!(0xFF0000),
        //         width: 0.0,
        //         radius: 0.0.into(),
        //     },
        //     ..Default::default()
        // })
        .into()
    }

    fn styled_slider(
        &self,
        text: &'static str,
        value: f32,
        range: RangeInclusive<f32>,
        message: impl Fn(f32) -> Message + 'static,
    ) -> iced::Element<'_, Message> {
        // iced::widget::container(
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
        // .style(|theme| self.style(theme))
        .center_x(Length::from(200.0))
        .center_y(Length::from(100.0))
        .padding(10.0)
        .blur_radius(50.0)
        .saturation(1.0)
        .lightness(0.0)
        .rim_width(1.0)
        // .edge_radius(self.edge_radius)
        // .edge_height(self.edge_height)
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

#[cfg(test)]
mod tests {
    use iced_glass::font;

    #[test]
    fn test_text() {
        use msdfgen::{Bitmap, FillRule, FontExt, Gray, MID_VALUE, MsdfGeneratorConfig, Range};
        use std::fs::File;
        use ttf_parser::Face;

        let font = Face::parse(font::FONT, 0).unwrap();
        let glyph = font.glyph_index('D').unwrap();
        let mut shape = font.glyph_shape(glyph).unwrap();
        // let mut shape = font.(glyph).unwrap();

        let width = 32;
        let height = 32;

        let bound = shape.get_bound();
        let framing = bound
            .autoframe(width, height, Range::Px(4.0), None)
            .unwrap();
        let fill_rule = FillRule::default();

        let mut bitmap = Bitmap::new(width, height);

        shape.edge_coloring_simple(3.0, 0);

        let config = MsdfGeneratorConfig::default();

        shape.generate_msdf(&mut bitmap, framing, config);

        // optionally
        shape.correct_sign(&mut bitmap, framing, fill_rule);
        shape.correct_msdf_error(&mut bitmap, framing, config);

        let error = shape.estimate_error(&mut bitmap, framing, 5, Default::default());

        println!("Estimated error: {}", error);

        bitmap.flip_y();

        let mut output = File::create("A-letter-msdf.png").unwrap();
        bitmap.write_png(&mut output).unwrap();

        let mut preview = Bitmap::<Gray<f32>>::new(width * 20, height * 20);
        bitmap.render(&mut preview, Default::default(), MID_VALUE);

        let mut output = File::create("A-letter-preview.png").unwrap();
        preview.write_png(&mut output).unwrap();
    }
}
