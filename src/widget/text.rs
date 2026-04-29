// use iced::advanced::Renderer as _;
use iced::{
    Color, Element, Length, Pixels, Rectangle, Size,
    advanced::{
        Layout, layout, mouse, renderer,
        text::paragraph,
        widget::{Tree, tree},
    },
    alignment,
    widget::{
        self,
        text::{self, Catalog, LineHeight, Shaping, Style, StyleFn, Wrapping},
    },
};

pub struct GlassText<'a, Renderer, Theme = iced::Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    id: Option<iced::widget::Id>,
    format: Format<Renderer::Font>,
    fragment: text::Fragment<'a>,

    // padding: Padding,
    // width: Length,
    // height: Length,
    // max_width: f32,
    // max_height: f32,
    // horizontal_alignment: iced::alignment::Horizontal,
    // vertical_alignment: iced::alignment::Vertical,
    // clip: bool,
    // content: String,
    class: Theme::Class<'a>,

    // font_size: f32,
    // line_height: f32,
    // font_data: FontData,

    // GlassText specific properties
    blur_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    edge_height: f32,
    refractive_index: f32,
    rim_width: f32,
    opacity: f32,
}

/// The format of some [`Text`].
///
/// Check out the methods of the [`Text`] widget
/// to learn more about each field.
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct Format<Font> {
    pub width: Length,
    pub height: Length,
    pub size: Option<Pixels>,
    pub font: Option<Font>,
    pub line_height: LineHeight,
    pub align_x: text::Alignment,
    pub align_y: alignment::Vertical,
    pub shaping: Shaping,
    pub wrapping: Wrapping,
}

impl<Font> Default for Format<Font> {
    fn default() -> Self {
        Self {
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
        }
    }
}

pub fn glass_text<'a, Renderer, Theme>(
    content: impl text::IntoFragment<'a>,
) -> GlassText<'a, Renderer, Theme>
where
    Theme: Catalog + 'a,
    Renderer: iced::advanced::text::Renderer,
{
    GlassText::new(content)
}

use cosmic_text::{Buffer, FontSystem, LayoutGlyph, Metrics};
use std::cell::RefCell;
use ttf_parser::GlyphId;

use crate::{font, pipeline::content_scale};
struct FontData {
    font_system: RefCell<FontSystem>,
    metrics: Metrics,
    buffer: RefCell<Buffer>,

    last_text: RefCell<Option<String>>,
    last_bounds: RefCell<Option<(f32, f32)>>,
    last_glyphs: RefCell<Option<Vec<GlyphData>>>,
    last_metrics: RefCell<Option<Metrics>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GlyphData {
    pub glyph_id: GlyphId,
    // pub glyph: char,
    pub x: f32,
    pub y: f32,
    pub run_line_y: f32,
    pub w: f32,
    pub y_offset: f32,
}

impl GlyphData {
    pub fn new(glyph: &LayoutGlyph, run_line_y: f32) -> Self {
        Self {
            glyph_id: GlyphId(glyph.glyph_id),
            x: glyph.x,
            y: glyph.y,
            run_line_y,
            w: glyph.w,
            y_offset: glyph.y_offset,
        }
    }
}

impl FontData {
    fn new(font_size: Pixels, line_height: LineHeight) -> Self {
        let mut font_system = FontSystem::new();
        // let f = include_bytes!("/System/Library/Fonts/Supplemental/Arial Unicode.ttf");

        // let font = Face::parse(f, 0).unwrap();
        font_system.db_mut().load_font_data(font::FONT.to_vec());
        // Metrics: font_size, line_height
        let lh = match line_height {
            LineHeight::Relative(factor) => Pixels(factor * font_size.0),
            LineHeight::Absolute(pixels) => pixels,
        };
        let metrics = Metrics::new(font_size.into(), lh.into());
        let buffer = Buffer::new(&mut font_system, metrics);
        // Set available width/height

        Self {
            font_system: RefCell::new(font_system),
            metrics,
            buffer: RefCell::new(buffer),

            last_text: RefCell::new(None),
            last_bounds: RefCell::new(None),
            last_glyphs: RefCell::new(None),
            last_metrics: RefCell::new(None),
        }
    }
}

impl<'a, Renderer, Theme> GlassText<'a, Renderer, Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new(fragment: impl text::IntoFragment<'a>) -> Self {
        Self {
            id: None,
            fragment: fragment.into_fragment(),
            format: Format::default(),
            class: Theme::default(),

            blur_radius: 0.0,
            saturation: 1.0,
            lightness: 0.0,
            edge_radius: 0.0,
            edge_height: 0.0,
            refractive_index: 1.5,
            rim_width: 1.0,
            opacity: 1.0,
        }
    }

