use std::time::{Duration, Instant};

use iced::{
    Alignment, Animation, Background, Border, Color, ContentFit, Element, Font, Gradient, Length,
    Padding, Size, Task,
    animation::Easing,
    font::{self, Family, Stretch},
    gradient::Linear,
    widget::{
        Row, column, container, image, mouse_area, row,
        slider::{self, Handle, Rail},
        space, stack, svg, text,
    },
};

use iced_glass::widget::{
    EdgeType, container as glass_container, slider as glass_slider, text as glass_text,
};

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
    lightness: HoverInfo,
    opacity: Animation<bool>,
    edge_radius: Animation<bool>,
    brightness: f32,
    volume: f32,
}

#[derive(Debug, Clone)]
pub struct HoverInfo {
    index: usize,
    is_hovered: bool,
    animation: Animation<bool>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Hovered(usize),
    ClearHover,
    ToggleMenu,
    Brightness(f32),
    Volume(f32),
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
            lightness: HoverInfo {
                index: 0,
                is_hovered: false,
                animation: Animation::new(false).duration(Duration::from_millis(150)),
            },
            opacity: Animation::new(false).slow(),
            edge_radius: Animation::new(false)
                .slow()
                .delay(Duration::from_millis(100))
                .easing(Easing::EaseIn),
            brightness: 0.6,
            volume: 0.3,
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
                let old_index = self.lightness.index;
                self.hovered = Some(index);
                let now = Instant::now();
                if self.lightness.animation.is_animating(now) && old_index != index {
                    self.lightness.animation =
                        Animation::new(false).duration(Duration::from_millis(150));
                }
                self.lightness.animation.go_mut(true, now);
                Task::none()
            }
            Message::ClearHover => {
                self.lightness.is_hovered = false;
                self.lightness.animation.go_mut(false, Instant::now());
                // self.hover_info.event_time = Instant::now();
                Task::none()
            }
            Message::ToggleMenu => {
                let new_state = !self.opacity.value();
                self.opacity.go_mut(new_state, Instant::now());
                self.edge_radius.go_mut(new_state, Instant::now());
                Task::none()
            }
            Message::Brightness(value) => {
                self.brightness = value;
                Task::none()
            }
            Message::Volume(value) => {
                self.volume = value;
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let now = Instant::now();
        if self.opacity.is_animating(now)
            || self.edge_radius.is_animating(now)
            || self.lightness.animation.is_animating(now)
        {
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
        } else {
            iced::Subscription::none()
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(stack![
            self.wallpaper(),
            self.desktop_elements(),
            self.clock()
        ])
        .center(Length::Fill)
        .into()
    }

    fn wallpaper(&self) -> Element<'_, Message> {
        image("examples/desktop/assets/ship.jpg")
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Cover)
            .into()
    }

    fn desktop_elements(&self) -> Element<'_, Message> {
        column![
            self.top_bar(),
            if self.get_opacity() > 0.0 {
                self.settings()
            } else {
                space().into()
            },
            space().height(Length::FillPortion(1)),
            self.dock(),
        ]
        .align_x(Alignment::Center)
        .padding(Padding {
            bottom: 10.0,
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
    }

    fn clock(&self) -> Element<'_, Message> {
        container(
            column![
                space().height(Length::FillPortion(1)),
                glass_text("12:45")
                    .size(200.0)
                    .font(FONT_BOLD)
                    .glass_style(|_theme| iced_glass::Style {
                        blur_radius: 500.0,
                        lightness: 1.5,
                        edge_radius: 8.0,
                        edge_height: 100.0,
                        rim_width: 1.0,
                        ..Default::default()
                    }),
                glass_text("Fri May 1")
                    .size(50.0)
                    .font(FONT_BOLD)
                    .glass_style(|_theme| iced_glass::Style {
                        blur_radius: 500.0,
                        lightness: 1.5,
                        edge_radius: 3.0,
                        edge_height: 100.0,
                        rim_width: 1.0,
                        ..Default::default()
                    }),
                space().height(Length::FillPortion(3)),
            ]
            .align_x(Alignment::Center),
        )
        .center(Length::Fill)
        .into()
    }

    fn top_bar(&self) -> Element<'_, Message> {
        iced::widget::container(
            row![
                mouse_area(
                    svg(icon_path_outline("desktop"))
                        .style(self.svg_white())
                        .width(30.0)
                        .height(30.0)
                )
                .on_press(Message::ToggleMenu)
            ]
            .align_y(Alignment::Center)
            .width(50.0),
        )
        .width(Length::Fill)
        .padding(5.0)
        .height(40.0)
        .align_x(Alignment::End)
        .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        container(
            glass_container(
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
                    self.slider(
                        9,
                        "Display",
                        "moon",
                        "sunny",
                        self.brightness,
                        Message::Brightness
                    ),
                    self.slider(
                        10,
                        "Sound",
                        "volume-off",
                        "volume-high",
                        self.volume,
                        Message::Volume
                    ),
                ]
                .spacing(20.0),
            )
            .glass_style(|_theme| iced_glass::Style {
                blur_radius: 50.0,
                edge_radius: 20.0,
                opacity: self.get_opacity(),
                edge_type: EdgeType::SoftEdge,
                ..Default::default()
            }),
        )
        .align_right(Length::Fill)
        .padding(20.0)
        .into()
    }

    fn small_icon_with_one_line(
        &self,
        index: usize,
        icon: &'static str,
        top_text: &'static str,
    ) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                row![
                    container(
                        svg(icon_path_outline(icon))
                            .style(self.svg_white())
                            .opacity(self.get_opacity())
                            .width(25.0)
                            .height(25.0)
                    )
                    .center(40.0)
                    .style(self.border_radius_blue(20.0)),
                    text(top_text)
                        .size(12.0)
                        .wrapping(text::Wrapping::None)
                        .style(self.text_white())
                        .font(FONT_BOLD),
                ]
                .align_y(Alignment::Center)
                .spacing(10.0),
            )
            .padding(15.0)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .center_y(70.0)
            .width(160.0)
            .style(border_radius(50.0)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn small_icon_with_two_lines(
        &self,
        index: usize,
        icon: &'static str,
        top_text: &'static str,
        bottom_text: &'static str,
    ) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                row![
                    container(
                        svg(icon_path_outline(icon))
                            .style(self.svg_blue())
                            .opacity(self.get_opacity())
                            .width(25.0)
                            .height(25.0)
                    )
                    .center(40.0)
                    .style(self.border_radius_white(20.0)),
                    column![
                        text(top_text)
                            .size(12.0)
                            .wrapping(text::Wrapping::None)
                            .style(self.text_white())
                            .font(FONT_BOLD),
                        text(bottom_text)
                            .size(10.0)
                            .style(self.text_white())
                            .font(FONT_NORMAL)
                    ]
                ]
                .align_y(Alignment::Center)
                .spacing(10.0),
            )
            .padding(15.0)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .center_y(70.0)
            .width(160.0)
            .style(border_radius(50.0)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn get_lightness(&self, index: usize) -> f32 {
        if let Some(idx) = self.hovered
            && idx == index
        {
            self.lightness
                .animation
                .interpolate(-0.5, -0.25, Instant::now())
        } else {
            -0.5
        }
    }

    fn get_blur_radius(&self, index: usize) -> f32 {
        if let Some(idx) = self.hovered
            && idx == index
        {
            self.lightness
                .animation
                .interpolate(50.0, 100.0, Instant::now())
        } else {
            50.0
        }
    }

    fn get_edge_height(&self, index: usize) -> f32 {
        if let Some(idx) = self.hovered
            && idx == index
        {
            self.lightness
                .animation
                .interpolate(100.0, 150.0, Instant::now())
        } else {
            100.0
        }
    }

    fn get_opacity(&self) -> f32 {
        self.opacity.interpolate(0.0, 1.0, Instant::now())
    }

    fn get_edge_radius(&self) -> f32 {
        self.edge_radius.interpolate(0.0, 8.0, Instant::now())
    }

    fn settings_glass_style(&self, _theme: &iced::Theme, index: usize) -> iced_glass::Style {
        iced_glass::Style {
            blur_radius: self.get_blur_radius(index),
            saturation: 1.1,
            lightness: self.get_lightness(index),
            edge_radius: self.get_edge_radius(),
            edge_height: self.get_edge_height(index),
            rim_width: 1.0,
            opacity: self.get_opacity(),
            ..Default::default()
        }
    }

    fn large_box(&self, index: usize) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                column![
                    image("examples/desktop/assets/album_cover.jpg")
                        .border_radius(15.0)
                        .opacity(self.get_opacity())
                        .width(60.0),
                    column![
                        text("Deep Meridian")
                            .width(Length::Fill)
                            .size(12.0)
                            .style(self.text_white())
                            .font(FONT_BOLD),
                        text("Terra Pulse - 2021")
                            .size(10.0)
                            .width(Length::Fill)
                            .style(self.text_white())
                            .font(FONT_NORMAL)
                    ],
                    row![
                        svg(icon_path_filled("play-back"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity()),
                        svg(icon_path_filled("play"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity()),
                        svg(icon_path_filled("play-forward"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity())
                    ]
                ]
                .align_x(Alignment::Center)
                .spacing(10.0),
            )
            .padding(15.0)
            .center_y(160.0)
            .width(160.0)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .style(border_radius(35.0)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn circular_icon(&self, index: usize, icon: &'static str) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                svg(icon_path_outline(icon))
                    .style(self.svg_white())
                    .opacity(self.get_opacity()),
            )
            .padding(20.0)
            .center(70.0)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .style(border_radius(50.0)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn slider(
        &self,
        index: usize,
        label: &'static str,
        left_icon: &'static str,
        right_icon: &'static str,
        value: f32,
        message: impl Fn(f32) -> Message + 'static,
    ) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                column![
                    text(label)
                        .size(13.0)
                        .style(self.text_white())
                        .font(FONT_BOLD),
                    row![
                        svg(icon_path_outline(left_icon))
                            .style(self.svg_white())
                            .opacity(self.get_opacity()),
                        glass_slider(0.0..=1.0, value, message)
                            .step(0.01_f32)
                            .width(250.0)
                            .style(self.slider_style()),
                        svg(icon_path_outline(right_icon))
                            .style(self.svg_white())
                            .opacity(self.get_opacity())
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(20.0)
                ]
                .spacing(10.0),
            )
            .padding(12.5)
            .height(75.0)
            .width(340.0)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .style(border_radius(25.0)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
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
            .glass_style(|_theme| iced_glass::Style {
                blur_radius: 50.0,
                edge_radius: 20.0,
                edge_height: 100.0,
                rim_width: 1.0,
                ..Default::default()
            })
            .padding(20.0)
            .style(border_radius(40.0))
            .into()
    }

    fn icon(&self, icon: &'static str, color: BaseColor) -> Element<'_, Message> {
        container(
            svg(icon_path_outline(icon))
                .width(45.0)
                .height(45.0)
                .style(self.svg_white()),
        )
        .center(70.0)
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

    fn border_radius_white(
        &self,
        radius: f32,
    ) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_theme| iced::widget::container::Style {
            border: iced::Border {
                radius: radius.into(),
                ..Default::default()
            },
            background: Some(Background::Color(color)),
            ..Default::default()
        }
    }

    fn border_radius_blue(
        &self,
        radius: f32,
    ) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
        let color = color_opacity(BaseColor::Blue.base(), self.get_opacity());
        move |_theme| iced::widget::container::Style {
            border: iced::Border {
                radius: radius.into(),
                ..Default::default()
            },
            background: Some(Background::Color(color)),
            ..Default::default()
        }
    }

    fn svg_white(&self) -> impl Fn(&iced::Theme, svg::Status) -> svg::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }

    fn svg_blue(&self) -> impl Fn(&iced::Theme, svg::Status) -> svg::Style {
        let color = color_opacity(BaseColor::Blue.base(), self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }

    fn text_white(&self) -> impl Fn(&iced::Theme) -> text::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_| text::Style { color: Some(color) }
    }

    fn slider_style(&self) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        let background_color = color_opacity(Color::from_rgb(0.3, 0.3, 0.3), self.get_opacity());
        move |_, status| {
            let handle_color = match status {
                slider::Status::Active => color_opacity(color, 0.0),
                _ => color,
            };
            slider::Style {
                rail: Rail {
                    backgrounds: (
                        Background::Color(color),
                        Background::Color(background_color),
                    ),
                    width: 3.0,
                    border: Border {
                        color,
                        width: 0.0,
                        radius: 5.0.into(),
                    },
                },
                handle: Handle {
                    shape: slider::HandleShape::Circle { radius: 5.0 },
                    background: Background::Color(handle_color),
                    border_width: 0.0,
                    border_color: Color::WHITE,
                },
            }
        }
    }
}

fn color_opacity(base: Color, opacity: f32) -> Color {
    Color::from_rgba(base.r, base.g, base.b, opacity)
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

fn icon_path_filled(name: &str) -> String {
    format!("examples/desktop/assets/{name}.svg")
}

fn icon_path_outline(name: &str) -> String {
    format!("examples/desktop/assets/{name}-outline.svg")
}

#[allow(unused)]
fn red_border(_theme: &iced::Theme) -> container::Style {
    container::Style {
        border: Border {
            color: Color::from_rgb(1.0, 0.0, 0.0),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}
