use super::*;

/// A builder for a content stream.
pub struct Content {
    buf: Buf,
    q_depth: usize,
}

/// Core methods.
impl Content {
    /// Create a new content stream with the default buffer capacity
    /// (currently 1 KB).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new content stream with the specified initial buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Buf::with_capacity(capacity), q_depth: 0 }
    }

    /// Start writing an arbitrary operation.
    #[inline]
    pub fn op<'a>(&'a mut self, operator: &'a str) -> Operation<'a> {
        Operation::start(&mut self.buf, operator)
    }

    /// Return the buffer of the content stream.
    ///
    /// The buffer is essentially a thin wrapper around two objects:
    /// - A [`Limits`] object, which can optionally be used to keep track of
    ///   data such as the largest used integer or the longest string used in
    ///   the content streams, which is useful information for some export
    ///   modes.
    /// - The actual underlying data of the content stream, which can be written
    ///   to a chunk (and optionally apply a filter before doing so).
    pub fn finish(mut self) -> Buf {
        if self.buf.last() == Some(&b'\n') {
            self.buf.inner.pop();
        }
        self.buf
    }
}

/// Writer for an _operation_ in a content stream.
///
/// This struct is created by [`Content::op`].
pub struct Operation<'a> {
    buf: &'a mut Buf,
    op: &'a str,
    first: bool,
}

impl<'a> Operation<'a> {
    #[inline]
    pub(crate) fn start(buf: &'a mut Buf, op: &'a str) -> Self {
        Self { buf, op, first: true }
    }

    /// Write a primitive operand.
    #[inline]
    pub fn operand<T: Primitive>(&mut self, value: T) -> &mut Self {
        self.obj().primitive(value);
        self
    }

    /// Write a sequence of primitive operands.
    #[inline]
    pub fn operands<T, I>(&mut self, values: I) -> &mut Self
    where
        T: Primitive,
        I: IntoIterator<Item = T>,
    {
        for value in values {
            self.operand(value);
        }
        self
    }

    /// Start writing an an arbitrary object operand.
    #[inline]
    pub fn obj(&mut self) -> Obj<'_> {
        if !self.first {
            self.buf.push(b' ');
        }
        self.first = false;
        Obj::direct(self.buf, 0)
    }
}

impl Drop for Operation<'_> {
    #[inline]
    fn drop(&mut self) {
        if !self.first {
            self.buf.push(b' ');
        }
        self.buf.extend(self.op.as_bytes());
        self.buf.push(b'\n');
    }
}

/// General graphics state.
impl Content {
    /// `w`: Set the stroke line width.
    ///
    /// Panics if `width` is negative.
    #[inline]
    pub fn set_line_width(&mut self, width: f32) -> &mut Self {
        assert!(width >= 0.0, "line width must be positive");
        self.op("w").operand(width);
        self
    }

    /// `J`: Set the line cap style.
    #[inline]
    pub fn set_line_cap(&mut self, cap: LineCapStyle) -> &mut Self {
        self.op("J").operand(cap.to_int());
        self
    }

    /// `j`: Set the line join style.
    #[inline]
    pub fn set_line_join(&mut self, join: LineJoinStyle) -> &mut Self {
        self.op("j").operand(join.to_int());
        self
    }

    /// `M`: Set the miter limit.
    #[inline]
    pub fn set_miter_limit(&mut self, limit: f32) -> &mut Self {
        self.op("M").operand(limit);
        self
    }

    /// `d`: Set the line dash pattern.
    #[inline]
    pub fn set_dash_pattern(
        &mut self,
        array: impl IntoIterator<Item = f32>,
        phase: f32,
    ) -> &mut Self {
        let mut op = self.op("d");
        op.obj().array().items(array);
        op.operand(phase);
        op.finish();
        self
    }

    /// `ri`: Set the color rendering intent to the parameter. PDF 1.1+.
    #[inline]
    pub fn set_rendering_intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.op("ri").operand(intent.to_name());
        self
    }

    /// `i`: Set the flatness tolerance in device pixels.
    ///
    /// Panics if `tolerance` is negative or larger than 100.
    #[inline]
    pub fn set_flatness(&mut self, tolerance: i32) -> &mut Self {
        assert!(
            matches!(tolerance, 0..=100),
            "flatness tolerance must be between 0 and 100",
        );
        self.op("i").operand(tolerance);
        self
    }

    /// `gs`: Set the parameters from an `ExtGState` dictionary. PDF 1.2+.
    #[inline]
    pub fn set_parameters(&mut self, dict: Name) -> &mut Self {
        self.op("gs").operand(dict);
        self
    }
}

/// How to terminate lines.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LineCapStyle {
    /// Square the line of at the endpoints of the path.
    ButtCap,
    /// Round the line off at its end with a semicircular arc as wide as the
    /// stroke.
    RoundCap,
    /// End the line with a square cap that protrudes by half the width of the
    /// stroke.
    ProjectingSquareCap,
}

impl LineCapStyle {
    #[inline]
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::ButtCap => 0,
            Self::RoundCap => 1,
            Self::ProjectingSquareCap => 2,
        }
    }
}

/// How to join lines at corners.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LineJoinStyle {
    /// Join the lines with a sharp corner where the outsides of the lines
    /// intersect.
    MiterJoin,
    /// Join the lines with a smooth circular segment.
    RoundJoin,
    /// End both lines with butt caps and join them with a triangle.
    BevelJoin,
}

impl LineJoinStyle {
    #[inline]
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::MiterJoin => 0,
            Self::RoundJoin => 1,
            Self::BevelJoin => 2,
        }
    }
}
/// How the output device should aim to render colors.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RenderingIntent {
    /// Only consider the light source, not the output's white point.
    AbsoluteColorimetric,
    /// Consider both the light source and the output's white point.
    RelativeColorimetric,
    /// Preserve saturation.
    Saturation,
    /// Preserve a pleasing visual appearance.
    Perceptual,
}

