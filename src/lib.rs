/*!
A step-by-step PDF writer.

The entry point into the API is the [`Pdf`] struct, which constructs the
document into one big internal buffer. The top-level writer has many methods to
create specialized writers for specific PDF objects. These all follow the same
general pattern: They borrow the main buffer mutably, expose a builder pattern
for writing individual fields in a strongly typed fashion and finish up the
object when dropped.

There are a few more top-level structs with internal buffers, like the
[`Content`] stream builder and the [`Chunk`], but wherever possible buffers
are borrowed from parent writers to minimize allocations.

# Writers
The writers contained is this crate fall roughly into two categories.

**Core writers** enable you to write arbitrary PDF objects.

- The [`Obj`] writer allows to write most fundamental PDF objects (numbers,
  strings, arrays, dictionaries, ...). It is exposed through
  [`Chunk::indirect`] to write top-level indirect objects and through
  [`Array::push`] and [`Dict::insert`] to compose objects.
- Streams are exposed through a separate [`Chunk::stream`] method since
  they _must_ be indirect objects.

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
use pdf_writer::{Pdf, Rect, Ref};

# fn main() -> std::io::Result<()> {
// Define some indirect reference ids we'll use.
let catalog_id = Ref::new(1);
let page_tree_id = Ref::new(2);
let page_id = Ref::new(3);

// Write a document catalog and a page tree with one A4 page that uses no resources.
let mut pdf = Pdf::new();
pdf.catalog(catalog_id).pages(page_tree_id);
pdf.pages(page_tree_id).kids([page_id]).count(1);
pdf.page(page_id)
    .parent(page_tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", pdf.finish())?;
# Ok(())
# }
```

For more examples, check out the [examples folder] in the repository.

# Note
This crate is rather low-level. It does not allocate or validate indirect
reference IDs for you and it does not check whether you write all required
fields for an object. Refer to the [PDF specification] to make sure you create
valid PDFs.

[page]: writers::Page
[image]: writers::ImageXObject
[`filter()`]: Stream::filter
[examples folder]: https://github.com/typst/pdf-writer/tree/main/examples
[PDF specification]: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf
*/

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::wrong_self_convention)]

#[macro_use]
mod macros;
mod actions;
mod annotations;
mod attributes;
mod buf;
mod chunk;
mod color;
mod content;
mod files;
mod font;
mod forms;
mod functions;
mod object;
mod renditions;
mod renumber;
mod structure;
mod transitions;
mod xobject;

/// Strongly typed writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use actions::{Action, AdditionalActions, Fields};
    pub use annotations::{
        Annotation, Appearance, AppearanceCharacteristics, AppearanceEntry, BorderStyle,
        IconFit,
    };
    pub use attributes::{
        ArtifactAttributes, Attributes, FENoteAttributes, FieldAttributes,
        LayoutAttributes, ListAttributes, TableAttributes, TrackSizes, UserProperty,
    };
    pub use color::{
        ColorSpace, DeviceN, DeviceNAttrs, DeviceNMixingHints, DeviceNProcess,
        FunctionShading, IccProfile, OutputIntent, Separation, SeparationInfo,
        ShadingPattern, StreamShading, StreamShadingType, TilingPattern,
    };
    pub use content::{
        Artifact, ExtGraphicsState, MarkContent, Operation, PositionedItems,
        PropertyList, Resources, ShowPositioned, SoftMask,
    };
    pub use files::{EmbeddedFile, EmbeddingParams, FileSpec};
    pub use font::{
        CidFont, Cmap, Differences, Encoding, FontDescriptor, FontDescriptorOverride,
        Type0Font, Type1Font, Type3Font, WMode, Widths,
    };
    pub use forms::{Field, Form};
    pub use functions::{
        ExponentialFunction, PostScriptFunction, SampledFunction, StitchingFunction,
    };
    pub use object::{
        DecodeParms, NameTree, NameTreeEntries, NumberTree, NumberTreeEntries,
    };
    pub use renditions::{MediaClip, MediaPermissions, MediaPlayParams, Rendition};
    pub use structure::{
        Catalog, ClassMap, Destination, DeveloperExtension, DocumentInfo, MarkInfo,
        MarkedRef, Metadata, Names, Namespace, NamespaceRoleMap, ObjectRef, Outline,
        OutlineItem, Page, PageLabel, Pages, RoleMap, StructChildren, StructElement,
        StructTreeRoot, ViewerPreferences,
    };
    pub use transitions::Transition;
    pub use xobject::{FormXObject, Group, ImageXObject, Reference};
}