    // Creates a [`Container`] with the given content.
    // pub fn new(content: impl Into<String>) -> Self {
    //     let content = content.into();
    //     // let size = content.as_widget().size_hint();

    //     GlassText {
    //         id: None,
    //         format: Format {
    //             width: Length::Fill,
    //             height: Length::Fill,
    //             size: None,
    //             font: None,
    //             line_height: LineHeight::default(),
    //             align_x: text::Alignment::Left,
    //             align_y: alignment::Vertical::Top,
    //             shaping: Shaping::Advanced,
    //             wrapping: Wrapping::default(),
    //         },
    //         // padding: Padding::ZERO,
    //         // width: 1.0.into(),
    //         // height: 1.0.into(),
    //         // max_width: f32::INFINITY,
    //         // max_height: f32::INFINITY,
    //         // horizontal_alignment: alignment::Horizontal::Left,
    //         // vertical_alignment: alignment::Vertical::Top,
    //         // clip: false,
    //         class: Theme::default(),
    //         content,

    //         // font_size: 48.0,
    //         // line_height: 56.0,
    //         // font_data: FontData::new(48.0, 56.0),
    //         blur_radius: 0.0,
    //         saturation: 1.0,
    //         lightness: 0.0,
    //         edge_radius: 0.0,
    //         edge_height: 0.0,
    //         refractive_index: 1.5,
    //         rim_width: 1.0,
    //         opacity: 1.0,
    //     }
    // }

    fn parse_text(font_data: &FontData, s: &str, bounds: &Rectangle) -> Vec<GlyphData> {
        use cosmic_text::{Attrs, Shaping};

        let needs_reshape = font_data.last_text.borrow().as_deref() != Some(s)
            || font_data.last_bounds.borrow().as_ref() != Some(&(bounds.width, bounds.height))
            || font_data.last_glyphs.borrow().is_none()
            || font_data.last_metrics.borrow().as_ref() != Some(&font_data.metrics);

        // println!("{}", font_data.metrics.font_size);
        if !needs_reshape {
            return font_data.last_glyphs.borrow().as_ref().unwrap().clone();
        }
        // println!("Reshaping text");
        // let now = std::time::Instant::now();

        let mut font_system = font_data.font_system.borrow_mut();
        let mut buffer = font_data.buffer.borrow_mut();

        let mut buffer = buffer.borrow_with(&mut font_system);

        buffer.set_size(Some(bounds.width), Some(bounds.height));
        let attrs = Attrs::new().family(font::FAMILY);
        buffer.set_text(s, &attrs, Shaping::Advanced, None);

        // let elapsed1 = now.elapsed();
        // println!("Time taken to set text: {:?}", elapsed1);
        // let now = std::time::Instant::now();

        font_data.last_text.replace(Some(s.to_string()));
        font_data
            .last_bounds
            .replace(Some((bounds.width, bounds.height)));
        let glyphs: Vec<GlyphData> = buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter().map(move |glyph| (run.line_y, glyph)))
            .map(|(run_line_y, glyph)| GlyphData::new(glyph, run_line_y))
            .collect();
        font_data.last_glyphs.replace(Some(glyphs.clone()));
        font_data.last_metrics.replace(Some(font_data.metrics));
        // let elapsed2 = now.elapsed();
        // println!("Time taken to parse text: {:?}", elapsed2);
        glyphs
    }

    /// Sets the [`widget::Id`] of the [`Container`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.format.size = Some(size.into());
        // self.font_data.metrics.font_size = font_size;
        //     // self.font_data
        //     //     .buffer
        //     //     .borrow_mut()
        //     //     .set_metrics(self.font_data.metrics);

        self
    }

    /// Sets the [`LineHeight`] of the [`Text`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.format.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.format.font = Some(font.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`], if `Some`.
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font_maybe(mut self, font: Option<impl Into<Renderer::Font>>) -> Self {
        self.format.font = font.map(Into::into);
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.format.width = width.into();
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.format.height = height.into();
        self
    }

    /// Centers the [`Text`], both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Text`].
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.format.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Text`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.format.align_y = alignment.into();
        self
    }

    /// Sets the [`Shaping`] strategy of the [`Text`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.format.shaping = shaping;
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Text`].
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.format.wrapping = wrapping;
        self
    }

    /// Sets the style of the [`Text`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the [`Color`] of the [`Text`], if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style { color })
    }

    // #[cfg(feature = "advanced")]
    // #[must_use]
    // pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
    //     self.class = class.into();
    //     self
    // }

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

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    // pub fn font_size(mut self, font_size: f32) -> Self {
    //     self.font_size = font_size;
    //     // self.font_data.metrics.font_size = font_size;
    //     // self.font_data
    //     //     .buffer
    //     //     .borrow_mut()
    //     //     .set_metrics(self.font_data.metrics);
    //     self
    // }

    // pub fn line_height(mut self, line_height: f32) -> Self {
    //     self.line_height = line_height;
    //     // self.font_data.metrics.line_height = line_height;
    //     // self.font_data
    //     //     .buffer
    //     //     .borrow_mut()
    //     //     .set_metrics(self.font_data.metrics);
    //     self
    // }
}

