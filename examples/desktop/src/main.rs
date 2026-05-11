use std::time::{Duration, Instant};

use iced::{
    Alignment, Animation, Background, Border, Color, ContentFit, Element, Font, Gradient, Length,
    Padding, Size, Subscription, Task, Theme, Vector,
    animation::Easing,
    font::{self, Family, Stretch, Weight},
    gradient::Linear,
    widget::{
        Row, column, container, image, mouse_area, row,
        slider::{self, Handle, Rail},
        space, stack, svg, text, text_input,
    },
};

use iced_glass::{
    glass_stack,
    widget::{
        EdgeType, StackOffset, container as glass_container, slider as glass_slider,
        text as glass_text,
    },
};

const FONT_BOLD: Font = Font {
    family: Family::Name("Noto Sans"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

const FONT_NORMAL: Font = Font {
    family: Family::Name("Noto Sans"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

#[derive(Debug, Clone)]
pub struct Ui {
    hovered: Option<usize>,
    lightness: HoverInfo,
    opacity: Animation<bool>,
    search_opacity: Animation<bool>,
    search_icons: Animation<bool>,
    search_blend: Animation<bool>,
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
    KeyPress(iced::keyboard::Key),
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
            search_opacity: Animation::new(false).quick().easing(Easing::EaseOut),
            search_icons: Animation::new(false)
                .delay(Duration::from_millis(1000))
                .easing(Easing::EaseOutBack)
                .duration(Duration::from_millis(700)),
            search_blend: Animation::new(false)
                .delay(Duration::from_millis(1100))
                .easing(Easing::Linear)
                .duration(Duration::from_millis(700)),
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
            Message::KeyPress(key) => {
                println!("Key pressed: {:?}", key);
                if key == iced::keyboard::Key::Character("m".into()) {
                    let new_state = !self.opacity.value();
                    self.opacity.go_mut(new_state, Instant::now());
                    self.edge_radius.go_mut(new_state, Instant::now());
                }
                if key == iced::keyboard::Key::Character("s".into()) {
                    let new_state = !self.search_opacity.value();
                    self.search_opacity.go_mut(new_state, Instant::now());
                    self.search_icons.go_mut(new_state, Instant::now());
                    self.search_blend.go_mut(new_state, Instant::now());
                }
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let now = Instant::now();
        let animation = if self.opacity.is_animating(now)
            || self.edge_radius.is_animating(now)
            || self.lightness.animation.is_animating(now)
            || self.search_opacity.is_animating(now)
            || self.search_icons.is_animating(now)
            || self.search_blend.is_animating(now)
        {
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
        } else {
            Subscription::none()
        };
        let keyboard = iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyPressed { key, .. } => Some(Message::KeyPress(key)),
            _ => None,
        });
        Subscription::batch(vec![animation, keyboard])
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
            row![
                if self.get_search_opacity() > 0.0 {
                    self.search_bar()
                } else {
                    space().width(Length::FillPortion(1)).into()
                },
                if self.get_opacity() > 0.0 {
                    self.settings()
                } else {
                    space().width(Length::FillPortion(1)).into()
                },
            ],
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
        container(
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

    fn search_bar(&self) -> Element<'_, Message> {
        let opacity = self.get_search_opacity();
        let icons_opacity = self.get_search_icons_value();
        let icons_blend = self.get_search_blend_value();
        let b = (1.0 - (-40.0 * icons_blend).exp()) * (1.0 - (-2.0 * (1.0 - icons_blend)).exp());
        container(
            glass_stack![
                container(
                    row![
                        svg(icon_path_outline("search"))
                            .opacity(self.get_search_opacity())
                            .style(self.svg_white())
                            .width(30.0)
                            .height(30.0),
                        text_input("Search...", "")
                            .width(Length::Fill)
                            .style(|_theme, _status| text_input::Style {
                                border: Border {
                                    color: color_opacity(Color::WHITE, self.get_search_opacity()),
                                    width: 0.0,
                                    radius: 50.0.into(),
                                },
                                background: Background::Color(Color::TRANSPARENT),
                                icon: Color::WHITE,
                                placeholder: color_opacity(Color::WHITE, self.get_search_opacity()),
                                value: color_opacity(Color::WHITE, self.get_search_opacity()),
                                selection: color_opacity(Color::WHITE, self.get_search_opacity()),
                            }),
                    ]
                    .padding(10.0)
                )
                .width(640.0 - icons_opacity * 200.0 - 40.0 * opacity)
                .with_offset(Vector::new(0.0, 0.0)),
                container(
                    svg(icon_path_outline("camera"))
                        .style(self.svg_white())
                        .opacity(icons_opacity * opacity)
                        .width(30.0)
                        .height(30.0)
                )
                .center(50.0)
                .with_offset(Vector::new(410.0, 0.0)),
                container(
                    svg(icon_path_outline("airplane"))
                        .style(self.svg_white())
                        .opacity(icons_opacity * opacity)
                        .width(30.0)
                        .height(30.0)
                )
                .center(50.0)
                .with_offset(Vector::new(410.0 + icons_opacity * 60.0, 0.0)),
                container(
                    svg(icon_path_outline("sunny"))
                        .style(self.svg_white())
                        .opacity(icons_opacity * opacity)
                        .width(30.0)
                        .height(30.0)
                )
                .center(50.0)
                .with_offset(Vector::new(410.0 + icons_opacity * 120.0, 0.0)),
                container(
                    svg(icon_path_outline("terminal"))
                        .style(self.svg_white())
                        .opacity(icons_opacity * opacity)
                        .width(30.0)
                        .height(30.0)
                )
                .center(50.0)
                .with_offset(Vector::new(410.0 + icons_opacity * 180.0, 0.0)),
            ]
            .width(700.0)
            .height(50.0)
            .glass_style(|_theme| iced_glass::Style {
                blur_radius: 200.0,
                edge_radius: 25.0,
                edge_height: 150.0,
                rim_width: 1.0,
                rim_angle: 1.0,
                opacity: self.get_search_opacity(),
                lightness: -2.0,
                ..Default::default()
            })
            .blending_factor(40.0 * b)
            .corner_radius(25.0),
        )
        .padding(Padding {
            top: 200.0,
            right: 0.0,
            bottom: 0.0,
            left: 200.0 + 20.0 * opacity,
        })
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

    fn get_search_opacity(&self) -> f32 {
        self.search_opacity.interpolate(0.0, 1.0, Instant::now())
    }

    fn get_search_icons_value(&self) -> f32 {
        self.search_icons.interpolate(0.0, 1.0, Instant::now())
    }

    fn get_search_blend_value(&self) -> f32 {
        self.search_blend.interpolate(0.0, 1.0, Instant::now())
    }

    fn get_edge_radius(&self) -> f32 {
        self.edge_radius.interpolate(0.0, 8.0, Instant::now())
    }

    fn settings_glass_style(&self, _theme: &Theme, index: usize) -> iced_glass::Style {
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

    fn border_radius_white(&self, radius: f32) -> impl Fn(&Theme) -> container::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_theme| container::Style {
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            background: Some(Background::Color(color)),
            ..Default::default()
        }
    }

    fn border_radius_blue(&self, radius: f32) -> impl Fn(&Theme) -> container::Style {
        let color = color_opacity(BaseColor::Blue.base(), self.get_opacity());
        move |_theme| container::Style {
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            background: Some(Background::Color(color)),
            ..Default::default()
        }
    }

    fn svg_white(&self) -> impl Fn(&Theme, svg::Status) -> svg::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }

    fn svg_blue(&self) -> impl Fn(&Theme, svg::Status) -> svg::Style {
        let color = color_opacity(BaseColor::Blue.base(), self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }

    fn text_white(&self) -> impl Fn(&Theme) -> text::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_| text::Style { color: Some(color) }
    }

    fn slider_style(&self) -> impl Fn(&Theme, slider::Status) -> slider::Style {
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

fn border_radius(radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        border: Border {
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
fn red_border(_theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: Color::from_rgb(1.0, 0.0, 0.0),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}
