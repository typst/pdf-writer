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

    /// `scn`: Set the fill pattern. PDF 1.2+.
    ///
    /// The `name` parameter is the name of a pattern. If this is an uncolored
    /// pattern, a tint color in the current `Pattern` base color space must be
    /// given, otherwise, the `color` iterator shall remain empty.
    pub fn fill_pattern(
        &mut self,
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        for val in color.into_iter() {
            self.buf.push_val(val);
            self.buf.push(b' ');
        }

        self.buf.push_val(name);
        self.buf.push_bytes(b" scn\n");
        self
    }

    /// `SCN`: Set the stroke pattern. PDF 1.2+.
    ///
    /// The `name` parameter is the name of a pattern. If this is an uncolored
    /// pattern, a tint color in the current `Pattern` base color space must be
    /// given, otherwise, the `color` iterator shall remain empty.
    pub fn stroke_pattern(
        &mut self,
        color: impl IntoIterator<Item = f32>,
        name: Name,
    ) -> &mut Self {
        for val in color.into_iter() {
            self.buf.push_val(val);
            self.buf.push(b' ');
        }

        self.buf.push_val(name);
        self.buf.push_bytes(b" SCN\n");
        self
    }

    /// `ri`: Set the color rendering intent to the parameter. PDF 1.1+.
    pub fn rendering_intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.buf.push_val(intent.to_name());
        self.buf.push_bytes(b" ri\n");
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

    /// `sh`: Fill the whole drawing area with the specified shading.
    pub fn shading(&mut self, shading: Name) -> &mut Self {
        self.buf.push_val(shading);
        self.buf.push_bytes(b" sh\n");
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

    /// Write the `/Intent` attribute. PDF 1.1+.
    pub fn intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.pair(Name(b"Intent"), intent.to_name());
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

    /// Write a `Pattern` color space for uncolored patterns. PDF 1.2+.
    ///
    /// The `base` attribute is the color space in which the pattern color is
    /// specified upon use.
    pub fn pattern(&mut self, name: Name, base: Name) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Pattern.to_name());
        array.item(base);
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

/// How the document should aim to render colors.
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
    fn to_name(self) -> Name<'static> {
        match self {
            Self::AbsoluteColorimetric => Name(b"AbsoluteColorimetric"),
            Self::RelativeColorimetric => Name(b"RelativeColorimetric"),
            Self::Saturation => Name(b"Saturation"),
            Self::Perceptual => Name(b"Perceptual"),
        }
    }
}

/// Writer for a _tiling pattern stream_.
pub struct TilingStream<'a> {
    stream: Stream<'a>,
}

impl<'a> TilingStream<'a> {
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"Pattern"));
        stream.pair(Name(b"PatternType"), PatternType::Tiling.to_int());
        Self { stream }
    }

    /// Write the `/PaintType` attribute.
    ///
    /// Sets whether to use external or stream color. Required.
    pub fn paint_type(&mut self, paint_type: PaintType) -> &mut Self {
        self.stream.pair(Name(b"PaintType"), paint_type.to_int());
        self
    }

    /// Write the `/TilingType` attribute.
    ///
    /// Sets how to stretch and space the pattern. Required.
    pub fn tiling_type(&mut self, tiling_type: TilingType) -> &mut Self {
        self.stream.pair(Name(b"TilingType"), tiling_type.to_int());
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the pattern in the pattern's coordinate system.
    /// Required.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.stream.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/XStep` attribute.
    ///
    /// Sets the horizontal spacing between pattern cells. Required.
    ///
    /// Panics if zero.
    pub fn x_step(&mut self, x_step: f32) -> &mut Self {
        assert!(x_step != 0.0);
        self.stream.pair(Name(b"XStep"), x_step);
        self
    }

    /// Write the `/YStep` attribute.
    ///
    /// Sets the vertical spacing between pattern cells. Required.
    ///
    /// Panics if zero.
    pub fn y_step(&mut self, y_step: f32) -> &mut Self {
        assert!(y_step != 0.0);
        self.stream.pair(Name(b"YStep"), y_step);
        self
    }

    /// Start writing the `/Resources` dictionary.
    ///
    /// Sets the resources used by the pattern. Required.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.key(Name(b"Resources")))
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the pattern coordinate system to the parent content stream
    /// coordinates. The default is the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.stream.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }
}

deref!('a, TilingStream<'a> => Stream<'a>, stream);

/// Type of pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PatternType {
    /// A tiling pattern.
    Tiling,
    /// A shading pattern.
    Shading,
}

impl PatternType {
    fn to_int(self) -> i32 {
        match self {
            Self::Tiling => 1,
            Self::Shading => 2,
        }
    }
}

/// Type of paint for a tiling pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PaintType {
    /// Paint the pattern with the colors specified in the stream.
    Colored,
    /// Paint the pattern with the colors active when the pattern was painted.
    Uncolored,
}

