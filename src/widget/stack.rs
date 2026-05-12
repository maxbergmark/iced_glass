//! Display content on top of other content.
use iced::{
    Color, Element, Event, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
};

use crate::uniforms::ChildRaw;

/// A container that displays children on top of each other.
///
/// The first [`Element`] dictates the intrinsic [`Size`] of a [`Stack`] and
/// will be displayed as the base layer. Every consecutive [`Element`] will be
/// rendered on top; on its own layer.
///
/// You can use [`push_under`](Self::push_under) to push an [`Element`] under
/// the current [`Stack`] without affecting its intrinsic [`Size`].
///
/// Keep in mind that too much layering will normally produce bad UX as well as
/// introduce certain rendering overhead. Use this widget sparingly!
pub struct Stack<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    children: Vec<InnerContent<'a, Message, Theme, Renderer>>,
    clip: bool,
    base_layer: usize,
    glass_style: crate::StyleFn<'a, Theme>,
    corner_radius: f32,
    tint: Color,
    blending_factor: f32,
}

/// A container that displays a child element on top of the current [`Stack`].
pub struct InnerContent<'a, Message, Theme, Renderer> {
    /// The element to display on top of the [`Stack`].
    pub container: Element<'a, Message, Theme, Renderer>,
    /// The offset of the [`InnerContent`] relative to the [`Stack`].
    pub offset: Vector,
}

impl<Message, Theme, Renderer> std::fmt::Debug for Stack<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stack")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

impl<Message, Theme, Renderer> std::fmt::Debug for InnerContent<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerContent")
            .field("width", &self.container.as_widget().size_hint().width)
            .field("height", &self.container.as_widget().size_hint().height)
            .field("offset", &self.offset)
            .finish_non_exhaustive()
    }
}

struct State {
    id: u64,
}
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1000);

impl<'a, Message, Theme, Renderer> Stack<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates an empty [`Stack`].
    #[must_use]
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Creates a [`Stack`] with the given capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_vec(Vec::with_capacity(capacity))
    }

    /// Creates a [`Stack`] with the given elements.
    pub fn with_children(
        children: impl IntoIterator<Item = InnerContent<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0).extend(iterator)
    }

    /// Creates a [`Stack`] from an already allocated [`Vec`].
    ///
    /// Keep in mind that the [`Stack`] will not inspect the [`Vec`], which means
    /// it won't automatically adapt to the sizing strategy of its contents.
    ///
    /// If any of the children have a [`Length::Fill`] strategy, you will need to
    /// call [`Stack::width`] or [`Stack::height`] accordingly.
    #[must_use]
    pub fn from_vec(children: Vec<InnerContent<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            children,
            clip: false,
            base_layer: 0,
            glass_style: Box::new(|_| crate::Style::default()),
            corner_radius: 50.0,
            tint: Color::WHITE,
            blending_factor: 1.0,
        }
    }

    /// Sets the width of the [`Stack`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Stack`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Adds an element on top of the [`Stack`].
    #[must_use]
    pub fn push(mut self, child: impl Into<InnerContent<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();
        let child_size = child.container.as_widget().size_hint();

        if !child_size.is_void() {
            if self.children.is_empty() {
                self.width = self.width.enclose(child_size.width);
                self.height = self.height.enclose(child_size.height);
            }

            self.children.push(child);
        }

        self
    }

    /// Adds an element under the [`Stack`].
    #[must_use]
    pub fn push_under(
        mut self,
        child: impl Into<InnerContent<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.children.insert(0, child.into());
        self.base_layer += 1;
        self
    }

    /// Extends the [`Stack`] with the given children.
    #[must_use]
    pub fn extend(
        self,
        children: impl IntoIterator<Item = InnerContent<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Stack::push)
    }

    /// Sets whether the [`Stack`] should clip overflowing content.
    ///
    /// It has a slight performance overhead during presentation.
    ///
    /// By default, it is set to `false`.
    #[must_use]
    pub const fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the glass style of the [`Stack`].
    #[must_use]
    pub fn glass_style(mut self, style: impl Fn(&Theme) -> crate::Style + 'a) -> Self {
        self.glass_style = Box::new(style) as crate::StyleFn<'a, Theme>;
        self
    }

    /// Sets the corner radius of the [`Stack`].
    #[must_use]
    pub const fn corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    /// Sets the tint of the [`Stack`].
    #[must_use]
    pub const fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Sets the blending factor of the [`Stack`].
    #[must_use]
    pub const fn blending_factor(mut self, blending_factor: f32) -> Self {
        self.blending_factor = blending_factor;
        self
    }
}

