use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::num::NonZeroI32;

use super::*;

/// A primitive PDF object.
pub trait Primitive {
    /// Write the object into a buffer.
    fn write(self, buf: &mut Vec<u8>);
}

impl<T: Primitive> Primitive for &T
where
    T: Copy,
{
    #[inline]
    fn write(self, buf: &mut Vec<u8>) {
        (*self).write(buf);
    }
}

impl Primitive for bool {
    #[inline]
    fn write(self, buf: &mut Vec<u8>) {
        if self {
            buf.extend(b"true");
        } else {
            buf.extend(b"false");
        }
    }
}

impl Primitive for i32 {
    #[inline]
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_int(self);
    }
}

impl Primitive for f32 {
    #[inline]
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
        // Fall back to hex formatting if the string contains a:
        // - backslash because it is used for escaping,
        // - parenthesis because they are the delimiters,
        // - carriage return (0x0D) because it would be silently
        //   transformed into a newline (0x0A).
        if self.0.iter().any(|b| matches!(b, b'\\' | b'(' | b')' | b'\r')) {
            buf.reserve(2 + 2 * self.0.len());
            buf.push(b'<');
            for &byte in self.0 {
                buf.push_hex(byte);
            }
            buf.push(b'>');
        } else {
            buf.push(b'(');
            buf.extend(self.0);
            buf.push(b')');
        }
    }
}

/// A unicode text string object.
///
/// This is written as a [`Str`] containing a byte order mark followed by
/// UTF-16-BE bytes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextStr<'a>(pub &'a str);

impl Primitive for TextStr<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        let mut bytes = vec![254, 255];
        for v in self.0.encode_utf16() {
            bytes.extend(v.to_be_bytes());
        }
        Str(&bytes).write(buf);
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
            // Quite a few characters must be escaped in names, so we take the
            // safe route and allow only ASCII letters and numbers.
            if byte.is_ascii_alphanumeric() {
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
    #[inline]
    fn write(self, buf: &mut Vec<u8>) {
        buf.extend(b"null");
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
    /// Panics if `id` is out of the valid range.
    #[inline]
    pub fn new(id: i32) -> Ref {
        let val = if id > 0 { NonZeroI32::new(id) } else { None };
        Self(val.expect("indirect reference out of valid range"))
    }

    /// Return the underlying number as a primitive type.
    #[inline]
    pub fn get(self) -> i32 {
        self.0.get()
    }
}

impl Primitive for Ref {
    #[inline]
    fn write(self, buf: &mut Vec<u8>) {
        buf.push_int(self.0.get());
        buf.extend(b" 0 R");
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
    #[inline]
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Convert this rectangle into 8 floats describing the four corners of the
    /// rectangle in counterclockwise order.
    #[inline]
    pub fn to_quad_points(self) -> [f32; 8] {
        [
            self.x1, self.y1, self.x2, self.y1, self.x2, self.y2, self.x1, self.y2,
        ]
    }
}

impl Primitive for Rect {
    #[inline]
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

/// A date, represented as a text string.
///
/// A field is only respected if all superior fields are supplied. For example,
/// to set the minute, the hour, day, etc. have to be set. Similarly, in order
/// for the time zone information to be written, all time information (including
/// seconds) must be written. `utc_offset_minute` is optional if supplying time
/// zone info. It must only be used to specify sub-hour time zone offsets.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Date {
    /// The year (0-9999).
    year: u16,
    /// The month (0-11).
    month: Option<u8>,
    /// The month (0-30).
    day: Option<u8>,
    /// The hour (0-23).
    hour: Option<u8>,
    /// The minute (0-59).
    minute: Option<u8>,
    /// The second (0-59).
    second: Option<u8>,
    /// The hour offset from UTC (-23 through 23).
    utc_offset_hour: Option<i8>,
    /// The minute offset from UTC (0-59). Will carry over the sign from
    /// `utc_offset_hour`.
    utc_offset_minute: u8,
}

impl Date {
    /// Create a new, minimal date. The year will be clamped within the range
    /// 0-9999.
    #[inline]
    pub fn new(year: u16) -> Self {
        Self {
            year: year.min(9999),
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            utc_offset_hour: None,
            utc_offset_minute: 0,
        }
    }

    /// Add the month field. It will be clamped within the range 1-12.
    #[inline]
    pub fn month(mut self, month: u8) -> Self {
        self.month = Some(month.clamp(1, 12));
        self
    }

    /// Add the day field. It will be clamped within the range 1-31.
    #[inline]
    pub fn day(mut self, day: u8) -> Self {
        self.day = Some(day.clamp(1, 31));
        self
    }

    /// Add the hour field. It will be clamped within the range 0-23.
    #[inline]
    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = Some(hour.min(23));
        self
    }

    /// Add the minute field. It will be clamped within the range 0-59.
    #[inline]
    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = Some(minute.min(59));
        self
    }

    /// Add the second field. It will be clamped within the range 0-59.
    #[inline]
    pub fn second(mut self, second: u8) -> Self {
        self.second = Some(second.min(59));
        self
    }

    /// Add the offset from UTC in hours. If not specified, the time will be
    /// assumed to be local to the viewer's time zone. It will be clamped within
    /// the range -23-23.
    #[inline]
    pub fn utc_offset_hour(mut self, hour: i8) -> Self {
        self.utc_offset_hour = Some(hour.clamp(-23, 23));
        self
    }

    /// Add the offset from UTC in minutes. This will have the same sign as set in
    /// [`Self::utc_offset_hour`]. It will be clamped within the range 0-59.
    #[inline]
    pub fn utc_offset_minute(mut self, minute: u8) -> Self {
        self.utc_offset_minute = minute.min(59);
        self
    }
}

