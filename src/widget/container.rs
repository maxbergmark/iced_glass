use iced::{
    Alignment, Background, Color, Element, Event, Length, Padding, Pixels, Rectangle, Renderer,
    Size, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    alignment,
    widget::{
        self,
        container::{self, Catalog, Style, StyleFn},
    },
};
use iced_wgpu::primitive::Renderer as _;

pub struct GlassContainer<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
    // Renderer: iced::advanced::Renderer,
{
    id: Option<iced::widget::Id>,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: iced::alignment::Horizontal,
    vertical_alignment: iced::alignment::Vertical,
    clip: bool,
    content: Element<'a, Message, Theme>,
    class: Theme::Class<'a>,

    // GlassContainer specific properties
    blur_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
    rim_width: f32,
}

pub fn glass_container<'a, Message, Theme>(
    content: impl Into<Element<'a, Message, Theme>>,
) -> GlassContainer<'a, Message, Theme>
where
    Theme: container::Catalog + 'a,
    // Renderer: ::Renderer,
{
    GlassContainer::new(content)
}

impl<'a, Message, Theme> GlassContainer<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// Creates a [`Container`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        GlassContainer {
            id: None,
            padding: Padding::ZERO,
            width: size.width.fluid(),
            height: size.height.fluid(),
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            clip: false,
            class: Theme::default(),
            content,

            blur_radius: 0.0,
            saturation: 1.0,
            lightness: 0.0,
            edge_radius: 0.0,
            edge_height: 0.0,
            refractive_index: 1.5,
            rim_width: 1.0,
        }
    }

    /// Sets the [`widget::Id`] of the [`Container`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`Container`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets the width of the [`Container`] and centers its contents horizontally.
    pub fn center_x(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Center)
    }

    /// Sets the height of the [`Container`] and centers its contents vertically.
    pub fn center_y(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Center)
    }

    /// Sets the width and height of the [`Container`] and centers its contents in
    /// both the horizontal and vertical axes.
    ///
    /// This is equivalent to chaining [`center_x`] and [`center_y`].
    ///
    /// [`center_x`]: Self::center_x
    /// [`center_y`]: Self::center_y
    pub fn center(self, length: impl Into<Length>) -> Self {
        let length = length.into();

        self.center_x(length).center_y(length)
    }

    /// Sets the width of the [`Container`] and aligns its contents to the left.
    pub fn align_left(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Left)
    }

    /// Sets the width of the [`Container`] and aligns its contents to the right.
    pub fn align_right(self, width: impl Into<Length>) -> Self {
        self.width(width).align_x(alignment::Horizontal::Right)
    }

    /// Sets the height of the [`Container`] and aligns its contents to the top.
    pub fn align_top(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Top)
    }

    /// Sets the height of the [`Container`] and aligns its contents to the bottom.
    pub fn align_bottom(self, height: impl Into<Length>) -> Self {
        self.height(height).align_y(alignment::Vertical::Bottom)
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.horizontal_alignment = alignment.into();
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.vertical_alignment = alignment.into();
        self
    }

    /// Sets whether the contents of the [`Container`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Container`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Container`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }

    pub fn saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }

    pub fn lightness(mut self, lightness: f32) -> Self {
        self.lightness = lightness;
        self
    }

    pub fn edge_radius(mut self, edge_radius: f32) -> Self {
        self.edge_radius = edge_radius;
        self
    }

    pub fn edge_height(mut self, edge_height: f32) -> Self {
        self.edge_height = edge_height;
        self
    }

    pub fn refractive_index(mut self, refractive_index: f32) -> Self {
        self.refractive_index = refractive_index;
        self
    }

    pub fn rim_width(mut self, rim_width: f32) -> Self {
        self.rim_width = rim_width;
        self
    }
}

struct State {
    id: u64,
}
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<Message, Theme> iced::advanced::Widget<Message, Theme, Renderer>
    for GlassContainer<'_, Message, Theme>
where
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        // self.content.as_widget().tag()
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        // self.content.as_widget().state()
        tree::State::new(State {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        })
    }

    fn children(&self) -> Vec<Tree> {
        // self.content.as_widget().children()
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        // self.content.as_widget().diff(tree);
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| {
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(self.id.as_ref(), layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let style = theme.style(&self.class);

        renderer.draw_primitive(
            bounds,
            crate::primitive::Primitive {
                id: state.id,
                uniforms: crate::uniforms::Uniforms {
                    blur_radius: self.blur_radius,
                    // TODO: don't just use the bottom left corner radius
                    corner_radius: style.border.radius.bottom_left,
                    saturation: self.saturation,
                    lightness: self.lightness,
                    edge_radius: self.edge_radius,
                    height: self.edge_height,
                    refractive_index: self.refractive_index,
                    rim_width: self.rim_width,
                },
            },
        );

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            draw_background(renderer, &style, bounds);
            renderer.with_layer(bounds, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    &renderer::Style {
                        text_color: style.text_color.unwrap_or(renderer_style.text_color),
                    },
                    layout.children().next().unwrap(),
                    cursor,
                    if self.clip {
                        &clipped_viewport
                    } else {
                        viewport
                    },
                );
            });
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }
}

impl<'a, Message, Theme> From<GlassContainer<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a,
    Theme: Catalog + 'a,
{
    fn from(container: GlassContainer<'a, Message, Theme>) -> Element<'a, Message, Theme> {
        Element::new(container)
    }
}

#[allow(clippy::too_many_arguments)]
/// Computes the layout of a [`Container`].
pub fn layout(
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    layout_content: impl FnOnce(&layout::Limits) -> layout::Node,
) -> layout::Node {
    layout::positioned(
        &limits.max_width(max_width).max_height(max_height),
        width,
        height,
        padding,
        |limits| layout_content(&limits.loose()),
        |content, size| {
            content.align(
                Alignment::from(horizontal_alignment),
                Alignment::from(vertical_alignment),
                size,
            )
        },
    )
}

/// Draws the background of a [`Container`] given its [`Style`] and its `bounds`.
pub fn draw_background<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle)
where
    Renderer: iced::advanced::Renderer,
{
    if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                shadow: style.shadow,
                snap: style.snap,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}
