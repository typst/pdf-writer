/*!
A step-by-step PDF writer.

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

**Specialized writers** for things like a _[page]_ or an _[image]_ expose the
core writer's capabilities in a strongly typed fashion.

- A [`Page`] writer, for example, is just a thin wrapper around a [`Dict`] and
  it even derefs to a dictionary in case you need to write a field that is not
  yet exposed by the typed API.
- Similarly, the [`Image`] derefs to a [`Stream`], so that the [`filter()`]
  function can be shared by all kinds of streams. The [`Stream`] in turn derefs
  to a [`Dict`] so that you can add arbitrary fields to the stream dictionary.

When you bind a writer to a variable instead of just writing a chained builder
pattern, you may need to manually drop it before starting a new object using
[`finish()`](Finish::finish) or [`drop()`].

# Minimal example
The following example creates a PDF with a single, empty A4 page.

```
use pdf_writer::{PdfWriter, Rect, Ref};

# fn main() -> std::io::Result<()> {
// Define some indirect reference ids we'll use.
let catalog_id = Ref::new(1);
let page_tree_id = Ref::new(2);
let page_id = Ref::new(3);

// Write a document catalog and a page tree with one A4 page that uses no resources.
let mut writer = PdfWriter::new();
writer.catalog(catalog_id).pages(page_tree_id);
writer.pages(page_tree_id).kids([page_id]);
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
This crate is rather low-level. It does not allocate or validate indirect reference
ids for you and it does not check you write all required fields for an object. Refer
to the [PDF specification] to make sure you create valid PDFs.

[page]: writers::Page
[image]: writers::Image
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
mod color;
mod content;
mod font;
mod functions;
mod object;
mod structure;
mod transitions;
mod xobject;

/// Writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use annotations::{
        Action, Annotation, Annotations, BorderStyle, EmbedParams, EmbeddedFile, FileSpec,
    };
    pub use color::{ColorSpaces, Shading, ShadingPattern, TilingPattern};
    pub use content::{ExtGraphicsState, Operation, PositionedItems, ShowPositioned};
    pub use font::{CidFont, Cmap, FontDescriptor, Type0Font, Type1Font, Widths};
    pub use functions::{
        ExponentialFunction, PostScriptFunction, SampledFunction, StitchingFunction,
    };
    pub use structure::{
        Catalog, Destination, Destinations, Outline, OutlineItem, Page, Pages, Resources,
        ViewerPreferences,
    };
    pub use transitions::Transition;
    pub use xobject::{FormXObject, Group, Image, Reference};
}

/// Types used by specific PDF structures.
pub mod types {
    use super::*;
    pub use annotations::{
        ActionType, AnnotationFlags, AnnotationIcon, AnnotationType, BorderType,
        HighlightEffect,
    };
    pub use color::{ColorSpace, PaintType, ShadingType, TilingType};
    pub use content::{LineCapStyle, LineJoinStyle, RenderingIntent, TextRenderingMode};
    pub use font::{CidFontType, FontFlags, SystemInfo};
    pub use functions::{InterpolationOrder, PostScriptOp};
    pub use structure::{Direction, OutlineItemFlags, PageLayout, PageMode, TabOrder};
    pub use transitions::{TransitionAngle, TransitionStyle};
}

pub use content::Content;
pub use font::UnicodeCmap;
pub use object::*;

use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::io::Write;

use buf::BufExt;
use types::CidFontType;
use writers::*;

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
}

/// Core methods.
impl PdfWriter {
    /// Create a new PDF writer with the default buffer capacity
    /// (currently 8 KB).
    pub fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }

    /// Create a new PDF writer with the specified initial buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Vec::with_capacity(capacity);
        buf.extend(b"%PDF-1.7\n%\x80\x80\x80\x80\n\n");
        Self { buf, offsets: vec![] }
    }

    /// Set the PDF version.
    ///
    /// The version is not semantically important to the writer, but must be
    /// present in the output document.
    ///
    /// _Default value_: 1.7.
    pub fn set_version(&mut self, major: u8, minor: u8) {
        if major < 10 {
            self.buf[5] = b'0' + major;
        }
        if minor < 10 {
            self.buf[7] = b'0' + minor;
        }
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    pub fn finish(mut self, catalog_id: Ref) -> Vec<u8> {
        let (xref_len, xref_offset) = self.xref_table();
        self.trailer(catalog_id, xref_len, xref_offset);
        self.buf
    }

    /// The number of bytes that were written so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    fn xref_table(&mut self) -> (i32, usize) {
        self.offsets.sort();

        let xref_len = 1 + self.offsets.last().map_or(0, |p| p.0.get());
        let xref_offset = self.buf.len();

        self.buf.extend(b"xref\n0 ");
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
        self.buf.extend(b"trailer\n");

        Obj::direct(&mut self.buf, 0)
            .dict()
            .pair(Name(b"Size"), xref_len)
            .pair(Name(b"Root"), catalog_id);

        // Write where the cross-reference table starts.
        self.buf.extend(b"\nstartxref\n");
        write!(self.buf, "{}", xref_offset).unwrap();

        // Write the end of file marker.
        self.buf.extend(b"\n%%EOF");
    }
}

/// Indirect objects.
impl PdfWriter {
    /// Start writing an indirectly referenceable object.
    pub fn indirect(&mut self, id: Ref) -> Obj<'_> {
        self.offsets.push((id, self.buf.len()));
        Obj::indirect(&mut self.buf, id)
    }

    /// Start writing a document catalog.
    pub fn catalog(&mut self, id: Ref) -> Catalog<'_> {
        Catalog::new(self.indirect(id))
    }

    /// Start writing a page tree.
    pub fn pages(&mut self, id: Ref) -> Pages<'_> {
        Pages::new(self.indirect(id))
    }

    /// Start writing a page.
    pub fn page(&mut self, id: Ref) -> Page<'_> {
        Page::new(self.indirect(id))
    }

    /// Start writing an outline.
    pub fn outline(&mut self, id: Ref) -> Outline<'_> {
        Outline::new(self.indirect(id))
    }

    /// Start writing an outline item.
    pub fn outline_item(&mut self, id: Ref) -> OutlineItem<'_> {
        OutlineItem::new(self.indirect(id))
    }

    /// Start writing a named destination dictionary.
    pub fn destinations(&mut self, id: Ref) -> Destinations<'_> {
        Destinations::new(self.indirect(id))
    }

    /// Start writing a Type-1 font.
    pub fn type1_font(&mut self, id: Ref) -> Type1Font<'_> {
        Type1Font::new(self.indirect(id))
    }

    /// Start writing a Type-0 font.
    pub fn type0_font(&mut self, id: Ref) -> Type0Font<'_> {
        Type0Font::new(self.indirect(id))
    }

    /// Start writing a CID font.
    pub fn cid_font(&mut self, id: Ref, subtype: CidFontType) -> CidFont<'_> {
        CidFont::new(self.indirect(id), subtype)
    }

    /// Start writing a font descriptor.
    pub fn font_descriptor(&mut self, id: Ref) -> FontDescriptor<'_> {
        FontDescriptor::new(self.indirect(id))
    }

    /// Start writing a dictionary for a shading pattern.
    pub fn shading_pattern(&mut self, id: Ref) -> ShadingPattern<'_> {
        ShadingPattern::new(self.indirect(id))
    }

    /// Start writing an exponential function dictionary.
    pub fn exponential_function(&mut self, id: Ref) -> ExponentialFunction<'_> {
        ExponentialFunction::new(self.indirect(id))
    }

    /// Start writing a stitching function dictionary.
    pub fn stitching_function(&mut self, id: Ref) -> StitchingFunction<'_> {
        StitchingFunction::new(self.indirect(id))
    }

    /// Start writing an external graphics state dictionary.
    pub fn ext_graphics(&mut self, id: Ref) -> ExtGraphicsState<'_> {
        ExtGraphicsState::new(self.indirect(id))
    }

    /// Start writing a file specification dictionary.
    pub fn file_spec(&mut self, id: Ref) -> FileSpec<'_> {
        FileSpec::new(self.indirect(id))
    }
}

/// Streams.
impl PdfWriter {
    /// Start writing an indirectly referenceable stream.
    ///
    /// The stream data and the `/Length` field are written automatically. You
    /// can add additional key-value pairs to the stream dictionary with the
    /// returned stream writer.
    ///
    /// This crate does not do any compression for you. If you want to compress
    /// a stream, you have to pass already compressed data into this function
    /// and specify the appropriate filter in the stream dictionary.
    ///
    /// For example, if you want to have a [content](Content) stream that is
    /// compressed with DEFLATE, you could do something like this:
    /// ```
    /// use pdf_writer::{PdfWriter, Content, Ref, Filter, Name, Str};
    /// use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
    ///
    /// // Create a writer and a simple content stream.
    /// let mut writer = PdfWriter::new();
    /// let mut content = Content::new();
    /// content.rect(50.0, 50.0, 50.0, 50.0);
    /// content.stroke();
    ///
    /// // Compress and write the stream.
    /// let level = CompressionLevel::DefaultLevel as u8;
    /// let compressed = compress_to_vec_zlib(&content.finish(), level);
    /// writer.stream(Ref::new(1), compressed).filter(Filter::FlateDecode);
    /// ```
    /// For all the specialized stream functions below, it works the same way:
    /// You can pass compressed data and specify a filter.
    pub fn stream<'a, T>(&'a mut self, id: Ref, data: T) -> Stream<'a>
    where
        T: Into<Cow<'a, [u8]>>,
    {
        Stream::new(self.indirect(id), data.into())
    }

    /// Start writing a character map stream.
    ///
    /// If you want to use this for a `/ToUnicode` CMap, you can create the
    /// bytes using a [`UnicodeCmap`] builder.
    pub fn cmap<'a>(&'a mut self, id: Ref, cmap: &'a [u8]) -> Cmap<'a> {
        Cmap::new(self.stream(id, cmap))
    }

    /// Start writing an XObject image stream.
    ///
    /// The samples should be encoded according to the stream's filter, color
    /// space and bits per component.
    pub fn image<'a>(&'a mut self, id: Ref, samples: &'a [u8]) -> Image<'a> {
        Image::new(self.stream(id, samples))
    }

    /// Start writing an form XObject stream.
    ///
    /// These can be used as transparency groups.
    pub fn form_xobject<'a>(&'a mut self, id: Ref, data: &'a [u8]) -> FormXObject<'a> {
        FormXObject::new(self.stream(id, data))
    }

    /// Start writing a tiling pattern stream.
    ///
    /// You can create the content bytes using a [`Content`] builder.
    pub fn tiling_pattern<'a>(
        &'a mut self,
        id: Ref,
        content: &'a [u8],
    ) -> TilingPattern<'a> {
        TilingPattern::new(self.stream(id, content))
    }

    /// Start writing a sampled function stream.
    pub fn sampled_function<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> SampledFunction<'a> {
        SampledFunction::new(self.stream(id, samples))
    }

    /// Start writing a PostScript function stream.
    ///
    /// You can create the code bytes using [`PostScriptOp::encode`](types::PostScriptOp::encode).
    pub fn post_script_function<'a>(
        &'a mut self,
        id: Ref,
        code: &'a [u8],
    ) -> PostScriptFunction<'a> {
        PostScriptFunction::new(self.stream(id, code))
    }

    /// Start writing an embedded file stream.
    pub fn embedded_file<'a>(&'a mut self, id: Ref, bytes: &'a [u8]) -> EmbeddedFile<'a> {
        EmbeddedFile::new(self.stream(id, bytes))
    }
}

impl Debug for PdfWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("PdfWriter(..)")
    }
}

/// Finish objects in postfix-style.
///
/// In many cases you can use writers in builder-pattern style so that they are
/// automatically dropped at the appropriate time. Sometimes though you need to
/// bind a writer to a variable and still want to regain access to the
/// [`PdfWriter`] in the same scope. In that case, you need to manually invoke
/// the writer's `Drop` implementation. You can of course, just write
/// `drop(array)` to finish your array, but you might find it more aesthetically
/// pleasing to write `array.finish()`. That's what this trait is for.
///
/// ```
/// # use pdf_writer::{PdfWriter, Ref, Finish, Name, Str};
/// # let mut writer = PdfWriter::new();
/// let mut array = writer.indirect(Ref::new(1)).array();
/// array.obj().dict().pair(Name(b"Key"), Str(b"Value"));
/// array.item(2);
/// array.finish(); // instead of drop(array)
///
/// // Do more stuff with the writer ...
/// ```
pub trait Finish: Sized {
    /// Does nothing but move `self`, equivalent to [`drop`].
    #[inline]
    fn finish(self) {}
}

impl<T> Finish for T {}
