use super::*;

/// Builder for a content stream.
pub struct Content {
    buf: Vec<u8>,
}

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

    /// `q`: Save the graphics state on the stack.
    pub fn save_state(&mut self) -> &mut Self {
        self.buf.push_bytes(b"q\n");
        self
    }

    /// `Q`: Restore the graphics state from the stack.
    pub fn restore_state(&mut self) -> &mut Self {
        self.buf.push_bytes(b"Q\n");
        self
    }

    /// `cm`: Modify the transformation matrix.
    pub fn matrix(
        &mut self,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
        f: f32,
    ) -> &mut Self {
        self.buf.push_val(a);
        self.buf.push(b' ');
        self.buf.push_val(b);
        self.buf.push(b' ');
        self.buf.push_val(c);
        self.buf.push(b' ');
        self.buf.push_val(d);
        self.buf.push(b' ');
        self.buf.push_val(e);
        self.buf.push(b' ');
        self.buf.push_val(f);
        self.buf.push_bytes(b" cm\n");
        self
    }

    /// `w`: Set the stroke line width.
    ///
    /// Panics if width is negative.
    pub fn line_width(&mut self, width: f32) -> &mut Self {
        if width < 0.0 {
            panic!("width must be positive");
        }

        self.buf.push_val(width);
        self.buf.push_bytes(b" w\n");
        self
    }

    /// `J`: Set the line cap style.
    pub fn line_cap(&mut self, cap: LineCapStyle) -> &mut Self {
        self.buf.push_val(cap.to_int());
        self.buf.push_bytes(b" J\n");
        self
    }

    /// `rg`: Set the fill color to the parameter and the color space to
    /// `DeviceRGB`.
    pub fn fill_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.buf.push_val(r);
        self.buf.push(b' ');
        self.buf.push_val(g);
        self.buf.push(b' ');
        self.buf.push_val(b);
        self.buf.push_bytes(b" rg\n");
        self
    }

    /// `RG`: Set the stroke color to the parameter and the color space to
    /// `DeviceRGB`.
    pub fn stroke_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.buf.push_val(r);
        self.buf.push(b' ');
        self.buf.push_val(g);
        self.buf.push(b' ');
        self.buf.push_val(b);
        self.buf.push_bytes(b" RG\n");
        self
    }

    /// `k`: Set the fill color to the parameter and the color space to
    /// `DeviceCMYK`.
    pub fn fill_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.buf.push_val(c);
        self.buf.push(b' ');
        self.buf.push_val(m);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push(b' ');
        self.buf.push_val(k);
        self.buf.push_bytes(b" k\n");
        self
    }

    /// `K`: Set the stroke color to the parameter and the color space to
    /// `DeviceCMYK`.
    pub fn stroke_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.buf.push_val(c);
        self.buf.push(b' ');
        self.buf.push_val(m);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push(b' ');
        self.buf.push_val(k);
        self.buf.push_bytes(b" K\n");
        self
    }

    /// `cs`: Set the fill color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    pub fn fill_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.buf.push_val(space.to_name());
        self.buf.push_bytes(b" cs\n");
        self
    }

    /// `CS`: Set the stroke color space to the parameter. PDF 1.1+.
    ///
    /// The parameter must be the name of a parameter-less color space or of a
    /// color space dictionary within the current resource dictionary.
    pub fn stroke_color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.buf.push_val(space.to_name());
        self.buf.push_bytes(b" CS\n");
        self
    }

    /// `scn`: Set the fill color to the parameter within the current color
    /// space. PDF 1.2+.
    pub fn fill_color(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        for (i, val) in color.into_iter().enumerate() {
            if i != 0 {
                self.buf.push(b' ');
            }
            self.buf.push_val(val);
        }
        self.buf.push_bytes(b" scn\n");
        self
    }

    /// `SCN`: Set the stroke color to the parameter within the current color
    /// space. PDF 1.2+.
    pub fn stroke_color(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        for (i, val) in color.into_iter().enumerate() {
            if i != 0 {
                self.buf.push(b' ');
            }
            self.buf.push_val(val);
        }
        self.buf.push_bytes(b" SCN\n");
        self
    }
    /// `re`: Draw a rectangle.
    pub fn rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        stroke: bool,
        fill: bool,
    ) -> &mut Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push(b' ');
        self.buf.push_val(width);
        self.buf.push(b' ');
        self.buf.push_val(height);
        self.buf.push_bytes(b" re\n");
        terminate_path(&mut self.buf, stroke, fill);
        self
    }

    /// `... h / S / f / B`: Start drawing a bezier path.
    pub fn path(&mut self, stroke: bool, fill: bool) -> Path<'_> {
        Path::start(self, stroke, fill)
    }

    /// `BT ... ET`: Start writing a text object.
    pub fn text(&mut self) -> Text<'_> {
        Text::start(self)
    }

    /// `Do`: Write an external object.
    pub fn x_object(&mut self, name: Name) -> &mut Self {
        self.buf.push_val(name);
        self.buf.push_bytes(b" Do\n");
        self
    }

    /// Return the raw constructed byte stream.
    pub fn finish(mut self) -> Vec<u8> {
        if self.buf.last() == Some(&b'\n') {
            self.buf.pop();
        }
        self.buf
    }
}

