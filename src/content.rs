use super::*;
use crate::types::ColorSpace;

/// A builder for a content stream.
pub struct Content {
    buf: Vec<u8>,
}

/// Core methods.
impl Content {
    /// Create a new content stream with the default buffer capacity
    /// (currently 1 KB).
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new content stream with the specified initial buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Vec::with_capacity(capacity) }
    }

    /// Start writing an arbitrary operation.
    #[inline]
    pub fn op<'a>(&'a mut self, operator: &'a str) -> Operation<'a> {
        Operation::new(&mut self.buf, operator)
    }

    /// Return the raw constructed byte stream.
    pub fn finish(mut self) -> Vec<u8> {
        if self.buf.last() == Some(&b'\n') {
            self.buf.pop();
        }
        self.buf
    }
}

/// Writer for an _operation_ in a content stream.
///
/// This struct is created by [`Content::op`].
pub struct Operation<'a> {
    buf: &'a mut Vec<u8>,
    op: &'a str,
    first: bool,
}

impl<'a> Operation<'a> {
    #[inline]
    fn new(buf: &'a mut Vec<u8>, op: &'a str) -> Self {
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

    /// Write an an arbitrary object operand.
    #[inline]
    pub fn obj(&mut self) -> Obj<'_> {
        if !self.first {
            self.buf.push(b' ');
        }
        self.first = false;
        Obj::direct(&mut self.buf, 0)
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

/// Graphics state.
impl Content {
    /// `q`: Save the graphics state on the stack.
    #[inline]
    pub fn save_state(&mut self) -> &mut Self {
        self.op("q");
        self
    }

    /// `Q`: Restore the graphics state from the stack.
    #[inline]
    pub fn restore_state(&mut self) -> &mut Self {
        self.op("Q");
        self
    }

    /// `cm`: Concatenate the current transformation matrix with `matrix`.
    #[inline]
    pub fn concat_matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.op("cm").operands(matrix);
        self
    }

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
    pub fn set_line_dash_pattern(
        &mut self,
        array: impl IntoIterator<Item = f32>,
        phase: f32,
    ) -> &mut Self {
        let mut op = self.op("d");
        op.obj().array().typed().items(array);
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
            matches!(tolerance, 0 ..= 100),
            "flatness tolerance must be between 0 and 100",
        );
        self.op("i").operand(tolerance);
        self
    }

    /// `gs`: Set the parameters from an `EXTGState` dictionary. PDF 1.2+.
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

    /// `s`: Close and stroke the current path.
    #[inline]
    pub fn close_and_stroke(&mut self) -> &mut Self {
        self.op("s");
        self
    }

    /// `f`: Fill the current path using the nonzero winding rule.
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

    /// `B`: Fill and then stroke the current path using the nonzero winding
    /// rule.
    #[inline]
    pub fn fill_and_stroke_nonzero(&mut self) -> &mut Self {
        self.op("B");
        self
    }

    /// `B*`: Fill and then stroke the current path using the even-odd rule.
    #[inline]
    pub fn fill_and_stroke_even_odd(&mut self) -> &mut Self {
        self.op("B*");
        self
    }

    /// `b`: Close, fill and then stroke the current path using the nonzero
    /// winding rule.
    #[inline]
    pub fn close_fill_and_stroke_nonzero(&mut self) -> &mut Self {
        self.op("b");
        self
    }

    /// `b*`: Close, fill and then stroke the current path using the even-odd
    /// rule.
    #[inline]
    pub fn close_fill_and_stroke_even_odd(&mut self) -> &mut Self {
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
    /// nonzero winding rule.
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

    /// `T*`: Move to the start of the next line, determing the vertical offset
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

    /// `TJ`: Show text with individual glyph positioning.
    #[inline]
    pub fn show_positioned(&mut self) -> ShowPositioned<'_> {
        ShowPositioned::new(self.op("TJ"))
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
    fn new(op: Operation<'a>) -> Self {
        Self { op }
    }

    /// Write the array of strings and adjustments. Required.
    #[inline]
    pub fn items(&mut self) -> PositionedItems<'_> {
        PositionedItems::new(self.op.obj())
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
    fn new(obj: Obj<'a>) -> Self {
        Self { array: obj.array() }
    }

    /// Show a continous string without adjustments.
    ///
    /// The encoding of the text is up to you.
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

// TODO: Type 3 fonts.

/// Color.
impl Content {
    /// `CS`: Set the stroke color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    #[inline]
    pub fn set_stroke_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.op("CS").operand(space.to_name());
        self
    }

    /// `cs`: Set the fill color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    #[inline]
    pub fn set_fill_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.op("cs").operand(space.to_name());
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
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("SCN").operands(color).operand(name);
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
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("scn").operands(color).operand(name);
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

/// Shading patterns.
impl Content {
    /// `sh`: Fill the whole drawing area with the specified shading.
    #[inline]
    pub fn shading(&mut self, shading: Name) -> &mut Self {
        self.op("sh").operand(shading);
        self
    }
}

// TODO: Inline images.

/// XObjects.
impl Content {
    /// `Do`: Write an external object.
    #[inline]
    pub fn x_object(&mut self, name: Name) -> &mut Self {
        self.op("Do").operand(name);
        self
    }
}

// TODO: Marked content.

/// Compatibility.
impl Content {
    /// `BX`: Begin a compatability section.
    #[inline]
    pub fn begin_compat(&mut self) -> &mut Self {
        self.op("BX");
        self
    }

    /// `EX`: End a compatability section.
    #[inline]
    pub fn end_compat(&mut self) -> &mut Self {
        self.op("EX");
        self
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
            .set_line_dash_pattern([7.0, 2.0], 4.0)
            .x_object(Name(b"MyImage"))
            .set_fill_pattern([2.0, 3.5], Name(b"MyPattern"))
            .restore_state();

        assert_eq!(
            content.finish(),
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
            content.finish(),
            b"/F1 12 Tf\nBT\n[] TJ\n[(AB) 2 (CD)] TJ\nET"
        );
    }
}
