/*!
A step-by-step, zero-unsafe PDF writer.

The entry point into the API is the main [`PdfWriter`], which constructs the
document into one big internal buffer. The top-level writer has many methods to
create specialized writers for specific PDF objects. These all follow the same
general pattern: They borrow the main buffer mutably, expose a builder pattern
for writing individual fields in a strongly typed fashion and finish up the
object when dropped.

There are a few more top-level structs with internal buffers, like the builder
for [`Content`] streams, but wherever possible buffers are borrowed from parent
writers to minimize allocations.

# Writers
The writers contained is this crate fall roughly into two categories.

**Core writers** enable you to write arbitrary PDF objects.

- The [`Obj`] writer allows to write most fundamental PDF objects (numbers,
  strings, arrays, dictionaries, ...). It is exposed through
  [`PdfWriter::indirect`] to write top-level indirect objects and through
  [`Array::obj`] and [`Dict::key`] to compose objects.
- Streams are exposed through a separate [`PdfWriter::stream`] method since they
  _must_ be indirect objects.

**Specialized writers** for things like a _[page]_ or an _[image stream]_ expose
the core writer's capabilities in a strongly typed fashion.

- A [`Page`] writer, for example, is just a thin wrapper around a [`Dict`] and
  it even derefs to a dictionary in case you need to write a field that is not
  yet exposed by the typed API.
- Similarly, the [`ImageStream`] derefs to a [`Stream`], so that the
  [`filter()`] function can be shared by all kinds of streams. The [`Stream`] in
  turn derefs to a [`Dict`] so that you can add arbitrary fields to the stream
  dictionary.

When you bind a writer to a variable instead of just writing a chained builder
pattern, you may need to manually [`drop()`] it before starting a new object.

# Minimal example
The following example creates a PDF with a single, empty A4 page.

```
use pdf_writer::{PdfWriter, Rect, Ref};

# fn main() -> std::io::Result<()> {
// Define some indirect reference ids we'll use.
let catalog_id = Ref::new(1);
let page_tree_id = Ref::new(2);
let page_id = Ref::new(3);

// Start writing with the PDF version 1.7 header.
let mut writer = PdfWriter::new(1, 7);

// The document catalog and a page tree with one A4 page that uses no resources.
writer.catalog(catalog_id).pages(page_tree_id);
writer.pages(page_tree_id).kids(vec![page_id]);
writer.page(page_id)
    .parent(page_tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", writer.finish(catalog_id))?;
# Ok(())
# }
```

For a more comprehensive overview, check out the [hello world example] in the
repository, which creates a document with text and a link in it.

# Note
This crate does not validate whether you use the correct indirect reference ids
or whether you write all required fields for an object. Refer to the
[PDF specification] to make sure you create valid PDFs.

[page]: writers::Page
[image stream]: writers::ImageStream
[`filter()`]: Stream::filter
[hello world example]: https://github.com/typst/pdf-writer/tree/main/examples/hello.rs
[PDF specification]: https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf
*/

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[macro_use]
mod macros;
mod annotations;
mod buf;
mod content;
mod font;
mod functions;
mod object;
mod stream;
mod structure;
mod transitions;

/// Writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use annotations::{Action, Annotation, Annotations, BorderStyle, FileSpec};
    pub use content::{ImageStream, Path, PositionedText, Text};
    pub use font::{CidFont, CmapStream, FontDescriptor, Type0Font, Type1Font, Widths};
    pub use functions::{
        ExponentialFunction, PostScriptFunction, SampledFunction, StitchingFunction,
    };
    pub use structure::{
        Catalog, Destination, Destinations, Outline, OutlineItem, Page, Pages, Resources,
        ViewerPreferences,
    };
    pub use transitions::Transition;
}

pub use annotations::{
    ActionType, AnnotationFlags, AnnotationIcon, AnnotationType, BorderType,
    HighlightEffect,
};
pub use content::{ColorSpace, Content, LineCapStyle};
pub use font::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
pub use functions::{InterpolationOrder, PostScriptOp};
pub use object::*;
pub use stream::*;
pub use structure::{Direction, OutlineItemFlags, PageLayout, PageMode};
pub use transitions::{TransitionAngle, TransitionStyle};

use std::borrow::Cow;
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
        self.offsets.sort();

        let xref_len = 1 + self.offsets.last().map_or(0, |p| p.0.get());
        let xref_offset = self.buf.len();

        self.buf.push_bytes(b"xref\n0 ");
        self.buf.push_int(xref_len);
        self.buf.push(b'\n');

        let mut idx = 0;
        let mut free = 0;

        loop {
            // Find the next free entry.
            let start = idx;
            let mut link = free + 1;
            while self.offsets.get(idx).map_or(false, |(id, _)| link == id.get()) {
                idx += 1;
                link += 1;
            }

            // A free entry links to the next free entry.
            let gen = if free == 0 { "65535" } else { "00000" };
            write!(self.buf, "{:010} {} f\r\n", link % xref_len, gen).unwrap();
            free = link;

            // A used entry contains the offset of the object in the file.
            for &(_, offset) in &self.offsets[start .. idx] {
                write!(self.buf, "{:010} 00000 n\r\n", offset).unwrap();
            }

            if idx >= self.offsets.len() {
                break;
            }
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

    /// Start writing an outline.
    pub fn outline(&mut self, id: Ref) -> Outline<'_> {
        Outline::start(self.indirect(id))
    }

    /// Start writing an outline item.
    pub fn outline_item(&mut self, id: Ref) -> OutlineItem<'_> {
        OutlineItem::start(self.indirect(id))
    }

    /// Start writing a named destination dictionary.
    pub fn destinations(&mut self, id: Ref) -> Destinations<'_> {
        Destinations::start(self.indirect(id))
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

    /// Start writing an exponential function dictionary.
    pub fn exponential_function(&mut self, id: Ref) -> ExponentialFunction<'_> {
        ExponentialFunction::start(self.indirect(id))
    }

    /// Start writing a stitching function dictionary.
    pub fn stitching_function(&mut self, id: Ref) -> StitchingFunction<'_> {
        StitchingFunction::start(self.indirect(id))
    }
}

/// Streams.
impl PdfWriter {
    /// Start writing an indirectly referenceable stream.
    ///
    /// The stream data and the `/Length` field are written automatically. You
    /// can add additional key-value pairs to the stream dictionary with the
    /// returned stream writer.
    pub fn stream<'a, T>(&'a mut self, id: Ref, data: T) -> Stream<'a>
    where
        T: Into<Cow<'a, [u8]>>,
    {
        let indirect = IndirectGuard::start(self, id);
        Stream::start(self, data.into(), indirect)
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

    /// Start writing a sampled function stream.
    pub fn sampled_function<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> SampledFunction<'a> {
        SampledFunction::start(self.stream(id, samples))
    }

    /// Start writing a PostScript function stream.
    pub fn postscript_function<'a>(
        &'a mut self,
        id: Ref,
        ops: &[PostScriptOp],
    ) -> PostScriptFunction<'a> {
        let bytes = PostScriptOp::encode_slice(ops);
        PostScriptFunction::start(self.stream(id, bytes))
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
