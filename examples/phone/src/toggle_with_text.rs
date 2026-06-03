use std::collections::HashSet;

use iced::{
    Alignment, Background, Border, Color, Element, Font, Theme,
    font::{self, Family, Stretch, Weight},
    widget::{column, container, mouse_area, row, svg, text},
};
use iced_glass::widget::container as glass_container;

use crate::{
    Message, Skin, icons,
    spacing::{n_cols, n_rows},
};

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

pub struct ToggleWithText {
    index: usize,
    icon: &'static str,
    top_text: &'static str,
    off_text: &'static str,
    on_text: &'static str,
    size: iced::Size,
    toggled: bool,
}

impl ToggleWithText {
    pub fn new(
        index: usize,
        icon: &'static str,
        top_text: &'static str,
        off_text: &'static str,
        on_text: &'static str,
        size: iced::Size,
        toggled: &HashSet<usize>,
    ) -> Self {
        let toggled = toggled.contains(&index);
        Self {
            index,
            icon,
            top_text,
            off_text,
            on_text,
            size,
            toggled,
        }
    }

    pub fn view<'a>(self, skin: Skin) -> Element<'a, Message> {
        let w = self.size.width.min(self.size.height);
        mouse_area(
            glass_container(
                row![
                    container(
                        svg(icons::svg_handle(self.icon))
                            .style(icon_toggled(self.toggled))
                            .opacity(skin.opacity)
                    )
                    .center(0.1 * w)
                    .padding(0.02 * w)
                    .style(border_radius_toggled(
                        w,
                        self.toggled,
                        skin.opacity
                    )),
                    column![
                        text(self.top_text)
                            .size(0.033 * w)
                            .wrapping(text::Wrapping::None)
                            .style(skin.text_white())
                            .font(FONT_BOLD),
                        text(if self.toggled {
                            self.on_text
                        } else {
                            self.off_text
                        })
                        .size(0.03 * w)
                        .style(skin.text_white())
                        .font(FONT_NORMAL)
                    ]
                ]
                .align_y(Alignment::Center)
                .spacing(0.03 * w),
            )
            .padding(0.04 * w)
            .glass_style(move |_theme| skin.glass_style(self.index))
            .center_y(n_rows(self.size, 1))
            .width(n_cols(self.size, 2))
            .style(border_radius(w)),
        )
        .on_enter(Message::Hovered(self.index))
        .on_press(Message::Toggle(self.index))
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

fn border_radius_toggled(
    radius: f32,
    toggled: bool,
    opacity: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        border: Border {
            radius: radius.into(),
            ..Default::default()
        },
        background: if toggled {
            Some(Background::Color(Color::from_rgba(0.3, 0.3, 1.0, opacity)))
        } else {
            Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, opacity)))
        },
        ..Default::default()
    }
}

fn icon_toggled(toggled: bool) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |_theme, _status| svg::Style {
        color: if toggled {
            Some(Color::WHITE)
        } else {
            Some(Color::from_rgba(0.3, 0.3, 1.0, 1.0))
        },
    }
}