impl RenderingIntent {
    #[inline]
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::AbsoluteColorimetric => Name(b"AbsoluteColorimetric"),
            Self::RelativeColorimetric => Name(b"RelativeColorimetric"),
            Self::Saturation => Name(b"Saturation"),
            Self::Perceptual => Name(b"Perceptual"),
        }
    }
}

/// Special graphics state.
impl Content {
    /// `q`: Save the graphics state on the stack.
    #[inline]
    pub fn save_state(&mut self) -> &mut Self {
        self.op("q");
        self.q_depth = self.q_depth.saturating_add(1);
        self
    }

    /// `Q`: Restore the graphics state from the stack.
    #[inline]
    pub fn restore_state(&mut self) -> &mut Self {
        self.op("Q");
        self.q_depth = self.q_depth.saturating_sub(1);
        self
    }

    /// The current `q` nesting depth.
    #[inline]
    pub fn state_nesting_depth(&self) -> usize {
        self.q_depth
    }

    /// `cm`: Pre-concatenate the `matrix` with the current transformation
    /// matrix.
    #[inline]
    pub fn transform(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.op("cm").operands(matrix);
        self
    }
}

/// Path construction.
impl Content {
    /// `m`: Begin a new subpath at (x, y).
    #[inline]
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("m").operands([x, y]);
        self
    }

    /// `l`: Append a straight line to (x, y).
    #[inline]
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("l").operands([x, y]);
        self
    }

    /// `c`: Append a cubic Bézier segment to (x3, y3) with (x1, y1), (x2, y2)
    /// as control points.
    #[inline]
    pub fn cubic_to(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) -> &mut Self {
        self.op("c").operands([x1, y1, x2, y2, x3, y3]);
        self
    }

    /// `v`: Append a cubic Bézier segment to (x3, y3) with (x2, y2) as control
    /// point.
    #[inline]
    pub fn cubic_to_initial(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.op("v").operands([x2, y2, x3, y3]);
        self
    }

    /// `y`: Append a cubic Bézier segment to (x3, y3) with (x1, y1) as control
    /// point.
    #[inline]
    pub fn cubic_to_final(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) -> &mut Self {
        self.op("y").operands([x1, y1, x3, y3]);
        self
    }

    /// `h`: Close the current subpath with a straight line.
    #[inline]
    pub fn close_path(&mut self) -> &mut Self {
        self.op("h");
        self
    }

    /// `re`: Append a rectangle to the current path.
    #[inline]
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op("re").operands([x, y, width, height]);
        self
    }
}

/// Path painting.
impl Content {
    /// `S`: Stroke the current path.
    #[inline]
    pub fn stroke(&mut self) -> &mut Self {
        self.op("S");
        self
    }

    /// `s`: Close the current path and then stroke it.
    #[inline]
    pub fn close_and_stroke(&mut self) -> &mut Self {
        self.op("s");
        self
    }

    /// `f`: Fill the current path using the nonzero winding number rule.
    #[inline]
    pub fn fill_nonzero(&mut self) -> &mut Self {
        self.op("f");
        self
    }

    /// `f*`: Fill the current path using the even-odd rule.
    #[inline]
    pub fn fill_even_odd(&mut self) -> &mut Self {
        self.op("f*");
        self
    }

    /// `B`: Fill the current path using the nonzero winding number rule and
    /// then stroke it.
    #[inline]
    pub fn fill_nonzero_and_stroke(&mut self) -> &mut Self {
        self.op("B");
        self
    }

    /// `B*`: Fill the current path using the even-odd rule and then stroke it.
    #[inline]
    pub fn fill_even_odd_and_stroke(&mut self) -> &mut Self {
        self.op("B*");
        self
    }

    /// `b`: Close the current path, fill it using the nonzero winding number
    /// rule and then stroke it.
    #[inline]
    pub fn close_fill_nonzero_and_stroke(&mut self) -> &mut Self {
        self.op("b");
        self
    }

    /// `b*`: Close the current path, fill it using the even-odd rule and then
    /// stroke it.
    #[inline]
    pub fn close_fill_even_odd_and_stroke(&mut self) -> &mut Self {
        self.op("b*");
        self
    }

    /// `n`: End the current path without filling or stroking it.
    ///
    /// This is primarily used for clipping paths.
    #[inline]
    pub fn end_path(&mut self) -> &mut Self {
        self.op("n");
        self
    }
}

/// Clipping paths.
impl Content {
    /// `W`: Intersect the current clipping path with the current path using the
    /// nonzero winding number rule.
    #[inline]
    pub fn clip_nonzero(&mut self) -> &mut Self {
        self.op("W");
        self
    }

    /// `W*`: Intersect the current clipping path with the current path using
    /// the even-odd rule.
    #[inline]
    pub fn clip_even_odd(&mut self) -> &mut Self {
        self.op("W*");
        self
    }
}

/// Text objects.
impl Content {
    /// `BT`: Begin a text object.
    #[inline]
    pub fn begin_text(&mut self) -> &mut Self {
        self.op("BT");
        self
    }

    /// `ET`: End a text object.
    #[inline]
    pub fn end_text(&mut self) -> &mut Self {
        self.op("ET");
        self
    }
}

/// Text state.
impl Content {
    /// `Tc`: Set the character spacing.
    #[inline]
    pub fn set_char_spacing(&mut self, spacing: f32) -> &mut Self {
        self.op("Tc").operand(spacing);
        self
    }

    /// `Tw`: Set the word spacing.
    #[inline]
    pub fn set_word_spacing(&mut self, spacing: f32) -> &mut Self {
        self.op("Tw").operand(spacing);
        self
    }

    /// `Tz`: Set the horizontal scaling.
    #[inline]
    pub fn set_horizontal_scaling(&mut self, scaling: f32) -> &mut Self {
        self.op("Tz").operand(scaling);
        self
    }

    /// `TL`: Set the leading.
    #[inline]
    pub fn set_leading(&mut self, leading: f32) -> &mut Self {
        self.op("TL").operand(leading);
        self
    }

    /// `Tf`: Set font and font size.
    #[inline]
    pub fn set_font(&mut self, font: Name, size: f32) -> &mut Self {
        self.op("Tf").operand(font).operand(size);
        self
    }

