use std::marker::PhantomData;
use std::num::NonZeroI32;

use super::*;

/// A primitive PDF object.
pub trait Primitive {
    /// Write the object into a buffer.
    fn write(self, buf: &mut Vec<u8>);
}

impl Primitive for bool {
    fn write(self, buf: &mut Vec<u8>) {
        if self {
            buf.push_bytes(b"true");
        } else {
            buf.push_bytes(b"false");
        }
    }
}

impl Primitive for i32 {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_int(self);
    }
}

impl Primitive for f32 {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_float(self);
    }
}

/// A string object (any byte sequence).
///
/// This is usually written as `(Thing)`. However, it falls back to hexadecimal
/// form (e.g. `<2829>` for the string `"()"`) if the byte sequence contains any
/// of the three ASCII characters `\`, `(` or `)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Str<'a>(pub &'a [u8]);

impl Primitive for Str<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        if self.0.iter().any(|b| matches!(b, b'\\' | b'(' | b')')) {
            buf.reserve(2 + 2 * self.0.len());
            buf.push(b'<');
            for &byte in self.0 {
                buf.push_hex(byte);
            }
            buf.push(b'>');
        } else {
            buf.push(b'(');
            buf.push_bytes(self.0);
            buf.push(b')');
        }
    }
}

/// A UTF-16BE-encoded text string object.
///
/// This is written as `(BOM <bytes>)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextStr<'a>(pub &'a str);

impl Primitive for TextStr<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push(b'(');
        buf.push(254);
        buf.push(255);
        buf.extend(
            self.0
                .encode_utf16()
                .flat_map(|x| std::array::IntoIter::new(x.to_be_bytes())),
        );
        buf.push(b')');
    }
}

/// A name object.
///
/// Written as `/Thing`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a [u8]);

impl Primitive for Name<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push(b'/');
        for &byte in self.0 {
            if matches!(byte, b'!' ..= b'~') && byte != b'#' {
                buf.push(byte);
            } else {
                buf.push(b'#');
                buf.push_hex(byte);
            }
        }
    }
}

/// The null object.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;

impl Primitive for Null {
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_bytes(b"null");
    }
}

/// A reference to an indirect object.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ref(NonZeroI32);

impl Ref {
    /// Create a new indirect reference.
    ///
    /// The provided value must be greater than zero.
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

impl Primitive for Ref {
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

impl Primitive for Rect {
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

/// Writer for an arbitrary object.
#[must_use = "not consuming this leaves the writer in an inconsistent state"]
pub struct Obj<'a, G: Guard = ()> {
    w: &'a mut PdfWriter,
    guard: G,
}

impl<'a, G: Guard> Obj<'a, G> {
    pub(crate) fn new(w: &'a mut PdfWriter, guard: G) -> Self {
        Self { w, guard }
    }

    /// Write a primitive object.
    pub fn primitive<T: Primitive>(self, value: T) {
        value.write(&mut self.w.buf);
        self.guard.finish(self.w);
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
    pub(crate) fn start(w: &'a mut PdfWriter, guard: G) -> Self {
        w.buf.push(b'[');
        Self { w, len: 0, guard }
    }

    /// Write an item with a primitive object value.
    ///
    /// This is a shorthand for `array.obj().primitive(value)`.
    pub fn item<T: Primitive>(&mut self, value: T) -> &mut Self {
        self.obj().primitive(value);
        self
    }

    /// Write an item with an arbitrary object value.
    pub fn obj(&mut self) -> Obj<'_> {
        if self.len != 0 {
            self.w.buf.push(b' ');
        }
        self.len += 1;
        Obj::new(self.w, ())
    }

    /// The number of written items.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Primitive>(self) -> TypedArray<'a, T, G> {
        TypedArray::new(self)
    }
}

impl<G: Guard> Drop for Array<'_, G> {
    fn drop(&mut self) {
        self.w.buf.push(b']');
        self.guard.finish(self.w);
    }
}

/// Writer for an array with fixed primitive value type.
pub struct TypedArray<'a, T, G: Guard = ()> {
    array: Array<'a, G>,
    phantom: PhantomData<T>,
}

impl<'a, T: Primitive, G: Guard> TypedArray<'a, T, G> {
    /// Wrap an array to make it type-safe.
    pub fn new(array: Array<'a, G>) -> Self {
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
pub struct Dict<'a, G: Guard = ()> {
    w: &'a mut PdfWriter,
    len: i32,
    guard: G,
}

impl<'a, G: Guard> Dict<'a, G> {
    pub(crate) fn start(w: &'a mut PdfWriter, guard: G) -> Self {
        w.buf.push_bytes(b"<<\n");
        w.depth += 1;
        Self { w, len: 0, guard }
    }

    /// Write a pair with a primitive object value.
    ///
    /// This is a shorthand for `dict.key(key).primitive(value)`.
    pub fn pair<T: Primitive>(&mut self, key: Name, value: T) -> &mut Self {
        self.key(key).primitive(value);
        self
    }

    /// Write a pair with an arbitrary object value.
    pub fn key(&mut self, key: Name) -> Obj<'_> {
        if self.len != 0 {
            self.w.buf.push(b'\n');
        }
        self.len += 1;
        self.w.push_indent();
        self.w.buf.push_val(key);
        self.w.buf.push(b' ');
        Obj::new(self.w, ())
    }

    /// The number of written pairs.
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Convert into the typed version.
    pub fn typed<T: Primitive>(self) -> TypedDict<'a, T, G> {
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
        self.guard.finish(self.w);
    }
}

/// Writer for a dictionary with fixed primitive value type.
pub struct TypedDict<'a, T, G: Guard = ()> {
    dict: Dict<'a, G>,
    phantom: PhantomData<T>,
}

impl<'a, T: Primitive, G: Guard> TypedDict<'a, T, G> {
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
