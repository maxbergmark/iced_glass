use iced::{Length, Task, window};

pub mod basic;
pub mod large_slider;
pub mod scroll_view;

#[derive(Debug, Clone)]
pub enum Message {
    SetScene(Scene),
    Basic(basic::Message),
    ScrollView(scroll_view::Message),
    LargeSlider(large_slider::Message),
    WindowResized(iced::Size),
}

#[derive(Debug, Clone)]
pub enum Scene {
    Basic(basic::Ui),
    ScrollView(scroll_view::Ui),
    LargeSlider(large_slider::Ui),
}

impl Default for Scene {
    fn default() -> Self {
        Self::Basic(basic::Ui::default())
        // Self::ScrollView(scroll_view::Ui::default())
        // Self::LargeSlider(large_slider::Ui::default())
    }
}

#[derive(Default)]
pub struct Ui {
    scene: Scene,
    window_size: iced::Size,
}

impl Ui {
    pub fn boot() -> (Ui, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetScene(scene) => {
                self.scene = scene;
                Task::none()
            }
            Message::Basic(message) => {
                if let Scene::Basic(ui) = &mut self.scene {
                    ui.update(message).map(Message::Basic)
                } else {
                    Task::none()
                }
            }
            Message::ScrollView(message) => {
                if let Scene::ScrollView(ui) = &mut self.scene {
                    ui.update(message).map(Message::ScrollView)
                } else {
                    Task::none()
                }
            }
            Message::LargeSlider(message) => {
                if let Scene::LargeSlider(ui) = &mut self.scene {
                    ui.update(message).map(Message::LargeSlider)
                } else {
                    Task::none()
                }
            }
            Message::WindowResized(size) => {
                self.window_size = size;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        window::resize_events().map(|(_id, size)| Message::WindowResized(size))
    }

    // pub fn subscription(&self) -> iced::Subscription<Message> {
    //     match &self.scene {
    //         Scene::Basic(ui) => ui.subscription().map(Message::Basic),
    //         Scene::ScrollView(_) => iced::Subscription::none(),
    //         Scene::LargeSlider(_) => iced::Subscription::none(),
    //     }
    // }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let scene = match &self.scene {
            Scene::Basic(ui) => ui.view(self.window_size).map(Message::Basic),
            Scene::ScrollView(ui) => ui.view().map(Message::ScrollView),
            Scene::LargeSlider(ui) => ui.view().map(Message::LargeSlider),
        };
        iced::widget::column![self.scene_selector(), scene].into()
    }

    fn scene_selector(&self) -> iced::Element<'_, Message> {
        iced::widget::row![
            iced::widget::button("Basic")
                .on_press(Message::SetScene(Scene::Basic(basic::Ui::default()))),
            iced::widget::button("ScrollView").on_press(Message::SetScene(Scene::ScrollView(
                scroll_view::Ui::default()
            ))),
            iced::widget::button("LargeSlider").on_press(Message::SetScene(Scene::LargeSlider(
                large_slider::Ui::default()
            ))),
        ]
        .spacing(10.0)
        .padding(10.0)
        .height(Length::from(60.0))
        .into()
    }
}
