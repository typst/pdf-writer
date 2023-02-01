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
  [`Array::push`] and [`Dict::insert`] to compose objects.
- Streams are exposed through a separate [`PdfWriter::stream`] method since they
  _must_ be indirect objects.

**Specialized writers** for things like a _[page]_ or an _[image]_ expose the
core writer's capabilities in a strongly typed fashion.

- A [`Page`] writer, for example, is just a thin wrapper around a [`Dict`] and
  it even derefs to a dictionary in case you need to write a field that is not
  yet exposed by the typed API.
- Similarly, the [`ImageXObject`] derefs to a [`Stream`], so that the [`filter()`]
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
writer.pages(page_tree_id).kids([page_id]).count(1);
writer.page(page_id)
    .parent(page_tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", writer.finish())?;
# Ok(())
# }
```

For more examples, check out the [examples folder] in the repository.

# Note
This crate is rather low-level. It does not allocate or validate indirect reference
ids for you and it does not check you write all required fields for an object. Refer
to the [PDF specification] to make sure you create valid PDFs.

[page]: writers::Page
[image]: writers::ImageXObject
[`filter()`]: Stream::filter
[examples folder]: https://github.com/typst/pdf-writer/tree/main/examples
[PDF specification]: https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf
*/

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[macro_use]
mod macros;
mod annotations;
mod attributes;
mod buf;
mod color;
mod content;
mod files;
mod font;
mod functions;
mod object;
mod structure;
mod transitions;
mod xobject;

/// Strongly typed writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use annotations::{Action, Annotation, BorderStyle};
    pub use attributes::{
        Attributes, FieldAttributes, LayoutAttributes, ListAttributes, TableAttributes,
        UserProperty,
    };
    pub use color::{
        ColorSpace, DeviceNAttrs, DeviceNMixingHints, DeviceNProcess, DeviceNWithAttrs,
        IccProfile, Shading, ShadingPattern, TilingPattern,
    };
    pub use content::{
        Artifact, ExtGraphicsState, MarkContent, Operation, PositionedItems,
        PropertyList, Resources, ShowPositioned, SoftMask,
    };
    pub use files::{EmbeddedFile, EmbeddingParams, FileSpec};
    pub use font::{
        CidFont, Cmap, Differences, Encoding, FontDescriptor, Type0Font, Type1Font,
        Type3Font, Widths,
    };
    pub use functions::{
        ExponentialFunction, PostScriptFunction, SampledFunction, StitchingFunction,
    };
    pub use object::{NameTree, NameTreeEntries, NumberTree, NumberTreeEntries};
    pub use structure::{
        Catalog, ClassMap, Destination, DeveloperExtension, DocumentInfo, MarkInfo,
        MarkedRef, Names, ObjectRef, Outline, OutlineItem, Page, PageLabel, Pages,
        RoleMap, StructChildren, StructElement, StructTreeRoot, ViewerPreferences,
    };
    pub use transitions::Transition;
    pub use xobject::{FormXObject, Group, ImageXObject, Reference};
}

/// Types used by specific PDF structures.
pub mod types {
    use super::*;
    pub use annotations::{
        ActionType, AnnotationFlags, AnnotationIcon, AnnotationType, BorderType,
        HighlightEffect,
    };
    pub use attributes::{
        AttributeOwner, BlockAlign, FieldRole, FieldState, InlineAlign,
        LayoutBorderStyle, ListNumbering, Placement, RubyAlign, RubyPosition,
        TableHeaderScope, TextAlign, TextDecorationType, WritingMode,
    };
    pub use color::{DeviceNSubtype, PaintType, ShadingType, TilingType};
    pub use content::{
        ArtifactAttachment, ArtifactSubtype, ArtifactType, ColorSpaceOperand,
        LineCapStyle, LineJoinStyle, MaskType, OverprintMode, ProcSet, RenderingIntent,
        TextRenderingMode,
    };
    pub use font::UnicodeCmap;
    pub use font::{CidFontType, FontFlags, FontStretch, SystemInfo};
    pub use functions::{InterpolationOrder, PostScriptOp};
    pub use structure::{
        Direction, NumberingStyle, OutlineItemFlags, PageLayout, PageMode, StructRole,
        TabOrder, TrappingStatus,
    };
    pub use transitions::{TransitionAngle, TransitionStyle};
    pub use xobject::SMaskInData;
}

pub use content::Content;
pub use object::{
    Array, Date, Dict, Filter, Finish, Name, Null, Obj, Primitive, Rect, Ref, Rewrite,
    Str, Stream, TextStr, TypedArray, TypedDict, Writer,
};

use std::fmt::{self, Debug, Formatter};
use std::io::Write;

use buf::BufExt;
use writers::*;

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
    catalog_id: Option<Ref>,
    info_id: Option<Ref>,
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
        Self {
            buf,
            offsets: vec![],
            catalog_id: None,
            info_id: None,
        }
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

    /// The number of bytes that were written so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    ///
    /// Panics if any indirect reference id was used twice.
    pub fn finish(mut self) -> Vec<u8> {
        self.offsets.sort();

        let xref_len = 1 + self.offsets.last().map_or(0, |p| p.0.get());
        let xref_offset = self.buf.len();

        self.buf.extend(b"xref\n0 ");
        self.buf.push_int(xref_len);
        self.buf.push(b'\n');

        if self.offsets.is_empty() {
            write!(self.buf, "0000000000 65535 f\r\n").unwrap();
        }

        let mut written = 0;
        for (i, (object_id, offset)) in self.offsets.iter().enumerate() {
            if written > object_id.get() {
                panic!("duplicate indirect reference id: {}", object_id.get());
            }

            // Fill in free list.
            for free_id in written..object_id.get() {
                let mut next = free_id + 1;
                if next == object_id.get() {
                    // Find next free id.
                    for (used_id, _) in &self.offsets[i..] {
                        if next < used_id.get() {
                            break;
                        } else {
                            next = used_id.get() + 1;
                        }
                    }
                }

                let gen = if free_id == 0 { "65535" } else { "00000" };
                write!(self.buf, "{:010} {} f\r\n", next % xref_len, gen).unwrap();
                written += 1;
            }

            write!(self.buf, "{:010} 00000 n\r\n", offset).unwrap();
            written += 1;
        }

        // Write the trailer dictionary.
        self.buf.extend(b"trailer\n");

        let mut trailer = Obj::direct(&mut self.buf, 0).dict();
        trailer.pair(Name(b"Size"), xref_len);

        if let Some(catalog_id) = self.catalog_id {
            trailer.pair(Name(b"Root"), catalog_id);
        }

        if let Some(info_id) = self.info_id {
            trailer.pair(Name(b"Info"), info_id);
        }

        trailer.finish();

        // Write where the cross-reference table starts.
        self.buf.extend(b"\nstartxref\n");
        write!(self.buf, "{}", xref_offset).unwrap();

        // Write the end of file marker.
        self.buf.extend(b"\n%%EOF");
        self.buf
    }
}

/// Indirect objects and streams.
impl PdfWriter {
    /// Start writing an indirectly referenceable object.
    pub fn indirect(&mut self, id: Ref) -> Obj<'_> {
        self.offsets.push((id, self.buf.len()));
        Obj::indirect(&mut self.buf, id)
    }

    /// Start writing an indirectly referenceable stream.
    ///
    /// The stream data and the `/Length` field are written automatically. You
    /// can add additional key-value pairs to the stream dictionary with the
    /// returned stream writer.
    ///
    /// You can use this function together with a [`Content`] stream builder to
    /// provide a [page's contents](Page::contents).
    /// ```
    /// use pdf_writer::{PdfWriter, Content, Ref};
    ///
    /// // Create a simple content stream.
    /// let mut content = Content::new();
    /// content.rect(50.0, 50.0, 50.0, 50.0);
    /// content.stroke();
    ///
    /// // Create a writer and write the stream.
    /// let mut writer = PdfWriter::new();
    /// writer.stream(Ref::new(1), &content.finish());
    /// ```
    ///
    /// This crate does not do any compression for you. If you want to compress
    /// a stream, you have to pass already compressed data into this function
    /// and specify the appropriate filter in the stream dictionary.
    ///
    /// For example, if you want to compress your content stream with DEFLATE,
    /// you could do something like this:
    /// ```
    /// use pdf_writer::{PdfWriter, Content, Ref, Filter};
    /// use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
    ///
    /// // Create a simple content stream.
    /// let mut content = Content::new();
    /// content.rect(50.0, 50.0, 50.0, 50.0);
    /// content.stroke();
    ///
    /// // Compress the stream.
    /// let level = CompressionLevel::DefaultLevel as u8;
    /// let compressed = compress_to_vec_zlib(&content.finish(), level);
    ///
    /// // Create a writer, write the compressed stream and specify that it
    /// // needs to be decoded with a FLATE filter.
    /// let mut writer = PdfWriter::new();
    /// writer.stream(Ref::new(1), &compressed).filter(Filter::FlateDecode);
    /// ```
    /// For all the specialized stream functions below, it works the same way:
    /// You can pass compressed data and specify a filter.
    ///
    /// Panics if the stream length exceeds `i32::MAX`.
    pub fn stream<'a>(&'a mut self, id: Ref, data: &'a [u8]) -> Stream<'a> {
        Stream::start(self.indirect(id), data)
    }
}