impl Primitive for Date {
    fn write(self, buf: &mut Vec<u8>) {
        write!(buf, "(D:{:04}", self.year).unwrap();

        self.month
            .and_then(|month| {
                write!(buf, "{:02}", month).unwrap();
                self.day
            })
            .and_then(|day| {
                write!(buf, "{:02}", day).unwrap();
                self.hour
            })
            .and_then(|hour| {
                write!(buf, "{:02}", hour).unwrap();
                self.minute
            })
            .and_then(|minute| {
                write!(buf, "{:02}", minute).unwrap();
                self.second
            })
            .and_then(|second| {
                write!(buf, "{:02}", second).unwrap();
                self.utc_offset_hour
            })
            .map(|utc_offset_hour| {
                if utc_offset_hour == 0 && self.utc_offset_minute == 0 {
                    buf.push(b'Z');
                } else {
                    write!(buf, "{:+03}'{:02}", utc_offset_hour, self.utc_offset_minute)
                        .unwrap();
                }
            });

        buf.push(b')');
    }
}

/// Writer for an arbitrary object.
#[must_use = "not consuming this leaves the writer in an inconsistent state"]
pub struct Obj<'a> {
    buf: &'a mut Vec<u8>,
    indirect: bool,
    indent: u8,
}

impl<'a> Obj<'a> {
    /// Start a new direct object.
    #[inline]
    pub(crate) fn direct(buf: &'a mut Vec<u8>, indent: u8) -> Self {
        Self { buf, indirect: false, indent }
    }

    /// Start a new indirect object.
    #[inline]
    pub(crate) fn indirect(buf: &'a mut Vec<u8>, id: Ref) -> Self {
        buf.push_int(id.get());
        buf.extend(b" 0 obj\n");
        Self { buf, indirect: true, indent: 0 }
    }

    /// Write a primitive object.
    #[inline]
    pub fn primitive<T: Primitive>(self, value: T) {
        value.write(self.buf);
        if self.indirect {
            self.buf.extend(b"\nendobj\n\n");
        }
    }

    /// Write an array.
    #[inline]
    pub fn array(self) -> Array<'a> {
        self.start()
    }

    /// Write a dictionary.
    #[inline]
    pub fn dict(self) -> Dict<'a> {
        self.start()
    }

    /// Start writing with an arbitrary writer.
    ///
    /// For example, using this, you could write a Type 1 font directly into
    /// a page's resource directionary.
    /// ```
    /// use pdf_writer::{PdfWriter, Ref, Name, writers::Type1Font};
    ///
    /// let mut w = PdfWriter::new();
    /// w.page(Ref::new(1))
    ///     .resources()
    ///     .fonts()
    ///     .insert(Name(b"F1"))
    ///     .start::<Type1Font>()
    ///     .base_font(Name(b"Helvetica"));
    /// ```
    #[inline]
    pub fn start<W: Writer<'a>>(self) -> W {
        W::start(self)
    }
}

/// A writer for a specific type of PDF object.
pub trait Writer<'a> {
    /// Start writing the object.
    fn start(obj: Obj<'a>) -> Self;
}

/// Writer for an array.
pub struct Array<'a> {
    buf: &'a mut Vec<u8>,
    indirect: bool,
    indent: u8,
    len: i32,
}

