use std::time::Instant;

use iced::{
    Alignment, Animation, Background, Border, Color, Element, Font, Length, Size, Subscription,
    Task,
    font::Weight,
    widget::{
        Row, button, column, container, image, mouse_area, row, scrollable,
        slider::{self, Handle, Rail},
        space, stack, svg, text, text_input,
    },
};
use iced_glass::widget::{container as glass_container, slider as glass_slider};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Ui {
    album_cards: Vec<AlbumCard>,
    current_album: Option<AlbumCard>,
    hover_info: HoverInfo,
    search_value: String,
    playback_time: f32,
    playing: bool,
    opacity: Animation<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct HoverInfo {
    index: usize,
    is_hovered: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchValueChange(String),
    SetCurrentAlbum(AlbumCard),
    SetHoverAlbum(usize),
    ClearHoverAlbum,
    SetPlaybackTime(f32),
    TogglePlayback,
    Noop,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumCard {
    file: String,
    artist: String,
    title: String,
    year: i32,
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
        let album_cards = include_str!("../assets/album_covers/albums.json");
        let album_cards: Vec<AlbumCard> =
            serde_json::from_str(album_cards).expect("Failed to parse album cards");
        Self {
            album_cards,
            search_value: String::new(),
            current_album: None,
            hover_info: HoverInfo {
                index: 0,
                is_hovered: false,
            },
            playback_time: 0.3,
            playing: false,
            opacity: Animation::new(false).quick(),
        }
    }
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchValueChange(search_value) => {
                self.search_value = search_value;
                self.opacity.go_mut(false, Instant::now());
                self.hover_info.is_hovered = false;
            }
            Message::SetCurrentAlbum(album_card) => {
                self.current_album = Some(album_card);
            }
            Message::SetPlaybackTime(playback_time) => {
                self.playback_time = playback_time;
            }
            Message::TogglePlayback => {
                self.playing = !self.playing;
            }
            Message::SetHoverAlbum(index) => {
                let old_index = self.hover_info.index;
                self.hover_info = HoverInfo {
                    index,
                    is_hovered: true,
                    // event_time: Instant::now(),
                };
                let now = Instant::now();
                if self.opacity.is_animating(now) && old_index != index {
                    self.opacity = Animation::new(false).quick();
                }
                self.opacity.go_mut(true, now);
            }
            Message::ClearHoverAlbum => {
                self.hover_info.is_hovered = false;
                self.opacity.go_mut(false, Instant::now());
                // self.hover_info.event_time = Instant::now();
            }
            Message::Noop => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Only request frames while the animation is running
        let now = Instant::now();
        if self.opacity.is_animating(now) {
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
        } else {
            Subscription::none()
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![stack![self.scroll_view(), self.glass(),],])
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::BLACK)),
                ..Default::default()
            })
            .into()
    }

    fn scroll_view(&self) -> Element<'_, Message> {
        scrollable(
            container(
                column![
                    space().height(Length::from(100.0)),
                    self.album_list(),
                    space().height(Length::from(150.0)),
                ]
                .width(200.0 * 4.0 + 20.0 * 3.0),
            )
            .height(Length::Fill)
            .center_x(Length::Fill),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(20.0)
        .into()
    }

    fn album_list(&self) -> Element<'_, Message> {
        container(
            Row::with_children(
                self.album_cards
                    .iter()
                    .enumerate()
                    .filter(|(_, album_card)| self.matches_search(album_card))
                    .map(|(idx, album_card)| album_card.view(idx, self.hover_info, &self.opacity)),
            )
            .spacing(20.0)
            .wrap(),
        )
        .width(Length::Fill)
        .into()
    }

    fn matches_search(&self, album_card: &AlbumCard) -> bool {
        album_card
            .title
            .to_lowercase()
            .contains(&self.search_value.to_lowercase())
    }

    fn glass(&self) -> Element<'_, Message> {
        container(column![
            // space().height(Length::from(20.0)),
            self.search_bar(),
            space().height(Length::Fill),
            self.playback_bar(),
        ])
        .center_x(Length::Fill)
        .align_top(Length::Fill)
        .padding(30.0)
        .into()
    }

    fn search_bar(&self) -> Element<'_, Message> {
        glass_container(
            row![
                svg("examples/scroll_view/assets/search.svg")
                    .width(Length::from(30.0))
                    .height(Length::from(30.0))
                    .style(|theme, _status| self.icon_style(theme)),
                text_input("Search...", self.search_value.as_str())
                    .style(|theme, _status| self.input_style(theme))
                    .on_input(Message::SearchValueChange)
            ]
            .padding(iced::Padding {
                top: 10.0,
                right: 30.0,
                bottom: 10.0,
                left: 30.0,
            })
            .spacing(20.0)
            .align_y(Alignment::Center),
        )
        .width(Length::from(600.0))
        .center_y(Length::from(60.0))
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 50.0,
            lightness: -1.5,
            edge_radius: 10.0,
            edge_height: 100.0,
            refractive_index: 1.5,
            ..Default::default()
        })
        .style(|theme| self.style(theme))
        .into()
    }

    fn playback_bar(&self) -> Element<'_, Message> {
        glass_container(
            button(
                container(
                    row![
                        self.navigation_buttons(),
                        self.current_album
                            .as_ref()
                            .map(|album_card| album_card.mini_view(self.playback_time))
                            .unwrap_or_else(|| text("No album selected").size(20.0).into()),
                    ]
                    .padding(iced::Padding {
                        top: 15.0,
                        right: 50.0,
                        bottom: 15.0,
                        left: 50.0,
                    })
                    .spacing(30.0)
                    .align_y(Alignment::Center),
                )
                .center_y(Length::Fill)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .on_press(Message::Noop)
            .style(|theme, _status| button_style(theme)),
        )
        .width(Length::from(600.0))
        .center_y(Length::from(100.0))
        .glass_style(|_theme| iced_glass::Style {
            blur_radius: 50.0,
            lightness: -1.5,
            edge_radius: 10.0,
            edge_height: 100.0,
            refractive_index: 1.5,
            rim_width: 2.0,
            ..Default::default()
        })
        .style(|theme| self.style(theme))
        .into()
    }

    fn navigation_buttons(&self) -> Element<'_, Message> {
        let play_icon = if self.playing {
            "examples/scroll_view/assets/pause.svg"
        } else {
            "examples/scroll_view/assets/play.svg"
        };
        row![
            svg("examples/scroll_view/assets/back.svg")
                .width(Length::from(20.0))
                .height(Length::from(20.0))
                .style(|theme, _status| self.icon_style(theme)),
            button(svg(play_icon).style(|theme, _status| self.icon_style(theme)))
                .width(Length::from(40.0))
                .height(Length::from(40.0))
                .padding(0.0)
                .on_press(Message::TogglePlayback)
                .style(|theme, _status| button_style(theme)),
            svg("examples/scroll_view/assets/forward.svg")
                .width(Length::from(20.0))
                .height(Length::from(20.0))
                .style(|theme, _status| self.icon_style(theme))
        ]
        .spacing(15.0)
        .align_y(Alignment::Center)
        .into()
    }

    fn style(&self, _theme: &iced::Theme) -> container::Style {
        container::Style {
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: iced::Border {
                radius: 100.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn input_style(&self, _theme: &iced::Theme) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            icon: Color::WHITE,
            placeholder: Color::from_rgb(0.8, 0.8, 0.8),
            value: Color::WHITE,
            selection: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border::default(),
        }
    }

    fn icon_style(&self, _theme: &iced::Theme) -> svg::Style {
        svg::Style {
            color: Some(Color::WHITE),
        }
    }
}

