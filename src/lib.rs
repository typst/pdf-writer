/*!
A PDF writer.

# Minimal example
```
use pdf_writer::{Name, PdfWriter, Rect, Ref};

fn main() -> std::io::Result<()> {
    let mut writer = PdfWriter::new();
    writer.set_indent(2);

    let catalog = Ref::new(1);
    let tree = Ref::new(2);
    let page = Ref::new(3);
    let font = Ref::new(4);

    // Write the PDF-1.7 header.
    writer.start(1, 7);

    // Write the document catalog and a page tree with one page.
    writer.catalog(catalog).pages(tree);
    writer.pages(tree).kids(vec![page]);
    writer.page(page)
        .parent(tree)
        .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
        .resources()
        .fonts()
        .pair("F1", font);

    // The font we want to use (one of the base-14 fonts) and a line of text.
    writer.type1_font(font).base_font(Name("Helvetica"));

    // Finish with the cross-reference table and file trailer.
    writer.end(catalog);

    std::fs::write("target/hello.pdf", writer.into_buf())
}
```
*/

#![deny(missing_docs)]

use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::marker::PhantomData;
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

    /// Start writing an indirectly referencable object.
    pub fn indirect(&mut self, id: Ref) -> Object<'_> {
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
        dict.pair("Size", xref_len);
        dict.pair("Root", root);
        drop(dict);

        // Write where the cross-reference table starts.
        writeln!(self, "startxref");
        writeln!(self, xref_offset);

        // Write the end of file marker.
        writeln!(self, "%%EOF");
    }

    fn start_indirect(&mut self, id: Ref) {
        assert_eq!(self.depth, 0);
        self.depth += 1;
        self.offsets.push((id, self.buf.len()));
        writeln!(self, "{} obj", id);
        self.write_indent();
    }

    fn end_indirect(&mut self) {
        self.depth -= 1;
        writeln!(self);
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

/// A primtive PDF type.
pub trait Primitive {
    #[doc(hidden)]
    fn write(self, w: &mut PdfWriter);
}

impl Primitive for bool {
    fn write(self, w: &mut PdfWriter) {
        write!(w, self);
    }
}

impl Primitive for i32 {
    fn write(self, w: &mut PdfWriter) {
        write!(w, self);
    }
}

impl Primitive for f32 {
    fn write(self, w: &mut PdfWriter) {
        write!(w, self);
    }
}

impl Primitive for Ref {
    fn write(self, w: &mut PdfWriter) {
        write!(w, "{} R", self);
    }
}

impl Primitive for Name<'_> {
    fn write(self, w: &mut PdfWriter) {
        write!(w, "/{}", self.0);
    }
}

impl Primitive for Rect {
    fn write(self, w: &mut PdfWriter) {
        write!(w, "[{} {} {} {}]", self.x1, self.y1, self.x2, self.y2);
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
        value.write(self.w);
        if self.indirect {
            self.w.end_indirect();
        }
    }

    // TODO: String (simple & streaming).

    /// Write an array.
    pub fn array(self) -> Array<'a> {
        Array::start(self.w, self.indirect)
    }

    /// Write a dictionary.
    pub fn dict(self) -> Dict<'a> {
        Dict::start(self.w, self.indirect)
    }

    /// Write a typed dictionary.
    pub fn typed_dict<T: Primitive>(self) -> TypedDict<'a, T> {
        TypedDict::new(Dict::start(self.w, self.indirect))
    }

    // TODO: Stream.
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
        write!(w, "[");
        Self { w, len: 0, indirect }
    }

    /// Write a primitive item.
    ///
    /// This is a shorthand for `array.obj().primitive(value)`.
    pub fn item<T: Primitive>(&mut self, value: T) {
        self.obj().primitive(value);
    }

    /// Write a sequence of primitive item.
    pub fn items<T: Primitive>(&mut self, values: impl IntoIterator<Item = T>) {
        for value in values {
            self.item(value);
        }
    }

    /// Write any object item.
    pub fn obj(&mut self) -> Object<'_> {
        if self.len != 0 {
            write!(self.w, " ");
        }
        self.len += 1;
        Object::new(self.w, false)
    }

    /// The number of written elements.
    pub fn len(&self) -> i32 {
        self.len
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

/// Writer for a dictionary with fixed primitive value type.
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
    pub fn item(&mut self, value: T) {
        self.array.item(value);
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
        writeln!(w, "<<");
        w.depth += 1;
        Self { w, len: 0, indirect }
    }

    /// Write a pair with primitive value.
    ///
    /// This is a shorthand for `dict.key(key).primitive(value)`.
    pub fn pair<T: Primitive>(&mut self, key: &str, value: T) {
        self.key(key).primitive(value);
    }

    /// Write a pair with any object as the value.
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
        self.w.depth -= 1;
        if self.len != 0 {
            writeln!(self.w);
        }
        self.w.write_indent();
        write!(self.w, ">>");
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
    /// Wrap an dictionary to make it type-safe.
    pub fn new(dict: Dict<'a>) -> Self {
        Self { dict, phantom: PhantomData }
    }

    /// Write a key-value pair.
    pub fn pair(&mut self, key: &str, value: T) {
        self.dict.pair(key, value);
    }
}

impl PdfWriter {
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
}

/// Writer for a _document catalog_ dictionary.
pub struct Catalog<'a> {
    dict: Dict<'a>,
}

impl<'a> Catalog<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Catalog"));
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the root page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.dict.pair("Pages", id);
        self
    }
}

/// Writer for a _page tree_ dictionary.
pub struct Pages<'a> {
    dict: Dict<'a>,
}

impl<'a> Pages<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Pages"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) {
        self.dict.pair("Parent", parent);
    }

    /// Write the `/Kids` and `/Count` attributes.
    pub fn kids(&mut self, kids: impl IntoIterator<Item = Ref>) {
        let mut array = self.dict.key("Kids").array();
        for kid in kids {
            array.item(kid);
        }
        let len = array.len();
        drop(array);
        self.dict.pair("Count", len);
    }
}

/// Writer for a _page_ dictionary.
pub struct Page<'a> {
    dict: Dict<'a>,
}

impl<'a> Page<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Page"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair("Parent", parent);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.dict.pair("MediaBox", rect);
        self
    }

    /// Start writing a `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.dict.key("Resources"))
    }
}

/// Writer for a _resource_ dictionary.
pub struct Resources<'a> {
    dict: Dict<'a>,
}

impl<'a> Resources<'a> {
    fn new(obj: Object<'a>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<Ref> {
        self.dict.key("Font").typed_dict()
    }
}

/// Writer for a _Type-1 font_ dictionary.
pub struct Type1Font<'a> {
    dict: Dict<'a>,
}

impl<'a> Type1Font<'a> {
    fn start(obj: Object<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Font"));
        dict.pair("Subtype", Name("Type1"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) {
        self.dict.pair("BaseFont", name);
    }
}