/// Document structure.
impl PdfWriter {
    /// Start writing the document catalog. Required.
    ///
    /// This will also register the document catalog with the file trailer,
    /// meaning that you don't need to provide the given `id` anywhere else.
    pub fn catalog(&mut self, id: Ref) -> Catalog<'_> {
        self.catalog_id = Some(id);
        self.indirect(id).start()
    }

    /// Start writing the document information.
    ///
    /// This will also register the document information dictionary with the
    /// file trailer, meaning that you don't need to provide the given `id` anywhere
    /// else.
    pub fn document_info(&mut self, id: Ref) -> DocumentInfo<'_> {
        self.info_id = Some(id);
        self.indirect(id).start()
    }

    /// Start writing a page tree.
    pub fn pages(&mut self, id: Ref) -> Pages<'_> {
        self.indirect(id).start()
    }

    /// Start writing a page.
    pub fn page(&mut self, id: Ref) -> Page<'_> {
        self.indirect(id).start()
    }

    /// Start writing an outline.
    pub fn outline(&mut self, id: Ref) -> Outline<'_> {
        self.indirect(id).start()
    }

    /// Start writing an outline item.
    pub fn outline_item(&mut self, id: Ref) -> OutlineItem<'_> {
        self.indirect(id).start()
    }

    /// Start writing a named destination dictionary.
    pub fn destinations(&mut self, id: Ref) -> TypedDict<'_, Destination> {
        self.indirect(id).dict().typed()
    }

    /// Start writing a file specification dictionary.
    pub fn file_spec(&mut self, id: Ref) -> FileSpec<'_> {
        self.indirect(id).start()
    }

    /// Start writing an embedded file stream.
    pub fn embedded_file<'a>(&'a mut self, id: Ref, bytes: &'a [u8]) -> EmbeddedFile<'a> {
        EmbeddedFile::start(self.stream(id, bytes))
    }

    /// Start writing a structure tree element.
    pub fn struct_element(&mut self, id: Ref) -> StructElement<'_> {
        self.indirect(id).start()
    }
}