/// Writer for a _text object_.
///
/// This struct is created by [`Content::text`].
pub struct Text<'a> {
    buf: &'a mut Vec<u8>,
}

impl<'a> Text<'a> {
    pub(crate) fn start(content: &'a mut Content) -> Self {
        let buf = &mut content.buf;
        buf.push_bytes(b"BT\n");
        Self { buf }
    }

    /// `Tf`: Set font and font size.
    pub fn font(&mut self, font: Name, size: f32) -> &mut Self {
        self.buf.push_val(font);
        self.buf.push(b' ');
        self.buf.push_val(size);
        self.buf.push_bytes(b" Tf\n");
        self
    }

    /// `Td`: Move to the start of the next line.
    pub fn next_line(&mut self, x: f32, y: f32) -> &mut Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push_bytes(b" Td\n");
        self
    }

    /// `Tm`: Set the text matrix.
    pub fn matrix(&mut self, values: [f32; 6]) -> &mut Self {
        for x in values {
            self.buf.push_val(x);
            self.buf.push(b' ');
        }
        self.buf.push_bytes(b"Tm\n");
        self
    }

    /// `Tj`: Show text.
    ///
    /// The encoding of the text is up to you.
    pub fn show(&mut self, text: Str) -> &mut Self {
        self.buf.push_val(text);
        self.buf.push_bytes(b" Tj\n");
        self
    }

    /// `TJ`: Show text with individual glyph positioning.
    pub fn show_positioned(&mut self) -> PositionedText<'_> {
        PositionedText::start(self)
    }
}

impl Drop for Text<'_> {
    fn drop(&mut self) {
        self.buf.push_bytes(b"ET\n");
    }
}

/// Writer for text with _individual glyph positioning_ in a text object.
///
/// This struct is created by [`Text::show_positioned`].
pub struct PositionedText<'a> {
    buf: &'a mut Vec<u8>,
    first: bool,
}

impl<'a> PositionedText<'a> {
    pub(crate) fn start(text: &'a mut Text) -> Self {
        let buf = &mut text.buf;
        buf.push(b'[');
        Self { buf, first: true }
    }

    /// Show a continous text string without adjustments.
    ///
    /// The encoding of the text is up to you.
    pub fn show(&mut self, text: Str) -> &mut Self {
        if !self.first {
            self.buf.push(b' ');
        }
        self.first = false;
        self.buf.push_val(text);
        self
    }

    /// Specify an adjustment between two glyphs.
    ///
    /// The `amount` is specified in thousands of units of text space and is
    /// subtracted from the current writing-mode dependent coordinate.
    pub fn adjust(&mut self, amount: f32) -> &mut Self {
        if !self.first {
            self.buf.push(b' ');
        }
        self.first = false;
        self.buf.push_val(amount);
        self
    }
}

