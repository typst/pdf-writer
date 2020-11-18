/*!
A PDF writer.

# Example

Writing a document with one A4 page containing the text "Hello World from Rust".
```
use pdf_writer::{Name, PdfWriter, Rect, Ref, TextStream};

# fn main() -> std::io::Result<()> {
let catalog_id = Ref::new(1);
let tree_id = Ref::new(2);
let page_id = Ref::new(3);
let font_id = Ref::new(4);
let text_id = Ref::new(5);

// Write the PDF-1.7 header.
let mut writer = PdfWriter::new(1, 7);
writer.set_indent(2);

// Write the document catalog and a page tree with one page.
writer.catalog(catalog_id).pages(tree_id);
writer.pages(tree_id).kids(vec![page_id]);
writer.page(page_id)
    .parent(tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .contents(text_id)
    .resources()
    .fonts()
    .pair("F1", font_id);

// The font we want to use (one of the base-14 fonts) and a line of text.
writer.type1_font(font_id).base_font(Name("Helvetica"));
writer.stream(
    text_id,
    TextStream::new()
        .tf(Name("F1"), 14.0)
        .td(108.0, 734.0)
        .tj(b"Hello World from Rust!")
        .end(),
);

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/hello.pdf", writer.end(catalog_id))?;
# Ok(())
# }
```
*/

#![deny(missing_docs)]

mod structure;
mod text;

pub use structure::*;
pub use text::*;

use std::convert::TryFrom;
use std::marker::PhantomData;
use std::num::NonZeroI32;

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
    depth: usize,
    indent: usize,
}

impl PdfWriter {
    /// Create a new PDF writer with the default buffer capacity (currently 8 KB).
    ///
    /// The buffer will grow as necessary, but you can override this initial value by
    /// using [`with_capacity`].
    ///
    /// This already writes the PDF header containing the version.
    ///
    /// [`with_capacity`]: #method.with_capacity
    pub fn new(major: i32, minor: i32) -> Self {
        Self::with_capacity(8 * 1024, major, minor)
    }

    /// Create a new PDF writer with the specified buffer capacity.
    ///
    /// This already writes the PDF header containing the version.
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
    pub fn indirect(&mut self, id: Ref) -> Object<'_> {
        self.start_indirect(id);
        Object::new(self, true)
    }

    /// Write an indirectly referenceable stream.
    pub fn stream(&mut self, id: Ref, data: impl AsRef<[u8]>) {
        let data = data.as_ref();
        let len = i32::try_from(data.len()).expect("data is too long");

        self.start_indirect(id);

        Dict::start(self, false).pair("Length", len);

        self.buf.push_bytes(b"\nstream\n");
        self.buf.push_bytes(data);
        self.buf.push_bytes(b"\nendstream");

        self.end_indirect();
    }

    /// Write the cross-reference table and file trailer and return the underlying buffer.
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

        Dict::start(self, false)
            .pair("Size", xref_len)
            .pair("Root", catalog_id);

        // Write where the cross-reference table starts.
        self.buf.push_bytes(b"\nstartxref\n");
        self.buf.push_int(xref_offset);

        // Write the end of file marker.
        self.buf.push_bytes(b"\n%%EOF");
    }

    fn start_indirect(&mut self, id: Ref) {
        assert_eq!(self.depth, 0);
        self.depth += 1;
        self.offsets.push((id, self.buf.len()));
        self.buf.push_int(id.0.get());
        self.buf.push_bytes(b" 0 obj\n");
        self.push_indent();
    }

    fn end_indirect(&mut self) {
        self.depth -= 1;
        self.buf.push_bytes(b"\nendobj\n\n");
    }

    fn push_indent(&mut self) {
        let width = self.indent * self.depth;
        for _ in 0 .. width {
            self.buf.push(b' ');
        }
    }

    /// Start writing the document catalog.
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
}

trait BufExt {
    fn push_val<T: Primitive>(&mut self, primitive: T);
    fn push_bytes(&mut self, bytes: &[u8]);
    fn push_str(&mut self, s: &str);
    fn push_int<I: itoa::Integer>(&mut self, value: I);
    fn push_int_aligned<I: itoa::Integer>(&mut self, value: I, align: usize);
    fn push_float<F: ryu::Float>(&mut self, value: F);
    fn push_hex(&mut self, value: u8);
}

