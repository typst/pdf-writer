//! A PDF writer.
//!
//! # Minimal example
//! ```
//! use pdf_writer::{PdfWriter, Ref};
//!
//! fn main() -> std::io::Result<()> {
//!     let mut writer = PdfWriter::new();
//!     writer.set_indent(2);
//!
//!     // Write the PDF-1.7 header, document catalog, page tree and
//!     // finish with the cross-reference table and file trailer.
//!     writer.start(1, 7);
//!     writer.catalog(Ref::new(1)).pages(Ref::new(2));
//!
//!     let mut pages = writer.pages(Ref::new(2));
//!     pages.count(1);
//!     pages.kids().item().id(Ref::new(3));
//!     drop(pages);
//!
//!     writer.end(Ref::new(1));
//!
//!     std::fs::write("target/hello.pdf", writer.into_buf())
//! }
//! ```

#![deny(missing_docs)]

use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::num::NonZeroI32;

macro_rules! write {
    ($w:expr, $fmt:literal) => {{
        $w.buf.extend($fmt.as_bytes());
    }};
    ($w:expr, $value:expr) => {{
        write!($w, "{}", $value);
    }};
    ($w:expr, $fmt:literal, $($rest:tt)*) => {{
        $w.buf.write_fmt(format_args!($fmt, $($rest)*)).unwrap();
    }};
}

macro_rules! writeln {
    ($w:expr) => {{
        $w.buf.push(b'\n');
    }};
    ($w:expr, $($rest:tt)*) => {{
        write!($w, $($rest)*);
        writeln!($w);
    }};
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
    /// Panics if `id` is zero.
    pub fn new(id: u32) -> Ref {
        Self(
            id.try_into()
                .ok()
                .and_then(NonZeroI32::new)
                .expect("indirect reference out of valid range"),
        )
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

/// The root writer.
pub struct PdfWriter {
    buf: Vec<u8>,
    offsets: Vec<(Ref, usize)>,
    depth: usize,
    indent: usize,
}

impl PdfWriter {
    /// Create a new PDF writer.
    pub fn new() -> Self {
        Self {
            buf: vec![],
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

    /// Write the PDF header, containing the version.
    ///
    /// That is, the following portion:
    /// ```text
    /// %PDF-{major}-{minor}
    /// ```
    pub fn start(&mut self, major: u32, minor: u32) {
        writeln!(self, "%PDF-{}.{}\n", major, minor);
    }

    /// Start writing an arbitrary indirect object.
    pub fn obj(&mut self, id: Ref) -> Object<'_> {
        self.start_indirect(id);
        Object::new(self, true)
    }

    /// Write the cross-reference table and file trailer.
    pub fn end(&mut self, root: Ref) {
        assert_eq!(self.depth, 0);
        let (xref_len, xref_offset) = self.xref_table();
        self.trailer(root, xref_len, xref_offset)
    }

    /// Return the underlying buffer.
    pub fn into_buf(self) -> Vec<u8> {
        self.buf
    }

    fn xref_table(&mut self) -> (i32, usize) {
        let mut offsets = std::mem::take(&mut self.offsets);
        offsets.sort();

        let xref_len = 1 + offsets.last().map(|p| p.0.get()).unwrap_or(0);
        let xref_offset = self.buf.len();

        writeln!(self, "xref");
        writeln!(self, "0 {}", xref_len);

        // Always write the initial entry for unusable id zero.
        write!(self, "0000000000 65535 f\r\n");
        let mut next = 1;

        for (id, offset) in &offsets {
            let id = id.get();
            while next < id {
                // TODO: Form linked list of free items.
                write!(self, "0000000000 65535 f\r\n");
                next += 1;
            }

            write!(self, "{:010} 00000 n\r\n", offset);
            next = id + 1;
        }

        (xref_len, xref_offset)
    }

    fn trailer(&mut self, root: Ref, xref_len: i32, xref_offset: usize) {
        // Write the trailer dictionary.
        writeln!(self, "trailer");

        let mut dict = Dict::start(self, false);
        dict.key("Size").int(xref_len);
        dict.key("Root").id(root);
        drop(dict);

        // Write where the cross-reference table starts.
        writeln!(self, "startxref");
        writeln!(self, xref_offset);

        // Write the end of file marker.
        writeln!(self, "%%EOF");
    }

    fn start_indirect(&mut self, id: Ref) {
        assert_eq!(self.depth, 0);
        self.offsets.push((id, self.buf.len()));
        writeln!(self, "{} obj", id);
    }

    fn end_indirect(&mut self) {
        writeln!(self, "endobj");
        writeln!(self);
    }

    fn write_indent(&mut self) {
        let width = self.indent * self.depth;
        for _ in 0 .. width {
            self.buf.push(b' ');
        }
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

    /// Write a boolean.
    pub fn bool(self, value: bool) {
        write!(self.w, value);
    }

    /// Write an integer number.
    pub fn int(self, value: i32) {
        write!(self.w, value);
    }

    /// Write a real number.
    pub fn real(self, value: f32) {
        write!(self.w, value);
    }

    // TODO: String (simple & streaming).

    /// Write a name object.
    pub fn name(self, name: &str) {
        write!(self.w, "/{}", name);
    }

    /// Write an array.
    pub fn array(self) -> Array<'a> {
        Array::start(self.w, self.indirect)
    }

    /// Write a dictionary.
    pub fn dict(self) -> Dict<'a> {
        Dict::start(self.w, self.indirect)
    }

    // TODO: Stream.
    // TODO: Null object.

    /// Write a reference to an indirect object.
    pub fn id(self, id: Ref) {
        write!(self.w, "{} R", id);
    }
}

/// Writer for an array.
pub struct Array<'a> {
    w: &'a mut PdfWriter,
    indirect: bool,
    len: usize,
}

impl<'a> Array<'a> {
    fn start(w: &'a mut PdfWriter, indirect: bool) -> Self {
        write!(w, "[");
        Self { w, len: 0, indirect }
    }

    /// Write an item.
    pub fn item(&mut self) -> Object<'_> {
        if self.len != 0 {
            write!(self.w, " ");
        }
        self.len += 1;
        Object::new(self.w, false)
    }
}