    /// `Tr`: Set the text rendering mode.
    #[inline]
    pub fn set_text_rendering_mode(&mut self, mode: TextRenderingMode) -> &mut Self {
        self.op("Tr").operand(mode.to_int());
        self
    }

    /// `Ts`: Set the rise.
    #[inline]
    pub fn set_rise(&mut self, rise: f32) -> &mut Self {
        self.op("Ts").operand(rise);
        self
    }
}

/// How to render text.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TextRenderingMode {
    /// Just fill the text.
    Fill,
    /// Just stroke the text.
    Stroke,
    /// First fill and then stroke the text.
    FillStroke,
    /// Don't fill and don't stroke the text.
    Invisible,
    /// Fill the text, then apply the text outlines to the current clipping
    /// path.
    FillClip,
    /// Stroke the text, then apply the text outlines to the current clipping
    /// path.
    StrokeClip,
    /// First fill, then stroke the text and finally apply the text outlines to
    /// the current clipping path.
    FillStrokeClip,
    /// Apply the text outlines to the current clipping path.
    Clip,
}

impl TextRenderingMode {
    #[inline]
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Fill => 0,
            Self::Stroke => 1,
            Self::FillStroke => 2,
            Self::Invisible => 3,
            Self::FillClip => 4,
            Self::StrokeClip => 5,
            Self::FillStrokeClip => 6,
            Self::Clip => 7,
        }
    }
}

/// Text positioning.
impl Content {
    /// `Td`: Move to the start of the next line.
    #[inline]
    pub fn next_line(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("Td").operands([x, y]);
        self
    }

    /// `TD`: Move to the start of the next line and set the text state's
    /// leading parameter to `-y`.
    #[inline]
    pub fn next_line_and_set_leading(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("TD").operands([x, y]);
        self
    }

    /// `Tm`: Set the text matrix.
    #[inline]
    pub fn set_text_matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.op("Tm").operands(matrix);
        self
    }

    /// `T*`: Move to the start of the next line, determining the vertical offset
    /// through the text state's leading parameter.
    #[inline]
    pub fn next_line_using_leading(&mut self) -> &mut Self {
        self.op("T*");
        self
    }
}

/// Text showing.
impl Content {
    /// `Tj`: Show text.
    ///
    /// The encoding of the text depends on the font.
    #[inline]
    pub fn show(&mut self, text: Str) -> &mut Self {
        self.op("Tj").operand(text);
        self
    }

    /// `'`: Move to the next line and show text.
    #[inline]
    pub fn next_line_show(&mut self, text: Str) -> &mut Self {
        self.op("'").operand(text);
        self
    }

    /// `"`: Move to the next line, show text and set the text state's word and
    /// character spacing.
    #[inline]
    pub fn next_line_show_and_set_word_and_char_spacing(
        &mut self,
        word_spacing: f32,
        char_spacing: f32,
        text: Str,
    ) -> &mut Self {
        self.op("\"").operands([word_spacing, char_spacing]).operand(text);
        self
    }

    /// `TJ`: Start showing text with individual glyph positioning.
    #[inline]
    pub fn show_positioned(&mut self) -> ShowPositioned<'_> {
        ShowPositioned::start(self.op("TJ"))
    }
}

/// Writer for an _individual glyph positioning operation_.
///
/// This struct is created by [`Content::show_positioned`].
pub struct ShowPositioned<'a> {
    op: Operation<'a>,
}

impl<'a> ShowPositioned<'a> {
    #[inline]
    pub(crate) fn start(op: Operation<'a>) -> Self {
        Self { op }
    }

    /// Start writing the array of strings and adjustments. Required.
    #[inline]
    pub fn items(&mut self) -> PositionedItems<'_> {
        PositionedItems::start(self.op.obj())
    }
}

deref!('a, ShowPositioned<'a> => Operation<'a>, op);

/// Writer for a _positioned items array_.
///
/// This struct is created by [`ShowPositioned::items`].
pub struct PositionedItems<'a> {
    array: Array<'a>,
}

impl<'a> PositionedItems<'a> {
    #[inline]
    pub(crate) fn start(obj: Obj<'a>) -> Self {
        Self { array: obj.array() }
    }

    /// Show a continuous string without adjustments.
    ///
    /// The encoding of the text depends on the font.
    #[inline]
    pub fn show(&mut self, text: Str) -> &mut Self {
        self.array.item(text);
        self
    }

    /// Specify an adjustment between two glyphs.
    ///
    /// The `amount` is specified in thousands of units of text space and is
    /// subtracted from the current writing-mode dependent coordinate.
    #[inline]
    pub fn adjust(&mut self, amount: f32) -> &mut Self {
        self.array.item(amount);
        self
    }
}