/// Graphics and content.
impl PdfWriter {
    /// Start writing an image XObject stream.
    ///
    /// The samples should be encoded according to the stream's filter, color
    /// space and bits per component.
    pub fn image_xobject<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> ImageXObject<'a> {
        ImageXObject::start(self.stream(id, samples))
    }

    /// Start writing a form XObject stream.
    ///
    /// These can be used as transparency groups.
    ///
    /// Note that these have nothing to do with forms that have fields to fill
    /// out. Rather, they are a way to encapsulate and reuse content across the
    /// file.
    ///
    /// You can create the content bytes using a [`Content`] builder.
    pub fn form_xobject<'a>(&'a mut self, id: Ref, content: &'a [u8]) -> FormXObject<'a> {
        FormXObject::start(self.stream(id, content))
    }

    /// Start writing an external graphics state dictionary.
    pub fn ext_graphics(&mut self, id: Ref) -> ExtGraphicsState<'_> {
        self.indirect(id).start()
    }
}

/// Fonts.
impl PdfWriter {
    /// Start writing a Type-1 font.
    pub fn type1_font(&mut self, id: Ref) -> Type1Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a Type-3 font.
    pub fn type3_font(&mut self, id: Ref) -> Type3Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a Type-0 font.
    pub fn type0_font(&mut self, id: Ref) -> Type0Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a CID font.
    pub fn cid_font(&mut self, id: Ref) -> CidFont<'_> {
        self.indirect(id).start()
    }

    /// Start writing a font descriptor.
    pub fn font_descriptor(&mut self, id: Ref) -> FontDescriptor<'_> {
        self.indirect(id).start()
    }

    /// Start writing a character map stream.
    ///
    /// If you want to use this for a `/ToUnicode` CMap, you can create the
    /// bytes using a [`UnicodeCmap`](types::UnicodeCmap) builder.
    pub fn cmap<'a>(&'a mut self, id: Ref, cmap: &'a [u8]) -> Cmap<'a> {
        Cmap::start(self.stream(id, cmap))
    }
}

