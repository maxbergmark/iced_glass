mod container;
mod slider;
/// Public text widget
pub mod text;

pub use container::glass_container as container;
pub use slider::glass_slider as slider;
pub use text::glass_text as text;

/// Selects whether to use glass refraction or smooth blending for the edge
#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    /// Use glass refraction for styling the edge
    GlassEdge,
    /// Smoothly blend opacity between 0 and 1 over edge radius
    SoftEdge,
}
