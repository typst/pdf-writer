/*!
A step-by-step PDF writer.

The entry point into the API is the main [`PdfWriter`]. The document is written
into one big internal buffer, but otherwise the API is largely non-allocating.

# Minimal example
The following example creates a PDF with a single, empty A4 page.

```
use pdf_writer::{PdfWriter, Rect, Ref};

# fn main() -> std::io::Result<()> {
// Start writing with the PDF version 1.7 header.
let mut writer = PdfWriter::new(1, 7);

// The document catalog and a page tree with one A4 page that uses no resources.
writer.catalog(Ref::new(1)).pages(Ref::new(2));
writer.pages(Ref::new(2)).kids(vec![Ref::new(3)]);
writer.page(Ref::new(3))
    .parent(Ref::new(2))
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", writer.finish(Ref::new(1)))?;
# Ok(())
# }
```

For a more comprehensive overview, check out the [hello world example] in the
repository, which creates a document with text in it.

[hello world example]: https://github.com/typst/pdf-writer/tree/main/examples/hello.rs
*/

#![deny(missing_docs)]

mod buf;
mod content;
mod font;
mod object;
mod stream;
mod structure;

/// Writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use content::{ImageStream, Path, Text};
    pub use font::{CidFont, CmapStream, FontDescriptor, Type0Font, Type1Font, Widths};
    pub use structure::{Catalog, Page, Pages, Resources};
}

pub use content::{ColorSpace, Content, LineCapStyle};
pub use font::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
pub use object::*;
pub use stream::*;

use std::fmt::{self, Debug, Formatter};
use std::io::Write;

use buf::BufExt;
use writers::*;

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
    depth: usize,
    indent: usize,
}

/// Core methods.
impl PdfWriter {
    /// Create a new PDF writer with the default buffer capacity
    /// (currently 8 KB).
    ///
    /// This already writes the PDF header containing the (major, minor)
    /// version.
    pub fn new(major: i32, minor: i32) -> Self {
        Self::with_capacity(8 * 1024, major, minor)
    }

    /// Create a new PDF writer with the specified initial buffer capacity.
    ///
    /// This already writes the PDF header containing the (major, minor)
    /// version.
    pub fn with_capacity(capacity: usize, major: i32, minor: i32) -> Self {
        let mut buf = Vec::with_capacity(capacity);
        buf.push_bytes(b"%PDF-");
        buf.push_int(major);
        buf.push(b'.');
        buf.push_int(minor);
        buf.push_bytes(b"\n\n");
        Self {
            buf,
            offsets: vec![],
            depth: 0,
            indent: 0,
        }
    }

    /// Set the indent level per layer of nested objects.
    ///
    /// _Default value_: 0.
    pub fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    pub fn finish(mut self, catalog_id: Ref) -> Vec<u8> {
        assert_eq!(self.depth, 0, "unfinished object");
        let (xref_len, xref_offset) = self.xref_table();
        self.trailer(catalog_id, xref_len, xref_offset);
        self.buf
    }

    /// The number of bytes that were written so far.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    fn xref_table(&mut self) -> (i32, usize) {
        let mut offsets = std::mem::take(&mut self.offsets);
        offsets.sort();

        let xref_len = 1 + offsets.last().map(|p| p.0.get()).unwrap_or(0);
        let xref_offset = self.buf.len();

        self.buf.push_bytes(b"xref\n0 ");
        self.buf.push_int(xref_len);

        // Always write the initial entry for unusable id zero.
        self.buf.push_bytes(b"\n0000000000 65535 f\r\n");
        let mut next = 1;

        for &(id, offset) in &offsets {
            let id = id.get();
            while next < id {
                // TODO: Form linked list of free items.
                self.buf.push_bytes(b"0000000000 65535 f\r\n");
                next += 1;
            }

            write!(self.buf, "{:010} 00000 n\r\n", offset).unwrap();
            next = id + 1;
        }