impl BufExt for Vec<u8> {
    fn push_val<T: Primitive>(&mut self, primitive: T) {
        primitive.format(self);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn push_str(&mut self, s: &str) {
        self.push_bytes(s.as_bytes());
    }

    fn push_int<I: itoa::Integer>(&mut self, value: I) {
        self.push_str(itoa::Buffer::new().format(value));
    }

    fn push_int_aligned<I: itoa::Integer>(&mut self, value: I, align: usize) {
        let mut buffer = itoa::Buffer::new();
        let number = buffer.format(value);
        for _ in 0 .. align.saturating_sub(number.len()) {
            self.push(b'0');
        }
        self.push_str(number);
    }

    fn push_float<F: ryu::Float>(&mut self, value: F) {
        self.push_str(ryu::Buffer::new().format(value));
    }

    fn push_hex(&mut self, value: u8) {
        fn hex(b: u8) -> u8 {
            if b < 10 { b'0' + b } else { b'A' + (b - 10) }
        }

        self.push(hex(value >> 4));
        self.push(hex(value & 0xF));
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

/// A name: `/Thing`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a str);

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

/// A basic PDF type.
pub trait Primitive {
    #[doc(hidden)]
    fn format(self, buf: &mut Vec<u8>);
}

impl Primitive for bool {
    fn format(self, buf: &mut Vec<u8>) {
        if self {
            buf.push_bytes(b"true");
        } else {
            buf.push_bytes(b"false");
        }
    }
}

impl Primitive for i32 {
    fn format(self, buf: &mut Vec<u8>) {
        buf.push_int(self);
    }
}

impl Primitive for f32 {
    fn format(self, buf: &mut Vec<u8>) {
        buf.push_float(self);
    }
}

impl Primitive for Ref {
    fn format(self, buf: &mut Vec<u8>) {
        buf.push_int(self.0.get());
        buf.push_bytes(b" 0 R");
    }
}

impl Primitive for Name<'_> {
    fn format(self, buf: &mut Vec<u8>) {
        buf.push(b'/');
        buf.push_str(self.0);
    }
}

impl Primitive for Rect {
    fn format(self, buf: &mut Vec<u8>) {
        buf.push(b'[');
        buf.push_float(self.x1);
        buf.push(b' ');
        buf.push_float(self.y1);
        buf.push(b' ');
        buf.push_float(self.x2);
        buf.push(b' ');
        buf.push_float(self.y2);
        buf.push(b']');
    }
}

/// Writer for an arbitrary object.
pub struct Object<'a> {
    w: &'a mut PdfWriter,
    indirect: bool,
}

impl<'a> Object<'a> {
    fn new(w: &'a mut PdfWriter, indirect: bool) -> Self {
        Self { w, indirect }
    }

    /// Write a primitive.
    pub fn primitive<T: Primitive>(self, value: T) {
        value.format(&mut self.w.buf);
        if self.indirect {
            self.w.end_indirect();
        }
    }

    // TODO: String (simple & streaming?).

    /// Write an array.
    pub fn array(self) -> Array<'a> {
        Array::start(self.w, self.indirect)
    }

    /// Write a dictionary.
    pub fn dict(self) -> Dict<'a> {
        Dict::start(self.w, self.indirect)
    }

    // TODO: Null object.
}

/// Writer for an array.
pub struct Array<'a> {
    w: &'a mut PdfWriter,
    indirect: bool,
    len: i32,
}

impl<'a> Array<'a> {
    fn start(w: &'a mut PdfWriter, indirect: bool) -> Self {
        w.buf.push(b'[');
        Self { w, len: 0, indirect }
    }

    /// Write an item.
    ///
    /// This is a shorthand for `array.obj().primitive(value)`.
    pub fn item<T: Primitive>(&mut self, value: T) -> &mut Self {
        self.obj().primitive(value);
        self
    }

    /// Write any object item.
    pub fn obj(&mut self) -> Object<'_> {
        if self.len != 0 {
            self.w.buf.push(b' ');
        }
        self.len += 1;
        Object::new(self.w, false)
    }

    /// The number of written items.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Primitive>(self) -> TypedArray<'a, T> {
        TypedArray::new(self)
    }
}

impl Drop for Array<'_> {
    fn drop(&mut self) {
        self.w.buf.push(b']');
        if self.indirect {
            self.w.end_indirect();
        }
    }
}

/// Writer for an array with fixed primitive value type.
pub struct TypedArray<'a, T> {
    array: Array<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T: Primitive> TypedArray<'a, T> {
    /// Wrap an array to make it type-safe.
    pub fn new(array: Array<'a>) -> Self {
        Self { array, phantom: PhantomData }
    }

    /// Write an item.
    pub fn item(&mut self, value: T) -> &mut Self {
        self.array.obj().primitive(value);
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
pub struct Dict<'a> {
    w: &'a mut PdfWriter,
    indirect: bool,
    len: i32,
}

impl<'a> Dict<'a> {
    fn start(w: &'a mut PdfWriter, indirect: bool) -> Self {
        w.buf.push_bytes(b"<<\n");
        w.depth += 1;
        Self { w, len: 0, indirect }
    }

    /// Write a pair with a primitive value.
    ///
    /// This is a shorthand for `dict.key(key).primitive(value)`.
    pub fn pair<T: Primitive>(&mut self, key: &str, value: T) -> &mut Self {
        self.key(key).primitive(value);
        self
    }

    /// Write a pair with any object as the value.
    pub fn key(&mut self, key: &str) -> Object<'_> {
        if self.len != 0 {
            self.w.buf.push(b'\n');
        }
        self.len += 1;
        self.w.push_indent();
        self.w.buf.push(b'/');
        self.w.buf.push_str(key);
        self.w.buf.push(b' ');
        Object::new(self.w, false)
    }

    /// The number of written pairs.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Primitive>(self) -> TypedDict<'a, T> {
        TypedDict::new(self)
    }
}

impl Drop for Dict<'_> {
    fn drop(&mut self) {
        self.w.depth -= 1;
        if self.len != 0 {
            self.w.buf.push(b'\n');
        }
        self.w.push_indent();
        self.w.buf.push_bytes(b">>");
        if self.indirect {
            self.w.end_indirect();
        }
    }
}

/// Writer for a dictionary with fixed primitive value type.
pub struct TypedDict<'a, T> {
    dict: Dict<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T: Primitive> TypedDict<'a, T> {
    /// Wrap a dictionary to make it type-safe.
    pub fn new(dict: Dict<'a>) -> Self {
        Self { dict, phantom: PhantomData }
    }

    /// Write a key-value pair.
    pub fn pair(&mut self, key: &str, value: T) -> &mut Self {
        self.dict.pair(key, value);
        self
    }

    /// The number of written pairs.
    pub fn len(&self) -> i32 {
        self.dict.len()
    }
}
