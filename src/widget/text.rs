use cosmic_text::{Attrs, Buffer, FontSystem, LayoutGlyph, Metrics};
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
use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use crate::pipeline::{text::content_scale, text_atlas::GlyphId};

/// A widget that renders text with a glass effect.
#[must_use]
pub struct Text<'a, Renderer, Theme = iced::Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    id: Option<iced::widget::Id>,
    format: Format<Renderer::Font>,
    fragment: text::Fragment<'a>,
    class: Theme::Class<'a>,
    glass_style: crate::StyleFn<'a, Theme>,
}

impl<Renderer, Theme> std::fmt::Debug for Text<'_, Renderer, Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Font: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
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

struct FontData {
    font_system: RefCell<FontSystem>,
    metrics: Metrics,
    font: Option<iced::Font>,
    buffer: RefCell<Buffer>,

    // font_cache: RefCell<HashMap<FontKey, (Vec<u8>, u32)>>,
    #[allow(clippy::type_complexity)]
    font_cache: RefCell<HashMap<cosmic_text::fontdb::ID, Arc<(Vec<u8>, u32)>>>,

    last_text: RefCell<Option<String>>,
    last_bounds: RefCell<Option<(f32, f32)>>,
    last_glyphs: RefCell<Option<Vec<GlyphData>>>,
    last_metrics: RefCell<Option<Metrics>>,
    last_font: RefCell<Option<iced::Font>>,
}

