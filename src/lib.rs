//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use iced_glass::widget::container;
//! use iced::widget::text;
//! use iced::Element;
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container(
//!         text("Hello, world!")
//!     )
//!     .blur_radius(10.0)
//!     .saturation(0.8)
//!     .lightness(-2.0)
//!     .edge_radius(20.0)
//!     .edge_height(100.0)
//!     .refractive_index(1.5)
//!     .rim_width(2.0)
//!     .opacity(1.0)
//!     .style(|_theme| iced::widget::container::Style {
//!         // set colored glass tint
//!         background: Some(iced::Background::Color(iced::Color::from_rgb(0.8, 0.9, 1.0))),
//!         border: iced::Border {
//!             // set rounded corners, does not support individual corner radii yet
//!             radius: 10.0.into(),
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     })
//!     .into()
//! }
//! ```
pub mod font;
pub mod pipeline;
pub mod primitive;
pub mod shader;
pub mod uniforms;
pub mod widget;
