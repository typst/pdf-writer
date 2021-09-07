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
/// This struct is created by [`Content::op`] and [`Text::op`].
pub struct Operation<'a> {
    buf: &'a mut Vec<u8>,
    op: &'a str,
    first: bool,
}

impl<'a> Operation<'a> {
    fn new(buf: &'a mut Vec<u8>, op: &'a str) -> Self {
        Self { buf, op, first: true }
    }

    /// Write a primitive operand.
    pub fn operand<T: Primitive>(&mut self, value: T) -> &mut Self {
        self.obj().primitive(value);
        self
    }

    /// Write a sequence of primitive operands.
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
    pub fn obj(&mut self) -> Obj<'_> {
        if !self.first {
            self.buf.push(b' ');
        }
        self.first = false;
        Obj::direct(&mut self.buf, 0)
    }
}

impl Drop for Operation<'_> {
    fn drop(&mut self) {
        if !self.first {
            self.buf.push(b' ');
        }
        self.buf.push_bytes(self.op.as_bytes());
        self.buf.push(b'\n');
    }
}

/// State.
impl Content {
    /// `q`: Save the graphics state on the stack.
    pub fn save_state(&mut self) -> &mut Self {
        self.op("q");
        self
    }

    /// `Q`: Restore the graphics state from the stack.
    pub fn restore_state(&mut self) -> &mut Self {
        self.op("Q");
        self
    }

    /// `cm`: Modify the current transformation matrix.
    pub fn set_matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.op("cm").operands(matrix);
        self
    }

    /// `w`: Set the stroke line width.
    ///
    /// Panics if `width` is negative.
    pub fn set_line_width(&mut self, width: f32) -> &mut Self {
        assert!(width >= 0.0, "line width must be positive");
        self.op("w").operand(width);
        self
    }

    /// `J`: Set the line cap style.
    pub fn set_line_cap(&mut self, cap: LineCapStyle) -> &mut Self {
        self.buf.push_val(cap.to_int());
        self.buf.push_bytes(b" J\n");
        self
    }
}

/// Color.
impl Content {
    /// `rg`: Set the fill color to the parameter and the color space to
    /// `DeviceRGB`.
    pub fn set_fill_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op("rg").operands([r, g, b]);
        self
    }

    /// `k`: Set the fill color to the parameter and the color space to
    /// `DeviceCMYK`.
    pub fn set_fill_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op("k").operands([c, m, y, k]);
        self
    }

    /// `cs`: Set the fill color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    pub fn set_fill_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.op("cs").operand(space.to_name());
        self
    }

    /// `scn`: Set the fill color to the parameter within the current color
    /// space. PDF 1.2+.
    pub fn set_fill_color(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        self.op("scn").operands(color);
        self
    }

    /// `scn`: Set the fill pattern. PDF 1.2+.
    ///
    /// The `name` parameter is the name of a pattern. If this is an uncolored
    /// pattern, a tint color in the current `Pattern` base color space must be
    /// given, otherwise, the `color` iterator shall remain empty.
    pub fn set_fill_pattern(
        &mut self,
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("scn").operands(color).operand(name);
        self
    }

    /// `RG`: Set the stroke color to the parameter and the color space to
    /// `DeviceRGB`.
    pub fn set_stroke_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.op("RG").operands([r, g, b]);
        self
    }

    /// `K`: Set the stroke color to the parameter and the color space to
    /// `DeviceCMYK`.
    pub fn set_stroke_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.op("K").operands([c, m, y, k]);
        self
    }

    /// `CS`: Set the stroke color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    pub fn set_stroke_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.op("CS").operand(space.to_name());
        self
    }

    /// `SCN`: Set the stroke color to the parameter within the current color
    /// space. PDF 1.2+.
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
    pub fn set_stroke_pattern(
        &mut self,
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        self.op("SCN").operands(color).operand(name);
        self
    }

    /// `ri`: Set the color rendering intent to the parameter. PDF 1.1+.
    pub fn set_rendering_intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.op("ri").operand(intent.to_name());
        self
    }
}

/// Path construction.
impl Content {
    /// `m`: Begin a new subpath at (x, y).
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("m").operands([x, y]);
        self
    }

    /// `l`: Append a straight line to (x, y).
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("l").operands([x, y]);
        self
    }

    /// `c`: Append a cubic Bézier segment to (x3, y3) with (x1, y1), (x2, y2)
    /// as control points.
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
    pub fn cubic_to_initial(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.op("v").operands([x2, y2, x3, y3]);
        self
    }

    /// `y`: Append a cubic Bézier segment to (x3, y3) with (x1, y1) as control
    /// point.
    pub fn cubic_to_final(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) -> &mut Self {
        self.op("y").operands([x1, y1, x3, y3]);
        self
    }

    /// `h`: Close the current subpath with a straight line.
    pub fn close_path(&mut self) -> &mut Self {
        self.op("h");
        self
    }

    /// `re`: Append a rectangle to the current path.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) -> &mut Self {
        self.op("re").operands([x, y, width, height]);
        self
    }
}

/// Path painting.
impl Content {
    /// `S`: Stroke the current path.
    pub fn stroke(&mut self) -> &mut Self {
        self.op("S");
        self
    }

    /// `s`: Close and stroke the current path.
    pub fn close_and_stroke(&mut self) -> &mut Self {
        self.op("s");
        self
    }

    /// `f`: Fill the current path using the nonzero winding rule.
    pub fn fill_nonzero(&mut self) -> &mut Self {
        self.op("f");
        self
    }

    /// `f*`: Fill the current path using the even-odd rule.
    pub fn fill_even_odd(&mut self) -> &mut Self {
        self.op("f*");
        self
    }

