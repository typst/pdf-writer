/*!
A step-by-step PDF writer.

The entry point into the API is the main [`PdfWriter`].

# Minimal example
The following example creates a PDF with a single empty A4 page.

```
use pdf_writer::{PdfWriter, Rect, Ref};

# fn main() -> std::io::Result<()> {
// Start writing with PDF version 1.7 header.
let mut writer = PdfWriter::new(1, 7);

// The document catalog and a page tree with one A4 page that uses no resources.
writer.catalog(Ref::new(1)).pages(Ref::new(2));
writer.pages(Ref::new(2)).kids(vec![Ref::new(3)]);
writer.page(Ref::new(3))
    .parent(Ref::new(2))
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", writer.end(Ref::new(1)))?;
# Ok(())
# }
```

For a more comprehensive overview, check out the [hello world example] in the
repository, which creates a document with text in it.

[hello world example]: https://github.com/typst/pdf-writer/tree/main/examples/hello.rs
*/

#![deny(missing_docs)]

mod structure;
mod text;

/// Writers for specific PDF structures.
pub mod writers {
    pub use super::structure::{Catalog, Page, Pages, Resources};
    pub use super::text::{
        CidFont, CmapStream, FontDescriptor, Type0Font, Type1Font, Widths,
    };
}

pub use text::{CidFontType, FontFlags, SystemInfo, TextStream, UnicodeCmap};

use std::convert::TryFrom;
use std::marker::PhantomData;
use std::num::NonZeroI32;

use structure::*;
use text::*;

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
    depth: usize,
    indent: usize,
}

impl PdfWriter {
    /// Create a new PDF writer with the default buffer capacity (currently 8
    /// KB). The buffer will grow as necessary, but you can override this
    /// initial value by using [`with_capacity`](Self::with_capacity).
    ///
    /// This already writes the PDF header containing the (major, minor)
    /// version.
    pub fn new(major: i32, minor: i32) -> Self {
        Self::with_capacity(8 * 1024, major, minor)
    }

    /// Create a new PDF writer with the specified buffer capacity.
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

    /// Start writing an indirectly referenceable object.
    pub fn indirect(&mut self, id: Ref) -> Any<'_, IndirectGuard> {
        let indirect = IndirectGuard::start(self, id, ());
        Any::new(self, indirect)
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

    /// Start writing an indirectly referenceable stream.
    ///
    /// The `/Length` field is added to the stream's dictionary automatically.
    /// You can add additional key-value pairs with the returned dictionary
    /// writer. The stream data is written once the dictionary is dropped.
    pub fn stream<'a>(
        &'a mut self,
        id: Ref,
        data: &'a [u8],
    ) -> Dict<'a, StreamGuard<'a, IndirectGuard>> {
        let data = data.as_ref();
        let len = i32::try_from(data.len()).unwrap_or_else(|_| {
            panic!("data length (is `{}`) must be < i32::MAX");
        });

        let indirect = IndirectGuard::start(self, id, ());
        let stream = StreamGuard::new(data, indirect);

        let mut dict = Dict::start(self, stream);
        dict.pair(Name(b"Length"), len);
        dict
    }

    /// Start writing a character map stream.
    ///
    /// If you want to use this for a `/ToUnicode` CMap, you can construct the
    /// data with the [`UnicodeCmap`] builder.
    pub fn cmap_stream<'a>(&'a mut self, id: Ref, cmap: &'a [u8]) -> CmapStream<'a> {
        CmapStream::start(self.stream(id, cmap))
    }

    /// Write the cross-reference table and file trailer and return the
    /// underlying buffer.
    pub fn end(mut self, catalog_id: Ref) -> Vec<u8> {
        assert_eq!(self.depth, 0);
        let (xref_len, xref_offset) = self.xref_table();
        self.trailer(catalog_id, xref_len, xref_offset);
        self.buf
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

            self.buf.push_int_aligned(offset, 10);
            self.buf.push_bytes(b" 00000 n\r\n");
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
        self.buf.push_int(xref_offset);

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

trait BufExt {
    fn push_val<T: Object>(&mut self, value: T);
    fn push_bytes(&mut self, bytes: &[u8]);
    fn push_int<I: itoa::Integer>(&mut self, value: I);
    fn push_int_aligned<I: itoa::Integer>(&mut self, value: I, align: usize);
    fn push_float(&mut self, value: f32);
    fn push_hex(&mut self, value: u8);
    fn push_hex_u16(&mut self, value: u16);
}

impl BufExt for Vec<u8> {
    fn push_val<T: Object>(&mut self, value: T) {
        value.write(self);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn push_int<I: itoa::Integer>(&mut self, value: I) {
        self.push_bytes(itoa::Buffer::new().format(value).as_bytes());
    }

    fn push_int_aligned<I: itoa::Integer>(&mut self, value: I, align: usize) {
        let mut buffer = itoa::Buffer::new();
        let number = buffer.format(value);
        for _ in 0 .. align.saturating_sub(number.len()) {
            self.push(b'0');
        }
        self.push_bytes(number.as_bytes());
    }

    fn push_float(&mut self, value: f32) {
        let int = value as i32;
        if int as f32 == value {
            self.push_int(int);
        } else {
            self.push_bytes(ryu::Buffer::new().format(value).as_bytes());
        }
    }

    fn push_hex(&mut self, value: u8) {
        fn hex(b: u8) -> u8 {
            if b < 10 { b'0' + b } else { b'A' + (b - 10) }
        }

        self.push(hex(value >> 4));
        self.push(hex(value & 0xF));
    }

    fn push_hex_u16(&mut self, value: u16) {
        self.push_hex((value >> 8) as u8);
        self.push_hex(value as u8);
    }
}

/// A PDF object.
pub trait Object {
    /// Write the object into a buffer.
    fn write(self, buf: &mut Vec<u8>);
}

impl Object for bool {
    fn write(self, buf: &mut Vec<u8>) {
        if self {
            buf.push_bytes(b"true");
        } else {
            buf.push_bytes(b"false");
        }
    }
}

impl Object for i32 {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_int(self);
    }
}

impl Object for f32 {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_float(self);
    }
}