deref!('a, PositionedItems<'a> => Array<'a>, array);

/// Type 3 fonts.
///
/// These operators are only allowed in
/// [Type 3 CharProcs](crate::font::Type3Font::char_procs).
impl Content {
    /// `d0`: Starts a Type 3 glyph that contains color information.
    /// - `wx` defines the glyph's width
    /// - `wy` is set to 0.0 automatically
    #[inline]
    pub fn start_color_glyph(&mut self, wx: f32) -> &mut Self {
        self.op("d0").operands([wx, 0.0]);
        self
    }

    /// `d1`: Starts a Type 3 glyph that contains only shape information.
    /// - `wx` defines the glyph's width
    /// - `wy` is set to 0.0 automatically
    /// - `ll_x` and `ll_y` define the lower-left corner of the glyph bounding box
    /// - `ur_x` and `ur_y` define the upper-right corner of the glyph bounding box
    #[inline]
    pub fn start_shape_glyph(
        &mut self,
        wx: f32,
        ll_x: f32,
        ll_y: f32,
        ur_x: f32,
        ur_y: f32,
    ) -> &mut Self {
        self.op("d1").operands([wx, 0.0, ll_x, ll_y, ur_x, ur_y]);
        self
    }
}

/// Color.
impl Content {
    /// `CS`: Set the stroke color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    #[inline]
    pub fn set_stroke_color_space<'a>(
        &mut self,
        space: impl Into<ColorSpaceOperand<'a>>,
    ) -> &mut Self {
        self.op("CS").operand(space.into().to_name());
        self
    }

    /// `cs`: Set the fill color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    #[inline]
    pub fn set_fill_color_space<'a>(
        &mut self,
        space: impl Into<ColorSpaceOperand<'a>>,
    ) -> &mut Self {
        self.op("cs").operand(space.into().to_name());
        self
    }

    /// `SCN`: Set the stroke color to the parameter within the current color
    /// space. PDF 1.2+.
    #[inline]
    pub fn set_stroke_color(
        &mut self,
        color: impl IntoIterator<Item = f32>,
    ) -> &mut Self {
        self.op("SCN").operands(color);
        self
    }

    /// `SCN`: Set the stroke pattern. PDF 1.2+.
    ///
    /// The `name` parameter is the name of a pattern. If this is an uncolored
    /// pattern, a tint color in the current `Pattern` base color space must be
    /// given, otherwise, the `color` iterator shall remain empty.
    #[inline]
    pub fn set_stroke_pattern(
        &mut self,
        tint: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("SCN").operands(tint).operand(name);
        self
    }

    /// `scn`: Set the fill color to the parameter within the current color
    /// space. PDF 1.2+.
    #[inline]
    pub fn set_fill_color(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        self.op("scn").operands(color);
        self
    }

    /// `scn`: Set the fill pattern. PDF 1.2+.
    ///
    /// The `name` parameter is the name of a pattern. If this is an uncolored
    /// pattern, a tint color in the current `Pattern` base color space must be
    /// given, otherwise, the `color` iterator shall remain empty.
    #[inline]
    pub fn set_fill_pattern(
        &mut self,
        tint: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("scn").operands(tint).operand(name);
        self
    }

    /// `G`: Set the stroke color to the parameter and the color space to
    /// `DeviceGray`.
    #[inline]
    pub fn set_stroke_gray(&mut self, gray: f32) -> &mut Self {
        self.op("G").operand(gray);
        self
    }

    /// `g`: Set the fill color to the parameter and the color space to
    /// `DeviceGray`.
    #[inline]
    pub fn set_fill_gray(&mut self, gray: f32) -> &mut Self {
        self.op("g").operand(gray);
        self
    }

    /// `RG`: Set the stroke color to the parameter and the color space to
    /// `DeviceRGB`.
    #[inline]
    pub fn set_stroke_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op("RG").operands([r, g, b]);
        self
    }

    /// `rg`: Set the fill color to the parameter and the color space to
    /// `DeviceRGB`.
    #[inline]
    pub fn set_fill_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op("rg").operands([r, g, b]);
        self
    }

    /// `K`: Set the stroke color to the parameter and the color space to
    /// `DeviceCMYK`.
    #[inline]
    pub fn set_stroke_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op("K").operands([c, m, y, k]);
        self
    }

    /// `k`: Set the fill color to the parameter and the color space to
    /// `DeviceCMYK`.
    #[inline]
    pub fn set_fill_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op("k").operands([c, m, y, k]);
        self
    }
}

/// A color space operand to the [`CS`](Content::set_stroke_color_space) or
/// [`cs`](Content::set_fill_color_space) operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ColorSpaceOperand<'a> {
    /// The predefined, parameterless gray device-dependent color space. Needs
    /// no further entries in the resource dictionary. This is not a normed
    /// color space, meaning the exact look depends on the device.
    ///
    /// Writing `cs` with this is equivalent to writing `g`.
    DeviceGray,
    /// The predefined, parameterless RGB device-dependent color space. Needs no
    /// further entries in the resource dictionary.
    ///
    /// Writing `cs` with this is equivalent to writing `rg`.
    DeviceRgb,
    /// The predefined, parameterless CMYK device-dependent color space. Needs
    /// no further entries in the resource dictionary.
    ///
    /// Writing `cs` with this is equivalent to writing `k`.
    DeviceCmyk,
    /// A pattern with color defined by the pattern itself.
    ///
    /// When writing a `cs` operation with this, you must also write an `scn`
    /// operation with a name pointing to an entry in the current resource
    /// dictionary's [pattern dictionary](Resources::patterns).
    Pattern,
    /// A named color space defined in the current resource dictionary's [color
    /// space dictionary](Resources::color_spaces).
    ///
    /// When this points to a pattern color space, You must also write an `scn`
    /// operation with a name pointing to an entry in the current resource
    /// dictionary's [pattern dictionary](Resources::patterns). The color will
    /// be taken from the `tint` passed to `SCN` and not the pattern itself.
    Named(Name<'a>),
}

impl<'a> ColorSpaceOperand<'a> {
    #[inline]
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::DeviceRgb => Name(b"DeviceRGB"),
            Self::DeviceCmyk => Name(b"DeviceCMYK"),
            Self::Pattern => Name(b"Pattern"),
            Self::Named(name) => name,
        }
    }
}

impl<'a> From<Name<'a>> for ColorSpaceOperand<'a> {
    fn from(name: Name<'a>) -> Self {
        Self::Named(name)
    }
}

/// Shading patterns.
impl Content {
    /// `sh`: Fill the whole drawing area with the specified shading.
    #[inline]
    pub fn shading(&mut self, shading: Name) -> &mut Self {
        self.op("sh").operand(shading);
        self
    }
}

// TODO: Inline images. Also check clause 6.1.10 of PDF/A-2 spec.

/// XObjects.
impl Content {
    /// `Do`: Write an external object.
    #[inline]
    pub fn x_object(&mut self, name: Name) -> &mut Self {
        self.op("Do").operand(name);
        self
    }
}

/// Marked Content.
impl Content {
    /// `MP`: Write a marked-content point. PDF 1.2+.
    #[inline]
    pub fn marked_content_point(&mut self, tag: Name) -> &mut Self {
        self.op("MP").operand(tag);
        self
    }

