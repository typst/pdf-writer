use super::*;

/// A stream of text operations.
pub struct TextStream {
    buf: Vec<u8>,
}

impl TextStream {
    /// Create a new, empty text stream.
    pub fn new() -> Self {
        let mut buf = vec![];
        writeln!(buf, "BT");
        Self { buf }
    }

    /// `Tf` operator: Select a font by name and set the font size as a scale factor.
    pub fn tf(mut self, font: Name, size: f32) -> Self {
        writeln!(self.buf, "{} {} Tf", font, size);
        self
    }

    /// `Td` operator: Move to the start of the next line.
    pub fn td(mut self, x: f32, y: f32) -> Self {
        writeln!(self.buf, "{} {} Td", x, y);
        self
    }

    /// `Tm` operator: Set the text matrix.
    pub fn tm(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        writeln!(self.buf, "{} {} {} {} {} {} Tm", a, b, c, d, e, f);
        self
    }

    /// `Tj` operator: Write text.
    ///
    /// This function takes raw bytes. The encoding is up to the caller.
    pub fn tj(mut self, text: &[u8]) -> Self {
        // TODO: Move to general string formatting.
        // TODO: Select best encoding.
        // TODO: Reserve size upfront.
        write!(self.buf, "<");
        for &byte in text {
            write!(self.buf, "{:x}", byte);
        }
        write!(self.buf, ">");
        writeln!(self.buf, " Tj");
        self
    }

    /// Return the raw constructed byte stream.
    pub fn end(mut self) -> Vec<u8> {
        writeln!(self.buf, "ET");
        self.buf
    }
}

/// Writer for a _Type-1 font_ dictionary.
pub struct Type1Font<'a> {
    dict: Dict<'a>,
}

impl<'a> Type1Font<'a> {
    pub(crate) fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Font"));
        dict.pair("Subtype", Name("Type1"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair("BaseFont", name);
        self
    }
}

/// Writer for a _Type-0 (composite) font_ dictionary.
pub struct Type0Font<'a> {
    dict: Dict<'a>,
}

impl<'a> Type0Font<'a> {
    pub(crate) fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Font"));
        dict.pair("Subtype", Name("Type0"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair("BaseFont", name);
        self
    }

    /// Write the `/Encoding` attribute as a predefined encoding name.
    pub fn encoding_predefined(&mut self, encoding: Name) -> &mut Self {
        self.dict.pair("Encoding", encoding);
        self
    }

    /// Write the `/Encoding` attribute as a reference to a character map stream.
    pub fn encoding_cmap(&mut self, cmap: Ref) -> &mut Self {
        self.dict.pair("Encoding", cmap);
        self
    }

    /// Write the `/DescendantFonts` attribute as a one-element array containing a
    /// reference to a character map.
    pub fn descendant_font(&mut self, cid_font: Ref) -> &mut Self {
        self.dict.key("DescendantFonts").array().item(cid_font);
        self
    }

    /// Write the `/ToUnicode` attribute as a reference to a character map stream.
    pub fn to_unicode(&mut self, cmap: Ref) -> &mut Self {
        self.dict.pair("ToUnicode", cmap);
        self
    }
}
