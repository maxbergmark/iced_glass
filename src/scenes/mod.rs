use iced::{Alignment, Length, Task};

pub mod basic;
mod declaration;
pub mod large_slider;
pub mod scroll_view;
pub mod stress_test;
pub mod text;

#[derive(Debug, Clone)]
pub enum Message {
    SetScene(Scene),
    Basic(basic::Message),
    ScrollView(scroll_view::Message),
    LargeSlider(large_slider::Message),
    StressTest(stress_test::Message),
    Text(text::Message),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Scene {
    Basic(basic::Ui),
    ScrollView(scroll_view::Ui),
    LargeSlider(large_slider::Ui),
    StressTest(stress_test::Ui),
    Text(text::Ui),
}

impl Default for Scene {
    fn default() -> Self {
        // Self::Basic(basic::Ui::default())
        // Self::ScrollView(scroll_view::Ui::default())
        // Self::LargeSlider(large_slider::Ui::default())
        // Self::StressTest(stress_test::Ui::default())
        Self::Text(text::Ui::default())
    }
}

#[derive(Default)]
pub struct Ui {
    scene: Scene,
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
            Message::StressTest(message) => {
                if let Scene::StressTest(ui) = &mut self.scene {
                    ui.update(message).map(Message::StressTest)
                } else {
                    Task::none()
                }
            }
            Message::Text(message) => {
                if let Scene::Text(ui) = &mut self.scene {
                    ui.update(message).map(Message::Text)
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.scene_subscription()
    }

    fn scene_subscription(&self) -> iced::Subscription<Message> {
        match &self.scene {
            // Scene::Basic(ui) => ui.subscription().map(Message::Basic),
            Scene::ScrollView(ui) => ui.subscription().map(Message::ScrollView),
            // Scene::LargeSlider(ui) => ui.subscription().map(Message::LargeSlider),
            Scene::StressTest(ui) => ui.subscription().map(Message::StressTest),
            _ => iced::Subscription::none(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let scene = match &self.scene {
            Scene::Basic(ui) => ui.view().map(Message::Basic),
            Scene::ScrollView(ui) => ui.view().map(Message::ScrollView),
            Scene::LargeSlider(ui) => ui.view().map(Message::LargeSlider),
            Scene::StressTest(ui) => ui.view().map(Message::StressTest),
            Scene::Text(ui) => ui.view().map(Message::Text),
        };
        iced::widget::column![self.scene_selector(), scene]
            .align_x(Alignment::Center)
            .into()
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
            iced::widget::button("StressTest").on_press(Message::SetScene(Scene::StressTest(
                stress_test::Ui::default()
            ))),
            iced::widget::button("Text")
                .on_press(Message::SetScene(Scene::Text(text::Ui::default()))),
        ]
        .spacing(10.0)
        .padding(10.0)
        .height(Length::from(60.0))
        .into()
    }
}