    /// `DP`: Start writing a marked-content point operation. PDF 1.2+.
    #[inline]
    pub fn marked_content_point_with_properties(&mut self, tag: Name) -> MarkContent<'_> {
        let mut op = self.op("DP");
        op.operand(tag);
        MarkContent::start(op)
    }

    /// `BMC`: Begin a marked-content sequence. PDF 1.2+.
    #[inline]
    pub fn begin_marked_content(&mut self, tag: Name) -> &mut Self {
        self.op("BMC").operand(tag);
        self
    }

    /// `BDC`: Start writing a "begin marked content" operation. PDF 1.2+.
    #[inline]
    pub fn begin_marked_content_with_properties(&mut self, tag: Name) -> MarkContent<'_> {
        let mut op = self.op("BDC");
        op.operand(tag);
        MarkContent::start(op)
    }

    /// `EMC`: End a marked-content sequence. PDF 1.2+.
    #[inline]
    pub fn end_marked_content(&mut self) -> &mut Self {
        self.op("EMC");
        self
    }
}

/// Writer for a _begin marked content operation_. PDF 1.3+.
pub struct MarkContent<'a> {
    op: Operation<'a>,
}

impl<'a> MarkContent<'a> {
    #[inline]
    pub(crate) fn start(op: Operation<'a>) -> Self {
        Self { op }
    }

    /// Start writing this marked content's property list. Mutually exclusive
    /// with [`properties_named`](Self::properties_named).
    #[inline]
    pub fn properties(&mut self) -> PropertyList<'_> {
        self.op.obj().start()
    }

    /// Reference a property list from the Resource dictionary. These property
    /// lists can be written using the [`Resources::properties`] method.
    /// Mutually exclusive with [`properties`](Self::properties).
    #[inline]
    pub fn properties_named(mut self, name: Name) {
        self.op.operand(name);
    }
}

deref!('a, MarkContent<'a> => Operation<'a>, op);

/// Writer for _property list dictionary_. Can be used as a generic dictionary.
/// PDF 1.3+.
pub struct PropertyList<'a> {
    dict: Dict<'a>,
}

writer!(PropertyList: |obj| Self { dict: obj.dict() });

impl<'a> PropertyList<'a> {
    /// Write the `/MCID` marked content identifier.
    #[inline]
    pub fn identify(&mut self, identifier: i32) -> &mut Self {
        self.pair(Name(b"MCID"), identifier);
        self
    }

    /// Write the `/ActualText` attribute to indicate the text replacement of
    /// this marked content sequence. PDF 1.5+.
    #[inline]
    pub fn actual_text(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"ActualText"), text);
        self
    }

    /// Start writing artifact property list. The tag of the marked content
    /// operation must have been `/Artifact`. PDF 1.4+.
    #[inline]
    pub fn artifact(self) -> Artifact<'a> {
        Artifact::start_with_dict(self.dict)
    }
}

deref!('a, PropertyList<'a> => Dict<'a>, dict);

/// Writer for an _actifact property list dictionary_. PDF 1.4+.
///
/// Required for marking up pagination artifacts in some PDF/A profiles.
pub struct Artifact<'a> {
    dict: Dict<'a>,
}

writer!(Artifact: |obj| Self::start_with_dict(obj.dict()));

impl<'a> Artifact<'a> {
    #[inline]
    pub(crate) fn start_with_dict(dict: Dict<'a>) -> Self {
        Self { dict }
    }

    /// Write the `/Type` entry to set the type of artifact. Specific to
    /// artifacts. PDF 1.4+.
    #[inline]
    pub fn kind(&mut self, kind: ArtifactType) -> &mut Self {
        self.pair(Name(b"Type"), kind.to_name());
        self
    }

    /// Write the `/Subtype` entry to set the subtype of pagination or inline
    /// artifacts. Specific to artifacts. PDF 1.7+.
    #[inline]
    pub fn subtype(&mut self, subtype: ArtifactSubtype) -> &mut Self {
        self.pair(Name(b"Subtype"), subtype.to_name());
        self
    }

