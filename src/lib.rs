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

macro_rules! write {
    ($buf:expr, $value:expr) => {{
        write!($buf, "{}", $value);
    }};
    ($buf:expr, $fmt:literal, $($rest:tt)*) => {{
        $buf.write_fmt(format_args!($fmt, $($rest)*)).unwrap();
    }};
}

macro_rules! writeln {
    ($buf:expr) => {{
        $buf.push(b'\n');
    }};
    ($buf:expr, $($rest:tt)*) => {{
        write!($buf, $($rest)*);
        writeln!($buf);
    }};
}

mod structure;
mod text;

pub use structure::*;
pub use text::*;

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
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
    /// Create a new PDF writer.
    ///
    /// This already writes the PDF header, containing the version, that is:
    /// ```text
    /// %PDF-{major}-{minor}
    /// ```
    pub fn new(major: u32, minor: u32) -> Self {
        let mut buf = vec![];
        writeln!(buf, "%PDF-{}.{}", major, minor);
        writeln!(buf);
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

        writeln!(self.buf, "stream");
        self.buf.extend(data);
        writeln!(self.buf, "endstream");

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

        writeln!(self.buf, "xref");
        writeln!(self.buf, "0 {}", xref_len);

        // Always write the initial entry for unusable id zero.
        write!(self.buf, "0000000000 65535 f\r\n");
        let mut next = 1;

        for (id, offset) in &offsets {
            let id = id.get();
            while next < id {
                // TODO: Form linked list of free items.
                write!(self.buf, "0000000000 65535 f\r\n");
                next += 1;
            }

            write!(self.buf, "{:010} 00000 n\r\n", offset);
            next = id + 1;
        }

        (xref_len, xref_offset)
    }

    fn trailer(&mut self, catalog_id: Ref, xref_len: i32, xref_offset: usize) {
        // Write the trailer dictionary.
        writeln!(self.buf, "trailer");

        Dict::start(self, false)
            .pair("Size", xref_len)
            .pair("Root", catalog_id);

        // Write where the cross-reference table starts.
        writeln!(self.buf, "startxref");
        writeln!(self.buf, xref_offset);

        // Write the end of file marker.
        writeln!(self.buf, "%%EOF");
    }

    fn start_indirect(&mut self, id: Ref) {
        assert_eq!(self.depth, 0);
        self.depth += 1;
        self.offsets.push((id, self.buf.len()));
        writeln!(self.buf, "{} obj", id);
        self.write_indent();
    }

    fn end_indirect(&mut self) {
        self.depth -= 1;
        writeln!(self.buf);
        writeln!(self.buf, "endobj");
        writeln!(self.buf);
    }

    fn write_indent(&mut self) {
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

impl Display for Ref {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // We do not use any generations other than zero.
        std::write!(f, "{} 0", self.0)
    }
}

/// A name: `/Thing`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a str);

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        std::write!(f, "/{}", self.0)
    }
}

/// A rectangle, specified by two opposite corners.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
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
    /// Write the primitive into a byte buffer.
    fn write(self, buf: &mut Vec<u8>);
}

impl Primitive for bool {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, self);
    }
}

impl Primitive for i32 {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, self);
    }
}

impl Primitive for f32 {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, self);
    }
}

impl Primitive for Ref {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, "{} R", self);
    }
}

impl Primitive for Name<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, "{}", self);
    }
}

impl Primitive for Rect {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, "[{} {} {} {}]", self.x1, self.y1, self.x2, self.y2);
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
        value.write(&mut self.w.buf);
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
        write!(w.buf, "[");
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
            write!(self.w.buf, " ");
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
        write!(self.w.buf, "]");
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
        writeln!(w.buf, "<<");
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
            writeln!(self.w.buf);
        }
        self.len += 1;
        self.w.write_indent();
        write!(self.w.buf, "/{} ", key);
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
            writeln!(self.w.buf);
        }
        self.w.write_indent();
        write!(self.w.buf, ">>");
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