const BOLD: Font = Font {
    weight: Weight::Bold,
    family: iced::font::Family::Name("Arial"),
    ..Font::DEFAULT
};

impl AlbumCard {
    fn view(
        &self,
        idx: usize,
        hover_info: HoverInfo,
        opacity: &Animation<bool>,
    ) -> Element<'_, Message> {
        let overlay = self.play_button_overlay(hover_info, idx, opacity);

        container(
            mouse_area(column![stack![self.image(), overlay,], self.album_info(),])
                .on_press(Message::SetCurrentAlbum(self.clone()))
                .on_enter(Message::SetHoverAlbum(idx))
                .on_exit(Message::ClearHoverAlbum),
        )
        .width(Length::from(200.0))
        .center_y(Length::from(250.0))
        .style(|theme| self.style(theme, 1.0))
        .into()
    }

    fn image(&self) -> Element<'_, Message> {
        image(format!(
            "examples/scroll_view/assets/album_covers/{}",
            self.file
        ))
        .width(200.0)
        .height(200.0)
        .into()
    }

    fn play_button_overlay(
        &self,
        hover_info: HoverInfo,
        idx: usize,
        opacity: &Animation<bool>,
    ) -> Element<'_, Message> {
        let is_hovered = hover_info.index == idx;
        if is_hovered {
            let opacity = opacity.interpolate(0.0, 1.0, Instant::now());

            container(
                glass_container(
                    svg("examples/scroll_view/assets/play.svg")
                        .width(Length::from(15.0))
                        .height(Length::from(15.0))
                        .style(|theme, _status| self.icon_style(theme))
                        .opacity(opacity),
                )
                .center(Length::from(40.0))
                .glass_style(move |_theme| iced_glass::Style {
                    blur_radius: 25.0,
                    lightness: -1.5,
                    edge_radius: 10.0,
                    edge_height: 30.0,
                    refractive_index: 1.5,
                    opacity,
                    ..Default::default()
                })
                .style(move |theme| self.style(theme, opacity)),
            )
            .align_right(Length::from(200.0))
            .align_bottom(Length::from(200.0))
            .padding(10.0)
            .into()
        } else {
            space().into()
        }
    }

    fn album_info(&self) -> Element<'_, Message> {
        let title = if self.title.len() > 40 {
            self.title[..40].to_string() + "..."
        } else {
            self.title.clone()
        };

        container(column![
            text(title)
                .size(15.0)
                .style(|theme| self.text_style(theme))
                .font(BOLD),
            row![
                text(self.year.to_string())
                    .size(12.0)
                    .style(|theme| self.text_style_gray(theme)),
                text("•")
                    .size(12.0)
                    .style(|theme| self.text_style_gray(theme)),
                text(self.artist.clone())
                    .size(12.0)
                    .style(|theme| self.text_style_gray(theme)),
            ]
            .spacing(5.0)
        ])
        .height(Length::from(40.0))
        .into()
    }

    fn mini_view(&self, playback_time: f32) -> Element<'_, Message> {
        let title = if self.title.len() > 25 {
            self.title[..25].to_string() + "..."
        } else {
            self.title.clone()
        };
        container(
            row![
                image(format!(
                    "examples/scroll_view/assets/album_covers/{}",
                    self.file.clone()
                ))
                .border_radius(10.0)
                .width(Length::from(60.0))
                .height(Length::from(60.0)),
                column![
                    text(title).size(15.0).font(BOLD),
                    row![
                        text(self.year.to_string())
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme)),
                        text("•")
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme)),
                        text(self.artist.clone())
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme))
                    ]
                    .spacing(5.0),
                    glass_slider(0.0..=1.0, playback_time, Message::SetPlaybackTime)
                        .step(0.01_f32)
                        .style(self.slider_style()),
                ],
            ]
            .spacing(15.0),
        )
        .into()
    }

    fn style(&self, _theme: &iced::Theme, opacity: f32) -> container::Style {
        container::Style {
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25 * opacity),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            border: iced::Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn text_style(&self, _theme: &iced::Theme) -> text::Style {
        text::Style {
            color: Some(Color::WHITE),
        }
    }

    fn text_style_gray(&self, _theme: &iced::Theme) -> text::Style {
        text::Style {
            color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
        }
    }

    fn slider_style(&self) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
        let color = Color::WHITE;
        let background_color = Color::from_rgb(0.3, 0.3, 0.3);
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

    fn icon_style(&self, _theme: &iced::Theme) -> svg::Style {
        svg::Style {
            color: Some(Color::WHITE),
        }
    }
}

fn button_style(_theme: &iced::Theme) -> button::Style {
    button::Style {
        background: None,
        text_color: Color::WHITE,
        border: Border::default(),
        ..Default::default()
    }
}

fn color_opacity(base: Color, opacity: f32) -> Color {
    Color::from_rgba(base.r, base.g, base.b, opacity)
}
