use iced::time::Instant;
use std::time::Duration;

use iced::{
    Alignment, Animation, Background, Border, Color, ContentFit, Element, Font, Length, Padding,
    Shadow, Size, Subscription, Task, Theme,
    animation::Easing,
    font::{self, Family, Stretch, Weight},
    widget::{
        button, column, container, image, mouse_area, responsive, row,
        slider::{self, Handle, Rail},
        space, stack, svg, text,
    },
};

use iced_glass::{
    SliderType,
    widget::{EdgeType, container as glass_container, slider as glass_slider},
};

mod icons;

const FONT_BOLD: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

const FONT_NORMAL: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: font::Style::Normal,
};

const BACKGROUND: &[u8] = include_bytes!("../assets/stars.jpg");
const ALBUM: &[u8] = include_bytes!("../assets/album_cover.jpg");

#[derive(Debug, Clone)]
pub struct Ui {
    hovered: Option<usize>,
    lightness: HoverInfo,
    opacity: Animation<bool>,
    edge_radius: Animation<bool>,
    brightness: f32,
    volume: f32,
    background: image::Handle,
    album_cover: image::Handle,
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
        .window_size(Size::new(1080.0, 1920.0))
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
            background: image::Handle::from_bytes(BACKGROUND),
            album_cover: image::Handle::from_bytes(ALBUM),
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
                if key == iced::keyboard::Key::Character("m".into()) {
                    let new_state = !self.opacity.value();
                    self.opacity.go_mut(new_state, Instant::now());
                    self.edge_radius.go_mut(new_state, Instant::now());
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
            // self.clock(),
            self.desktop_elements(),
        ])
        .center(Length::Fill)
        .into()
    }

    fn wallpaper(&self) -> Element<'_, Message> {
        image(&self.background)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Cover)
            .into()
    }

    fn desktop_elements(&self) -> Element<'_, Message> {
        stack![
            if self.get_opacity() > 0.0 {
                self.settings()
            } else {
                space().width(Length::Fill).into()
            },
            container(
                button(svg(icons::svg_handle("gear")).style(self.svg_white()))
                    .style(|_theme, _status| {
                        button::Style {
                            background: None,
                            text_color: Color::TRANSPARENT,
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: false,
                        }
                    })
                    .width(100.0)
                    .height(100.0)
                    .on_press(Message::ToggleMenu)
            )
            .center_x(Length::Fill)
            .align_bottom(Length::Fill)
            .padding(20.0)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        glass_container(responsive(|size| {
            column![
                row![
                    column![
                        self.small_icon_with_two_lines(0, size, "wifi", "Wi-Fi", "Connected"),
                        self.small_icon_with_two_lines(1, size, "bluetooth", "Bluetooth", "On")
                    ]
                    .spacing(Self::spacing(size)),
                    self.large_box(2, size),
                ]
                .spacing(Self::spacing(size)),
                row![
                    self.small_icon_with_two_lines(3, size, "airplane", "Airplane", "Off"),
                    self.circular_icon(4, size, "camera"),
                    self.circular_icon(5, size, "desktop"),
                ]
                .spacing(Self::spacing(size)),
                row![
                    self.circular_icon(6, size, "finger-print"),
                    self.circular_icon(7, size, "at"),
                    self.small_icon_with_one_line(8, size, "moon", "Focus"),
                ]
                .spacing(Self::spacing(size)),
                self.slider(9, size, "sunny", self.brightness, Message::Brightness),
                self.slider(10, size, "volume-high", self.volume, Message::Volume),
            ]
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .spacing(Self::spacing(size))
            .into()
        }))
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 200.0,
            edge_radius: 0.0,
            lightness: -0.25,
            opacity: self.get_opacity(),
            edge_type: EdgeType::SoftEdge,
            ..Default::default()
        })
        .center(Length::Fill)
        .padding(Padding::default().horizontal(40.0).top(100.0))
        .into()
    }

    fn small_icon_with_one_line(
        &self,
        index: usize,
        size: iced::Size,
        icon: &'static str,
        top_text: &'static str,
    ) -> Element<'_, Message> {
        let w = size.width;
        mouse_area(
            glass_container(
                row![
                    container(
                        svg(icons::svg_handle(icon))
                            .style(self.svg_white())
                            .opacity(self.get_opacity())
                    )
                    .center(0.1 * w)
                    .padding(0.02 * w)
                    .style(self.border_radius_blue(w)),
                    text(top_text)
                        .size(0.033 * w)
                        .wrapping(text::Wrapping::None)
                        .style(self.text_white())
                        .font(FONT_BOLD),
                ]
                .align_y(Alignment::Center)
                .spacing(0.028 * w),
            )
            .padding(0.04 * w)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .center_y(Self::n_rows(size, 1))
            .width(Self::n_cols(size, 2))
            .style(border_radius(w)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn small_icon_with_two_lines(
        &self,
        index: usize,
        size: iced::Size,
        icon: &'static str,
        top_text: &'static str,
        bottom_text: &'static str,
    ) -> Element<'_, Message> {
        let w = size.width;
        mouse_area(
            glass_container(
                row![
                    container(
                        svg(icons::svg_handle(icon))
                            .style(self.svg_blue())
                            .opacity(self.get_opacity())
                    )
                    .center(0.1 * w)
                    .padding(0.02 * w)
                    .style(self.border_radius_white(w)),
                    column![
                        text(top_text)
                            .size(0.033 * w)
                            .wrapping(text::Wrapping::None)
                            .style(self.text_white())
                            .font(FONT_BOLD),
                        text(bottom_text)
                            .size(0.03 * w)
                            .style(self.text_white())
                            .font(FONT_NORMAL)
                    ]
                ]
                .align_y(Alignment::Center)
                .spacing(0.03 * w),
            )
            .padding(0.04 * w)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .center_y(Self::n_rows(size, 1))
            .width(Self::n_cols(size, 2))
            .style(border_radius(w)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn n_cols(size: iced::Size, n: usize) -> f32 {
        let n = n as f32;
        0.2 * size.width * n + Self::spacing(size) * (n - 1.0)
    }

    fn n_rows(size: iced::Size, n: usize) -> f32 {
        let n = n as f32;
        0.2 * size.width * n + Self::spacing(size) * (n - 1.0)
    }

    fn spacing(size: iced::Size) -> f32 {
        0.05 * size.width
    }

    fn get_lightness(&self, index: usize) -> f32 {
        if let Some(idx) = self.hovered
            && idx == index
        {
            self.lightness
                .animation
                .interpolate(-0.25, 0.0, Instant::now())
        } else {
            -0.25
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
                .interpolate(200.0, 300.0, Instant::now())
        } else {
            200.0
        }
    }

    fn get_opacity(&self) -> f32 {
        self.opacity.interpolate(0.0, 1.0, Instant::now())
    }

    fn get_edge_radius(&self) -> f32 {
        self.edge_radius.interpolate(0.0, 24.0, Instant::now())
    }

    fn settings_glass_style(&self, _theme: &Theme, index: usize) -> iced_glass::Style {
        iced_glass::Style {
            blur_radius: self.get_blur_radius(index),
            saturation: 1.1,
            lightness: self.get_lightness(index),
            edge_radius: self.get_edge_radius(),
            edge_height: self.get_edge_height(index),
            rim_width: 2.0,
            rim_angle: 0.5,
            opacity: self.get_opacity(),
            ..Default::default()
        }
    }

    fn large_box(&self, index: usize, size: iced::Size) -> Element<'_, Message> {
        let w = size.width;
        mouse_area(
            glass_container(
                column![
                    image(&self.album_cover)
                        .border_radius(0.04 * w)
                        .opacity(self.get_opacity())
                        .width(0.17 * w),
                    column![
                        text("Deep Meridian")
                            .width(Length::Fill)
                            .size(0.033 * w)
                            .style(self.text_white())
                            .font(FONT_BOLD),
                        text("Terra Pulse - 2021")
                            .size(0.028 * w)
                            .width(Length::Fill)
                            .style(self.text_white())
                            .font(FONT_NORMAL)
                    ],
                    row![
                        svg(icons::svg_handle("play-back"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity()),
                        svg(icons::svg_handle("play"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity()),
                        svg(icons::svg_handle("play-forward"))
                            .style(self.svg_white())
                            .opacity(self.get_opacity())
                    ]
                ]
                .align_x(Alignment::Center)
                .spacing(0.028 * w),
            )
            .padding(0.04 * w)
            .center_y(Self::n_rows(size, 2))
            .width(Self::n_cols(size, 2))
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .style(border_radius(Self::n_rows(size, 1) * 0.5)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn circular_icon(
        &self,
        index: usize,
        size: iced::Size,
        icon: &'static str,
    ) -> Element<'_, Message> {
        mouse_area(
            glass_container(
                svg(icons::svg_handle(icon))
                    .style(self.svg_white())
                    .opacity(self.get_opacity()),
            )
            .padding(0.05 * size.width)
            .center(0.2 * size.width)
            .glass_style(move |theme| self.settings_glass_style(theme, index))
            .style(border_radius(0.1 * size.width)),
        )
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
        .into()
    }

    fn slider(
        &self,
        index: usize,
        size: iced::Size,
        icon: &'static str,
        value: f32,
        message: impl Fn(f32) -> Message + 'static,
    ) -> Element<'_, Message> {
        let height = Self::n_rows(size, 1);
        let w = size.width;
        mouse_area(stack![
            glass_slider(0.0..=1.0, value, message)
                .slider_type(SliderType::Filled)
                .step(0.001_f32)
                .width(w)
                .height(height)
                .style(self.slider_style())
                .glass_style(move |theme| self.settings_glass_style(theme, index)),
            container(
                svg(icons::svg_handle(icon))
                    .style(self.svg_blue())
                    .opacity(self.get_opacity())
                    .width(0.08 * w)
                    .height(0.08 * w)
            )
            .center_y(height)
            .align_left(w)
            .padding(0.05 * w),
        ])
        .on_enter(Message::Hovered(index))
        .on_exit(Message::ClearHover)
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
        let color = color_opacity(Color::from_rgb(0.3, 0.3, 1.0), self.get_opacity());
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
        let color = color_opacity(Color::from_rgb(0.3, 0.3, 1.0), self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }

    fn text_white(&self) -> impl Fn(&Theme) -> text::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_| text::Style { color: Some(color) }
    }

    fn slider_style(&self) -> impl Fn(&Theme, slider::Status) -> slider::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        // let background_color = color_opacity(Color::from_rgb(0.3, 0.3, 0.3), self.get_opacity());
        let background_color = Color::TRANSPARENT;
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
                    width: 10.0,
                    border: Border {
                        color,
                        width: 0.0,
                        radius: 150.0.into(),
                    },
                },
                handle: Handle {
                    shape: slider::HandleShape::Circle { radius: 15.0 },
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

fn border_radius(radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        border: Border {
            radius: radius.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