impl<'a> Writer<'a> for Array<'a> {
    #[inline]
    fn start(obj: Obj<'a>) -> Self {
        obj.buf.push(b'[');
        Self {
            buf: obj.buf,
            indirect: obj.indirect,
            indent: obj.indent,
            len: 0,
        }
    }
}

impl<'a> Array<'a> {
    /// The number of written items.
    #[inline]
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Start writing an arbitrary item.
    #[inline]
    pub fn push(&mut self) -> Obj<'_> {
        if self.len != 0 {
            self.buf.push(b' ');
        }
        self.len += 1;
        Obj::direct(self.buf, self.indent)
    }

    /// Write an item with a primitive value.
    ///
    /// This is a shorthand for `array.push().primitive(value)`.
    #[inline]
    pub fn item<T: Primitive>(&mut self, value: T) -> &mut Self {
        self.push().primitive(value);
        self
    }

    /// Write a sequence of items with primitive values.
    #[inline]
    pub fn items<T: Primitive>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> &mut Self {
        for value in values {
            self.item(value);
        }
        self
    }

    /// Convert into a typed version.
    #[inline]
    pub fn typed<T: Primitive>(self) -> TypedArray<'a, T> {
        TypedArray::wrap(self)
    }
}

impl Drop for Array<'_> {
    #[inline]
    fn drop(&mut self) {
        self.buf.push(b']');
        if self.indirect {
            self.buf.extend(b"\nendobj\n\n");
        }
    }
}

/// Writer for an array of a fixed primitive type.
pub struct TypedArray<'a, T> {
    array: Array<'a>,
    phantom: PhantomData<fn() -> T>,
}

impl<'a, T> TypedArray<'a, T>
where
    T: Primitive,
{
    /// Wrap an array to make it type-safe.
    #[inline]
    pub fn wrap(array: Array<'a>) -> Self {
        Self { array, phantom: PhantomData }
    }

    /// The number of written items.
    #[inline]
    pub fn len(&self) -> i32 {
        self.array.len()
    }

    /// Write an item.
    #[inline]
    pub fn item(&mut self, value: T) -> &mut Self {
        self.array.item(value);
        self
    }

    /// Write a sequence of items.
    #[inline]
    pub fn items(&mut self, values: impl IntoIterator<Item = T>) -> &mut Self {
        self.array.items(values);
        self
    }
}

/// Writer for a dictionary.
pub struct Dict<'a> {
    buf: &'a mut Vec<u8>,
    indirect: bool,
    indent: u8,
    len: i32,
}

impl<'a> Writer<'a> for Dict<'a> {
    #[inline]
    fn start(obj: Obj<'a>) -> Self {
        obj.buf.extend(b"<<");
        Self {
            buf: obj.buf,
            indirect: obj.indirect,
            indent: obj.indent.saturating_add(2),
            len: 0,
        }
    }
}

impl<'a> Dict<'a> {
    /// The number of written pairs.
    #[inline]
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Start writing a pair with an arbitrary value.
    #[inline]
    pub fn insert(&mut self, key: Name) -> Obj<'_> {
        self.len += 1;
        self.buf.push(b'\n');

        for _ in 0 .. self.indent {
            self.buf.push(b' ');
        }

        self.buf.push_val(key);
        self.buf.push(b' ');

        Obj::direct(self.buf, self.indent)
    }

    /// Write a pair with a primitive value.
    ///
    /// This is a shorthand for `dict.insert(key).primitive(value)`.
    #[inline]
    pub fn pair<T: Primitive>(&mut self, key: Name, value: T) -> &mut Self {
        self.insert(key).primitive(value);
        self
    }

    /// Write a sequence of pairs with primitive values.
    pub fn pairs<'n, T: Primitive>(
        &mut self,
        pairs: impl IntoIterator<Item = (Name<'n>, T)>,
    ) -> &mut Self {
        for (key, value) in pairs {
            self.pair(key, value);
        }
        self
    }

    /// Convert into a typed version.
    #[inline]
    pub fn typed<T: Primitive>(self) -> TypedDict<'a, T> {
        TypedDict::wrap(self)
    }
}

impl Drop for Dict<'_> {
    #[inline]
    fn drop(&mut self) {
        if self.len != 0 {
            self.buf.push(b'\n');
            for _ in 0 .. self.indent - 2 {
                self.buf.push(b' ');
            }
        }
        self.buf.extend(b">>");
        if self.indirect {
            self.buf.extend(b"\nendobj\n\n");
        }
    }
}