struct State<P: iced::advanced::text::Paragraph> {
    id: u64,
    font_data: FontData,
    paragraph: paragraph::Plain<P>,
}
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<Message, Theme, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for GlassText<'_, Renderer, Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer + iced_wgpu::primitive::Renderer,
{
    fn tag(&self) -> tree::Tag {
        // self.content.as_widget().tag()
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        // self.content.as_widget().state()
        tree::State::new(State {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            font_data: FontData::new(
                self.format.size.unwrap_or(16.0.into()),
                self.format.line_height,
            ),
            paragraph: paragraph::Plain::<Renderer::Paragraph>::default(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![]
        // vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        // self.content.as_widget().diff(tree);
        // tree.diff_children(std::slice::from_ref(&self.content));
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let fs = self.format.size.unwrap_or(16.0.into());
        let lh = match self.format.line_height {
            LineHeight::Relative(factor) => Pixels(factor * fs.0),
            LineHeight::Absolute(pixels) => pixels,
        };
        if state.font_data.metrics.font_size != fs.0 || state.font_data.metrics.line_height != lh.0
        {
            let new_metrics = Metrics::new(fs.0, lh.0);
            state.font_data.metrics = new_metrics;
            state.font_data.buffer.borrow_mut().set_metrics(new_metrics);
            // Invalidate cached glyphs so parse_text reshapes
            *state.font_data.last_text.borrow_mut() = None;
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.format.width,
            height: self.format.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            &mut tree
                .state
                .downcast_mut::<State<Renderer::Paragraph>>()
                .paragraph,
            renderer,
            limits,
            &self.fragment,
            self.format,
        )
        // layout(
        //     limits,
        //     self.width,
        //     self.height,
        //     self.max_width,
        //     self.max_height,
        //     self.padding,
        //     self.horizontal_alignment,
        //     self.vertical_alignment,
        //     |limits| layout::atomic(limits, self.width, self.height),
        // )
    }

    // fn operate(
    //     &mut self,
    //     tree: &mut Tree,
    //     layout: Layout<'_>,
    //     renderer: &Renderer,
    //     operation: &mut dyn Operation,
    // ) {
    //     operation.container(self.id.as_ref(), layout.bounds());
    //     operation.traverse(&mut |operation| {
    //         self.content.as_widget_mut().operate(
    //             &mut tree.children[0],
    //             layout.children().next().unwrap(),
    //             renderer,
    //             operation,
    //         );
    //     });
    // }

    // fn update(
    //     &mut self,
    //     tree: &mut Tree,
    //     event: &Event,
    //     layout: Layout<'_>,
    //     cursor: mouse::Cursor,
    //     renderer: &Renderer,
    //     clipboard: &mut dyn Clipboard,
    //     shell: &mut Shell<'_, Message>,
    //     viewport: &Rectangle,
    // ) {
    //     self.content.as_widget_mut().update(
    //         &mut tree.children[0],
    //         event,
    //         layout.children().next().unwrap(),
    //         cursor,
    //         renderer,
    //         clipboard,
    //         shell,
    //         viewport,
    //     );
    // }

    // fn mouse_interaction(
    //     &self,
    //     tree: &Tree,
    //     layout: Layout<'_>,
    //     cursor: mouse::Cursor,
    //     viewport: &Rectangle,
    //     renderer: &Renderer,
    // ) -> mouse::Interaction {
    //     self.content.as_widget().mouse_interaction(
    //         &tree.children[0],
    //         layout.children().next().unwrap(),
    //         cursor,
    //         viewport,
    //         renderer,
    //     )
    // }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _renderer_style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // let now = std::time::Instant::now();
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let style = theme.style(&self.class);
        let tint = style.color.unwrap_or(Color::WHITE);

        let glyphs = Self::parse_text(&state.font_data, &self.fragment, &bounds);

        renderer.draw_primitive(
            bounds,
            crate::primitive::text::TextPrimitive {
                id: state.id,
                text: self.fragment.to_string(),
                glyphs,
                font_size: self.format.size.unwrap_or(16.0.into()).0,
                uniforms: crate::uniforms::Uniforms {
                    blur_radius: self.blur_radius,
                    // TODO: implement corner radius for text
                    corner_radius: 0.0,
                    saturation: self.saturation,
                    lightness: self.lightness,
                    edge_radius: self.edge_radius,
                    height: self.edge_height,
                    refractive_index: self.refractive_index,
                    rim_width: self.rim_width,
                    opacity: self.opacity,
                    tint,
                    content_scale: content_scale(bounds.size()),
                },
            },
        );

        // let elapsed = now.elapsed();
        // println!("Time taken to draw text: {:?}", elapsed);
    }

    // fn overlay<'b>(
    //     &'b mut self,
    //     tree: &'b mut Tree,
    //     layout: Layout<'b>,
    //     renderer: &Renderer,
    //     viewport: &Rectangle,
    //     translation: Vector,
    // ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
    //     self.content.as_widget_mut().overlay(
    //         &mut tree.children[0],
    //         layout.children().next().unwrap(),
    //         renderer,
    //         viewport,
    //         translation,
    //     )
    // }

    // fn size_hint(&self) -> Size<Length> {
    //     self.size()
    // }
}

impl<'a, Message, Theme, Renderer> From<GlassText<'a, Renderer, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: iced::advanced::text::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(text: GlassText<'a, Renderer, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}

// #[allow(clippy::too_many_arguments)]
// /// Computes the layout of a [`Container`].
// pub fn layout(
//     limits: &layout::Limits,
//     width: Length,
//     height: Length,
//     max_width: f32,
//     max_height: f32,
//     padding: Padding,
//     horizontal_alignment: alignment::Horizontal,
//     vertical_alignment: alignment::Vertical,
//     layout_content: impl FnOnce(&layout::Limits) -> layout::Node,
// ) -> layout::Node {
//     layout::positioned(
//         &limits.max_width(max_width).max_height(max_height),
//         width,
//         height,
//         padding,
//         |limits| layout_content(&limits.loose()),
//         |content, size| {
//             content.align(
//                 Alignment::from(horizontal_alignment),
//                 Alignment::from(vertical_alignment),
//                 size,
//             )
//         },
//     )
// }

/// Produces the [`layout::Node`] of a [`Text`] widget.
pub fn layout<Renderer>(
    paragraph: &mut paragraph::Plain<Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    content: &str,
    format: Format<Renderer::Font>,
) -> layout::Node
where
    Renderer: iced::advanced::text::Renderer,
{
    layout::sized(limits, format.width, format.height, |limits| {
        let bounds = limits.max();

        let size = format.size.unwrap_or_else(|| renderer.default_size());
        let font = format.font.unwrap_or_else(|| renderer.default_font());

        let _ = paragraph.update(iced::advanced::Text {
            content,
            bounds,
            size,
            line_height: format.line_height,
            font,
            align_x: format.align_x,
            align_y: format.align_y,
            shaping: format.shaping,
            wrapping: format.wrapping,
        });

        paragraph.min_bounds()
    })
}

// Draws the background of a [`Container`] given its [`Style`] and its `bounds`.
// pub fn draw_background<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle)
// where
//     Renderer: iced::advanced::Renderer,
// {
//     if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
//         renderer.fill_quad(
//             renderer::Quad {
//                 bounds,
//                 border: style.border,
//                 shadow: style.shadow,
//                 snap: style.snap,
//             },
//             style
//                 .background
//                 .unwrap_or(Background::Color(Color::TRANSPARENT)),
//         );
//     }
// }