impl Drop for PositionedText<'_> {
    fn drop(&mut self) {
        self.buf.push_bytes(b"] TJ\n");
    }
}

/// Writer for a _path_.
///
/// This struct is created by [`Content::path`].
pub struct Path<'a> {
    buf: &'a mut Vec<u8>,
    stroke: bool,
    fill: bool,
}

impl<'a> Path<'a> {
    /// Create a new path.
    pub(crate) fn start(content: &'a mut Content, stroke: bool, fill: bool) -> Self {
        Self { buf: &mut content.buf, stroke, fill }
    }

    /// `m`: Move to (x, y).
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push_bytes(b" m\n");
        self
    }

    /// `l`: Draw a straight line to (x, y).
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push_bytes(b" l\n");
        self
    }

    /// `c`: Create a cubic Bézier segment to (x3, y3) with (x1, y1), (x2, y2)
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
        self.buf.push_val(x1);
        self.buf.push(b' ');
        self.buf.push_val(y1);
        self.buf.push(b' ');
        self.buf.push_val(x2);
        self.buf.push(b' ');
        self.buf.push_val(y2);
        self.buf.push(b' ');
        self.buf.push_val(x3);
        self.buf.push(b' ');
        self.buf.push_val(y3);
        self.buf.push_bytes(b" c\n");
        self
    }

    /// `v`: Create a cubic Bézier segment to (x3, y3) with (x2, y2)
    /// as control point.
    pub fn cubic_to_initial(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.buf.push_val(x2);
        self.buf.push(b' ');
        self.buf.push_val(y2);
        self.buf.push(b' ');
        self.buf.push_val(x3);
        self.buf.push(b' ');
        self.buf.push_val(y3);
        self.buf.push_bytes(b" v\n");
        self
    }

    /// `y`: Create a cubic Bézier segment to (x3, y3) with (x1, y1)
    /// as control point.
    pub fn cubic_to_final(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) -> &mut Self {
        self.buf.push_val(x1);
        self.buf.push(b' ');
        self.buf.push_val(y1);
        self.buf.push(b' ');
        self.buf.push_val(x3);
        self.buf.push(b' ');
        self.buf.push_val(y3);
        self.buf.push_bytes(b" y\n");
        self
    }

    /// `h`: Closes the path with a straight line.
    pub fn close_path(&mut self) -> &mut Self {
        self.buf.push_bytes(b"h\n");
        self
    }
}

impl Drop for Path<'_> {
    fn drop(&mut self) {
        terminate_path(self.buf, self.stroke, self.fill);
    }
}

fn terminate_path(buf: &mut Vec<u8>, stroke: bool, fill: bool) {
    if stroke && fill {
        buf.push_bytes(b"B\n");
    } else if stroke {
        buf.push_bytes(b"S\n");
    } else if fill {
        buf.push_bytes(b"f\n");
    } else {
        buf.push_bytes(b"n\n");
    }
}

/// Writer for an _image XObject_.
///
/// This struct is created by [`PdfWriter::image`].
pub struct ImageStream<'a> {
    stream: Stream<'a>,
}

impl<'a> ImageStream<'a> {
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"XObject"));
        stream.pair(Name(b"Subtype"), Name(b"Image"));
        Self { stream }
    }

    /// Write the `/Width` attribute.
    pub fn width(&mut self, width: i32) -> &mut Self {
        self.pair(Name(b"Width"), width);
        self
    }

    /// Write the `/Height` attribute.
    pub fn height(&mut self, height: i32) -> &mut Self {
        self.pair(Name(b"Height"), height);
        self
    }

    /// Write the `/ColorSpace` attribute.
    pub fn color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.pair(Name(b"ColorSpace"), space.to_name());
        self
    }

    /// Write the `/BitsPerComponent` attribute.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        self.pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/SMask` attribute.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.pair(Name(b"SMask"), x_object);
        self
    }
}