    /// Write the `/BBox` entry to set the bounding box of the artifact.
    /// Specific to artifacts. Required for background artifacts. PDF 1.4+.
    #[inline]
    pub fn bounding_box(&mut self, bbox: Rect) -> &mut Self {
        self.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/Attached` entry to set where the artifact is attached to the
    /// page. Only for pagination and full-page background artifacts. Specific
    /// to artifacts. PDF 1.4+.
    #[inline]
    pub fn attached(
        &mut self,
        attachment: impl IntoIterator<Item = ArtifactAttachment>,
    ) -> &mut Self {
        self.insert(Name(b"Attached"))
            .array()
            .typed()
            .items(attachment.into_iter().map(ArtifactAttachment::to_name));
        self
    }
}

deref!('a, Artifact<'a> => Dict<'a>, dict);

/// The various types of layout artifacts.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArtifactType {
    /// Artifacts of the pagination process like headers, footers, page numbers.
    Pagination,
    /// Artifacts of the layout process such as footnote rules.
    Layout,
    /// Artifacts of the page, like printer's marks.
    Page,
    /// Background image artifacts. PDF 1.7+.
    Background,
    /// Artefacts related to inline content, such as line numbers or redactions. PDF 2.0+.
    Inline,
}

impl ArtifactType {
    #[inline]
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Pagination => Name(b"Pagination"),
            Self::Layout => Name(b"Layout"),
            Self::Page => Name(b"Page"),
            Self::Background => Name(b"Background"),
            Self::Inline => Name(b"Inline"),
        }
    }
}

/// The various subtypes of pagination artifacts.
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactSubtype<'a> {
    /// Headers.
    Header,
    /// Footers.
    Footer,
    /// Background watermarks.
    Watermark,
    /// Page numbers. PDF 2.0+
    PageNumber,
    /// Bates numbering. PDF 2.0+
    Bates,
    /// Line numbers. PDF 2.0+
    LineNumber,
    /// Redactions. PDF 2.0+
    Redaction,
    /// Custom subtype named according to ISO 32000-1:2008 Annex E.
    Custom(Name<'a>),
}

impl<'a> ArtifactSubtype<'a> {
    #[inline]
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::Header => Name(b"Header"),
            Self::Footer => Name(b"Footer"),
            Self::Watermark => Name(b"Watermark"),
            Self::PageNumber => Name(b"PageNum"),
            Self::Bates => Name(b"Bates"),
            Self::LineNumber => Name(b"LineNum"),
            Self::Redaction => Name(b"Redaction"),
            Self::Custom(name) => name,
        }
    }
}

/// Where a layout [`Artifact`] is attached to the page edge.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum ArtifactAttachment {
    Left,
    Top,
    Right,
    Bottom,
}

impl ArtifactAttachment {
    #[inline]
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Left => Name(b"Left"),
            Self::Top => Name(b"Top"),
            Self::Right => Name(b"Right"),
            Self::Bottom => Name(b"Bottom"),
        }
    }
}

/// Compatibility.
impl Content {
    /// `BX`: Begin a compatibility section.
    #[inline]
    pub fn begin_compat(&mut self) -> &mut Self {
        self.op("BX");
        self
    }

    /// `EX`: End a compatibility section.
    #[inline]
    pub fn end_compat(&mut self) -> &mut Self {
        self.op("EX");
        self
    }
}

/// Writer for a _resource dictionary_.
///
/// This struct is created by [`Pages::resources`], [`Page::resources`],
/// [`FormXObject::resources`], and [`TilingPattern::resources`].
pub struct Resources<'a> {
    dict: Dict<'a>,
}

writer!(Resources: |obj| Self { dict: obj.dict() });

impl Resources<'_> {
    /// Start writing the `/XObject` dictionary.
    ///
    /// Relevant types:
    /// - [`ImageXObject`]
    /// - [`FormXObject`]
    pub fn x_objects(&mut self) -> Dict<'_> {
        self.insert(Name(b"XObject")).dict()
    }

    /// Start writing the `/Font` dictionary.
    ///
    /// Relevant types:
    /// - [`Type1Font`]
    /// - [`Type3Font`]
    /// - [`Type0Font`]
    pub fn fonts(&mut self) -> Dict<'_> {
        self.insert(Name(b"Font")).dict()
    }

    /// Start writing the `/ColorSpace` dictionary. PDF 1.1+.
    ///
    /// Relevant types:
    /// - [`ColorSpace`]
    pub fn color_spaces(&mut self) -> Dict<'_> {
        self.insert(Name(b"ColorSpace")).dict()
    }

    /// Start writing the `/Pattern` dictionary. PDF 1.2+.
    ///
    /// Relevant types:
    /// - [`TilingPattern`]
    /// - [`ShadingPattern`]
    pub fn patterns(&mut self) -> Dict<'_> {
        self.insert(Name(b"Pattern")).dict()
    }

    /// Start writing the `/Shading` dictionary. PDF 1.3+.
    ///
    /// Relevant types:
    /// - [`FunctionShading`]
    pub fn shadings(&mut self) -> Dict<'_> {
        self.insert(Name(b"Shading")).dict()
    }

    /// Start writing the `/ExtGState` dictionary. PDF 1.2+.
    ///
    /// Relevant types:
    /// - [`ExtGraphicsState`]
    pub fn ext_g_states(&mut self) -> Dict<'_> {
        self.insert(Name(b"ExtGState")).dict()
    }

    /// Write the `/ProcSet` attribute.
    ///
    /// This defines what procedure sets are sent to an output device when
    /// printing the file as PostScript. The attribute is only used for PDFs
    /// with versions below 1.4.
    pub fn proc_sets(&mut self, sets: impl IntoIterator<Item = ProcSet>) -> &mut Self {
        self.insert(Name(b"ProcSet"))
            .array()
            .items(sets.into_iter().map(ProcSet::to_name));
        self
    }

    /// Write the `/ProcSet` attribute with all available procedure sets.
    ///
    /// The PDF 1.7 specification recommends that modern PDFs either omit the
    /// attribute or specify all available procedure sets, as this function
    /// does.
    pub fn proc_sets_all(&mut self) -> &mut Self {
        self.proc_sets([
            ProcSet::Pdf,
            ProcSet::Text,
            ProcSet::ImageGrayscale,
            ProcSet::ImageColor,
            ProcSet::ImageIndexed,
        ])
    }

    /// Start writing the `/Properties` attribute.
    ///
    /// This allows to write property lists with indirect objects for
    /// marked-content sequences. These properties can be used by property lists
    /// using the [`MarkContent::properties_named`] method. PDF 1.2+.
    pub fn properties(&mut self) -> TypedDict<'_, PropertyList> {
        self.insert(Name(b"Properties")).dict().typed()
    }
}

deref!('a, Resources<'a> => Dict<'a>, dict);

/// What procedure sets to send to a PostScript printer or other output device.
///
/// This enumeration provides compatibility for printing PDFs of versions 1.3 and
/// below.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ProcSet {
    /// Painting and graphics state.
    Pdf,
    /// Text.
    Text,
    /// Grayscale images and masks.
    ImageGrayscale,
    /// Color images.
    ImageColor,
    /// Images with color tables.
    ImageIndexed,
}

impl ProcSet {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            ProcSet::Pdf => Name(b"PDF"),
            ProcSet::Text => Name(b"Text"),
            ProcSet::ImageGrayscale => Name(b"ImageB"),
            ProcSet::ImageColor => Name(b"ImageC"),
            ProcSet::ImageIndexed => Name(b"ImageI"),
        }
    }
}

/// Writer for a _dictionary with additional parameters for the graphics state._
///
/// This struct is created by [`Chunk::ext_graphics`] and
/// [`ShadingPattern::ext_graphics`].
pub struct ExtGraphicsState<'a> {
    dict: Dict<'a>,
}

writer!(ExtGraphicsState: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"ExtGState"));
    Self { dict }
});

impl ExtGraphicsState<'_> {
    /// Write the `LW` attribute to set the line width. PDF 1.3+.
    pub fn line_width(&mut self, width: f32) -> &mut Self {
        self.pair(Name(b"LW"), width);
        self
    }

    /// Write the `LC` attribute to set the line cap style. PDF 1.3+.
    pub fn line_cap(&mut self, cap: LineCapStyle) -> &mut Self {
        self.pair(Name(b"LC"), cap.to_int());
        self
    }

    /// Write the `LJ` attribute to set the line join style. PDF 1.3+.
    pub fn line_join(&mut self, join: LineJoinStyle) -> &mut Self {
        self.pair(Name(b"LJ"), join.to_int());
        self
    }

    /// Write the `ML` attribute to set the miter limit. PDF 1.3+.
    pub fn miter_limit(&mut self, limit: f32) -> &mut Self {
        self.pair(Name(b"ML"), limit);
        self
    }

    /// Write the `D` attribute to set the dash pattern. PDF 1.3+.
    pub fn dash_pattern(
        &mut self,
        pattern: impl IntoIterator<Item = f32>,
        phase: f32,
    ) -> &mut Self {
        let mut array = self.insert(Name(b"D")).array();
        array.push().array().items(pattern);
        array.item(phase);
        array.finish();
        self
    }

    /// Write the `RI` attribute to set the rendering intent. PDF 1.3+.
    pub fn rendering_intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.pair(Name(b"RI"), intent.to_name());
        self
    }

    /// Write the `OP` attribute to set the overprint mode for all operations,
    /// except if an `op` entry is present. If so, only influence the stroking
    /// operations. PDF 1.2+.
    pub fn overprint(&mut self, overprint: bool) -> &mut Self {
        self.pair(Name(b"OP"), overprint);
        self
    }

    /// Write the `op` attribute to set the overprint mode for fill operations.
    /// PDF 1.3+.
    pub fn overprint_fill(&mut self, overprint: bool) -> &mut Self {
        self.pair(Name(b"op"), overprint);
        self
    }

    /// Write the `OPM` attribute to set the overprint mode for components that
    /// have been zeroed out. PDF 1.3+.
    ///
    /// Note that this attribute is restricted by PDF/A.
    pub fn overprint_mode(&mut self, mode: OverprintMode) -> &mut Self {
        self.pair(Name(b"OPM"), mode.to_int());
        self
    }

    /// Write the `Font` attribute to set the font. PDF 1.3+.
    pub fn font(&mut self, font: Name, size: f32) -> &mut Self {
        let mut array = self.insert(Name(b"Font")).array();
        array.item(font);
        array.item(size);
        array.finish();
        self
    }

    /// Write the `BG` attribute to set the black generation function.
    pub fn black_generation(&mut self, func: Ref) -> &mut Self {
        self.pair(Name(b"BG"), func);
        self
    }

    /// Write the `BG2` attribute to set the black-generation function back to
    /// the function that has been in effect at the beginning of the page. PDF
    /// 1.3+.
    pub fn black_generation_default(&mut self) -> &mut Self {
        self.pair(Name(b"BG2"), Name(b"Default"));
        self
    }

    /// Write the `UCR` attribute to set the undercolor removal function.
    pub fn undercolor_removal(&mut self, func: Ref) -> &mut Self {
        self.pair(Name(b"UCR"), func);
        self
    }

    /// Write the `UCR2` attribute to set the undercolor removal function back
    /// to the function that has been in effect at the beginning of the page.
    /// PDF 1.3+.
    pub fn undercolor_removal_default(&mut self) -> &mut Self {
        self.pair(Name(b"UCR2"), Name(b"Default"));
        self
    }

    /// Write the `TR` attribute to set the transfer function.
    ///
    /// Note that this key is illegal in PDF/A.
    pub fn transfer(&mut self, func: Ref) -> &mut Self {
        self.pair(Name(b"TR"), func);
        self
    }

    /// Write the `TR2` attribute to set the transfer function back to the
    /// function that has been in effect at the beginning of the page. PDF 1.3+.
    pub fn transfer_default(&mut self) -> &mut Self {
        self.pair(Name(b"TR2"), Name(b"Default"));
        self
    }

    /// Write the `HT` attribute to set the halftone.
    ///
    /// Note that this value may be ignored in PDF/A.
    pub fn halftone(&mut self, ht: Ref) -> &mut Self {
        self.pair(Name(b"HT"), ht);
        self
    }

    /// Write the `HT` attribute to set the halftone back to the one that has
    /// been in effect at the beginning of the page.
    pub fn halftone_default(&mut self) -> &mut Self {
        self.pair(Name(b"HT"), Name(b"Default"));
        self
    }

    /// Write the `FL` attribute to set the flatness tolerance. PDF 1.3+.
    ///
    /// Note that this key may be ignored in PDF/A.
    pub fn flatness(&mut self, tolerance: f32) -> &mut Self {
        self.pair(Name(b"FL"), tolerance);
        self
    }

    /// Write the `SM` attribute to set the smoothness tolerance. PDF 1.3+.
    pub fn smoothness(&mut self, tolerance: f32) -> &mut Self {
        self.pair(Name(b"SM"), tolerance);
        self
    }

    /// Write the `SA` attribute to set automatic stroke adjustment.
    pub fn stroke_adjustment(&mut self, adjust: bool) -> &mut Self {
        self.pair(Name(b"SA"), adjust);
        self
    }

    /// Write the `BM` attribute to set the blend mode. PDF 1.4+.
    ///
    /// Note that this key is restricted in PDF/A-1.
    pub fn blend_mode(&mut self, mode: BlendMode) -> &mut Self {
        self.pair(Name(b"BM"), mode.to_name());
        self
    }

    /// Start writing the `SMask` attribute. PDF 1.4+.
    ///
    /// Note that this key is forbidden in PDF/A-1.
    pub fn soft_mask(&mut self) -> SoftMask<'_> {
        self.insert(Name(b"SMask")).start()
    }

    /// Write the `SMask` attribute using a name. PDF 1.4+.
    ///
    /// Note that this key is forbidden in PDF/A-1.
    pub fn soft_mask_name(&mut self, mask: Name) -> &mut Self {
        self.pair(Name(b"SMask"), mask);
        self
    }

    /// Write the `CA` attribute to set the stroking alpha constant. PDF 1.4+.
    ///
    /// Note that this key is restricted in PDF/A-1.
    pub fn stroking_alpha(&mut self, alpha: f32) -> &mut Self {
        self.pair(Name(b"CA"), alpha);
        self
    }

    /// Write the `ca` attribute to set the non-stroking alpha constant. PDF
    /// 1.4+.
    ///
    /// Note that this key is restricted in PDF/A-1.
    pub fn non_stroking_alpha(&mut self, alpha: f32) -> &mut Self {
        self.pair(Name(b"ca"), alpha);
        self
    }

    /// Write the `AIS` attribute to set the alpha source flag. `CA` and `ca`
    /// values as well as the `SMask` will be interpreted as shape instead of
    /// opacity. PDF 1.4+.
    pub fn alpha_source(&mut self, source: bool) -> &mut Self {
        self.pair(Name(b"AIS"), source);
        self
    }

    /// Write the `TK` attribute to set the text knockout flag. PDF 1.4+.
    pub fn text_knockout(&mut self, knockout: bool) -> &mut Self {
        self.pair(Name(b"TK"), knockout);
        self
    }
}

deref!('a, ExtGraphicsState<'a> => Dict<'a>, dict);

/// How to blend source and backdrop.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl BlendMode {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            BlendMode::Normal => Name(b"Normal"),
            BlendMode::Multiply => Name(b"Multiply"),
            BlendMode::Screen => Name(b"Screen"),
            BlendMode::Overlay => Name(b"Overlay"),
            BlendMode::Darken => Name(b"Darken"),
            BlendMode::Lighten => Name(b"Lighten"),
            BlendMode::ColorDodge => Name(b"ColorDodge"),
            BlendMode::ColorBurn => Name(b"ColorBurn"),
            BlendMode::HardLight => Name(b"HardLight"),
            BlendMode::SoftLight => Name(b"SoftLight"),
            BlendMode::Difference => Name(b"Difference"),
            BlendMode::Exclusion => Name(b"Exclusion"),
            BlendMode::Hue => Name(b"Hue"),
            BlendMode::Saturation => Name(b"Saturation"),
            BlendMode::Color => Name(b"Color"),
            BlendMode::Luminosity => Name(b"Luminosity"),
        }
    }
}

/// How to behave when overprinting for colorants with the value zero.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum OverprintMode {
    /// An overprint operation will always discard the underlying color, even if
    /// one of the colorants is zero.
    OverrideAllColorants,
    /// An overprint operation will only discard the underlying colorant
    /// component (e.g. cyan in CMYK) if the new corresponding colorant is
    /// non-zero.
    ///
    /// Note that this value is forbidden by PDF/A for ICCBased color spaces
    /// when overprinting is enabled.
    IgnoreZeroChannel,
}

impl OverprintMode {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            OverprintMode::OverrideAllColorants => 0,
            OverprintMode::IgnoreZeroChannel => 1,
        }
    }
}

/// Writer for a _soft mask dictionary_.
///
/// This struct is created by [`ExtGraphicsState::soft_mask`].
pub struct SoftMask<'a> {
    dict: Dict<'a>,
}

writer!(SoftMask: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Mask"));
    Self { dict }
});

impl SoftMask<'_> {
    /// Write the `S` attribute to set the soft mask subtype. Required.
    pub fn subtype(&mut self, subtype: MaskType) -> &mut Self {
        self.pair(Name(b"S"), subtype.to_name());
        self
    }

    /// Write the `G` attribute to set the transparency group XObject. The group
    /// has to have a color space set in the `/CS` attribute if the mask subtype
    /// is `Luminosity`. Required.
    pub fn group(&mut self, group: Ref) -> &mut Self {
        self.pair(Name(b"G"), group);
        self
    }

    /// Write the `BC` attribute to set the background color for the
    /// transparency group. Only applicable if the mask subtype is `Luminosity`.
    /// Has to be set in the group's color space.
    pub fn backdrop(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"BC")).array().items(color);
        self
    }

    /// Write the `TR` attribute, a function that maps from the group's output
    /// values to the mask opacity.
    pub fn transfer_function(&mut self, function: Ref) -> &mut Self {
        self.pair(Name(b"TR"), function);
        self
    }
}

deref!('a, SoftMask<'a> => Dict<'a>, dict);

/// What property in the mask influences the target alpha.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MaskType {
    /// The alpha values from the mask are applied to the target.
    Alpha,
    /// A single-channel luminosity value is calculated for the colors in the
    /// mask.
    Luminosity,
}

impl MaskType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            MaskType::Alpha => Name(b"Alpha"),
            MaskType::Luminosity => Name(b"Luminosity"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_encoding() {
        let mut content = Content::new();
        content
            .save_state()
            .rect(1.0, 2.0, 3.0, 4.0)
            .fill_nonzero()
            .set_dash_pattern([7.0, 2.0], 4.0)
            .x_object(Name(b"MyImage"))
            .set_fill_pattern([2.0, 3.5], Name(b"MyPattern"))
            .restore_state();

        assert_eq!(
            content.finish().into_vec(),
            b"q\n1 2 3 4 re\nf\n[7 2] 4 d\n/MyImage Do\n2 3.5 /MyPattern scn\nQ"
        );
    }

    #[test]
    fn test_content_text() {
        let mut content = Content::new();

        content.set_font(Name(b"F1"), 12.0);
        content.begin_text();
        content.show_positioned().items();
        content
            .show_positioned()
            .items()
            .show(Str(b"AB"))
            .adjust(2.0)
            .show(Str(b"CD"));
        content.end_text();

        assert_eq!(
            content.finish().into_vec(),
            b"/F1 12 Tf\nBT\n[] TJ\n[(AB) 2 (CD)] TJ\nET"
        );
    }
}