/// Types used by specific PDF structures.
pub mod types {
    use super::*;
    pub use actions::{ActionType, FormActionFlags, RenditionOperation};
    pub use annotations::{
        AnnotationFlags, AnnotationIcon, AnnotationType, BorderType, HighlightEffect,
        IconScale, IconScaleType, TextPosition,
    };
    pub use attributes::{
        AttributeOwner, BlockAlign, FieldRole, FieldState, GlyphOrientationVertical,
        InlineAlign, LayoutBorderStyle, LayoutTextPosition, LineHeight, ListNumbering,
        NoteType, Placement, RubyAlign, RubyPosition, Sides, TableHeaderScope, TextAlign,
        TextDecorationType, WritingMode,
    };
    pub use color::{
        DeviceNSubtype, FunctionShadingType, OutputIntentSubtype, PaintType, TilingType,
    };
    pub use content::{
        ArtifactAttachment, ArtifactSubtype, ArtifactType, BlendMode, ColorSpaceOperand,
        LineCapStyle, LineJoinStyle, MaskType, OverprintMode, ProcSet, RenderingIntent,
        TextRenderingMode,
    };
    pub use files::AssociationKind;
    pub use font::{
        CidFontType, CjkClass, FontFlags, FontStretch, GlyphId, SystemInfo, UnicodeCmap,
    };
    pub use forms::{
        CheckBoxState, ChoiceOptions, FieldFlags, FieldType, Quadding, SigFlags,
    };
    pub use functions::{InterpolationOrder, PostScriptOp};
    pub use object::Predictor;
    pub use renditions::{MediaClipType, RenditionType, TempFileType};
    pub use structure::{
        BlockLevelRoleSubtype, Direction, InlineLevelRoleSubtype,
        InlineLevelRoleSubtype2, NumberingStyle, OutlineItemFlags, PageLayout, PageMode,
        PhoneticAlphabet, RoleMapOpts, StructRole, StructRole2, StructRole2Compat,
        StructRoleType, StructRoleType2, TabOrder, TrappingStatus,
    };
    pub use transitions::{TransitionAngle, TransitionStyle};
    pub use xobject::SMaskInData;
}

pub use self::buf::{Buf, Limits};
pub use self::chunk::{Chunk, Settings};
pub use self::content::Content;
pub use self::object::{
    Array, Date, Dict, Filter, Finish, LanguageIdentifier, Name, Null, Obj, Primitive,
    Rect, Ref, Rewrite, Str, Stream, TextStr, TextStrLike, TextStrWithLang, TypedArray,
    TypedDict, Writer,
};

use std::fmt::{self, Debug, Formatter};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use self::writers::*;

/// A builder for a PDF file.
///
/// This type constructs a PDF file in-memory. Aside from a few specific
/// structures, a PDF file mostly consists of indirect objects. For more
/// flexibility, you can write these objects either directly into a [`Pdf`] or
/// into a [`Chunk`], which you can add to the [`Pdf`] (or another chunk) later.
/// Therefore, most writing methods are exposed on the chunk type, which this
/// type dereferences to.
pub struct Pdf {
    chunk: Chunk,
    trailer_data: TrailerData,
}

