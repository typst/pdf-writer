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
[PDF specification]: https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf
*/

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::wrong_self_convention)]

#[macro_use]
mod macros;
mod annotations;
mod attributes;
mod buf;
mod chunk;
mod color;
mod content;
mod files;
mod font;
mod functions;
mod object;
mod renumber;
mod structure;
mod transitions;
mod xobject;

/// Strongly typed writers for specific PDF structures.
pub mod writers {
    use super::*;
    pub use annotations::{Action, Annotation, Appearance, BorderStyle, IconFit};
    pub use attributes::{
        Attributes, FieldAttributes, LayoutAttributes, ListAttributes, TableAttributes,
        UserProperty,
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
        CidFont, Cmap, Differences, Encoding, FontDescriptor, Type0Font, Type1Font,
        Type3Font, Widths,
    };
    pub use functions::{
        ExponentialFunction, PostScriptFunction, SampledFunction, StitchingFunction,
    };
    pub use object::{NameTree, NameTreeEntries, NumberTree, NumberTreeEntries};
    pub use structure::{
        Catalog, ClassMap, Destination, DeveloperExtension, DocumentInfo, MarkInfo,
        MarkedRef, Metadata, Names, ObjectRef, Outline, OutlineItem, Page, PageLabel,
        Pages, RoleMap, StructChildren, StructElement, StructTreeRoot, ViewerPreferences,
    };
    pub use transitions::Transition;
    pub use xobject::{FormXObject, Group, ImageXObject, Reference};
}

/// Types used by specific PDF structures.
pub mod types {
    use super::*;
    pub use annotations::{
        ActionType, AnnotationFlags, AnnotationIcon, AnnotationType, BorderType,
        HighlightEffect, IconScale, IconScaleType, TextPosition,
    };
    pub use attributes::{
        AttributeOwner, BlockAlign, FieldRole, FieldState, InlineAlign,
        LayoutBorderStyle, ListNumbering, Placement, RubyAlign, RubyPosition,
        TableHeaderScope, TextAlign, TextDecorationType, WritingMode,
    };
    pub use color::{
        DeviceNSubtype, FunctionShadingType, OutputIntentSubtype, PaintType, TilingType,
    };
    pub use content::{
        ArtifactAttachment, ArtifactSubtype, ArtifactType, BlendMode, ColorSpaceOperand,
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

pub use self::chunk::Chunk;
pub use self::content::Content;
pub use self::object::{
    Array, Date, Dict, Filter, Finish, Name, Null, Obj, Primitive, Rect, Ref, Rewrite,
    Str, Stream, TextStr, TypedArray, TypedDict, Writer,
};

use std::fmt::{self, Debug, Formatter};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use self::buf::BufExt;
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
    catalog_id: Option<Ref>,
    info_id: Option<Ref>,
    file_id: Option<(Vec<u8>, Vec<u8>)>,
}

impl Pdf {
    /// Create a new PDF with the default buffer capacity (currently 8 KB).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_capacity(8 * 1024)
    }

    /// Create a new PDF with the specified initial buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut chunk = Chunk::with_capacity(capacity);
        chunk.buf.extend(b"%PDF-1.7\n%\x80\x80\x80\x80\n\n");
        Self {
            chunk,
            catalog_id: None,
            info_id: None,
            file_id: None,
        }
    }

    /// Set the PDF version.
    ///
    /// The version is not semantically important to the crate, but must be
    /// present in the output document.
    ///
    /// _Default value_: 1.7.
    pub fn set_version(&mut self, major: u8, minor: u8) {
        if major < 10 {
            self.chunk.buf[5] = b'0' + major;
        }
        if minor < 10 {
            self.chunk.buf[7] = b'0' + minor;
        }
    }

    /// Set the file identifier for the document.
    ///
    /// The file identifier is a pair of two byte strings that shall be used to
    /// uniquely identify a particular file. The first string should always stay
    /// the same for a document, the second should change for each revision. It
    /// is optional, but recommended. PDF 1.1+.
    pub fn set_file_id(&mut self, id: (Vec<u8>, Vec<u8>)) {
        self.file_id = Some(id);
    }

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
    /// file trailer, meaning that you don't need to provide the given `id`
    /// anywhere else.
    pub fn document_info(&mut self, id: Ref) -> DocumentInfo<'_> {
        self.info_id = Some(id);
        self.indirect(id).start()
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    ///
    /// Panics if any indirect reference id was used twice.
    pub fn finish(self) -> Vec<u8> {
        let Chunk { mut buf, mut offsets } = self.chunk;

        offsets.sort();

        let xref_len = 1 + offsets.last().map_or(0, |p| p.0.get());
        let xref_offset = buf.len();

        buf.extend(b"xref\n0 ");
        buf.push_int(xref_len);
        buf.push(b'\n');

        if offsets.is_empty() {
            write!(buf, "0000000000 65535 f\r\n").unwrap();
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

                let gen = if free_id == 0 { "65535" } else { "00000" };
                write!(buf, "{:010} {} f\r\n", next % xref_len, gen).unwrap();
                written += 1;
            }

            write!(buf, "{:010} 00000 n\r\n", offset).unwrap();
            written += 1;
        }

        // Write the trailer dictionary.
        buf.extend(b"trailer\n");

        let mut trailer = Obj::direct(&mut buf, 0).dict();
        trailer.pair(Name(b"Size"), xref_len);

        if let Some(catalog_id) = self.catalog_id {
            trailer.pair(Name(b"Root"), catalog_id);
        }

        if let Some(info_id) = self.info_id {
            trailer.pair(Name(b"Info"), info_id);
        }

        if let Some(file_id) = self.file_id {
            let mut ids = trailer.insert(Name(b"ID")).array();
            ids.item(Str(&file_id.0));
            ids.item(Str(&file_id.1));
        }

        trailer.finish();

        // Write where the cross-reference table starts.
        buf.extend(b"\nstartxref\n");
        write!(buf, "{}", xref_offset).unwrap();

        // Write the end of file marker.
        buf.extend(b"\n%%EOF");
        buf
    }
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
    pub fn slice<F>(f: F) -> Vec<u8>
    where
        F: FnOnce(&mut Pdf),
    {
        let mut w = Pdf::new();
        let start = w.len();
        f(&mut w);
        let end = w.len();
        let buf = w.finish();
        buf[start..end].to_vec()
    }

    /// Return the slice of bytes written for an object.
    pub fn slice_obj<F>(f: F) -> Vec<u8>
    where
        F: FnOnce(Obj<'_>),
    {
        let buf = slice(|w| f(w.indirect(Ref::new(1))));
        buf[8..buf.len() - 9].to_vec()
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
}
