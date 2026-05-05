#![warn(
    missing_docs,
    // unreachable_pub,
    keyword_idents,
    unexpected_cfgs,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    nonstandard_style,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use iced_glass::widget::container;
//! use iced_glass::widget::EdgeType;
//! use iced::widget::text;
//! use iced::Element;
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container(
//!         text("Hello, world!")
//!     )
//!     .glass_style(|_theme| iced_glass::Style {
//!         blur_radius: 10.0,
//!         saturation: 0.8,
//!         lightness: -2.0,
//!         edge_radius: 20.0,
//!         edge_height: 100.0,
//!         refractive_index: 1.5,
//!         chromatic_aberration: 0.1,
//!         rim_width: 2.0,
//!         opacity: 1.0,
//!         edge_type: EdgeType::GlassEdge,
//!     })
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

pub(crate) mod pipeline;
pub(crate) mod primitive;
pub(crate) mod shader;
pub(crate) mod uniforms;
/// Public widgets
pub mod widget;

pub use widget::{Style, StyleFn};