impl Pdf {
    /// Create a new PDF with the default settings and buffer capacity
    /// (currently 8 KB).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_settings(Settings::default())
    }

    /// Create a new PDF with the given settings and the default buffer capacity
    /// (currently 8 KB).
    pub fn with_settings(settings: Settings) -> Self {
        Self::with_settings_and_capacity(settings, 8 * 1024)
    }

    /// Create a new PDF with the default settings and the specified initial
    /// buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_settings_and_capacity(Settings::default(), capacity)
    }

    /// Create a new PDF with the given settings and the specified initial
    /// buffer capacity.
    pub fn with_settings_and_capacity(settings: Settings, capacity: usize) -> Self {
        let mut chunk = Chunk::with_settings_and_capacity(settings, capacity);
        chunk.buf.extend(b"%PDF-1.7\n%\x80\x80\x80\x80\n\n");
        Self { chunk, trailer_data: TrailerData::default() }
    }

    /// Set the binary marker in the header of the PDF.
    ///
    /// This can be useful if you want to ensure that your PDF consists of only
    /// ASCII characters, as this is not the case by default.
    ///
    /// _Default value_: \x80\x80\x80\x80
    pub fn set_binary_marker(&mut self, marker: &[u8; 4]) {
        self.chunk.buf.inner[10..14].copy_from_slice(marker);
    }

    /// Set the PDF version.
    ///
    /// The version is not semantically important to the crate, but must be
    /// present in the output document.
    ///
    /// _Default value_: 1.7.
    pub fn set_version(&mut self, major: u8, minor: u8) {
        if major < 10 {
            self.chunk.buf.inner[5] = b'0' + major;
        }
        if minor < 10 {
            self.chunk.buf.inner[7] = b'0' + minor;
        }
    }

    /// Set the file identifier for the document.
    ///
    /// The file identifier is a pair of two byte strings that shall be used to
    /// uniquely identify a particular file. The first string should always stay
    /// the same for a document, the second should change for each revision. It
    /// is optional, but recommended. In PDF/A, this is required. PDF 1.1+.
    pub fn set_file_id(&mut self, id: (Vec<u8>, Vec<u8>)) {
        self.trailer_data.file_id = Some(id);
    }

    /// Start writing the document catalog. Required.
    ///
    /// This will also register the document catalog with the file trailer,
    /// meaning that you don't need to provide the given `id` anywhere else.
    pub fn catalog(&mut self, id: Ref) -> Catalog<'_> {
        self.trailer_data.catalog_id = Some(id);
        self.indirect(id).start()
    }

    /// Start writing the document information.
    ///
    /// This will also register the document information dictionary with the
    /// file trailer, meaning that you don't need to provide the given `id`
    /// anywhere else.
    pub fn document_info(&mut self, id: Ref) -> DocumentInfo<'_> {
        self.trailer_data.info_id = Some(id);
        self.indirect(id).start()
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    ///
    /// Panics if any indirect reference id was used twice.
    pub fn finish(self) -> Vec<u8> {
        let Chunk { mut buf, offsets, settings } = self.chunk;
        let trailer_data = self.trailer_data;
        let xref_offset = buf.len();

        let mut writer = PlainXRefWriter::new(&mut buf);
        let xref_len = write_offsets(offsets, &mut writer);

        // Write the trailer dictionary.
        buf.extend(b"trailer\n");
        let mut trailer = Obj::direct(&mut buf, 0, settings, false).dict();
        trailer_data.write_into_dict(&mut trailer, xref_len);
        trailer.finish();

        finish_trailer(buf, xref_offset, b"\n")
    }

    /// Write the cross-reference stream and file trailer and return the
    /// underlying buffer. This method is functionally the same as
    /// [`Pdf::finish`], the difference being that the cross-reference
    /// information is written as a cross-reference stream instead of a
    /// cross-reference table. Cross-reference streams usually allow for
    /// smaller file sizes since they can also be compressed (see
    /// [`Pdf::finish_with_xref_stream_and_filter`]),
    /// but are only available from PDF 1.5 onwards. It is also necessary
    /// to call this method instead of [`Pdf::finish`] in case object stream
    /// are used anywhere in the document since normal xref tables do not
    /// support object streams.
    ///
    /// `xref_id` will be the object identifier used for the cross-reference
    /// stream. As in other cases, the identifier needs to be unique throughout
    /// the whole document.
    ///
    /// Panics if any indirect reference id was used twice.
    pub fn finish_with_xref_stream(self, xref_id: Ref) -> Vec<u8> {
        self.finish_with_xref_stream_inner(xref_id, |buf| (buf, None))
    }

    /// Write the cross-reference stream and file trailer and return the
    /// underlying buffer.
    ///
    /// This method is equivalent to [`Pdf::finish_with_xref_stream`], except
    /// that it allows you to apply one or multiple filters to the xref stream
    /// via the `filter` closure. The input of the closure will be the raw
    /// content of the xref stream, and the output should be the filtered data
    /// as well as a single filter or a list of filters that need to be
    /// applied to unfilter the data in the correct order.
    pub fn finish_with_xref_stream_and_filter(
        self,
        xref_id: Ref,
        filter: impl FnOnce(&[u8]) -> (Vec<u8>, XRefFilter),
    ) -> Vec<u8> {
        self.finish_with_xref_stream_inner(xref_id, |buf| {
            let (xref_data, filter) = filter(&buf);
            (xref_data, Some(filter))
        })
    }

    fn finish_with_xref_stream_inner(
        self,
        xref_id: Ref,
        filter: impl FnOnce(Vec<u8>) -> (Vec<u8>, Option<XRefFilter>),
    ) -> Vec<u8> {
        let Chunk { mut buf, mut offsets, settings } = self.chunk;
        let trailer_data = self.trailer_data;

        // Include the reference of the xref stream in the offsets as well!
        let xref_offset = buf.len();
        offsets.push((xref_id, xref_offset));
        let field_width = determine_field_width(xref_offset);

        let mut writer = XRefStreamWriter::new(field_width);
        let xref_len = write_offsets(offsets, &mut writer);

        let (xref_data, filter) = filter(writer.buf);

        let mut stream =
            Stream::start(Obj::indirect(&mut buf, xref_id, settings), &xref_data);

        stream.pair(Name(b"Type"), Name(b"XRef"));

        if let Some(filter) = filter {
            match filter {
                XRefFilter::Single(filter) => {
                    stream.filter(filter);
                }
                XRefFilter::Multiple(filters) => {
                    let mut arr = stream.insert(Name(b"Filter")).array();

                    for filter in filters {
                        arr.item(filter.to_name());
                    }
                }
            }
        }

        trailer_data.write_into_dict(stream.deref_mut(), xref_len);

        stream
            .insert(Name(b"W"))
            .array()
            .item(1)
            .item(field_width as i32)
            .item(2);

        stream.finish();

        finish_trailer(buf, xref_offset, &[])
    }
}