    /// `B`: Fill and then stroke the current path using the nonzero winding
    /// rule.
    pub fn fill_and_stroke_nonzero(&mut self) -> &mut Self {
        self.op("B");
        self
    }

    /// `B*`: Fill and then stroke the current path using the even-odd rule.
    pub fn fill_and_stroke_even_odd(&mut self) -> &mut Self {
        self.op("B*");
        self
    }

    /// `b`: Close, fill and then stroke the current path using the nonzero
    /// winding rule.
    pub fn close_fill_and_stroke_nonzero(&mut self) -> &mut Self {
        self.op("b");
        self
    }

    /// `b*`: Close, fill and then stroke the current path using the even-odd
    /// rule.
    pub fn close_fill_and_stroke_even_odd(&mut self) -> &mut Self {
        self.op("b*");
        self
    }

    /// `n`: End the current path without filling or stroking it.
    ///
    /// This is primarily used for clipping paths.
    pub fn end_path(&mut self) -> &mut Self {
        self.op("n");
        self
    }

    /// `W`: Intersect the current clipping path with the current path using the
    /// nonzero winding rule.
    pub fn clip_nonzero(&mut self) -> &mut Self {
        self.op("W");
        self
    }

    /// `W*`: Intersect the current clipping path with the current path using
    /// the even-odd rule.
    pub fn clip_even_odd(&mut self) -> &mut Self {
        self.op("W*");
        self
    }
}

/// Other objects.
impl Content {
    /// `BT ... ET`: Start writing a text object.
    pub fn text(&mut self) -> Text<'_> {
        Text::new(self)
    }

    /// `Do`: Write an external object.
    pub fn x_object(&mut self, name: Name) -> &mut Self {
        self.op("Do").operand(name);
        self
    }

    /// `sh`: Fill the whole drawing area with the specified shading.
    pub fn shading(&mut self, shading: Name) -> &mut Self {
        self.op("sh").operand(shading);
        self
    }
}

/// Writer for a _text object_ in a content stream.
///
/// This struct is created by [`Content::text`].
pub struct Text<'a> {
    buf: &'a mut Vec<u8>,
}

impl<'a> Text<'a> {
    fn new(content: &'a mut Content) -> Self {
        content.op("BT");
        let buf = &mut content.buf;
        Self { buf }
    }

    /// Start writing an arbitrary operation.
    pub fn op<'b>(&'b mut self, operator: &'b str) -> Operation<'b> {
        Operation::new(self.buf, operator)
    }

    /// `Tf`: Set font and font size.
    pub fn font(&mut self, font: Name, size: f32) -> &mut Self {
        self.op("Tf").operand(font).operand(size);
        self
    }

    /// `Td`: Move to the start of the next line.
    pub fn next_line(&mut self, x: f32, y: f32) -> &mut Self {
        self.op("Td").operands([x, y]);
        self
    }

    /// `Tm`: Set the text matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.op("Tm").operands(matrix);
        self
    }

    /// `Tj`: Show text.
    ///
    /// The encoding of the text is up to you.
    pub fn show(&mut self, text: Str) -> &mut Self {
        self.op("Tj").operand(text);
        self
    }

    /// `TJ`: Show text with individual glyph positioning.
    pub fn show_positioned(&mut self) -> ShowPositioned<'_> {
        ShowPositioned::new(self.op("TJ"))
    }
}

impl Drop for Text<'_> {
    fn drop(&mut self) {
        self.op("ET");
    }
}

/// Writer for an _individual glyph positioning operation_.
///
/// This struct is created by [`Text::show_positioned`].
pub struct ShowPositioned<'a> {
    op: Operation<'a>,
}

impl<'a> ShowPositioned<'a> {
    fn new(op: Operation<'a>) -> Self {
        Self { op }
    }

    /// Write the array of strings and adjustments. Required.
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
    fn new(obj: Obj<'a>) -> Self {
        Self { array: obj.array() }
    }

    /// Show a continous string without adjustments.
    ///
    /// The encoding of the text is up to you.
    pub fn show(&mut self, text: Str) -> &mut Self {
        self.array.item(text);
        self
    }

    /// Specify an adjustment between two glyphs.
    ///
    /// The `amount` is specified in thousands of units of text space and is
    /// subtracted from the current writing-mode dependent coordinate.
    pub fn adjust(&mut self, amount: f32) -> &mut Self {
        self.array.item(amount);
        self
    }
}

deref!('a, PositionedItems<'a> => Array<'a>, array);

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
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::ButtCap => 0,
            Self::RoundCap => 1,
            Self::ProjectingSquareCap => 2,
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
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::AbsoluteColorimetric => Name(b"AbsoluteColorimetric"),
            Self::RelativeColorimetric => Name(b"RelativeColorimetric"),
            Self::Saturation => Name(b"Saturation"),
            Self::Perceptual => Name(b"Perceptual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_encoding() {
        let mut c = Content::new();
        c.save_state();
        c.rect(1.0, 2.0, 3.0, 4.0);
        c.fill_nonzero();
        c.x_object(Name(b"MyImage"));
        c.set_fill_pattern([2.0, 3.5], Name(b"MyPattern"));
        c.restore_state();
        assert_eq!(
            c.finish(),
            b"q\n1 2 3 4 re\nf\n/MyImage Do\n2 3.5 /MyPattern scn\nQ"
        );
    }

    #[test]
    fn test_content_text() {
        let mut c = Content::new();
        let mut t = c.text();
        t.font(Name(b"F1"), 12.0);
        t.show_positioned().items();
        t.show_positioned()
            .items()
            .show(Str(b"AB"))
            .adjust(2.0)
            .show(Str(b"CD"));
        t.finish();
        assert_eq!(c.finish(), b"BT\n/F1 12 Tf\n[] TJ\n[(AB) 2 (CD)] TJ\nET");
    }
}
