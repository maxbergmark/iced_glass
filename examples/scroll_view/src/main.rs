use std::time::Instant;

use iced::{
    Alignment, Animation, Background, Border, Color, Font, Length, Size, Task, font::Weight,
};
use itertools::Itertools;
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
                Task::none()
            }
            Message::SetCurrentAlbum(album_card) => {
                self.current_album = Some(album_card);
                Task::none()
            }
            Message::SetPlaybackTime(playback_time) => {
                self.playback_time = playback_time;
                Task::none()
            }
            Message::TogglePlayback => {
                self.playing = !self.playing;
                Task::none()
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
                Task::none()
            }
            Message::ClearHoverAlbum => {
                self.hover_info.is_hovered = false;
                self.opacity.go_mut(false, Instant::now());
                // self.hover_info.event_time = Instant::now();
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Only request frames while the animation is running
        let now = Instant::now();
        if self.opacity.is_animating(now) {
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Noop)
        } else {
            iced::Subscription::none()
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![iced::widget::stack![
            self.scroll_view(),
            self.glass(),
        ],])
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::BLACK)),
            ..Default::default()
        })
        .into()
    }

    fn scroll_view(&self) -> iced::Element<'_, Message> {
        iced::widget::scrollable(iced::widget::column![
            iced::widget::space().height(Length::from(100.0)),
            iced::widget::container(
                iced::widget::Column::with_children(
                    self.album_cards
                        .iter()
                        .enumerate()
                        .filter(|(_, album_card)| {
                            album_card
                                .title
                                .to_lowercase()
                                .contains(&self.search_value.to_lowercase())
                        })
                        // .map(|album_card| album_card.view())
                        .chunks(4)
                        .into_iter()
                        .map(|chunk| {
                            iced::widget::Row::from_vec(
                                chunk
                                    .map(|(idx, album_card)| {
                                        album_card.view(idx, self.hover_info, &self.opacity)
                                    })
                                    .collect_vec(),
                            )
                            .spacing(20.0)
                            .into()
                        }),
                )
                .spacing(20.0),
            )
            .center_x(Length::Fill),
            iced::widget::space().height(Length::from(150.0)),
        ])
        .height(Length::Fill)
        .height(Length::Fill)
        .spacing(20.0)
        .into()
    }

    fn glass(&self) -> iced::Element<'_, Message> {
        iced::widget::container(iced::widget::column![
            // iced::widget::space().height(Length::from(20.0)),
            self.search_bar(),
            iced::widget::space().height(Length::Fill),
            self.playback_bar(),
        ])
        .center_x(Length::Fill)
        .align_top(Length::Fill)
        .padding(30.0)
        .into()
    }

    fn search_bar(&self) -> iced::Element<'_, Message> {
        iced_glass::widget::container(
            iced::widget::row![
                iced::widget::svg("assets/search.svg")
                    .width(Length::from(30.0))
                    .height(Length::from(30.0))
                    .style(|theme, _status| self.icon_style(theme)),
                // iced::widget::text("Album Search").size(20.0),
                iced::widget::text_input("Search", self.search_value.as_str())
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
        .blur_radius(50.0)
        // .saturation(1.0)
        .lightness(-1.5)
        .edge_radius(10.0)
        .edge_height(100.0)
        .refractive_index(1.5)
        .style(|theme| self.style(theme))
        .into()
    }

    fn playback_bar(&self) -> iced::Element<'_, Message> {
        let play_icon = if self.playing {
            "examples/scroll_view/assets/pause.svg"
        } else {
            "examples/scroll_view/assets/play.svg"
        };
        iced_glass::widget::container(
            iced::widget::button(
                iced::widget::container(
                    iced::widget::row![
                        iced::widget::row![
                            iced::widget::svg("examples/scroll_view/assets/back.svg")
                                .width(Length::from(20.0))
                                .height(Length::from(20.0))
                                .style(|theme, _status| self.icon_style(theme)),
                            iced::widget::button(
                                iced::widget::svg(play_icon)
                                    .width(Length::from(30.0))
                                    .height(Length::from(30.0))
                                    .style(|theme, _status| self.icon_style(theme))
                            )
                            .on_press(Message::TogglePlayback)
                            .style(|theme, _status| button_style(theme)),
                            iced::widget::svg("examples/scroll_view/assets/forward.svg")
                                .width(Length::from(20.0))
                                .height(Length::from(20.0))
                                .style(|theme, _status| self.icon_style(theme))
                        ]
                        .spacing(15.0)
                        .align_y(Alignment::Center),
                        self.current_album
                            .as_ref()
                            .map(|album_card| album_card.mini_view(self.playback_time))
                            .unwrap_or_else(|| iced::widget::text("No album selected")
                                .size(20.0)
                                .into()),
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
        .blur_radius(50.0)
        // .saturation(1.0)
        .edge_radius(10.0)
        .edge_height(100.0)
        .refractive_index(1.5)
        .rim_width(2.0)
        .lightness(-1.5)
        .style(|theme| self.style(theme))
        .into()
    }

    fn style(&self, _theme: &iced::Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
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

    fn input_style(&self, _theme: &iced::Theme) -> iced::widget::text_input::Style {
        iced::widget::text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            icon: Color::WHITE,
            placeholder: Color::from_rgb(0.8, 0.8, 0.8),
            value: Color::WHITE,
            selection: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border::default(),
        }
    }

    fn icon_style(&self, _theme: &iced::Theme) -> iced::widget::svg::Style {
        iced::widget::svg::Style {
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
    ) -> iced::Element<'_, Message> {
        let title = if self.title.len() > 40 {
            self.title[..40].to_string() + "..."
        } else {
            self.title.clone()
        };

        let is_hovered = hover_info.index == idx;
        let overlay: iced::Element<'_, Message> = if is_hovered {
            let opacity = opacity.interpolate(0.0, 1.0, Instant::now());

            iced::widget::container(
                iced_glass::widget::container(
                    iced::widget::svg("examples/scroll_view/assets/play.svg")
                        .width(Length::from(15.0))
                        .height(Length::from(15.0))
                        .style(|theme, _status| self.icon_style(theme))
                        .opacity(opacity),
                )
                .center(Length::from(40.0))
                .blur_radius(25.0)
                .edge_radius(10.0)
                .edge_height(30.0)
                .refractive_index(1.5)
                .lightness(-1.5)
                .opacity(opacity)
                .style(move |theme| self.style(theme, opacity)),
            )
            .align_right(Length::from(200.0))
            .align_bottom(Length::from(200.0))
            .padding(10.0)
            .into()
        } else {
            iced::widget::space().into()
        };

        iced::widget::container(
            iced::widget::mouse_area(iced::widget::column![
                iced::widget::stack![
                    iced::widget::image(format!(
                        "examples/scroll_view/assets/album_covers/{}",
                        self.file.clone()
                    ))
                    .width(200.0)
                    .height(200.0),
                    overlay,
                ],
                iced::widget::container(iced::widget::column![
                    iced::widget::text(title)
                        .size(15.0)
                        .style(|theme| self.text_style(theme))
                        .font(BOLD),
                    iced::widget::row![
                        iced::widget::text(self.year.to_string())
                            .size(12.0)
                            .style(|theme| self.text_style_gray(theme)),
                        iced::widget::text("•")
                            .size(12.0)
                            .style(|theme| self.text_style_gray(theme)),
                        iced::widget::text(self.artist.clone())
                            .size(12.0)
                            .style(|theme| self.text_style_gray(theme)),
                    ]
                    .spacing(5.0)
                ])
                .height(Length::from(40.0)),
            ])
            .on_press(Message::SetCurrentAlbum(self.clone()))
            .on_enter(Message::SetHoverAlbum(idx))
            .on_exit(Message::ClearHoverAlbum),
        )
        .width(Length::from(200.0))
        .center_y(Length::from(250.0))
        // .blur_radius(50.0)
        // .saturation(1.0)
        // .lightness(-0.5)
        .style(|theme| self.style(theme, 1.0))
        .into()
    }

    fn mini_view(&self, playback_time: f32) -> iced::Element<'_, Message> {
        let title = if self.title.len() > 25 {
            self.title[..25].to_string() + "..."
        } else {
            self.title.clone()
        };
        iced::widget::container(
            iced::widget::row![
                iced::widget::image(format!(
                    "examples/scroll_view/assets/album_covers/{}",
                    self.file.clone()
                ))
                .width(Length::from(60.0))
                .height(Length::from(60.0)),
                iced::widget::column![
                    iced::widget::text(title).size(15.0).font(BOLD),
                    iced::widget::row![
                        iced::widget::text(self.year.to_string())
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme)),
                        iced::widget::text("•")
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme)),
                        iced::widget::text(self.artist.clone())
                            .size(15.0)
                            .style(|theme| self.text_style_gray(theme))
                    ]
                    .spacing(5.0),
                    iced_glass::widget::slider(0.0..=1.0, playback_time, Message::SetPlaybackTime)
                        .step(0.01)
                        .style(|theme, status| self.slider_style(theme, status))
                        .edge_radius(5.0)
                        .edge_height(10.0)
                        .refractive_index(1.5),
                ],
            ]
            .spacing(10.0),
        )
        .into()
    }

    fn style(&self, _theme: &iced::Theme, opacity: f32) -> iced::widget::container::Style {
        iced::widget::container::Style {
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

    fn text_style(&self, _theme: &iced::Theme) -> iced::widget::text::Style {
        iced::widget::text::Style {
            color: Some(Color::WHITE),
        }
    }

    fn text_style_gray(&self, _theme: &iced::Theme) -> iced::widget::text::Style {
        iced::widget::text::Style {
            color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
        }
    }

    fn slider_style(
        &self,
        _theme: &iced::Theme,
        _status: iced::widget::slider::Status,
    ) -> iced::widget::slider::Style {
        let fill_color = iced::Color::from_rgba(0.3, 0.3, 1.0, 1.0);
        iced::widget::slider::Style {
            rail: iced::widget::slider::Rail {
                backgrounds: (
                    iced::Background::Color(fill_color),
                    iced::Background::Color(iced::Color::WHITE),
                ),
                width: 5.0,
                border: iced::Border {
                    radius: 100.0.into(),
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

    fn icon_style(&self, _theme: &iced::Theme) -> iced::widget::svg::Style {
        iced::widget::svg::Style {
            color: Some(Color::WHITE),
        }
    }
}

fn button_style(_theme: &iced::Theme) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: None,
        text_color: Color::WHITE,
        border: Border::default(),
        ..Default::default()
    }
}