deref!('a, ImageStream<'a> => Stream<'a>, stream);

/// A color space.
///
/// These are either the predefined, parameter-less color spaces like
/// `DeviceGray` or the ones defined by the user, accessed through the `Named`
/// variant. A custom color space of types like `CalRGB` or `Pattern` can be set
/// by registering it with the [`/ColorSpace`](ColorSpaces) dictionary in the
/// current [`Resources`] dictionary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum ColorSpace<'a> {
    DeviceGray,
    DeviceRgb,
    DeviceCmyk,
    Pattern,
    Named(Name<'a>),
}

impl<'a> ColorSpace<'a> {
    /// Get the corresponding [`Name`] primitive for the space.
    pub fn to_name(self) -> Name<'a> {
        match self {
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::DeviceRgb => Name(b"DeviceRGB"),
            Self::DeviceCmyk => Name(b"DeviceCMYK"),
            Self::Pattern => Name(b"Pattern"),
            Self::Named(name) => name,
        }
    }
}

/// A color space type that requires further parameters. These are for internal
/// use. Instances of these color spaces may be used by defining them and their
/// parameters through the [`/ColorSpace`](ColorSpaces) dictionary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ColorSpaceType {
    CalGray,
    CalRgb,
    Lab,
    #[allow(unused)]
    IccBased,
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

impl ColorSpaceType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::CalGray => Name(b"CalGray"),
            Self::CalRgb => Name(b"CalRGB"),
            Self::Lab => Name(b"Lab"),
            Self::IccBased => Name(b"ICCBased"),
            Self::Indexed => Name(b"Indexed"),
            Self::Pattern => Name(b"Pattern"),
            Self::Separation => Name(b"Separation"),
            Self::DeviceN => Name(b"DeviceN"),
        }
    }
}

/// Writer for a _color space dictionary_.
///
/// This struct is created by [`Resources::color_spaces`].
pub struct ColorSpaces<'a> {
    dict: Dict<'a>,
}

impl<'a> ColorSpaces<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write a `CalGray` color space.
    pub fn cal_gray(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::CalGray.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.pair(Name(b"Gamma"), gamma);
        }

        drop(dict);
        drop(array);
        self
    }

    /// Write a `CalRgb` color space.
    pub fn cal_rgb(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
        matrix: Option<[f32; 9]>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::CalRgb.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.pair(Name(b"Gamma"), gamma);
        }

        if let Some(matrix) = matrix {
            dict.key(Name(b"Matrix")).array().typed().items(matrix);
        }

        drop(dict);
        drop(array);
        self
    }

    /// Write a `Lab` color space.
    pub fn lab(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        range: Option<[f32; 4]>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Lab.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(range) = range {
            dict.key(Name(b"Range")).array().typed().items(range);
        }

        drop(dict);
        drop(array);
        self
    }

    /// Write an `Indexed` color space. PDF 1.2+.
    ///
    /// The length of the lookup slice must be the product of the dimensions of
    /// the base color space and (`hival + 1`).
    pub fn indexed(
        &mut self,
        name: Name,
        base: Name,
        hival: i32,
        lookup: &'a [u8],
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Indexed.to_name());
        array.item(base);
        array.item(hival);
        array.item(ByteStr(lookup));
        drop(array);
        self
    }

    /// Write a `Separation` color space. PDF 1.2+.
    pub fn separation(
        &mut self,
        name: Name,
        color_name: Name,
        base: Name,
        tint: Ref,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Separation.to_name());
        array.item(color_name);
        array.item(base);
        array.item(tint);
        drop(array);
        self
    }

    /// Write a `DeviceN` color space. PDF 1.3+.
    pub fn device_n(
        &mut self,
        name: Name,
        names: impl IntoIterator<Item = Name<'a>>,
        alternate_space: Name,
        tint: Ref,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::DeviceN.to_name());
        array.obj().array().typed().items(names);
        array.item(alternate_space);
        array.item(tint);
        drop(array);
        self
    }
}

deref!('a, ColorSpaces<'a> => Dict<'a>, dict);

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
    fn to_int(self) -> i32 {
        match self {
            Self::ButtCap => 0,
            Self::RoundCap => 1,
            Self::ProjectingSquareCap => 2,
        }
    }
}
