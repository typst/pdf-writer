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
            panic!("width parameter must be positive");
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

    /// `re`: Draws a rectangle.
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

    /// `m ... h / S / f / B`: Start drawing a path at (x, y).
    pub fn path(&mut self, x: f32, y: f32, stroke: bool, fill: bool) -> Path<'_> {
        Path::start(self, x, y, stroke, fill)
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
    fn start(content: &'a mut Content) -> Self {
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
        self.buf.push_bytes(b" Tm\n");
        self
    }

    /// `Tj`: Show text.
    ///
    /// This function takes raw bytes. The encoding is up to you.
    pub fn show(&mut self, text: &[u8]) -> &mut Self {
        // TODO: Move to general string formatting.
        self.buf.push(b'<');
        for &byte in text {
            self.buf.push_hex(byte);
        }
        self.buf.push_bytes(b"> Tj\n");
        self
    }
}

impl Drop for Text<'_> {
    fn drop(&mut self) {
        self.buf.push_bytes(b"ET\n");
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
    /// `m`: Create a new path at the current point.
    fn start(
        content: &'a mut Content,
        x: f32,
        y: f32,
        stroke: bool,
        fill: bool,
    ) -> Self {
        let buf = &mut content.buf;
        buf.push_val(x);
        buf.push(b' ');
        buf.push_val(y);
        buf.push_bytes(b" m\n");
        Self { buf, stroke, fill }
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
    pub fn cubic_to_initial(
        &mut self,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) -> &mut Self {
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
    pub fn cubic_to_final(
        &mut self,
        x1: f32,
        y1: f32,
        x3: f32,
        y3: f32,
    ) -> &mut Self {
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
        stream.inner().pair(Name(b"Type"), Name(b"XObject"));
        stream.inner().pair(Name(b"Subtype"), Name(b"Image"));
        Self { stream }
    }

    /// Write the `/Width` attribute.
    pub fn width(&mut self, width: i32) -> &mut Self {
        self.stream.inner().pair(Name(b"Width"), width);
        self
    }

    /// Write the `/Height` attribute.
    pub fn height(&mut self, height: i32) -> &mut Self {
        self.stream.inner().pair(Name(b"Height"), height);
        self
    }

    /// Write the `/ColorSpace` attribute.
    pub fn color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.stream.inner().pair(Name(b"ColorSpace"), space.to_name());
        self
    }

    /// Write the `/BitsPerComponent` attribute.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        self.stream.inner().pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/SMask` attribute.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.stream.inner().pair(Name(b"SMask"), x_object);
        self
    }

    /// Access the underlying stream writer.
    pub fn inner(&mut self) -> &mut Stream<'a> {
        &mut self.stream
    }
}

/// A color space.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum ColorSpace {
    DeviceGray,
    DeviceRgb,
    DeviceCmyk,
    CalGray,
    CalRgb,
    Lab,
    IccBased,
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

impl ColorSpace {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::DeviceRgb => Name(b"DeviceRGB"),
            Self::DeviceCmyk => Name(b"DeviceCMYK"),
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