        (xref_len, xref_offset)
    }

    fn trailer(&mut self, catalog_id: Ref, xref_len: i32, xref_offset: usize) {
        // Write the trailer dictionary.
        self.buf.push_bytes(b"trailer\n");

        Dict::start(self, ())
            .pair(Name(b"Size"), xref_len)
            .pair(Name(b"Root"), catalog_id);

        // Write where the cross-reference table starts.
        self.buf.push_bytes(b"\nstartxref\n");
        write!(self.buf, "{}", xref_offset).unwrap();

        // Write the end of file marker.
        self.buf.push_bytes(b"\n%%EOF");
    }

    fn push_indent(&mut self) {
        let width = self.indent * self.depth;
        for _ in 0 .. width {
            self.buf.push(b' ');
        }
    }
}

/// Indirect objects.
impl PdfWriter {
    /// Start writing an indirectly referenceable object.
    pub fn indirect(&mut self, id: Ref) -> Obj<'_, IndirectGuard> {
        let indirect = IndirectGuard::start(self, id);
        Obj::new(self, indirect)
    }

    /// Start writing a document catalog.
    pub fn catalog(&mut self, id: Ref) -> Catalog<'_> {
        Catalog::start(self.indirect(id))
    }

    /// Start writing a page tree.
    pub fn pages(&mut self, id: Ref) -> Pages<'_> {
        Pages::start(self.indirect(id))
    }

    /// Start writing a page.
    pub fn page(&mut self, id: Ref) -> Page<'_> {
        Page::start(self.indirect(id))
    }

    /// Start writing a Type-1 font.
    pub fn type1_font(&mut self, id: Ref) -> Type1Font<'_> {
        Type1Font::start(self.indirect(id))
    }

    /// Start writing a Type-0 font.
    pub fn type0_font(&mut self, id: Ref) -> Type0Font<'_> {
        Type0Font::start(self.indirect(id))
    }

    /// Start writing a CID font.
    pub fn cid_font(&mut self, id: Ref, subtype: CidFontType) -> CidFont<'_> {
        CidFont::start(self.indirect(id), subtype)
    }

    /// Start writing a font descriptor.
    pub fn font_descriptor(&mut self, id: Ref) -> FontDescriptor<'_> {
        FontDescriptor::start(self.indirect(id))
    }
}

/// Streams.
impl PdfWriter {
    /// Start writing an indirectly referenceable stream.
    ///
    /// The stream data and the `/Length` field are written automatically. You
    /// can add additional key-value pairs to the stream dictionary with the
    /// returned stream writer.
    pub fn stream<'a>(&'a mut self, id: Ref, data: &'a [u8]) -> Stream<'a> {
        let indirect = IndirectGuard::start(self, id);
        Stream::start(self, data, indirect)
    }

    /// Start writing a character map stream.
    ///
    /// If you want to use this for a `/ToUnicode` CMap, you can use the
    /// [`UnicodeCmap`] builder to construct the data.
    pub fn cmap<'a>(&'a mut self, id: Ref, cmap: &'a [u8]) -> CmapStream<'a> {
        CmapStream::start(self.stream(id, cmap))
    }

    /// Start writing an XObject image stream.
    pub fn image<'a>(&'a mut self, id: Ref, samples: &'a [u8]) -> ImageStream<'a> {
        ImageStream::start(self.stream(id, samples))
    }
}

impl Debug for PdfWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("PdfWriter(..)")
    }
}

/// Finishes an entity when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub trait Guard {
    /// Finish the entity.
    fn finish(&self, writer: &mut PdfWriter);
}

impl Guard for () {
    fn finish(&self, _: &mut PdfWriter) {}
}

/// A guard that finishes an indirect object when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub struct IndirectGuard;

impl IndirectGuard {
    pub(crate) fn start(w: &mut PdfWriter, id: Ref) -> Self {
        assert_eq!(w.depth, 0);
        w.depth += 1;
        w.offsets.push((id, w.buf.len()));
        w.buf.push_int(id.get());
        w.buf.push_bytes(b" 0 obj\n");
        w.push_indent();
        Self
    }
}

impl Guard for IndirectGuard {
    fn finish(&self, w: &mut PdfWriter) {
        w.depth -= 1;
        w.buf.push_bytes(b"\nendobj\n\n");
    }
}