impl<Message, Theme, Renderer> Default for Stack<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for Stack<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
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
        self.children
            .iter()
            .map(|c| Tree::new(&c.container))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(
            &self.children,
            |tree, c| tree.diff(&c.container),
            |c| Tree::new(&c.container),
        );
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    #[allow(clippy::shadow_unrelated)]
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let outer_limits = limits.width(self.width).height(self.height);
        // children get a min of zero — they're free to be any size up to the group's max.
        let child_limits = outer_limits.loose();

        if self.children.len() <= self.base_layer {
            return layout::Node::new(outer_limits.resolve(self.width, self.height, Size::ZERO));
        }

        let base_offset = self.children[self.base_layer].offset;

        let base = self.children[self.base_layer]
            .container
            .as_widget_mut()
            .layout(&mut tree.children[self.base_layer], renderer, &child_limits)
            .translate(base_offset);

        let (under, above) = self.children.split_at_mut(self.base_layer);
        let (tree_under, tree_above) = tree.children.split_at_mut(self.base_layer);

        let nodes: Vec<layout::Node> = under
            .iter_mut()
            .zip(tree_under)
            .map(|(layer, tree)| {
                let offset = layer.offset;
                layer
                    .container
                    .as_widget_mut()
                    .layout(tree, renderer, &child_limits)
                    .translate(offset)
            })
            .chain(std::iter::once(base))
            .chain(
                above[1..]
                    .iter_mut()
                    .zip(&mut tree_above[1..])
                    .map(|(layer, tree)| {
                        let offset = layer.offset;
                        layer
                            .container
                            .as_widget_mut()
                            .layout(tree, renderer, &child_limits)
                            .translate(offset)
                    }),
            )
            .collect();

        let intrinsic = nodes.iter().fold(Size::ZERO, |acc, n| {
            let b = n.bounds();
            Size::new(acc.width.max(b.x + b.width), acc.height.max(b.y + b.height))
        });

        // Parent size still respects the group's Fixed values via `outer_limits`.
        let size = outer_limits.resolve(self.width, self.height, intrinsic);

        layout::Node::with_children(size, nodes)
    }

    #[allow(clippy::shadow_unrelated)]
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .container
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    #[allow(clippy::shadow_unrelated)]
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        mut cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.children.is_empty() {
            return;
        }

        let is_over = cursor.is_over(layout.bounds());
        let end = self.children.len() - 1;

        for (i, ((child, tree), layout)) in self
            .children
            .iter_mut()
            .rev()
            .zip(tree.children.iter_mut().rev())
            .zip(layout.children().rev())
            .enumerate()
        {
            child.container.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );

            if shell.is_event_captured() {
                return;
            }

            if i < end && is_over && !cursor.is_levitating() {
                let interaction = child
                    .container
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer);

                if interaction != mouse::Interaction::None {
                    cursor = cursor.levitate();
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .rev()
            .zip(tree.children.iter().rev())
            .zip(layout.children().rev())
            .map(|((child, tree), layout)| {
                child
                    .container
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .find(|&interaction| interaction != mouse::Interaction::None)
            .unwrap_or_default()
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::shadow_unrelated)]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let glass_style = (self.glass_style)(theme);
        // info!("bounds: {:?}", bounds);

        let half_w = bounds.width * 0.5;
        let half_h = bounds.height * 0.5;
        let children: Vec<ChildRaw> = self
            .children
            .iter()
            .zip(layout.children())
            .map(|(_, child_layout)| {
                let cb = child_layout.bounds(); // absolute coords
                // child center relative to parent's top-left:
                let lx = cb.width.mul_add(0.5, cb.x) - bounds.x;
                let ly = cb.height.mul_add(0.5, cb.y) - bounds.y;
                // ...then re-center on the texture midpoint:
                ChildRaw {
                    center: [lx - half_w, ly - half_h],
                    half_size: [cb.width * 0.5, cb.height * 0.5],
                }
            })
            .collect();

        renderer.draw_primitive(
            bounds,
            crate::primitive::Primitive {
                id: state.id,
                uniforms: crate::uniforms::Uniforms {
                    blur_radius: glass_style.blur_radius,
                    // TODO: don't just use the bottom left corner radius
                    corner_radius: self.corner_radius,
                    saturation: glass_style.saturation,
                    lightness: glass_style.lightness,
                    edge_radius: glass_style.edge_radius,
                    height: glass_style.edge_height,
                    refractive_index: glass_style.refractive_index,
                    chromatic_aberration: glass_style.chromatic_aberration,
                    rim_width: glass_style.rim_width,
                    rim_angle: glass_style.rim_angle,
                    opacity: glass_style.opacity,
                    tint: self.tint,
                    content_scale: (1.0, 1.0),
                    edge_type: glass_style.edge_type,
                    num_children: children.len() as u32,
                    blending_factor: self.blending_factor,
                },
                children,
            },
        );

        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            renderer.with_layer(bounds, |renderer| {
                let viewport = if self.clip {
                    &clipped_viewport
                } else {
                    viewport
                };

                let layers_under = if cursor.is_over(layout.bounds()) {
                    self.children
                        .iter()
                        .rev()
                        .zip(tree.children.iter().rev())
                        .zip(layout.children().rev())
                        .position(|((layer, tree), layout)| {
                            let interaction = layer
                                .container
                                .as_widget()
                                .mouse_interaction(tree, layout, cursor, viewport, renderer);

                            interaction != mouse::Interaction::None
                        })
                        .map(|i| self.children.len() - i - 1)
                        .unwrap_or_default()
                } else {
                    0
                };

                let mut layers = self
                    .children
                    .iter()
                    .zip(&tree.children)
                    .zip(layout.children())
                    .enumerate();

                let layers = layers.by_ref();

                let mut draw_layer =
                    |i, layer: &Element<'a, Message, Theme, Renderer>, tree, layout, cursor| {
                        if i > 0 {
                            renderer.with_layer(*viewport, |renderer| {
                                layer
                                    .as_widget()
                                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
                            });
                        } else {
                            layer
                                .as_widget()
                                .draw(tree, renderer, theme, style, layout, cursor, viewport);
                        }
                    };

                for (i, ((layer, tree), layout)) in layers.take(layers_under) {
                    draw_layer(
                        i,
                        &layer.container,
                        tree,
                        layout,
                        mouse::Cursor::Unavailable,
                    );
                }

                for (i, ((layer, tree), layout)) in layers {
                    draw_layer(i, &layer.container, tree, layout, cursor);
                }
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
        let children = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| {
                child.container.as_widget_mut().overlay(
                    state,
                    layout,
                    renderer,
                    viewport,
                    translation,
                )
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Stack<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(stack: Stack<'a, Message, Theme, Renderer>) -> Self {
        Self::new(stack)
    }
}
