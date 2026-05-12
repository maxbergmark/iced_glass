use iced::{
    Alignment, Background, Color, Element, Event, Length, Padding, Pixels, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    alignment,
    widget::{
        self,
        container::{self, Catalog, Style, StyleFn},
    },
};

/// A container widget with a glass effect.
#[must_use]
pub struct Container<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
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
    content: Element<'a, Message, Theme, Renderer>,
    class: Theme::Class<'a>,
    glass_style: crate::StyleFn<'a, Theme>,
}

impl<Message, Theme, Renderer> std::fmt::Debug for Container<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// Creates a new [`Container`] with the given content.
pub fn glass_container<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: iced::advanced::Renderer,
{
    Container::new(content)
}

impl<'a, Message, Theme, Renderer> Container<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    /// Creates a [`Container`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Container {
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
            glass_style: Box::new(|_| crate::Style::default()),
            content,
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
    pub const fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Container`].
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the glass style of the [`Container`].
    pub fn glass_style(mut self, style: impl Fn(&Theme) -> crate::Style + 'a) -> Self {
        self.glass_style = Box::new(style) as crate::StyleFn<'a, Theme>;
        self
    }
}

struct State {
    id: u64,
}
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<Message, Theme, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for Container<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer,
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

        #[allow(clippy::expect_used)]
        operation.traverse(&mut |op| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().expect("No child found"),
                renderer,
                op,
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
        #[allow(clippy::expect_used)]
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().expect("No child found"),
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
        #[allow(clippy::expect_used)]
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().expect("No child found"),
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
        let glass_style = (self.glass_style)(theme);
        let tint = style.background.map_or_else(
            || Color::WHITE,
            |background| match background {
                Background::Color(color) => color,
                Background::Gradient(_) => Color::WHITE,
            },
        );

        renderer.draw_primitive(
            bounds,
            crate::primitive::Primitive {
                id: state.id,
                uniforms: crate::uniforms::Uniforms {
                    blur_radius: glass_style.blur_radius,
                    // TODO: don't just use the bottom left corner radius
                    corner_radius: style.border.radius.bottom_left,
                    saturation: glass_style.saturation,
                    lightness: glass_style.lightness,
                    edge_radius: glass_style.edge_radius,
                    height: glass_style.edge_height,
                    refractive_index: glass_style.refractive_index,
                    chromatic_aberration: glass_style.chromatic_aberration,
                    rim_width: glass_style.rim_width,
                    rim_angle: glass_style.rim_angle,
                    opacity: glass_style.opacity,
                    tint,
                    content_scale: (1.0, 1.0),
                    edge_type: glass_style.edge_type,
                    num_children: 0,
                    blending_factor: 1.0,
                },
                children: vec![],
            },
        );

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            #[allow(clippy::expect_used)]
            renderer.with_layer(bounds, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    &renderer::Style {
                        text_color: style.text_color.unwrap_or(renderer_style.text_color),
                    },
                    layout.children().next().expect("No child found"),
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
        #[allow(clippy::expect_used)]
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().expect("No child found"),
            renderer,
            viewport,
            translation,
        )
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }
}

impl<'a, Message, Theme, Renderer> From<Container<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(container: Container<'a, Message, Theme, Renderer>) -> Self {
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
