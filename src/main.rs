use iced::{Size, application};

mod scenes;
use scenes::Ui;

#[tokio::main]
async fn main() -> iced::Result {
    application(Ui::boot, Ui::update, Ui::view)
        .subscription(Ui::subscription)
        .antialiasing(true)
        .window_size(Size::new(2560.0, 1440.0))
        .run()
}