/// Writer for a dictionary mapping to a fixed primitive type.
pub struct TypedDict<'a, T> {
    dict: Dict<'a>,
    phantom: PhantomData<fn() -> T>,
}

impl<'a, T> TypedDict<'a, T>
where
    T: Primitive,
{
    /// Wrap a dictionary to make it type-safe.
    #[inline]
    pub fn wrap(dict: Dict<'a>) -> Self {
        Self { dict, phantom: PhantomData }
    }

    /// The number of written pairs.
    #[inline]
    pub fn len(&self) -> i32 {
        self.dict.len()
    }

    /// Write a key-value pair.
    #[inline]
    pub fn pair(&mut self, key: Name, value: T) -> &mut Self {
        self.dict.pair(key, value);
        self
    }

    /// Write a sequence of key-value pairs.
    #[inline]
    pub fn pairs<'n>(
        &mut self,
        pairs: impl IntoIterator<Item = (Name<'n>, T)>,
    ) -> &mut Self {
        self.dict.pairs(pairs);
        self
    }
}

/// Writer for an indirect stream object.
pub struct Stream<'a> {
    dict: ManuallyDrop<Dict<'a>>,
    data: &'a [u8],
}

impl<'a> Stream<'a> {
    /// Start writing a stream.
    ///
    /// Panics if the object writer is not indirect or the stream length exceeds
    /// `i32::MAX`.
    pub(crate) fn start(obj: Obj<'a>, data: &'a [u8]) -> Self {
        assert!(obj.indirect);

        let mut dict = obj.dict();
        dict.pair(
            Name(b"Length"),
            i32::try_from(data.len()).unwrap_or_else(|_| {
                panic!("data length (is `{}`) must be <= i32::MAX", data.len());
            }),
        );

        Self { dict: ManuallyDrop::new(dict), data }
    }

    /// Write the `/Filter` attribute.
    pub fn filter(&mut self, filter: Filter) -> &mut Self {
        self.pair(Name(b"Filter"), filter.to_name());
        self
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        self.dict.buf.extend(b"\n>>");
        self.dict.buf.extend(b"\nstream\n");
        self.dict.buf.extend(self.data.as_ref());
        self.dict.buf.extend(b"\nendstream");
        self.dict.buf.extend(b"\nendobj\n\n");
    }
}

deref!('a, Stream<'a> => Dict<'a>, dict);

/// A compression filter for a stream.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Filter {
    AsciiHexDecode,
    Ascii85Decode,
    LzwDecode,
    FlateDecode,
    RunLengthDecode,
    CcittFaxDecode,
    Jbig2Decode,
    DctDecode,
    JpxDecode,
    Crypt,
}

impl Filter {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::AsciiHexDecode => Name(b"ASCIIHexDecode"),
            Self::Ascii85Decode => Name(b"ASCII85Decode"),
            Self::LzwDecode => Name(b"LZWDecode"),
            Self::FlateDecode => Name(b"FlateDecode"),
            Self::RunLengthDecode => Name(b"RunLengthDecode"),
            Self::CcittFaxDecode => Name(b"CCITTFaxDecode"),
            Self::Jbig2Decode => Name(b"JBIG2Decode"),
            Self::DctDecode => Name(b"DCTDecode"),
            Self::JpxDecode => Name(b"JPXDecode"),
            Self::Crypt => Name(b"Crypt"),
        }
    }
}

/// Finish objects in postfix-style.
///
/// In many cases you can use writers in builder-pattern style so that they are
/// automatically dropped at the appropriate time. Sometimes though you need to
/// bind a writer to a variable and still want to regain access to the
/// [`PdfWriter`] in the same scope. In that case, you need to manually invoke
/// the writer's `Drop` implementation. You can of course, just write
/// `drop(array)` to finish your array, but you might find it more aesthetically
/// pleasing to write `array.finish()`. That's what this trait is for.
///
/// ```
/// # use pdf_writer::{PdfWriter, Ref, Finish, Name, Str};
/// # let mut writer = PdfWriter::new();
/// let mut array = writer.indirect(Ref::new(1)).array();
/// array.push().dict().pair(Name(b"Key"), Str(b"Value"));
/// array.item(2);
/// array.finish(); // instead of drop(array)
///
/// // Do more stuff with the writer ...
/// ```
pub trait Finish: Sized {
    /// Does nothing but move `self`, equivalent to [`drop`].
    #[inline]
    fn finish(self) {}
}

impl<T> Finish for T {}
