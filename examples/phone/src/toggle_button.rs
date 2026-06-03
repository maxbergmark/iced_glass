use std::collections::HashSet;

use iced::{
    Background, Border, Color, Element, Theme,
    widget::{container, mouse_area, svg},
};
use iced_glass::widget::container as glass_container;

use crate::{Message, Skin, icons, spacing};

pub struct ToggleButton {
    index: usize,
    icon: &'static str,
    size: iced::Size,
    toggled: bool,
}

impl ToggleButton {
    pub fn new(
        index: usize,
        icon: &'static str,
        size: iced::Size,
        toggled: &HashSet<usize>,
    ) -> Self {
        let toggled = toggled.contains(&index);
        Self {
            index,
            icon,
            size,
            toggled,
        }
    }

    pub fn view<'a>(self, skin: Skin) -> Element<'a, Message> {
        let w = self.size.width.min(self.size.height);
        let max_dim = self.size.width.max(self.size.height);
        let opacity = skin.opacity * f32::clamp(10.0 * max_dim / w - 12.633333, 0.0, 1.0);
        mouse_area(
            glass_container(
                svg(icons::svg_handle(self.icon))
                    .style(skin.svg_white())
                    .opacity(opacity),
            )
            .padding(0.05 * w)
            .center(spacing::n_cols(self.size, 1))
            .glass_style(move |_theme| skin.glass_style_opacity(self.index, opacity))
            .style(border_radius(w, self.toggled)),
        )
        .on_enter(Message::Hovered(self.index))
        .on_press(Message::Toggle(self.index))
        .on_exit(Message::ClearHover)
        .into()
    }
}

fn border_radius(radius: f32, toggled: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        border: Border {
            radius: radius.into(),
            ..Default::default()
        },
        background: if toggled {
            Some(Background::Color(Color::from_rgba(0.3, 0.3, 1.0, 0.6)))
        } else {
            None
        },
        ..Default::default()
    }
}
