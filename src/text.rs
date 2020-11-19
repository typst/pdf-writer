use super::*;

/// A stream of text operations.
pub struct TextStream {
    buf: Vec<u8>,
}

impl TextStream {
    /// Create a new, empty text stream.
    pub fn new() -> Self {
        let mut buf = Vec::new();
        buf.push_bytes(b"BT\n");
        Self { buf }
    }

    /// `Tf` operator: Select a font by name and set the font size as a scale factor.
    pub fn tf(mut self, font: Name, size: f32) -> Self {
        self.buf.push_val(font);
        self.buf.push(b' ');
        self.buf.push_val(size);
        self.buf.push_bytes(b" Tf\n");
        self
    }

    /// `Td` operator: Move to the start of the next line.
    pub fn td(mut self, x: f32, y: f32) -> Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push_bytes(b" Td\n");
        self
    }

    /// `Tm` operator: Set the text matrix.
    pub fn tm(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
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
        self.buf.push_bytes(b" Td\n");
        self
    }

    /// `Tj` operator: Write text.
    ///
    /// This function takes raw bytes. The encoding is up to the caller.
    pub fn tj(mut self, text: &[u8]) -> Self {
        // TODO: Move to general string formatting.
        self.buf.push(b'<');
        for &byte in text {
            self.buf.push_hex(byte);
        }
        self.buf.push_bytes(b"> Tj\n");
        self
    }

    /// Return the raw constructed byte stream.
    pub fn end(mut self) -> Vec<u8> {
        self.buf.push_bytes(b"ET");
        self.buf
    }
}

/// Writer for a _Type-1 font_ dictionary.
pub struct Type1Font<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Type1Font<'a> {
    pub(crate) fn start(any: Any<'a, Indirect>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Font"));
        dict.pair(Name(b"Subtype"), Name(b"Type1"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"BaseFont"), name);
        self
    }
}

/// Writer for a _Type-0 (composite) font_ dictionary.
pub struct Type0Font<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Type0Font<'a> {
    pub(crate) fn start(any: Any<'a, Indirect>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Font"));
        dict.pair(Name(b"Subtype"), Name(b"Type0"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"BaseFont"), name);
        self
    }

    /// Start writing the `/Encoding` attribute.
    pub fn encoding(&mut self) -> Encoding<'_> {
        Encoding::new(self.dict.key(Name(b"Encoding")))
    }

    /// Write the `/DescendantFonts` attribute as a one-element array containing a
    /// reference to a character map.
    pub fn descendant_font(&mut self, cid_font: Ref) -> &mut Self {
        self.dict.key(Name(b"DescendantFonts")).array().item(cid_font);
        self
    }

    /// Write the `/ToUnicode` attribute as a reference to a character map stream.
    pub fn to_unicode(&mut self, cmap: Ref) -> &mut Self {
        self.dict.pair(Name(b"ToUnicode"), cmap);
        self
    }
}

/// Writer for an _encoding_.
pub struct Encoding<'a> {
    any: Any<'a>,
}

impl<'a> Encoding<'a> {
    fn new(any: Any<'a>) -> Self {
        Self { any }
    }

    /// Write a predefined encoding name.
    pub fn predefined(self, encoding: Name) {
        self.any.obj(encoding);
    }

    /// Write a reference to a character map stream.
    pub fn cmap(self, cmap: Ref) {
        self.any.obj(cmap);
    }
}

/// Specifics about a character collection.
pub struct SystemInfo<'a> {
    /// The issuer of the collection.
    pub registry: Str<'a>,
    /// A unique name of the collection within the registry.
    pub ordering: Str<'a>,
    /// The supplement number (i.e. the version).
    pub supplement: i32,
}

/// Writer a character map object.
///
/// Defined here:
/// https://www.adobe.com/content/dam/acom/en/devnet/font/pdfs/5014.CIDFont_Spec.pdf
pub(crate) fn write_cmap(
    w: &mut PdfWriter,
    id: Ref,
    name: Name,
    info: SystemInfo,
    mapping: impl ExactSizeIterator<Item = (u16, char)>,
) {
    let mut buf = Vec::new();

    // Static header.
    buf.push_bytes(b"%!PS-Adobe-3.0 Resource-CMap\n");
    buf.push_bytes(b"%%DocumentNeededResources: procset CIDInit\n");
    buf.push_bytes(b"%%IncludeResource: procset CIDInit\n");

    // Dynamic header.
    buf.push_bytes(b"%%BeginResource: CMap ");
    buf.push_bytes(name.0);
    buf.push(b'\n');
    buf.push_bytes(b"%%Title: (");
    buf.push_bytes(name.0);
    buf.push(b' ');
    buf.push_bytes(info.registry.0);
    buf.push(b' ');
    buf.push_bytes(info.ordering.0);
    buf.push(b' ');
    buf.push_int(info.supplement);
    buf.push_bytes(b")\n");
    buf.push_bytes(b"%%Version: 1\n");
    buf.push_bytes(b"%%EndComments\n");

    // General body.
    buf.push_bytes(b"/CIDInit /ProcSet findresource begin\n");
    buf.push_bytes(b"9 dict begin\n");
    buf.push_bytes(b"begincmap\n");
    buf.push_bytes(b"/CIDSystemInfo 3 dict dup begin\n");
    buf.push_bytes(b"    /Registry ");
    buf.push_val(info.registry);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"    /Ordering ");
    buf.push_val(info.ordering);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"    /Supplement ");
    buf.push_val(info.supplement);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"end def\n");
    buf.push_bytes(b"/CMapName /");
    buf.push_val(name);
    buf.push_bytes(b"def\n");
    buf.push_bytes(b"/CMapVersion 1 def\n");
    buf.push_bytes(b"/CMapType 0 def\n");

    // We just cover the whole unicode codespace.
    buf.push_bytes(b"1 begincodespacerange\n");
    buf.push_bytes(b"<0000> <ffff>\n");
    buf.push_bytes(b"endcodespacerange\n");

    // The mappings.
    buf.push_int(mapping.len());
    buf.push_bytes(b" beginbfchar\n");

    for (cid, c) in mapping {
        buf.push(b'<');
        buf.push_hex_u16(cid);
        buf.push_bytes(b"> <");

        let mut utf16 = [0u16; 2];
        for &mut part in c.encode_utf16(&mut utf16) {
            buf.push_hex_u16(part);
        }

        buf.push_bytes(b">\n");
    }
    buf.push_bytes(b"endbfchar\n");

    // End of body.
    buf.push_bytes(b"endcmap\n");
    buf.push_bytes(b"CMapName currentdict /CMap defineresource pop\n");
    buf.push_bytes(b"end\n");
    buf.push_bytes(b"end\n");
    buf.push_bytes(b"%%EndResource\n");
    buf.push_bytes(b"%%EOF");

    w.stream(id, &buf)
        .pair(Name(b"Type"), Name(b"CMap"))
        .pair(Name(b"CMapName"), name)
        .key(Name(b"CIDSystemInfo"))
        .dict()
        .pair(Name(b"Registry"), info.registry)
        .pair(Name(b"Ordering"), info.ordering)
        .pair(Name(b"Supplement"), info.supplement);
}
