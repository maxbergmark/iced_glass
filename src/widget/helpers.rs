use std::ops::RangeInclusive;

use iced::{
    Element, Vector,
    widget::{container, slider, text},
};

use crate::widget::{InnerContent, container::Container, slider::Slider, text::Text};

/// Creates a [`Stack`] with the given children.
///
/// [`Stack`]: crate::widget::Stack
#[macro_export]
macro_rules! glass_stack {
    () => (
        $crate::widget::Stack::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Stack::with_children([$($crate::widget::InnerContent::from($x)),+])
    );
}

/// A trait for elements that can be offset in a stack.
pub trait StackOffset<'a, Message, Theme, Renderer> {
    /// Creates a new [`InnerContent`] with the given offset.
    ///
    /// [`InnerContent`]: crate::widget::InnerContent
    fn with_offset(self, x: f32, y: f32) -> InnerContent<'a, Message, Theme, Renderer>;
}

impl<'a, T, Message, Theme, Renderer> StackOffset<'a, Message, Theme, Renderer> for T
where
    T: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn with_offset(self, x: f32, y: f32) -> InnerContent<'a, Message, Theme, Renderer> {
        InnerContent {
            container: self.into(),
            offset: Vector::new(x, y),
        }
    }
}

/// Creates a new [`Container`] with the given content.
pub fn glass_container<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: iced::advanced::Renderer,
{
    Container::new(content)
}

/// Creates a new [`Slider`] with the given range, value, and `on_change` function.
pub fn glass_slider<'a, T, Message, Theme>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Theme: slider::Catalog + 'a,
{
    Slider::new(range, value, on_change)
}

/// Creates a new [`Text`] with the given content.
pub fn glass_text<'a, Renderer, Theme>(
    content: impl text::IntoFragment<'a>,
) -> Text<'a, Renderer, Theme>
where
    Theme: text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    Text::new(content)
}