/// Color spaces, shadings and patterns.
impl PdfWriter {
    /// Start writing a color space.
    pub fn color_space(&mut self, id: Ref) -> ColorSpace<'_> {
        self.indirect(id).start()
    }

    /// Start writing a shading.
    pub fn shading(&mut self, id: Ref) -> Shading<'_> {
        self.indirect(id).start()
    }

    /// Start writing a tiling pattern stream.
    ///
    /// You can create the content bytes using a [`Content`] builder.
    pub fn tiling_pattern<'a>(
        &'a mut self,
        id: Ref,
        content: &'a [u8],
    ) -> TilingPattern<'a> {
        TilingPattern::start_with_stream(self.stream(id, content))
    }

    /// Start writing a shading pattern.
    pub fn shading_pattern(&mut self, id: Ref) -> ShadingPattern<'_> {
        self.indirect(id).start()
    }

    /// Start writing a ICC profile stream.
    pub fn icc_profile<'a>(&'a mut self, id: Ref, profile: &'a [u8]) -> IccProfile<'a> {
        IccProfile::start(self.stream(id, profile))
    }
}

/// Functions.
impl PdfWriter {
    /// Start writing a sampled function stream.
    pub fn sampled_function<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> SampledFunction<'a> {
        SampledFunction::start(self.stream(id, samples))
    }

    /// Start writing an exponential function.
    pub fn exponential_function(&mut self, id: Ref) -> ExponentialFunction<'_> {
        self.indirect(id).start()
    }

    /// Start writing a stitching function.
    pub fn stitching_function(&mut self, id: Ref) -> StitchingFunction<'_> {
        self.indirect(id).start()
    }

    /// Start writing a PostScript function stream.
    ///
    /// You can create the code bytes using [`PostScriptOp::encode`](types::PostScriptOp::encode).
    pub fn post_script_function<'a>(
        &'a mut self,
        id: Ref,
        code: &'a [u8],
    ) -> PostScriptFunction<'a> {
        PostScriptFunction::start(self.stream(id, code))
    }
}

/// Tree data structures.
impl PdfWriter {
    /// Start writing a name tree node.
    pub fn name_tree<T: Primitive>(&mut self, id: Ref) -> NameTree<'_, T> {
        self.indirect(id).start()
    }

    /// Start writing a number tree node.
    pub fn number_tree<T: Primitive>(&mut self, id: Ref) -> NumberTree<'_, T> {
        self.indirect(id).start()
    }
}

impl Debug for PdfWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("PdfWriter(..)")
    }
}
