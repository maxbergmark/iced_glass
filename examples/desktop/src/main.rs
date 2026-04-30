use std::time::Instant;

use iced::{
    Alignment, Animation, Background, Border, Color, Element, Font, Gradient, Length, Padding,
    Size, Task,
    font::{self, Family, Stretch},
    gradient::Linear,
    widget::{Row, column, container, image, mouse_area, row, svg, text},
};

use iced_glass::widget::{container as glass_container, slider};

const FONT_BOLD: Font = Font {
    family: Family::SansSerif,
    weight: iced::font::Weight::Bold,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

const FONT_NORMAL: Font = Font {
    family: Family::SansSerif,
    weight: iced::font::Weight::Normal,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

#[derive(Debug, Clone)]
pub struct Ui {
    hovered: Option<usize>,
    hover_info: HoverInfo,
    lightness: Animation<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct HoverInfo {
    index: usize,
    is_hovered: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Hovered(usize),
    ClearHover,
    Noop,
}
// TODO: fix mouse area bounds
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
            hovered: None,
            hover_info: HoverInfo {
                index: 0,
                is_hovered: false,
            },
            lightness: Animation::new(false).very_quick(),
        }
    }
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Hovered(index) => {
                let old_index = self.hover_info.index;
                self.hovered = Some(index);
                let now = Instant::now();
                if self.lightness.is_animating(now) && old_index != index {
                    self.lightness = Animation::new(false).very_quick();
                }
                self.lightness.go_mut(true, now);
                Task::none()
            }
            Message::ClearHover => {
                self.hover_info.is_hovered = false;
                self.lightness.go_mut(false, Instant::now());
                // self.hover_info.event_time = Instant::now();
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Only request frames while the animation is running
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
    }

    pub fn view(&self) -> Element<'_, Message> {
        iced::widget::container(iced::widget::stack![
            iced::widget::image("examples/desktop/assets/ship.jpg")
                .width(Length::Fill)
                .height(Length::Fill),
            iced::widget::column![
                self.top_bar(),
                self.settings(),
                iced::widget::space().height(Length::FillPortion(1)),
                self.dock(),
            ]
            .align_x(Alignment::Center)
            .padding(Padding {
                bottom: 10.0,
                ..Default::default()
            })
        ])
        .center(Length::Fill)
        .into()
    }

    fn top_bar(&self) -> Element<'_, Message> {
        iced::widget::container(row![svg("")])
            .width(Length::Fill)
            .height(50.0)
            .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        container(
            column![
                row![
                    column![
                        self.small_icon_with_two_lines(0, "wifi", "Wi-Fi", "Connected"),
                        self.small_icon_with_two_lines(1, "bluetooth", "Bluetooth", "On")
                    ]
                    .spacing(20.0),
                    self.large_box(2),
                ]
                .spacing(20.0),
                row![
                    self.small_icon_with_two_lines(3, "airplane", "Airplane", "Off"),
                    self.circular_icon(4, "camera"),
                    self.circular_icon(5, "desktop"),
                ]
                .spacing(20.0),
                row![
                    self.circular_icon(6, "finger-print"),
                    self.circular_icon(7, "at"),
                    self.small_icon_with_one_line(8, "moon", "Focus"),
                ]
                .spacing(20.0),
                self.slider(9, "Display", "moon", "sunny"),
                self.slider(10, "Sound", "volume-off", "volume-high"),
            ]
            .spacing(20.0),
        )
        .into()
    }

    fn small_icon_with_one_line(
        &self,
        index: usize,
        icon: &'static str,
        top_text: &'static str,
    ) -> Element<'_, Message> {
        glass_container(
            mouse_area(
                row![
                    container(
                        svg(icon_path_outline(icon))
                            .style(svg_blue)
                            .width(25.0)
                            .height(25.0)
                    )
                    .center(40.0)
                    .style(border_radius_white(20.0)),
                    text(top_text)
                        .size(12.0)
                        .wrapping(text::Wrapping::None)
                        .font(FONT_BOLD),
                ]
                .align_y(Alignment::Center)
                .spacing(10.0),
            )
            .on_enter(Message::Hovered(index))
            .on_exit(Message::ClearHover),
        )
        .padding(15.0)
        .blur_radius(50.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .lightness(self.get_lightness(index))
        .saturation(1.1)
        .center_y(70.0)
        .width(160.0)
        .style(border_radius(50.0))
        .into()
    }

    fn small_icon_with_two_lines(
        &self,
        index: usize,
        icon: &'static str,
        top_text: &'static str,
        bottom_text: &'static str,
    ) -> Element<'_, Message> {
        glass_container(
            mouse_area(
                row![
                    container(
                        svg(icon_path_outline(icon))
                            .style(svg_blue)
                            .width(25.0)
                            .height(25.0)
                    )
                    .center(40.0)
                    .style(border_radius_white(20.0)),
                    column![
                        text(top_text)
                            .size(12.0)
                            .wrapping(text::Wrapping::None)
                            .font(FONT_BOLD),
                        text(bottom_text).size(10.0).font(FONT_NORMAL)
                    ]
                ]
                .align_y(Alignment::Center)
                .spacing(10.0),
            )
            .on_enter(Message::Hovered(index))
            .on_exit(Message::ClearHover),
        )
        .padding(15.0)
        .blur_radius(50.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .lightness(self.get_lightness(index))
        .saturation(1.1)
        .center_y(70.0)
        .width(160.0)
        .style(border_radius(50.0))
        .into()
    }

    fn get_lightness(&self, index: usize) -> f32 {
        if let Some(idx) = self.hovered
            && idx == index
        {
            self.lightness.interpolate(-0.5, 0.0, Instant::now())
        } else {
            -0.5
        }
    }

    fn large_box(&self, index: usize) -> Element<'_, Message> {
        glass_container(
            mouse_area(
                column![
                    image("examples/desktop/assets/album_cover.jpg")
                        .border_radius(15.0)
                        .width(60.0),
                    column![
                        text("Deep Meridian")
                            .width(Length::Fill)
                            .size(12.0)
                            .font(FONT_BOLD),
                        text("Terra Pulse - 2021")
                            .size(10.0)
                            .width(Length::Fill)
                            .font(FONT_NORMAL)
                    ],
                    row![
                        svg(icon_path_filled("play-back")).style(svg_white),
                        svg(icon_path_filled("play")).style(svg_white),
                        svg(icon_path_filled("play-forward")).style(svg_white)
                    ]
                ]
                .align_x(Alignment::Center)
                .spacing(10.0),
            )
            .on_enter(Message::Hovered(index))
            .on_exit(Message::ClearHover),
        )
        .padding(15.0)
        .blur_radius(50.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .lightness(self.get_lightness(index))
        .saturation(1.1)
        .center_y(160.0)
        .width(160.0)
        .style(border_radius(35.0))
        .into()
    }

    fn circular_icon(&self, index: usize, icon: &'static str) -> Element<'_, Message> {
        glass_container(
            mouse_area(svg(icon_path_outline(icon)).style(svg_white))
                .on_enter(Message::Hovered(index))
                .on_exit(Message::ClearHover),
        )
        .padding(20.0)
        .blur_radius(50.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .lightness(self.get_lightness(index))
        .saturation(1.1)
        .center(70.0)
        .style(border_radius(50.0))
        .into()
    }

    fn slider(
        &self,
        index: usize,
        label: &'static str,
        left_icon: &'static str,
        right_icon: &'static str,
    ) -> Element<'_, Message> {
        glass_container(
            mouse_area(
                column![
                    text(label).size(13.0).font(FONT_BOLD),
                    row![
                        svg(icon_path_outline(left_icon)).style(svg_white),
                        slider(0.0..=100.0, 30.0, |_| Message::Noop).width(250.0),
                        svg(icon_path_outline(right_icon)).style(svg_white)
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(20.0)
                ]
                .spacing(10.0),
            )
            .on_enter(Message::Hovered(index))
            .on_exit(Message::ClearHover),
        )
        .padding(12.5)
        .blur_radius(50.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .lightness(self.get_lightness(index))
        .saturation(1.1)
        .height(75.0)
        .width(340.0)
        .style(border_radius(25.0))
        .into()
    }

    fn dock(&self) -> Element<'_, Message> {
        let icons = [
            ("accessibility", BaseColor::Blue),
            ("alarm", BaseColor::Blue),
            ("calendar-number", BaseColor::Green),
            ("finger-print", BaseColor::Red),
            ("search", BaseColor::Blue),
            ("airplane", BaseColor::Cyan),
            ("at", BaseColor::Magenta),
            ("desktop", BaseColor::Blue),
            ("terminal", BaseColor::Blue),
        ];
        let icons = icons.into_iter().map(|(s, c)| self.icon(s, c)).collect();
        glass_container(Row::from_vec(icons).spacing(20.0))
            .blur_radius(50.0)
            .edge_radius(20.0)
            .edge_height(100.0)
            .rim_width(1.0)
            .padding(20.0)
            .style(border_radius(40.0))
            .into()
    }

    fn icon(&self, icon: &'static str, color: BaseColor) -> Element<'_, Message> {
        container(
            svg(icon_path_outline(icon))
                .width(50.0)
                .height(50.0)
                .style(svg_white),
        )
        .center(80.0)
        .style(move |_theme| container::Style {
            background: Some(Background::Gradient(Gradient::Linear(
                Linear::new(-0.0)
                    .add_stop(0.0, color.base())
                    .add_stop(1.0, color.highlight()),
            ))),
            border: Border {
                color: Color::WHITE,
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

enum BaseColor {
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
}

impl BaseColor {
    fn base(&self) -> Color {
        match self {
            BaseColor::Red => Color::from_rgb(1.0, 0.3, 0.3),
            BaseColor::Green => Color::from_rgb(0.3, 0.8, 0.3),
            BaseColor::Blue => Color::from_rgb(0.3, 0.3, 1.0),
            BaseColor::Cyan => Color::from_rgb(0.3, 0.8, 0.8),
            BaseColor::Magenta => Color::from_rgb(0.8, 0.3, 0.8),
        }
    }

    fn highlight(&self) -> Color {
        match self {
            BaseColor::Red => Color::from_rgb(1.0, 0.4, 0.4),
            BaseColor::Green => Color::from_rgb(0.4, 0.8, 0.4),
            BaseColor::Blue => Color::from_rgb(0.4, 0.4, 1.0),
            BaseColor::Cyan => Color::from_rgb(0.4, 0.8, 0.8),
            BaseColor::Magenta => Color::from_rgb(0.8, 0.4, 0.8),
        }
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

fn border_radius_white(radius: f32) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        border: iced::Border {
            radius: radius.into(),
            ..Default::default()
        },
        background: Some(Background::Color(Color::WHITE)),
        ..Default::default()
    }
}

fn icon_path_filled(name: &str) -> String {
    format!("examples/desktop/assets/{name}.svg")
}

fn icon_path_outline(name: &str) -> String {
    format!("examples/desktop/assets/{name}-outline.svg")
}

fn svg_white(_theme: &iced::Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(Color::WHITE),
    }
}

fn svg_blue(_theme: &iced::Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(BaseColor::Blue.base()),
    }
}