impl Drop for Array<'_> {
    fn drop(&mut self) {
        write!(self.w, "]");
        if self.indirect {
            self.w.end_indirect();
        }
    }
}

/// Writer for a dictionary.
pub struct Dict<'a> {
    w: &'a mut PdfWriter,
    indirect: bool,
    len: usize,
}

impl<'a> Dict<'a> {
    fn start(w: &'a mut PdfWriter, indirect: bool) -> Self {
        w.write_indent();
        writeln!(w, "<<");
        w.depth += 1;
        Self { w, len: 0, indirect }
    }

    /// Write a key-value pair.
    pub fn key(&mut self, key: &str) -> Object<'_> {
        if self.len != 0 {
            writeln!(self.w);
        }
        self.len += 1;
        self.w.write_indent();
        write!(self.w, "/{} ", key);
        Object::new(self.w, false)
    }
}

impl Drop for Dict<'_> {
    fn drop(&mut self) {
        if self.len != 0 {
            writeln!(self.w);
        }
        self.w.depth -= 1;
        self.w.write_indent();
        writeln!(self.w, ">>");
        if self.indirect {
            self.w.end_indirect();
        }
    }
}

impl PdfWriter {
    /// Start writing the document catalog.
    pub fn catalog(&mut self, id: Ref) -> Catalog<'_> {
        Catalog::start(self.obj(id))
    }

    /// Start writing the page tree.
    pub fn pages(&mut self, id: Ref) -> Pages<'_> {
        Pages::start(self.obj(id))
    }
}

/// Writer for the _document catalog_.
pub struct Catalog<'a> {
    dict: Dict<'a>,
}

impl<'a> Catalog<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.key("Type").name("Catalog");
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.dict.key("Pages").id(id);
        self
    }
}

/// Writer for the _page tree_.
pub struct Pages<'a> {
    dict: Dict<'a>,
}

impl<'a> Pages<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.key("Type").name("Pages");
        Self { dict }
    }

    /// Write the `/Count` attribute, indicating the number of elements in the `/Kids`
    /// array.
    pub fn count(&mut self, count: i32) {
        self.dict.key("Count").int(count);
    }

    /// Write the `/Kids` attributes.
    pub fn kids(&mut self) -> Array<'_> {
        self.dict.key("Kids").array()
    }
}
