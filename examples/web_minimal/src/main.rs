use iced::{
    Color, Element, Task,
    widget::{container, space, text},
};

#[derive(Debug, Default, Clone)]
pub struct Ui {}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Noop,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Warn);
    }

    iced::application(Ui::boot, Ui::update, Ui::view).run()
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Noop => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(text("Hello, world!").color(Color::WHITE).size(100.0))
            // container(space().width(100.0).height(100.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.7))),
                ..Default::default()
            })
            .into()
    }
}