impl PaintType {
    fn to_int(self) -> i32 {
        match self {
            Self::Colored => 1,
            Self::Uncolored => 2,
        }
    }
}

/// How to adjust tile spacing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TilingType {
    /// Constant space between each tile, tiles may be distorted by 1px.
    ConstantSpacing,
    /// Tile size is constant, spacing between may vary by 1px.
    NoDistortion,
    /// Constant space between each tile and faster drawing, tiles may be distorted.
    FastConstantSpacing,
}

impl TilingType {
    fn to_int(self) -> i32 {
        match self {
            Self::ConstantSpacing => 1,
            Self::NoDistortion => 2,
            Self::FastConstantSpacing => 3,
        }
    }
}

/// Writer for a _shading dictionary_.
///
/// This struct is created by [`PdfWriter::shading`].
pub struct Shading<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Shading<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write the `/ShadingType` attribute.
    ///
    /// Sets the type of shading. The available and required attributes change
    /// depending on this. Required.
    pub fn shading_type(&mut self, shading_type: ShadingType) -> &mut Self {
        self.dict.pair(Name(b"ShadingType"), shading_type.to_int());
        self
    }

    /// Write the `/ColorSpace` attribute.
    ///
    /// Sets the color space of the shading function. May not be a `Pattern`
    /// space. Required.
    pub fn color_space(&mut self, color_space: ColorSpace) -> &mut Self {
        self.dict.pair(Name(b"ColorSpace"), color_space.to_name());
        self
    }

    /// Write the `/Background` attribute.
    ///
    /// Sets the background color of the area to be shaded. The `background`
    /// iterator must contain exactly as many elements as the current
    /// `ColorSpace` has dimensions.
    pub fn background(&mut self, background: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.key(Name(b"Background")).array().typed().items(background);
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the shading in the target coordinate system.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.dict.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/AntiAlias` attribute.
    ///
    /// Sets whether to anti-alias the shading.
    pub fn anti_alias(&mut self, anti_alias: bool) -> &mut Self {
        self.dict.pair(Name(b"AntiAlias"), anti_alias);
        self
    }

    /// Write the `/Domain` attribute.
    ///
    /// Sets the domain of the shading function in a rectangle. Can be used for
    /// function, axial, or radial shadings. Will otherwise default to
    /// `[x_min = 0, x_max = 1, y_min = 0, y_max = 1]`
    pub fn domain(&mut self, domain: [f32; 4]) -> &mut Self {
        self.dict.key(Name(b"Domain")).array().typed().items(domain);
        self
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the shading domain rectangle to the target coordinate system. Can
    /// be used for function shadings. Will otherwise
    /// default to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }

    /// Write the `/Function` attribute.
    ///
    /// Sets the 2-in function to use for shading. Required.
    pub fn function(&mut self, function: Ref) -> &mut Self {
        self.dict.pair(Name(b"Function"), function);
        self
    }

    /// Write the `/Coords` attribute.
    ///
    /// Sets the coordinates of the start and end of the axis in terms of the
    /// target coordinate system. Required for axial (4 items) and radial (6
    /// items; centers and radii) shadings.
    pub fn coords(&mut self, coords: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.key(Name(b"Coords")).array().typed().items(coords);
        self
    }

    /// Write the `/Extend` attribute.
    ///
    /// Set whether the shading should extend beyond either side of the axis /
    /// circles. Can be used for axial and radial shadings.
    pub fn extend(&mut self, extend: [bool; 2]) -> &mut Self {
        self.dict.key(Name(b"Extend")).array().typed().items(extend);
        self
    }
}

deref!('a, Shading<'a> => Dict<'a, IndirectGuard>, dict);

/// What kind of shading to use.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ShadingType {
    /// The function specifies the color for each point in the domain.
    Function,
    /// The function specifies the color for each point on a line.
    Axial,
    /// The function specifies the color for each circle between two nested circles.
    Radial,
}

impl ShadingType {
    fn to_int(self) -> i32 {
        match self {
            Self::Function => 1,
            Self::Axial => 2,
            Self::Radial => 3,
        }
    }
}

/// Writer for a _shading pattern stream_.
pub struct ShadingPattern<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> ShadingPattern<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pattern"));
        dict.pair(Name(b"PatternType"), PatternType::Shading.to_int());
        Self { dict }
    }

    /// Write the `/Shading` attribute.
    ///
    /// Sets the shading object to use. Required.
    pub fn shading(&mut self, shading: Ref) -> &mut Self {
        self.dict.pair(Name(b"Shading"), shading);
        self
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Sets the matrix to use for the pattern. Defaults to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }
}

deref!('a, ShadingPattern<'a> => Dict<'a, IndirectGuard>, dict);