impl FontData {
    fn needs_reshape(&self, s: &str, bounds: &Rectangle) -> bool {
        self.last_text.borrow().as_deref() != Some(s)
            || self.last_bounds.borrow().as_ref() != Some(&(bounds.width, bounds.height))
            || self.last_glyphs.borrow().is_none()
            || self.last_metrics.borrow().as_ref() != Some(&self.metrics)
            || self.last_font.borrow().as_ref() != self.font.as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlyphData {
    pub glyph_id: GlyphId,
    pub font_id: cosmic_text::fontdb::ID,
    pub x: f32,
    pub run_line_y: f32,
}

impl GlyphData {
    #[must_use]
    pub const fn new(glyph: &LayoutGlyph, run_line_y: f32) -> Self {
        Self {
            glyph_id: GlyphId(glyph.glyph_id),
            font_id: glyph.font_id,
            x: glyph.x,
            // y: glyph.y,
            run_line_y,
            // w: glyph.w,
            // y_offset: glyph.y_offset,
        }
    }
}

impl FontData {
    fn new(font: Option<iced::Font>, font_size: Pixels, line_height: LineHeight) -> Self {
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(notosans::REGULAR_TTF.to_vec());
        font_system
            .db_mut()
            .load_font_data(notosans::ITALIC_TTF.to_vec());
        font_system
            .db_mut()
            .load_font_data(notosans::BOLD_TTF.to_vec());
        font_system
            .db_mut()
            .load_font_data(notosans::BOLD_ITALIC_TTF.to_vec());

        let lh = match line_height {
            LineHeight::Relative(factor) => Pixels(factor * font_size.0),
            LineHeight::Absolute(pixels) => pixels,
        };
        let metrics = Metrics::new(font_size.into(), lh.into());
        let buffer = Buffer::new(&mut font_system, metrics);

        Self {
            font_system: RefCell::new(font_system),
            metrics,
            buffer: RefCell::new(buffer),
            font,

            font_cache: RefCell::new(HashMap::new()),

            last_text: RefCell::new(None),
            last_bounds: RefCell::new(None),
            last_glyphs: RefCell::new(None),
            last_metrics: RefCell::new(None),
            last_font: RefCell::new(None),
        }
    }
}

impl<'a, Renderer, Theme> Text<'a, Renderer, Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new(fragment: impl text::IntoFragment<'a>) -> Self {
        Self {
            id: None,
            fragment: fragment.into_fragment(),
            format: Format::default(),
            class: Theme::default(),
            glass_style: Box::new(|_| crate::Style::default()),
        }
    }

    fn parse_text(font_data: &FontData, s: &str, bounds: &Rectangle) -> Vec<GlyphData> {
        if !font_data.needs_reshape(s, bounds) {
            #[allow(clippy::expect_used)]
            return font_data
                .last_glyphs
                .borrow()
                .as_ref()
                .expect("No glyphs found")
                .clone();
        }

        let mut font_system = font_data.font_system.borrow_mut();
        let mut buffer = font_data.buffer.borrow_mut();

        let mut buffer = buffer.borrow_with(&mut font_system);

        buffer.set_size(Some(bounds.width), Some(bounds.height));
        let family = iced_font_to_family(font_data.font);
        let weight = iced_weight(font_data.font);
        let style = iced_style(font_data.font);
        let stretch = iced_stretch(font_data.font);
        let attrs = Attrs::new()
            .family(family)
            .weight(weight)
            .style(style)
            .stretch(stretch);
        buffer.set_text(s, &attrs, cosmic_text::Shaping::Advanced, None);

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
        font_data.last_font.replace(font_data.font);

        let all_fonts: Vec<cosmic_text::fontdb::ID> =
            glyphs.iter().map(|glyph| glyph.font_id).unique().collect();

        for font_id in all_fonts {
            let _font_bytes = font_data
                .font_cache
                .borrow_mut()
                .entry(font_id)
                .or_insert_with(|| {
                    let data = font_system
                        .db()
                        .with_face_data(font_id, |bytes, index| (bytes.to_vec(), index));
                    #[allow(clippy::expect_used)]
                    Arc::new(data.expect("Failed to get face data"))
                });
        }
        glyphs
    }

    /// Sets the [`widget::Id`] of the [`Text`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.format.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`Text`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.format.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: iced::advanced::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.format.font = Some(font.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`], if `Some`.
    ///
    /// [`Font`]: iced::advanced::text::Renderer::Font
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
    pub const fn shaping(mut self, shaping: Shaping) -> Self {
        self.format.shaping = shaping;
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Text`].
    pub const fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.format.wrapping = wrapping;
        self
    }

    /// Sets the style of the [`Text`].
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

    /// Sets the glass style of the [`Text`].
    pub fn glass_style(mut self, style: impl Fn(&Theme) -> crate::Style + 'a) -> Self {
        self.glass_style = Box::new(style) as crate::StyleFn<'a, Theme>;
        self
    }
}

struct State<P: iced::advanced::text::Paragraph> {
    id: u64,
    font_data: FontData,
    paragraph: paragraph::Plain<P>,
}
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<Message, Theme, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for Text<'_, Renderer, Theme>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + iced_wgpu::primitive::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            font_data: FontData::new(
                self.format.font,
                self.format.size.unwrap_or_else(|| 16.0.into()),
                self.format.line_height,
            ),
            paragraph: paragraph::Plain::<Renderer::Paragraph>::default(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let fs = self.format.size.unwrap_or_else(|| 16.0.into());
        let lh = match self.format.line_height {
            LineHeight::Relative(factor) => Pixels(factor * fs.0),
            LineHeight::Absolute(pixels) => pixels,
        };

        if (state.font_data.metrics.font_size - fs.0).abs() > f32::EPSILON
            || (state.font_data.metrics.line_height - lh.0).abs() > f32::EPSILON
        {
            let new_metrics = Metrics::new(fs.0, lh.0);
            state.font_data.metrics = new_metrics;
            state.font_data.buffer.borrow_mut().set_metrics(new_metrics);
            // Invalidate cached glyphs so parse_text reshapes
            *state.font_data.last_text.borrow_mut() = None;
        }

        if state.font_data.font != self.format.font {
            state.font_data.font = self.format.font;
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
    }

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
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let style = theme.style(&self.class);
        let glass_style = (self.glass_style)(theme);
        let scrim = style.color.unwrap_or(Color::TRANSPARENT);

        let glyphs = Self::parse_text(&state.font_data, &self.fragment, &bounds);

        if glyphs.is_empty() {
            return;
        }

        let fonts = state.font_data.font_cache.borrow().clone();

        renderer.draw_primitive(
            bounds,
            crate::primitive::text::TextPrimitive {
                id: state.id,
                // text: self.fragment.to_string(),
                fonts,
                glyphs,
                font_size: self.format.size.unwrap_or_else(|| 16.0.into()).0,
                uniforms: crate::uniforms::Uniforms {
                    blur_radius: glass_style.blur_radius,
                    // TODO: implement corner radius for text
                    corner_radius: 0.0,
                    saturation: glass_style.saturation,
                    lightness: glass_style.lightness,
                    edge_radius: glass_style.edge_radius,
                    height: glass_style.edge_height,
                    refractive_index: glass_style.refractive_index,
                    chromatic_aberration: glass_style.chromatic_aberration,
                    rim_width: glass_style.rim_width,
                    rim_angle: glass_style.rim_angle,
                    opacity: glass_style.opacity,
                    tint: glass_style.tint,
                    scrim,
                    content_scale: content_scale(bounds.size()),
                    edge_type: glass_style.edge_type,
                    num_children: 0,
                    blending_factor: 1.0,
                    fill_level: 0.0,
                    fill_color: Color::TRANSPARENT,
                },
            },
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Text<'a, Renderer, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer:
        iced::advanced::text::Renderer<Font = iced::Font> + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(text: Text<'a, Renderer, Theme>) -> Self {
        Element::new(text)
    }
}

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

fn iced_font_to_family(font: Option<iced::Font>) -> cosmic_text::Family<'static> {
    match font.map_or(iced::font::Family::SansSerif, |f| f.family) {
        iced::font::Family::Name(name) => cosmic_text::Family::Name(name),
        iced::font::Family::SansSerif => cosmic_text::Family::SansSerif,
        iced::font::Family::Serif => cosmic_text::Family::Serif,
        iced::font::Family::Cursive => cosmic_text::Family::Cursive,
        iced::font::Family::Monospace => cosmic_text::Family::Monospace,
        iced::font::Family::Fantasy => cosmic_text::Family::Fantasy,
    }
}

fn iced_weight(font: Option<iced::Font>) -> cosmic_text::Weight {
    cosmic_text::Weight(
        match font.map_or(iced::font::Weight::Normal, |f| f.weight) {
            iced::font::Weight::Thin => 100,
            iced::font::Weight::ExtraLight => 200,
            iced::font::Weight::Light => 300,
            iced::font::Weight::Normal => 400,
            iced::font::Weight::Medium => 500,
            iced::font::Weight::Semibold => 600,
            iced::font::Weight::Bold => 700,
            iced::font::Weight::ExtraBold => 800,
            iced::font::Weight::Black => 900,
        },
    )
}

fn iced_style(font: Option<iced::Font>) -> cosmic_text::Style {
    match font.map_or(iced::font::Style::Normal, |f| f.style) {
        iced::font::Style::Normal => cosmic_text::Style::Normal,
        iced::font::Style::Italic => cosmic_text::Style::Italic,
        iced::font::Style::Oblique => cosmic_text::Style::Oblique,
    }
}

fn iced_stretch(font: Option<iced::Font>) -> cosmic_text::Stretch {
    match font.map_or(iced::font::Stretch::Normal, |f| f.stretch) {
        iced::font::Stretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
        iced::font::Stretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
        iced::font::Stretch::Condensed => cosmic_text::Stretch::Condensed,
        iced::font::Stretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
        iced::font::Stretch::Normal => cosmic_text::Stretch::Normal,
        iced::font::Stretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
        iced::font::Stretch::Expanded => cosmic_text::Stretch::Expanded,
        iced::font::Stretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
        iced::font::Stretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
    }
}