/// The filters used for the xref stream.
pub enum XRefFilter {
    /// A single filter.
    Single(Filter),
    /// An array of filters.
    Multiple(Vec<Filter>),
}

fn finish_trailer(mut buf: Buf, xref_offset: usize, pad: &[u8]) -> Vec<u8> {
    buf.extend(pad);
    // Write startxref pointing to the xref stream
    buf.extend(b"startxref\n");
    write!(buf.inner, "{}", xref_offset).unwrap();

    // Write EOF marker
    buf.extend(b"\n%%EOF");
    buf.into_vec()
}

fn write_offsets(mut offsets: Vec<(Ref, usize)>, writer: &mut impl XRefWriter) -> i32 {
    offsets.sort();

    let xref_len = 1 + offsets.last().map_or(0, |p| p.0.get());
    writer.prologue(xref_len);

    if offsets.is_empty() {
        writer.write_free_entry(0, 65535);
    }

    let mut written = 0;
    for (i, (object_id, offset)) in offsets.iter().enumerate() {
        if written > object_id.get() {
            panic!("duplicate indirect reference id: {}", object_id.get());
        }

        // Fill in free list.
        let start = written;
        for free_id in start..object_id.get() {
            let mut next = free_id + 1;
            if next == object_id.get() {
                // Find next free id.
                for (used_id, _) in &offsets[i..] {
                    if next < used_id.get() {
                        break;
                    } else {
                        next = used_id.get() + 1;
                    }
                }
            }

            let gen = if free_id == 0 { 65535 } else { 0 };
            writer.write_free_entry((next % xref_len) as usize, gen);
            written += 1;
        }

        writer.write_occupied_entry(*offset, 0);
        written += 1;
    }

    xref_len
}

