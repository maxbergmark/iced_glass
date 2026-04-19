use iced::mouse;

use crate::{Message, primitive::Primitive};

#[derive(Default, Clone, Copy, Debug)]
pub struct Program;

impl iced::widget::shader::Program<Message> for Program {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Primitive
    }
}
