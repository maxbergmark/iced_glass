use iced::{Length, Task, color};

#[derive(Debug, Clone)]
pub struct Ui {
    size: f32,
    edge_radius: f32,
    edge_height: f32,
    font_size: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Size(f32),
    EdgeRadius(f32),
    EdgeHeight(f32),
    FontSize(f32),
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            size: 700.0,
            edge_radius: 50.0,
            edge_height: 200.0,
            font_size: 200.0,
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
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::stack![
            iced::widget::image("assets/tulips.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            iced::widget::column![
                iced::widget::slider(0.0..=1000.0, self.size, Message::Size),
                iced::widget::slider(0.0..=100.0, self.edge_radius, Message::EdgeRadius),
                iced::widget::slider(0.0..=1000.0, self.edge_height, Message::EdgeHeight),
                iced::widget::slider(0.0..=400.0, self.font_size, Message::FontSize),
                iced::widget::space().height(200.0),
                iced::widget::container(iced::widget::row![
                    // self.styled_text("Hiello"),
                    // self.styled_text("你好齉蘭"),
                    self.styled_text("Hello"),
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
                .style(|_theme: &iced::Theme| iced::widget::container::Style {
                    border: iced::Border {
                        color: iced::Color::WHITE,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
            ],
        ]
        .into()
    }

    fn styled_text(&self, s: &str) -> iced::Element<'_, Message> {
        iced::widget::container(
            iced_glass::widget::text(s)
                .width(Length::Fill)
                .height(Length::Fill)
                .blur_radius(100.0)
                .edge_radius(self.edge_radius)
                .edge_height(self.edge_height)
                .refractive_index(1.5)
                .rim_width(3.0)
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