impl Debug for Pdf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("Pdf(..)")
    }
}

impl Deref for Pdf {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        &self.chunk
    }
}

impl DerefMut for Pdf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk
    }
}

#[derive(Default)]
struct TrailerData {
    catalog_id: Option<Ref>,
    info_id: Option<Ref>,
    file_id: Option<(Vec<u8>, Vec<u8>)>,
}

impl TrailerData {
    fn write_into_dict(&self, dict: &mut Dict, xref_len: i32) {
        dict.pair(Name(b"Size"), xref_len);

        if let Some(catalog_id) = self.catalog_id {
            dict.pair(Name(b"Root"), catalog_id);
        }

        if let Some(info_id) = self.info_id {
            dict.pair(Name(b"Info"), info_id);
        }

        if let Some(file_id) = &self.file_id {
            let mut ids = dict.insert(Name(b"ID")).array();
            ids.item(Str(&file_id.0));
            ids.item(Str(&file_id.1));
        }
    }
}

trait XRefWriter {
    fn prologue(&mut self, xref_len: i32);
    fn write_free_entry(&mut self, offset: usize, gen_number: u16);
    fn write_occupied_entry(&mut self, offset: usize, gen_number: u16);
}

struct XRefStreamWriter {
    buf: Vec<u8>,
    field_width: u32,
}

impl XRefStreamWriter {
    fn new(field_width: u32) -> Self {
        Self { buf: Vec::new(), field_width }
    }
}

impl XRefStreamWriter {
    fn write(&mut self, entry_type: u8, offset: usize, gen_number: u16) {
        let offset_bytes = (offset as u64).to_be_bytes();

        self.buf.push(entry_type);
        self.buf.extend(
            offset_bytes
                .iter()
                .skip(offset_bytes.len() - self.field_width as usize),
        );
        self.buf.extend_from_slice(&gen_number.to_be_bytes());
    }
}

impl XRefWriter for XRefStreamWriter {
    fn prologue(&mut self, _: i32) {}

    fn write_free_entry(&mut self, offset: usize, gen_number: u16) {
        self.write(0, offset, gen_number);
    }

    fn write_occupied_entry(&mut self, offset: usize, gen_number: u16) {
        self.write(1, offset, gen_number);
    }
}

struct PlainXRefWriter<'a> {
    buf: &'a mut Buf,
}

impl<'a> PlainXRefWriter<'a> {
    fn new(buf: &'a mut Buf) -> Self {
        Self { buf }
    }
}

impl<'a> XRefWriter for PlainXRefWriter<'a> {
    fn prologue(&mut self, xref_len: i32) {
        self.buf.extend(b"xref\n0 ");
        self.buf.push_int(xref_len);
        self.buf.push(b'\n');
    }

    fn write_free_entry(&mut self, offset: usize, gen_number: u16) {
        write!(self.buf.inner, "{offset:010} {gen_number:05} f\r\n").unwrap();
    }

    fn write_occupied_entry(&mut self, offset: usize, gen_number: u16) {
        write!(self.buf.inner, "{offset:010} {gen_number:05} n\r\n").unwrap();
    }
}

