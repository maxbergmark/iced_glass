use iced::{time::Instant, widget::image};
use once_cell::sync::Lazy;
use std::{collections::HashSet, time::Duration};

use iced::{
    Alignment, Animation, Border, Color, ContentFit, Element, Length, Padding, Shadow, Size,
    Subscription, Task, Theme,
    animation::Easing,
    widget::{button, column, container, responsive, row, stack, svg},
};

use iced_glass::widget::{EdgeType, container as glass_container};

use crate::{
    music_view::MusicView, skin::Skin, slider_with_icon::SliderWithIcon,
    toggle_button::ToggleButton, toggle_with_text::ToggleWithText,
};

mod icons;
mod music_view;
mod skin;
mod slider_with_icon;
mod spacing;
mod toggle_button;
mod toggle_with_text;

const BACKGROUND: &[u8] = include_bytes!("../assets/stars.jpg");
static BACKGROUND_HANDLE: Lazy<image::Handle> = Lazy::new(|| image::Handle::from_bytes(BACKGROUND));

#[derive(Debug, Clone)]
pub struct Ui {
    hovered: Option<usize>,
    toggled: HashSet<usize>,
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
    Toggle(usize),
    ClearHover,
    KeyPress(iced::keyboard::Key),
    ToggleMenu,
    Brightness(f32),
    Volume(f32),
    Noop,
}

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
            toggled: [8].into(), // focus toggle
            lightness: HoverInfo {
                index: 0,
                is_hovered: true,
                animation: Animation::new(true).duration(Duration::from_millis(150)),
            },
            opacity: Animation::new(true).slow(),
            edge_radius: Animation::new(true)
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
            Message::Toggle(index) => {
                if self.toggled.contains(&index) {
                    self.toggled.remove(&index);
                } else {
                    self.toggled.insert(index);
                }
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

    fn skin(&self) -> Skin {
        let now = Instant::now();
        Skin {
            opacity: self.opacity.interpolate(0.0, 1.0, now),
            hovered: self.hovered,
            hover_t: self.lightness.animation.interpolate(0.0, 1.0, now),
            edge_radius: self.edge_radius.interpolate(0.0, 24.0, now),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(stack![self.wallpaper(), self.desktop_elements(),])
            .center(Length::Fill)
            .into()
    }

    fn wallpaper(&self) -> Element<'_, Message> {
        image(BACKGROUND_HANDLE.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Cover)
            .into()
    }

    fn desktop_elements(&self) -> Element<'_, Message> {
        stack![
            self.is_visible().then_some(self.settings()),
            self.settings_button_overlay(),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn settings_button_overlay(&self) -> Element<'_, Message> {
        responsive(|size| {
            let overlay = container(self.settings_button());
            if size.width < size.height {
                overlay.center_x(Length::Fill).align_bottom(Length::Fill)
            } else {
                overlay.center_y(Length::Fill).align_right(Length::Fill)
            }
            .padding(20.0)
            .into()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn settings_button(&self) -> Element<'_, Message> {
        button(svg(icons::svg_handle("gear")).style(self.svg_white()))
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: Color::TRANSPARENT,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: false,
            })
            .width(80.0)
            .height(80.0)
            .on_press(Message::ToggleMenu)
            .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        self.blurred_background(move |size| {
            if size.width < size.height {
                self.portrait_settings(size)
            } else {
                self.landscape_settings(size)
            }
        })
    }

    fn blurred_background<'a, F>(&'a self, content: F) -> Element<'a, Message>
    where
        F: Fn(iced::Size) -> Element<'a, Message> + 'a,
    {
        glass_container(responsive(content))
            .glass_style(|_theme| iced_glass::Style {
                blur_radius: 200.0,
                edge_radius: 0.0,
                lightness: -0.25,
                opacity: self.get_opacity(),
                edge_type: EdgeType::SoftEdge,
                ..Default::default()
            })
            .center(Length::Fill)
            .padding(Padding::default().horizontal(40.0).top(100.0).bottom(100.0))
            .into()
    }

    fn portrait_settings(&self, size: iced::Size) -> Element<'_, Message> {
        let skin = self.skin();

        column![
            row![
                column![
                    self.wifi_toggle(size).view(skin),
                    self.bluetooth_toggle(size).view(skin),
                ]
                .spacing(spacing::spacing(size)),
                self.music_view(size).view(skin),
            ]
            .spacing(spacing::spacing(size)),
            row![
                column![
                    self.airplane_toggle(size).view(skin),
                    self.focus_toggle(size).view(skin),
                ]
                .spacing(spacing::spacing(size)),
                self.brightness_slider(size).view(skin),
                self.volume_slider(size).view(skin),
            ]
            .spacing(spacing::spacing(size)),
            (size.height > 1.2633333 * size.width).then_some(
                row![
                    self.camera_toggle(size).view(skin),
                    self.desktop_toggle(size).view(skin),
                    self.fingerprint_toggle(size).view(skin),
                    self.at_toggle(size).view(skin),
                ]
                .spacing(spacing::spacing(size)),
            )
        ]
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .spacing(spacing::spacing(size))
        .into()
    }

    fn landscape_settings(&self, size: iced::Size) -> Element<'_, Message> {
        let skin = self.skin();
        row![
            column![
                self.wifi_toggle(size).view(skin),
                self.bluetooth_toggle(size).view(skin),
                self.airplane_toggle(size).view(skin),
                self.focus_toggle(size).view(skin),
            ]
            .spacing(spacing::spacing(size)),
            column![
                self.music_view(size).view(skin),
                self.brightness_slider(size).view(skin),
                self.volume_slider(size).view(skin),
            ]
            .spacing(spacing::spacing(size)),
            (size.width > 1.2633333 * size.height).then_some(
                column![
                    self.camera_toggle(size).view(skin),
                    self.desktop_toggle(size).view(skin),
                    self.fingerprint_toggle(size).view(skin),
                    self.at_toggle(size).view(skin),
                ]
                .spacing(spacing::spacing(size)),
            )
        ]
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .spacing(spacing::spacing(size))
        .into()
    }

    fn wifi_toggle(&self, size: iced::Size) -> ToggleWithText {
        ToggleWithText::new(
            0,
            "wifi",
            "Wi-Fi",
            "Disconnected",
            "Connected",
            size,
            &self.toggled,
        )
    }

    fn bluetooth_toggle(&self, size: iced::Size) -> ToggleWithText {
        ToggleWithText::new(
            1,
            "bluetooth",
            "Bluetooth",
            "Off",
            "On",
            size,
            &self.toggled,
        )
    }

    fn focus_toggle(&self, size: iced::Size) -> ToggleWithText {
        ToggleWithText::new(2, "moon", "Focus", "Off", "On", size, &self.toggled)
    }

    fn airplane_toggle(&self, size: iced::Size) -> ToggleWithText {
        ToggleWithText::new(3, "airplane", "Airplane", "Off", "On", size, &self.toggled)
    }

    fn camera_toggle(&self, size: iced::Size) -> ToggleButton {
        ToggleButton::new(4, "camera", size, &self.toggled)
    }

    fn desktop_toggle(&self, size: iced::Size) -> ToggleButton {
        ToggleButton::new(5, "desktop", size, &self.toggled)
    }

    fn fingerprint_toggle(&self, size: iced::Size) -> ToggleButton {
        ToggleButton::new(6, "finger-print", size, &self.toggled)
    }

    fn at_toggle(&self, size: iced::Size) -> ToggleButton {
        ToggleButton::new(7, "at", size, &self.toggled)
    }

    fn volume_slider(&self, size: iced::Size) -> SliderWithIcon {
        SliderWithIcon::new(8, "volume-high", size, self.volume, Message::Volume)
    }

    fn brightness_slider(&self, size: iced::Size) -> SliderWithIcon {
        SliderWithIcon::new(9, "sunny", size, self.brightness, Message::Brightness)
    }

    fn music_view(&self, size: iced::Size) -> MusicView {
        MusicView::new(10, size)
    }

    fn get_opacity(&self) -> f32 {
        self.opacity.interpolate(0.0, 1.0, Instant::now())
    }

    fn is_visible(&self) -> bool {
        self.get_opacity() > 0.0
    }

    fn svg_white(&self) -> impl Fn(&Theme, svg::Status) -> svg::Style {
        let color = color_opacity(Color::WHITE, self.get_opacity());
        move |_, _| svg::Style { color: Some(color) }
    }
}

fn color_opacity(base: Color, opacity: f32) -> Color {
    Color::from_rgba(base.r, base.g, base.b, opacity)
}