/// A PDF string object (any byte sequence).
///
/// Written as `(Thing)` in a file.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Str<'a>(pub &'a [u8]);

impl Object for Str<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        // TODO: Escape when necessary, select best encoding, reserve size
        // upfront.
        buf.push(b'(');
        buf.push_bytes(self.0);
        buf.push(b')');
    }
}

/// A PDF name object.
///
/// Written as `/Thing` in a file.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a [u8]);

impl Object for Name<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push(b'/');
        buf.push_bytes(self.0);
    }
}

/// An indirect reference.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ref(NonZeroI32);

impl Ref {
    /// Create a new indirect reference.
    ///
    /// The provided value must be in the range `1..=i32::MAX`.
    ///
    /// # Panics
    /// Panics if `id` is out of the valid range.
    pub fn new(id: i32) -> Ref {
        let val = if id > 0 { NonZeroI32::new(id) } else { None };
        Self(val.expect("indirect reference out of valid range"))
    }

    /// Return the underlying number as a primitive type.
    pub fn get(self) -> i32 {
        self.0.get()
    }
}

impl Object for Ref {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_int(self.0.get());
        buf.push_bytes(b" 0 R");
    }
}

/// A rectangle, specified by two opposite corners.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    /// The x-coordinate of the first (typically, lower-left) corner.
    pub x1: f32,
    /// The y-coordinate of the first (typically, lower-left) corner.
    pub y1: f32,
    /// The x-coordinate of the second (typically, upper-right) corner.
    pub x2: f32,
    /// The y-coordinate of the second (typically, upper-right) corner.
    pub y2: f32,
}

impl Rect {
    /// Create a new rectangle from four coordinate values.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

impl Object for Rect {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push(b'[');
        buf.push_val(self.x1);
        buf.push(b' ');
        buf.push_val(self.y1);
        buf.push(b' ');
        buf.push_val(self.x2);
        buf.push(b' ');
        buf.push_val(self.y2);
        buf.push(b']');
    }
}

/// Finishes an entity when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub trait Guard {
    /// Finish the entity.
    fn end(&self, writer: &mut PdfWriter);
}

impl Guard for () {
    fn end(&self, _: &mut PdfWriter) {}
}

/// A guard that finishes an indirect object when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub struct IndirectGuard<G: Guard = ()> {
    guard: G,
}

impl<G: Guard> IndirectGuard<G> {
    fn start(w: &mut PdfWriter, id: Ref, guard: G) -> Self {
        assert_eq!(w.depth, 0);
        w.depth += 1;
        w.offsets.push((id, w.buf.len()));
        w.buf.push_int(id.0.get());
        w.buf.push_bytes(b" 0 obj\n");
        w.push_indent();
        Self { guard }
    }
}

impl<G: Guard> Guard for IndirectGuard<G> {
    fn end(&self, w: &mut PdfWriter) {
        w.depth -= 1;
        w.buf.push_bytes(b"\nendobj\n\n");
        self.guard.end(w);
    }
}

/// A guard that finishes a stream when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub struct StreamGuard<'a, G: Guard = ()> {
    data: &'a [u8],
    guard: G,
}

impl<'a, G: Guard> StreamGuard<'a, G> {
    fn new(data: &'a [u8], guard: G) -> Self {
        Self { data, guard }
    }
}

impl<G: Guard> Guard for StreamGuard<'_, G> {
    fn end(&self, w: &mut PdfWriter) {
        w.buf.push_bytes(b"\nstream\n");
        w.buf.push_bytes(self.data);
        w.buf.push_bytes(b"\nendstream");
        self.guard.end(w);
    }
}

