use iced::{
    Alignment, Border, Element, Font, Length, Theme,
    font::{self, Family, Stretch, Weight},
    widget::{column, container, image, mouse_area, row, svg, text},
};
use iced_glass::widget::container as glass_container;
use once_cell::sync::Lazy;

use crate::{Message, Skin, icons, spacing};

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

const ALBUM: &[u8] = include_bytes!("../assets/album_cover.jpg");
static ALBUM_HANDLE: Lazy<image::Handle> = Lazy::new(|| image::Handle::from_bytes(ALBUM));

pub struct MusicView {
    index: usize,
    // handle: &'a image::Handle,
    size: iced::Size,
}

impl MusicView {
    pub fn new(index: usize, size: iced::Size) -> Self {
        Self {
            index,
            // handle,
            size,
        }
    }

    pub fn view(self, skin: Skin) -> Element<'static, Message> {
        let w = self.size.width.min(self.size.height);
        mouse_area(
            glass_container(
                column![
                    image(ALBUM_HANDLE.clone())
                        .border_radius(0.04 * w)
                        .opacity(skin.opacity)
                        .width(0.17 * w),
                    column![
                        text("Deep Meridian")
                            .width(Length::Fill)
                            .size(0.033 * w)
                            .style(skin.text_white())
                            .font(FONT_BOLD),
                        text("Terra Pulse - 2021")
                            .size(0.028 * w)
                            .width(Length::Fill)
                            .style(skin.text_white())
                            .font(FONT_NORMAL)
                    ],
                    row![
                        svg(icons::svg_handle("play-back"))
                            .style(skin.svg_white())
                            .opacity(skin.opacity),
                        svg(icons::svg_handle("play"))
                            .style(skin.svg_white())
                            .opacity(skin.opacity),
                        svg(icons::svg_handle("play-forward"))
                            .style(skin.svg_white())
                            .opacity(skin.opacity)
                    ]
                ]
                .align_x(Alignment::Center)
                .spacing(0.028 * w),
            )
            .padding(0.04 * w)
            .center_y(spacing::n_rows(self.size, 2))
            .width(spacing::n_cols(self.size, 2))
            .glass_style(move |_theme| skin.glass_style(self.index))
            .style(border_radius(spacing::n_rows(self.size, 1) * 0.5)),
        )
        .on_enter(Message::Hovered(self.index))
        .on_exit(Message::ClearHover)
        .into()
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