fn determine_field_width(offset: usize) -> u32 {
    (usize::BITS - offset.leading_zeros()).div_ceil(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Print a chunk.
    #[allow(unused)]
    pub fn print_chunk(chunk: &Chunk) {
        println!("========== Chunk ==========");
        for &(id, offset) in &chunk.offsets {
            println!("[{}]: {}", id.get(), offset);
        }
        println!("---------------------------");
        print!("{}", String::from_utf8_lossy(&chunk.buf));
        println!("===========================");
    }

    /// Return the slice of bytes written during the execution of `f`.
    pub fn slice<F>(f: F, settings: Settings) -> Vec<u8>
    where
        F: FnOnce(&mut Pdf),
    {
        let mut w = Pdf::with_settings(settings);
        let start = w.len();
        f(&mut w);
        let end = w.len();
        let buf = w.finish();
        buf[start..end].to_vec()
    }

    /// Return the slice of bytes written for an object.
    pub fn slice_obj<F>(f: F, settings: Settings) -> Vec<u8>
    where
        F: FnOnce(Obj<'_>),
    {
        let buf = slice(|w| f(w.indirect(Ref::new(1))), settings);
        if settings.pretty {
            buf[8..buf.len() - 9].to_vec()
        } else {
            buf[8..buf.len() - 8].to_vec()
        }
    }

    #[test]
    fn test_minimal() {
        let w = Pdf::new();
        test!(
            w.finish(),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"xref\n0 1\n0000000000 65535 f\r",
            b"trailer\n<<\n  /Size 1\n>>",
            b"startxref\n16\n%%EOF",
        );
    }

    #[test]
    fn test_xref_free_list_short() {
        let mut w = Pdf::new();
        w.indirect(Ref::new(1)).primitive(1);
        w.indirect(Ref::new(2)).primitive(2);
        test!(
            w.finish(),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n1\nendobj\n",
            b"2 0 obj\n2\nendobj\n",
            b"xref",
            b"0 3",
            b"0000000000 65535 f\r",
            b"0000000016 00000 n\r",
            b"0000000034 00000 n\r",
            b"trailer",
            b"<<\n  /Size 3\n>>",
            b"startxref\n52\n%%EOF",
        )
    }

    #[test]
    fn test_xref_free_list_long() {
        let mut w = Pdf::new();
        w.set_version(1, 4);
        w.indirect(Ref::new(1)).primitive(1);
        w.indirect(Ref::new(2)).primitive(2);
        w.indirect(Ref::new(5)).primitive(5);
        test!(
            w.finish(),
            b"%PDF-1.4\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n1\nendobj\n",
            b"2 0 obj\n2\nendobj\n",
            b"5 0 obj\n5\nendobj\n",
            b"xref",
            b"0 6",
            b"0000000003 65535 f\r",
            b"0000000016 00000 n\r",
            b"0000000034 00000 n\r",
            b"0000000004 00000 f\r",
            b"0000000000 00000 f\r",
            b"0000000052 00000 n\r",
            b"trailer",
            b"<<\n  /Size 6\n>>",
            b"startxref\n70\n%%EOF",
        )
    }

    #[test]
    #[should_panic(expected = "duplicate indirect reference id: 3")]
    fn test_xref_free_list_duplicate() {
        let mut w = Pdf::new();
        w.indirect(Ref::new(3)).primitive(1);
        w.indirect(Ref::new(5)).primitive(2);
        w.indirect(Ref::new(13)).primitive(1);
        w.indirect(Ref::new(3)).primitive(1);
        w.indirect(Ref::new(6)).primitive(2);
        w.finish();
    }

    #[test]
    fn test_binary_marker() {
        let mut w = Pdf::new();
        w.set_binary_marker(b"ABCD");
        test!(
            w.finish(),
            b"%PDF-1.7\n%ABCD\n",
            b"xref\n0 1\n0000000000 65535 f\r",
            b"trailer\n<<\n  /Size 1\n>>",
            b"startxref\n16\n%%EOF",
        );
    }

    #[test]
    fn field_width() {
        assert_eq!(determine_field_width(128), 1);
        assert_eq!(determine_field_width(255), 1);
        assert_eq!(determine_field_width(256), 2);
        assert_eq!(determine_field_width(u16::MAX as usize), 2);
        assert_eq!(determine_field_width(u16::MAX as usize + 1), 3);
        assert_eq!(determine_field_width(u32::MAX as usize), 4);
    }

    #[test]
    fn test_xref_stream() {
        let mut w = Pdf::new();
        w.indirect(Ref::new(1)).primitive(1);
        w.indirect(Ref::new(2)).primitive(2);
        w.indirect(Ref::new(5)).primitive(5);
        test!(
            w.finish_with_xref_stream(Ref::new(6)),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n1\nendobj\n",
            b"2 0 obj\n2\nendobj\n",
            b"5 0 obj\n5\nendobj\n",
            b"6 0 obj\n<<\n  /Length 28\n  /Type /XRef\n  /Size 7\n  /W [1 1 2]\n>>\nstream",
            // [0, 3, 255, 255], [1, 16, 0, 0], [1, 34, 0, 0], [0, 4, 0, 0], [0, 0, 0, 0], [1, 52, 0, 0], [1, 70, 0, 0]
            b"\x00\x03\xFF\xFF\x01\x10\x00\x00\x01\x22\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x01\x34\x00\x00\x01\x46\x00\x00",
            b"endstream\nendobj\n",
            b"startxref\n70\n%%EOF",
        )
    }

    #[test]
    fn test_xref_stream_single_filter() {
        let mut w = Pdf::new();
        w.indirect(Ref::new(1)).primitive(1);
        test!(
            w.finish_with_xref_stream_and_filter(Ref::new(2), |_| (b"ABCDEFGH".to_vec(), XRefFilter::Single(Filter::FlateDecode))),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n1\nendobj\n",
            b"2 0 obj\n<<\n  /Length 8\n  /Type /XRef\n  /Filter /FlateDecode\n  /Size 3\n  /W [1 1 2]\n>>\nstream",
            b"ABCDEFGH",
            b"endstream\nendobj\n",
            b"startxref\n34\n%%EOF",
        )
    }

    #[test]
    fn test_xref_stream_multiple_filters() {
        let mut w = Pdf::new();
        w.indirect(Ref::new(1)).primitive(1);
        test!(
            w.finish_with_xref_stream_and_filter(Ref::new(2), |_| (b"ABCDEFGH".to_vec(), XRefFilter::Multiple(vec![Filter::AsciiHexDecode, Filter::FlateDecode]))),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n1\nendobj\n",
            b"2 0 obj\n<<\n  /Length 8\n  /Type /XRef\n  /Filter [/ASCIIHexDecode /FlateDecode]\n  /Size 3\n  /W [1 1 2]\n>>\nstream",
            b"ABCDEFGH",
            b"endstream\nendobj\n",
            b"startxref\n34\n%%EOF",
        )
    }

    #[test]
    fn test_xref_width2() {
        let mut w = Pdf::new();
        w.stream(Ref::new(1), &[b'0'; 256]);
        w.indirect(Ref::new(2)).primitive(1);
        test!(
            w.finish_with_xref_stream(Ref::new(3)),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj\n<<\n  /Length 256\n>>\nstream",
            b"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            b"endstream\nendobj\n",
            b"2 0 obj\n1\nendobj\n",
            b"3 0 obj\n<<\n  /Length 20\n  /Type /XRef\n  /Size 4\n  /W [1 2 2]\n>>\nstream",
            // [0, 0, 0, 255, 255], [1, 0, 16, 0, 0], [1, 0, 34, 0, 0], [1, 1, 32, 0, 0]
            b"\x00\x00\x00\xFF\xFF\x01\x00\x10\x00\x00\x01\x01\x46\x00\x00\x01\x01\x58\x00\x00",
            b"endstream\nendobj\n",
            b"startxref\n344\n%%EOF",
        )
    }
}