/// Writer for an arbitrary object.
pub struct Any<'a, G: Guard = ()> {
    w: &'a mut PdfWriter,
    guard: G,
}

impl<'a, G: Guard> Any<'a, G> {
    fn new(w: &'a mut PdfWriter, guard: G) -> Self {
        Self { w, guard }
    }

    /// Write a basic object.
    pub fn obj<T: Object>(self, object: T) {
        object.write(&mut self.w.buf);
    }

    /// Write an array.
    pub fn array(self) -> Array<'a, G> {
        Array::start(self.w, self.guard)
    }

    /// Write a dictionary.
    pub fn dict(self) -> Dict<'a, G> {
        Dict::start(self.w, self.guard)
    }
}

/// Writer for an array.
pub struct Array<'a, G: Guard = ()> {
    w: &'a mut PdfWriter,
    len: i32,
    guard: G,
}

impl<'a, G: Guard> Array<'a, G> {
    fn start(w: &'a mut PdfWriter, guard: G) -> Self {
        w.buf.push(b'[');
        Self { w, len: 0, guard }
    }

    /// Write an item with a basic object value.
    ///
    /// This is a shorthand for `array.any().obj(value)`.
    pub fn item<T: Object>(&mut self, object: T) -> &mut Self {
        self.any().obj(object);
        self
    }

    /// Write any object item.
    pub fn any(&mut self) -> Any<'_> {
        if self.len != 0 {
            self.w.buf.push(b' ');
        }
        self.len += 1;
        Any::new(self.w, ())
    }

    /// The number of written items.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Object>(self) -> TypedArray<'a, T, G> {
        TypedArray::new(self)
    }
}

impl<G: Guard> Drop for Array<'_, G> {
    fn drop(&mut self) {
        self.w.buf.push(b']');
        self.guard.end(self.w);
    }
}

/// Writer for an array with fixed primitive value type.
pub struct TypedArray<'a, T, G: Guard = ()> {
    array: Array<'a, G>,
    phantom: PhantomData<T>,
}

impl<'a, T: Object, G: Guard> TypedArray<'a, T, G> {
    /// Wrap an array to make it type-safe.
    pub fn new(array: Array<'a, G>) -> Self {
        Self { array, phantom: PhantomData }
    }

    /// Write an item.
    pub fn item(&mut self, value: T) -> &mut Self {
        self.array.any().obj(value);
        self
    }

    /// Write a sequence of items.
    pub fn items(&mut self, values: impl IntoIterator<Item = T>) -> &mut Self {
        for value in values {
            self.item(value);
        }
        self
    }

    /// The number of written items.
    pub fn len(&self) -> i32 {
        self.array.len()
    }
}

/// Writer for a dictionary.
pub struct Dict<'a, G: Guard = ()> {
    w: &'a mut PdfWriter,
    len: i32,
    guard: G,
}

impl<'a, G: Guard> Dict<'a, G> {
    fn start(w: &'a mut PdfWriter, guard: G) -> Self {
        w.buf.push_bytes(b"<<\n");
        w.depth += 1;
        Self { w, len: 0, guard }
    }

    /// Write a pair with a basic object value.
    ///
    /// This is a shorthand for `dict.key(key).obj(value)`.
    pub fn pair<T: Object>(&mut self, key: Name, object: T) -> &mut Self {
        self.key(key).obj(object);
        self
    }

    /// Write a pair with any object as the value.
    pub fn key(&mut self, key: Name) -> Any<'_> {
        if self.len != 0 {
            self.w.buf.push(b'\n');
        }
        self.len += 1;
        self.w.push_indent();
        self.w.buf.push_val(key);
        self.w.buf.push(b' ');
        Any::new(self.w, ())
    }

    /// The number of written pairs.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Object>(self) -> TypedDict<'a, T, G> {
        TypedDict::new(self)
    }
}

impl<G: Guard> Drop for Dict<'_, G> {
    fn drop(&mut self) {
        self.w.depth -= 1;
        if self.len != 0 {
            self.w.buf.push(b'\n');
        }
        self.w.push_indent();
        self.w.buf.push_bytes(b">>");
        self.guard.end(self.w);
    }
}

/// Writer for a dictionary with fixed primitive value type.
pub struct TypedDict<'a, T, G: Guard = ()> {
    dict: Dict<'a, G>,
    phantom: PhantomData<T>,
}

impl<'a, T: Object, G: Guard> TypedDict<'a, T, G> {
    /// Wrap a dictionary to make it type-safe.
    pub fn new(dict: Dict<'a, G>) -> Self {
        Self { dict, phantom: PhantomData }
    }

    /// Write a key-value pair.
    pub fn pair(&mut self, key: Name, value: T) -> &mut Self {
        self.dict.pair(key, value);
        self
    }

    /// The number of written pairs.
    pub fn len(&self) -> i32 {
        self.dict.len()
    }
}
