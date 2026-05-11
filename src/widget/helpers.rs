use iced::{Element, Vector};

use crate::widget::InnerContent;

/// Creates a [`GlassStack`] with the given children.
///
/// [`GlassStack`]: crate::GlassStack
#[macro_export]
macro_rules! glass_stack {
    () => (
        $crate::GlassStack::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::GlassStack::with_children([$($crate::widget::InnerContent::from($x)),+])
    );
}

/// A trait for elements that can be offset in a stack.
pub trait StackOffset<'a, Message, Theme, Renderer> {
    /// Creates a new [`InnerContent`] with the given offset.
    ///
    /// [`InnerContent`]: crate::InnerContent
    fn with_offset(self, offset: Vector) -> InnerContent<'a, Message, Theme, Renderer>;
}

impl<'a, T, Message, Theme, Renderer> StackOffset<'a, Message, Theme, Renderer> for T
where
    T: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn with_offset(self, offset: Vector) -> InnerContent<'a, Message, Theme, Renderer> {
        InnerContent {
            container: self.into(),
            offset,
        }
    }
}
