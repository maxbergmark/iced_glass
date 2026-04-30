use std::ops::RangeInclusive;

use iced::{
    Border, Color, Element, Event, Length, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{
            Tree,
            tree::{self},
        },
    },
    keyboard::{self, Key, key},
    touch,
    widget::slider::{self, Catalog, HandleShape, Status, Style, StyleFn},
    window,
};

use iced::advanced::Renderer as _;
use iced_wgpu::primitive::Renderer as _;

pub fn glass_slider<'a, T, Message, Theme>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> GlassSlider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Theme: slider::Catalog + 'a,
{
    GlassSlider::new(range, value, on_change)
}

#[must_use]
pub struct GlassSlider<'a, T, Message, Theme = iced::Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    on_release: Option<Message>,
    width: Length,
    height: f32,
    class: Theme::Class<'a>,
    status: Option<Status>,

    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
}

impl<T, Message, Theme> std::fmt::Debug for GlassSlider<'_, T, Message, Theme>
where
    T: std::fmt::Debug,
    Theme: Catalog,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlassSlider")
            .field("range", &self.range)
            .field("value", &self.value)
            .field("width", &self.width)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    id: u64,
    is_dragging: bool,
    keyboard_modifiers: keyboard::Modifiers,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_000_000);

impl<'a, T, Message, Theme> GlassSlider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
    Theme: Catalog,
{
    /// The default height of a [`Slider`].
    pub const DEFAULT_HEIGHT: f32 = 16.0;

    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///     It receives the new value of the [`Slider`] and must produce a
    ///     `Message`.
    pub fn new<F>(range: RangeInclusive<T>, value: T, on_change: F) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        GlassSlider {
            value,
            default: None,
            range,
            step: T::from(1),
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            height: Self::DEFAULT_HEIGHT,
            class: Theme::default(),
            status: None,

            edge_radius: 0.0,
            edge_height: 0.0,
            refractive_index: 1.5,
        }
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default [`on_change`] message could create too many events.
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Slider`].
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into().0;
        self
    }

    /// Sets the step size of the [`Slider`].
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the style of the [`Slider`].
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    // /// Sets the style class of the [`Slider`].
    // #[cfg(feature = "advanced")]
    // #[must_use]
    // pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
    //     self.class = class.into();
    //     self
    // }

    pub const fn edge_radius(mut self, edge_radius: f32) -> Self {
        self.edge_radius = edge_radius;
        self
    }

    pub const fn edge_height(mut self, edge_height: f32) -> Self {
        self.edge_height = edge_height;
        self
    }

    pub const fn refractive_index(mut self, refractive_index: f32) -> Self {
        self.refractive_index = refractive_index;
        self
    }
}

impl<T, Message, Theme> Widget<Message, Theme, Renderer> for GlassSlider<'_, T, Message, Theme>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Theme: Catalog,
    // Renderer: iced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        // tree::State::new(State::default())
        tree::State::new(State {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            is_dragging: false,
            keyboard_modifiers: keyboard::Modifiers::default(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    #[allow(clippy::too_many_lines)]
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let mut update = || {
            let current_value = self.value;

            let locate = |cursor_position: Point| -> Option<T> {
                let bounds = layout.bounds();

                if cursor_position.x <= bounds.x {
                    Some(*self.range.start())
                } else if cursor_position.x >= bounds.x + bounds.width {
                    Some(*self.range.end())
                } else {
                    let step = if state.keyboard_modifiers.shift() {
                        self.shift_step.unwrap_or(self.step)
                    } else {
                        self.step
                    }
                    .into();

                    let start = (*self.range.start()).into();
                    let end = (*self.range.end()).into();

                    let percent = f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width);

                    let steps = (percent * (end - start) / step).round();
                    let value = steps.mul_add(step, start);

                    T::from_f64(value.min(end))
                }
            };

            let increment = |value: T| -> Option<T> {
                let step = if state.keyboard_modifiers.shift() {
                    self.shift_step.unwrap_or(self.step)
                } else {
                    self.step
                }
                .into();

                let steps = (value.into() / step).round();
                let new_value = step * (steps + 1.0);

                if new_value > (*self.range.end()).into() {
                    return Some(*self.range.end());
                }

                T::from_f64(new_value)
            };

            let decrement = |value: T| -> Option<T> {
                let step = if state.keyboard_modifiers.shift() {
                    self.shift_step.unwrap_or(self.step)
                } else {
                    self.step
                }
                .into();

                let steps = (value.into() / step).round();
                let new_value = step * (steps - 1.0);

                if new_value < (*self.range.start()).into() {
                    return Some(*self.range.start());
                }

                T::from_f64(new_value)
            };

            let change = |new_value: T| {
                if (self.value.into() - new_value.into()).abs() > f64::EPSILON {
                    shell.publish((self.on_change)(new_value));

                    self.value = new_value;
                }
            };

            match &event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) = cursor.position_over(layout.bounds()) {
                        if state.keyboard_modifiers.command() {
                            let _ = self.default.map(change);
                            state.is_dragging = false;
                        } else {
                            let _ = locate(cursor_position).map(change);
                            state.is_dragging = true;
                        }

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. },
                ) => {
                    if state.is_dragging {
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        state.is_dragging = false;
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if state.is_dragging {
                        let _ = cursor.land().position().and_then(locate).map(change);

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if state.keyboard_modifiers.control() =>
                {
                    if cursor.is_over(layout.bounds()) {
                        let delta = match delta {
                            mouse::ScrollDelta::Lines { x: _, y }
                            | mouse::ScrollDelta::Pixels { x: _, y } => y,
                        };

                        if *delta < 0.0 {
                            let _ = decrement(current_value).map(change);
                        } else {
                            let _ = increment(current_value).map(change);
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        match key {
                            Key::Named(key::Named::ArrowUp) => {
                                let _ = increment(current_value).map(change);
                                shell.capture_event();
                            }
                            Key::Named(key::Named::ArrowDown) => {
                                let _ = decrement(current_value).map(change);
                                shell.capture_event();
                            }
                            _ => (),
                        }
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    state.keyboard_modifiers = *modifiers;
                }
                _ => {}
            }
        };

        update();

        let current_status = if state.is_dragging {
            Status::Dragged
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let style = theme.style(&self.class, self.status.unwrap_or(Status::Active));
        // style.handle.background = iced::Background::Color(iced::Color::WHITE);

        let (handle_width, handle_height, handle_border_radius) = match style.handle.shape {
            HandleShape::Circle { radius } => (radius * 2.0, radius * 2.0, radius.into()),
            HandleShape::Rectangle {
                width,
                border_radius,
            } => (f32::from(width), bounds.height, border_radius),
        };
        let corner_radius = handle_border_radius.bottom_left;

        let value = self.value.into() as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();

            (start.into() as f32, end.into() as f32)
        };

        let offset = if range_start >= range_end {
            0.0
        } else {
            (bounds.width - handle_width + corner_radius) * (value - range_start)
                / (range_end - range_start)
                - corner_radius / 2.0
            // (bounds.width - handle_width) * (value - range_start) / (range_end - range_start)
        };

        let rail_y = bounds.y + bounds.height / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y - style.rail.width / 2.0,
                    width: offset + handle_width / 2.0 + style.rail.width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.0,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset + handle_width / 2.0,
                    y: rail_y - style.rail.width / 2.0,
                    width: bounds.width - offset - handle_width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.1,
        );

        // Tinting is currently disabled, since it conflicts with the handle color
        // let tint = match style.handle.background {
        //     Background::Color(color) => color,
        //     _ => Color::WHITE,
        // };
        let tint = Color::WHITE;

        let state = tree.state.downcast_ref::<State>();
        if state.is_dragging {
            renderer.draw_primitive(
                Rectangle {
                    x: bounds.x + offset,
                    y: rail_y - handle_height / 2.0,
                    width: handle_width,
                    height: handle_height,
                },
                crate::primitive::Primitive {
                    id: state.id,
                    uniforms: crate::uniforms::Uniforms {
                        blur_radius: 0.0,
                        // TODO: don't just use the bottom left corner radius
                        corner_radius,
                        saturation: 1.0,
                        lightness: 0.0,
                        edge_radius: self.edge_radius,
                        height: self.edge_height,
                        refractive_index: self.refractive_index,
                        rim_width: 1.0,
                        opacity: 1.0,
                        tint,
                        content_scale: (1.0, 1.0),
                    },
                },
            );
        } else {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + offset,
                        y: rail_y - handle_height / 2.0,
                        width: handle_width,
                        height: handle_height,
                    },
                    border: Border {
                        radius: handle_border_radius,
                        width: style.handle.border_width,
                        color: style.handle.border_color,
                    },
                    ..renderer::Quad::default()
                },
                style.handle.background,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.is_dragging {
            // FIXME: Fall back to `Pointer` on Windows
            // See https://github.com/rust-windowing/winit/issues/1043
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grabbing
            }
        } else if cursor.is_over(layout.bounds()) {
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, T, Message, Theme> From<GlassSlider<'a, T, Message, Theme>> for Element<'a, Message, Theme>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    // Renderer: iced::Renderer + 'a,
{
    fn from(slider: GlassSlider<'a, T, Message, Theme>) -> Self {
        Element::new(slider)
    }
}
