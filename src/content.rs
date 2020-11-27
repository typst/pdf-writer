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

/// Writer for a text object.
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

/// Writer for an _image XObject stream_.
pub struct ImageStream<'a> {
    dict: Dict<'a, StreamGuard<'a, IndirectGuard>>,
}

impl<'a> ImageStream<'a> {
    pub(crate) fn start(mut dict: Dict<'a, StreamGuard<'a, IndirectGuard>>) -> Self {
        dict.pair(Name(b"Type"), Name(b"XObject"));
        dict.pair(Name(b"Subtype"), Name(b"Image"));
        Self { dict }
    }

    /// Write the `/Width` attribute.
    pub fn width(&mut self, width: i32) -> &mut Self {
        self.dict.pair(Name(b"Width"), width);
        self
    }

    /// Write the `/Height` attribute.
    pub fn height(&mut self, height: i32) -> &mut Self {
        self.dict.pair(Name(b"Height"), height);
        self
    }

    /// Write the `/ColorSpace` attribute.
    pub fn color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.dict.pair(Name(b"ColorSpace"), space.name());
        self
    }

    /// Write the `/BitsPerComponent` attribute.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        self.dict.pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/SMask` attribute.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.dict.pair(Name(b"SMask"), x_object);
        self
    }
}

/// A color space.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum ColorSpace {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    CalGray,
    CalRGB,
    Lab,
    ICCBased,
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

impl ColorSpace {
    fn name(self) -> Name<'static> {
        match self {
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::DeviceRGB => Name(b"DeviceRGB"),
            Self::DeviceCMYK => Name(b"DeviceCMYK"),
            Self::CalGray => Name(b"CalGray"),
            Self::CalRGB => Name(b"CalRGB"),
            Self::Lab => Name(b"Lab"),
            Self::ICCBased => Name(b"ICCBased"),
            Self::Indexed => Name(b"Indexed"),
            Self::Pattern => Name(b"Pattern"),
            Self::Separation => Name(b"Separation"),
            Self::DeviceN => Name(b"DeviceN"),
        }
    }
}
