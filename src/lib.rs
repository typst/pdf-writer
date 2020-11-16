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
//!     writer.pages(Ref::new(2)).count(0);
//!     writer.end(Ref::new(1));
//!
//!     std::fs::write("target/hello.pdf", writer.into_buf())
//! }
//! ```

#![deny(missing_docs)]

use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::num::NonZeroU32;

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

macro_rules! write_pair {
    ($w:expr, $key:expr, $($rest:tt)*) => {{
        $w.write_indent();
        write!($w, "/{} ", $key);
        writeln!($w, $($rest)*);
    }};
}

/// An indirect reference.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ref(NonZeroU32);

impl Ref {
    /// Create a new indirect reference. The provided value must be larger than zero.
    ///
    /// # Panics
    /// Panics if `id` is zero.
    pub fn new(id: u32) -> Ref {
        Ref(NonZeroU32::new(id).expect("indirect reference must be larger than zero"))
    }

    /// Return the underlying number as a primitive type.
    pub fn get(self) -> u32 {
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

    /// Start writing the document catalog.
    pub fn catalog(&mut self, id: Ref) -> Catalog {
        Catalog::start(self, id)
    }

    /// Start writing the page tree.
    pub fn pages(&mut self, id: Ref) -> Pages {
        Pages::start(self, id)
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

    fn xref_table(&mut self) -> (u32, usize) {
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

    fn trailer(&mut self, root: Ref, xref_len: u32, xref_offset: usize) {
        // Write the trailer dictionary.
        writeln!(self, "trailer");

        self.depth += 1;
        self.start_dict();
        write_pair!(self, "Size", xref_len);
        write_pair!(self, "Root", root);
        self.end_dict();
        self.depth -= 1;

        // Write where the cross-reference table starts.
        writeln!(self, "startxref");
        writeln!(self, xref_offset);

        // Write the end of file marker.
        writeln!(self, "%%EOF");
    }

    fn start_obj(&mut self, id: Ref) {
        self.write_indent();
        self.offsets.push((id, self.buf.len()));
        writeln!(self, "{} obj", id);
        self.depth += 1;
    }

    fn start_dict(&mut self) {
        self.write_indent();
        writeln!(self, "<<");
        self.depth += 1;
    }

    fn end_obj(&mut self) {
        assert!(self.depth > 0);
        self.depth -= 1;
        self.write_indent();
        writeln!(self, "endobj\n");
    }

    fn end_dict(&mut self) {
        assert!(self.depth > 0);
        self.depth -= 1;
        self.write_indent();
        writeln!(self, ">>");
    }

    fn write_indent(&mut self) {
        let width = self.indent * self.depth;
        for _ in 0..width {
            self.buf.push(b' ');
        }
    }
}

/// Writer for the document catalog.
pub struct Catalog<'a> {
    w: &'a mut PdfWriter,
}

impl<'a> Catalog<'a> {
    fn start(w: &'a mut PdfWriter, id: Ref) -> Self {
        w.start_obj(id);
        w.start_dict();
        write_pair!(w, "Type", "/Catalog");
        Self { w }
    }

    /// Write the `/Pages` attribute pointing to the page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        write_pair!(self.w, "Pages", "{} R", id);
        self
    }
}

impl Drop for Catalog<'_> {
    fn drop(&mut self) {
        self.w.end_dict();
        self.w.end_obj();
    }
}

/// Writer for the page tree.
pub struct Pages<'a> {
    w: &'a mut PdfWriter,
}

impl<'a> Pages<'a> {
    fn start(w: &'a mut PdfWriter, id: Ref) -> Self {
        w.start_obj(id);
        w.start_dict();
        write_pair!(w, "Type", "/Pages");
        Self { w }
    }

    /// Write the `/Count` attribute indicating the number of pages.
    pub fn count(&mut self, count: u32) -> &mut Self {
        write_pair!(self.w, "Count", count);
        self
    }
}

impl Drop for Pages<'_> {
    fn drop(&mut self) {
        self.w.end_dict();
        self.w.end_obj();
    }
}
