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

/// The default style function for glass effects
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Used to define styling for iced glass effects
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Sets the blur radius for the widget
    pub blur_radius: f32,
    /// Sets the saturation for the widget
    pub saturation: f32,
    /// Sets the lightness for the widget, measured in exposure steps
    pub lightness: f32,
    /// Sets the edge radius for the glass effect or smooth edge
    pub edge_radius: f32,
    /// Sets the edge height for the glass refraction
    pub edge_height: f32,
    /// Sets the refractive index for the glass refraction
    pub refractive_index: f32,
    /// Sets the rim width for the edge highlight effect
    pub rim_width: f32,
    /// Sets the opacity for the widget
    pub opacity: f32,
    /// Sets the edge type for the widget
    pub edge_type: EdgeType,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            blur_radius: 0.0,
            saturation: 1.0,
            lightness: 0.0,
            edge_radius: 0.0,
            edge_height: 0.0,
            refractive_index: 1.5,
            rim_width: 1.0,
            opacity: 1.0,
            edge_type: EdgeType::GlassEdge,
        }
    }
}

impl Style {
    /// Sets the blur radius of the widget
    #[must_use]
    pub const fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }

    /// Sets the saturation of the widget
    #[must_use]
    pub const fn saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }

    /// Sets the lightness of the widget
    #[must_use]
    pub const fn lightness(mut self, lightness: f32) -> Self {
        self.lightness = lightness;
        self
    }

    /// Sets the edge radius of the widget
    #[must_use]
    pub const fn edge_radius(mut self, edge_radius: f32) -> Self {
        self.edge_radius = edge_radius;
        self
    }

    /// Sets the edge height of the widget
    #[must_use]
    pub const fn edge_height(mut self, edge_height: f32) -> Self {
        self.edge_height = edge_height;
        self
    }

    /// Sets the refractive index of the widget
    #[must_use]
    pub const fn refractive_index(mut self, refractive_index: f32) -> Self {
        self.refractive_index = refractive_index;
        self
    }

    /// Sets the rim width of the widget
    #[must_use]
    pub const fn rim_width(mut self, rim_width: f32) -> Self {
        self.rim_width = rim_width;
        self
    }

    /// Sets the opacity of the widget
    #[must_use]
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Sets the edge type of the widget
    #[must_use]
    pub const fn edge_type(mut self, edge_type: EdgeType) -> Self {
        self.edge_type = edge_type;
        self
    }
}
