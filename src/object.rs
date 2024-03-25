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
/// This is written as `(Thing)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Str<'a>(pub &'a [u8]);

impl Str<'_> {
    /// Whether the parentheses in the byte string are balanced.
    fn is_balanced(self) -> bool {
        let mut depth = 0;
        for &byte in self.0 {
            match byte {
                b'(' => depth += 1,
                b')' if depth > 0 => depth -= 1,
                b')' => return false,
                _ => {}
            }
        }
        depth == 0
    }
}

impl Primitive for Str<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        // We use:
        // - Literal strings for ASCII with nice escape sequences to make it
        //   also be represented fully in visible ASCII. We also escape
        //   parentheses because they are delimiters.
        // - Hex strings for anything non-ASCII.
        if self.0.iter().all(|b| b.is_ascii()) {
            buf.reserve(self.0.len());
            buf.push(b'(');

            let mut balanced = None;
            for &byte in self.0 {
                match byte {
                    b'(' | b')' => {
                        if !*balanced
                            .get_or_insert_with(|| byte != b')' && self.is_balanced())
                        {
                            buf.push(b'\\');
                        }
                        buf.push(byte);
                    }
                    b'\\' => buf.extend(br"\\"),
                    b' '..=b'~' => buf.push(byte),
                    b'\n' => buf.extend(br"\n"),
                    b'\r' => buf.extend(br"\r"),
                    b'\t' => buf.extend(br"\t"),
                    b'\x08' => buf.extend(br"\b"),
                    b'\x0c' => buf.extend(br"\f"),
                    _ => {
                        buf.push(b'\\');
                        buf.push_octal(byte);
                    }
                }
            }

            buf.push(b')');
        } else {
            buf.reserve(2 + 2 * self.0.len());
            buf.push(b'<');

            for &byte in self.0 {
                buf.push_hex(byte);
            }

            buf.push(b'>');
        }
    }
}

/// A unicode text string object.
///
/// This is written as a [`Str`] containing either bare ASCII (if possible) or a
/// byte order mark followed by UTF-16-BE bytes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextStr<'a>(pub &'a str);

impl Primitive for TextStr<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        // ASCII and PDFDocEncoding match for 32 up to 126.
        if self.0.bytes().all(|b| matches!(b, 32..=126)) {
            Str(self.0.as_bytes()).write(buf);
        } else {
            buf.reserve(6 + 4 * self.0.len());
            buf.push(b'<');
            buf.push_hex(254);
            buf.push_hex(255);
            for value in self.0.encode_utf16() {
                buf.push_hex_u16(value);
            }
            buf.push(b'>');
        }
    }
}

/// A name object.
///
/// Written as `/Thing`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a [u8]);

impl Primitive for Name<'_> {
    fn write(self, buf: &mut Vec<u8>) {
        buf.reserve(1 + self.0.len());
        buf.push(b'/');
        for &byte in self.0 {
            // - Number sign shall use hexadecimal escape
            // - Regular characters within the range exlacamation mark .. tilde
            //   can be written directly
            if byte != b'#' && matches!(byte, b'!'..=b'~') && is_regular_character(byte) {
                buf.push(byte);
            } else {
                buf.push(b'#');
                buf.push_hex(byte);
            }
        }
    }
}

/// Regular characters are a PDF concept.
fn is_regular_character(byte: u8) -> bool {
    !matches!(
        byte,
        b'\0'
            | b'\t'
            | b'\n'
            | b'\x0C'
            | b'\r'
            | b' '
            | b'('
            | b')'
            | b'<'
            | b'>'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'/'
            | b'%'
    )
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
    #[track_caller]
    pub const fn new(id: i32) -> Ref {
        let option = if id > 0 { NonZeroI32::new(id) } else { None };
        match option {
            Some(val) => Self(val),
            None => panic!("indirect reference out of valid range"),
        }
    }

    /// Return the underlying number as a primitive type.
    #[inline]
    pub const fn get(self) -> i32 {
        self.0.get()
    }

    /// The next consecutive ID.
    #[inline]
    pub const fn next(self) -> Self {
        Self::new(self.get() + 1)
    }

    /// Increase this ID by one and return the old one. Useful to turn this ID
    /// into a bump allocator of sorts.
    #[inline]
    pub fn bump(&mut self) -> Self {
        let prev = *self;
        *self = self.next();
        prev
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
        [self.x1, self.y1, self.x2, self.y1, self.x2, self.y2, self.x1, self.y2]
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

/// A date, written as a text string.
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
        buf.extend(b"(D:");

        (|| {
            write!(buf, "{:04}", self.year).unwrap();
            write!(buf, "{:02}", self.month?).unwrap();
            write!(buf, "{:02}", self.day?).unwrap();
            write!(buf, "{:02}", self.hour?).unwrap();
            write!(buf, "{:02}", self.minute?).unwrap();
            write!(buf, "{:02}", self.second?).unwrap();
            let utc_offset_hour = self.utc_offset_hour?;
            if utc_offset_hour == 0 && self.utc_offset_minute == 0 {
                buf.push(b'Z');
            } else {
                write!(buf, "{:+03}'{:02}", utc_offset_hour, self.utc_offset_minute)
                    .unwrap();
            }
            Some(())
        })();

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

    /// Start writing an array.
    #[inline]
    pub fn array(self) -> Array<'a> {
        self.start()
    }

    /// Start writing a dictionary.
    #[inline]
    pub fn dict(self) -> Dict<'a> {
        self.start()
    }

    /// Start writing with an arbitrary writer.
    ///
    /// For example, using this, you could write a Type 1 font directly into
    /// a page's resource directionary.
    /// ```
    /// use pdf_writer::{Pdf, Ref, Name, writers::Type1Font};
    ///
    /// let mut pdf = Pdf::new();
    /// pdf.page(Ref::new(1))
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

/// Rewrites a writer's lifetime.
///
/// This is a workaround to ignore the `'b` lifetime in a
/// `TypedArray<'a, SomeWriter<'b>>` because that lifetime is meaningless. What
/// we actually want is each item's `SomeWriter` to borrow from the array itself.
pub trait Rewrite<'a> {
    /// The writer with the rewritten lifetime.
    type Output: Writer<'a>;
}

/// Writer for an array.
pub struct Array<'a> {
    buf: &'a mut Vec<u8>,
    indirect: bool,
    indent: u8,
    len: i32,
}

writer!(Array: |obj| {
    obj.buf.push(b'[');
    Self {
        buf: obj.buf,
        indirect: obj.indirect,
        indent: obj.indent,
        len: 0,
    }
});

impl<'a> Array<'a> {
    /// The number of written items.
    #[inline]
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Whether no items have been written so far.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
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
    pub fn typed<T>(self) -> TypedArray<'a, T> {
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

/// Writer for an array of items of a fixed type.
pub struct TypedArray<'a, T> {
    array: Array<'a>,
    phantom: PhantomData<fn() -> T>,
}

impl<'a, T> Writer<'a> for TypedArray<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { array: obj.array(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for TypedArray<'any, T> {
    type Output = TypedArray<'a, T>;
}

impl<'a, T> TypedArray<'a, T> {
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

    /// Whether no items have been written so far.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write an item.
    #[inline]
    pub fn item(&mut self, value: T) -> &mut Self
    where
        T: Primitive,
    {
        self.array.item(value);
        self
    }

    /// Write a sequence of items.
    #[inline]
    pub fn items(&mut self, values: impl IntoIterator<Item = T>) -> &mut Self
    where
        T: Primitive,
    {
        self.array.items(values);
        self
    }

    /// Start writing an item with the typed writer.
    ///
    /// Returns `T` but with its lifetime rewritten from `'a` to `'b`.
    #[inline]
    pub fn push<'b>(&'b mut self) -> <T as Rewrite>::Output
    where
        T: Writer<'a> + Rewrite<'b>,
    {
        <T as Rewrite>::Output::start(self.array.push())
    }
}

/// Writer for a dictionary.
pub struct Dict<'a> {
    buf: &'a mut Vec<u8>,
    indirect: bool,
    indent: u8,
    len: i32,
}

writer!(Dict: |obj| {
    obj.buf.extend(b"<<");
    Self {
        buf: obj.buf,
        indirect: obj.indirect,
        indent: obj.indent.saturating_add(2),
        len: 0,
    }
});

impl<'a> Dict<'a> {
    /// The number of written pairs.
    #[inline]
    pub fn len(&self) -> i32 {
        self.len
    }

    /// Whether no pairs have been written so far.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Start writing a pair with an arbitrary value.
    #[inline]
    pub fn insert(&mut self, key: Name) -> Obj<'_> {
        self.len += 1;
        self.buf.push(b'\n');

        for _ in 0..self.indent {
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
    pub fn typed<T>(self) -> TypedDict<'a, T> {
        TypedDict::wrap(self)
    }
}

impl Drop for Dict<'_> {
    #[inline]
    fn drop(&mut self) {
        if self.len != 0 {
            self.buf.push(b'\n');
            for _ in 0..self.indent - 2 {
                self.buf.push(b' ');
            }
        }
        self.buf.extend(b">>");
        if self.indirect {
            self.buf.extend(b"\nendobj\n\n");
        }
    }
}

/// Writer for a dictionary mapping to a fixed type.
pub struct TypedDict<'a, T> {
    dict: Dict<'a>,
    phantom: PhantomData<fn() -> T>,
}

impl<'a, T> Writer<'a> for TypedDict<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for TypedDict<'any, T> {
    type Output = TypedDict<'a, T>;
}

impl<'a, T> TypedDict<'a, T> {
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

    /// Whether no pairs have been written so far.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write a key-value pair.
    #[inline]
    pub fn pair(&mut self, key: Name, value: T) -> &mut Self
    where
        T: Primitive,
    {
        self.dict.pair(key, value);
        self
    }

    /// Write a sequence of key-value pairs.
    #[inline]
    pub fn pairs<'n>(
        &mut self,
        pairs: impl IntoIterator<Item = (Name<'n>, T)>,
    ) -> &mut Self
    where
        T: Primitive,
    {
        self.dict.pairs(pairs);
        self
    }

    /// Start writing a pair with the typed writer.
    ///
    /// Returns `T` but with its lifetime rewritten from `'a` to `'b`.
    #[inline]
    pub fn insert<'b>(&'b mut self, key: Name) -> <T as Rewrite>::Output
    where
        T: Writer<'a> + Rewrite<'b>,
    {
        <T as Rewrite>::Output::start(self.dict.insert(key))
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

    /// Start writing the `/DecodeParms` attribute.
    ///
    /// This is a dictionary that specifies parameters to be used in decoding
    /// the stream data using the filter specified by the
    /// [`/Filter`](Self::filter) attribute.
    pub fn decode_parms(&mut self) -> DecodeParms<'_> {
        self.insert(Name(b"DecodeParms")).start()
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

/// Writer for an _filter decode parameters dictionary_.
///
/// This struct is created by [`Stream::decode_parms`].
pub struct DecodeParms<'a> {
    dict: Dict<'a>,
}

writer!(DecodeParms: |obj| Self { dict: obj.dict() });

/// Properties for `FlateDecode` and `LzwDecode`.
impl DecodeParms<'_> {
    /// Write the `/Predictor` attribute for `FlateDecode` and `LzwDecode`.
    ///
    /// No predictor is used by default.
    pub fn predictor(&mut self, predictor: Predictor) -> &mut Self {
        self.pair(Name(b"Predictor"), predictor.to_i32());
        self
    }

    /// Write the `/Colors` attribute for `FlateDecode` and `LzwDecode`.
    ///
    /// Must be greater than 0. [`/Predictor`](Self::predictor) must be set.
    /// Defaults to 1.
    pub fn colors(&mut self, colors: i32) -> &mut Self {
        if colors <= 0 {
            panic!("`Columns` must be greater than 0");
        }

        self.pair(Name(b"Columns"), colors);
        self
    }

    /// Write the `/BitsPerComponent` attribute for `FlateDecode` and
    /// `LzwDecode`.
    ///
    /// Must be one of 1, 2, 4, 8, or 16. [`/Predictor`](Self::predictor) must
    /// be set. Defaults to 8.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        if ![1, 2, 4, 8, 16].contains(&bits) {
            panic!("`BitsPerComponent` must be one of 1, 2, 4, 8, or 16");
        }

        self.pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/Columns` attribute for `FlateDecode` and `LzwDecode` or
    /// `CcittFaxDecode`.
    ///
    /// When used with `FlateDecode` and `LzwDecode`, it indicates the number of
    /// samples in each row. In that case, [`/Predictor`](Self::predictor) must
    /// be set and the default is 1.
    ///
    /// When used with `CcittFaxDecode` it denominates the width of the image in
    /// pixels and defaults to 1728.
    pub fn columns(&mut self, columns: i32) -> &mut Self {
        self.pair(Name(b"Columns"), columns);
        self
    }

    /// Write the `/EarlyChange` attribute for `LzwDecode`.
    ///
    /// If `true` (1), the code length increases one code earlier, if `false`
    /// (0), length change is postponed as long as possible.
    ///
    /// Defaults to 1.
    pub fn early_change(&mut self, early_change: bool) -> &mut Self {
        self.pair(Name(b"EarlyChange"), if early_change { 1 } else { 0 });
        self
    }
}

/// Properties for `CcittFaxDecode`. Also see [`Self::columns`].
impl DecodeParms<'_> {
    /// Write the `/K` attribute for `CcittFaxDecode`.
    ///
    /// Defaults to 0.
    pub fn k(&mut self, k: i32) -> &mut Self {
        self.pair(Name(b"K"), k);
        self
    }

    /// Write the `/EndOfLine` attribute for `CcittFaxDecode`.
    ///
    /// Whether the EOL bit pattern is present in the encoding. Defaults to
    /// `false`.
    pub fn end_of_line(&mut self, eol: bool) -> &mut Self {
        self.pair(Name(b"EndOfLine"), eol);
        self
    }

    /// Write the `/EncodedByteAlign` attribute for `CcittFaxDecode`.
    ///
    /// Whether to expect zero bits before each encoded line. Defaults to
    /// `false`.
    pub fn encoded_byte_align(&mut self, encoded_byte_align: bool) -> &mut Self {
        self.pair(Name(b"EncodedByteAlign"), encoded_byte_align);
        self
    }

    /// Write the `/Rows` attribute for `CcittFaxDecode`.
    ///
    /// The image's height. Defaults to 0.
    pub fn rows(&mut self, rows: i32) -> &mut Self {
        self.pair(Name(b"Rows"), rows);
        self
    }

    /// Write the `/EndOfBlock` attribute for `CcittFaxDecode`.
    ///
    /// Whether to expect an EOB code at the end of the data. Defaults to
    /// `true`.
    pub fn end_of_block(&mut self, end_of_block: bool) -> &mut Self {
        self.pair(Name(b"EndOfBlock"), end_of_block);
        self
    }

    /// Write the `/BlackIs1` attribute for `CcittFaxDecode`.
    ///
    /// Whether to invert the bits in the image. Defaults to `false`.
    pub fn black_is_1(&mut self, black_is_1: bool) -> &mut Self {
        self.pair(Name(b"BlackIs1"), black_is_1);
        self
    }

    /// Write the `/DamagedRowsBeforeError` attribute for `CcittFaxDecode`.
    ///
    /// How many damaged rows are allowed before an error is raised. Defaults to
    /// 0.
    pub fn damaged_rows_before_error(&mut self, count: i32) -> &mut Self {
        self.pair(Name(b"DamagedRowsBeforeError"), count);
        self
    }
}

/// Properties for `Jbig2Decode`.
impl DecodeParms<'_> {
    /// Write the `/JBIG2Globals` attribute for `Jbig2Decode`.
    ///
    /// A reference to a stream containing global segments.
    pub fn jbig2_globals(&mut self, globals: Ref) -> &mut Self {
        self.pair(Name(b"JBIG2Globals"), globals);
        self
    }
}

/// Properties for `JpxDecode`.
impl DecodeParms<'_> {
    /// Write the `/ColorTransform` attribute for `JpxDecode`.
    ///
    /// How to handle color data. If `true` (1), images with three color
    /// channels shall be decoded from the YCbCr space and images with four
    /// color channels are decoded from YCbCrK. If `false` (0), no
    /// transformation is applied. The default depends on the `APP14` marker in
    /// the data stream.
    pub fn color_transform(&mut self, color_transform: bool) -> &mut Self {
        self.pair(Name(b"ColorTransform"), if color_transform { 1 } else { 0 });
        self
    }
}

/// Properties for `Crypt`.
impl DecodeParms<'_> {
    /// Write the `/Type` attribute for `Crypt` as `CryptFilterDecodeParms`.
    pub fn crypt_type(&mut self) -> &mut Self {
        self.pair(Name(b"Type"), Name(b"CryptFilterDecodeParms"));
        self
    }

    /// Write the `/Name` attribute for `Crypt`.
    ///
    /// The name of the crypt filter corresponding to a `CF` entry of the
    /// encryption dictionary.
    pub fn name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"Name"), name);
        self
    }
}

deref!('a, DecodeParms<'a> => Dict<'a>, dict);

/// Which kind of predictor to use for a `FlateDecode` or `LzwDecode` stream.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Predictor {
    /// No prediction.
    None,
    /// TIFF Predictor 2.
    Tiff,
    PngNone,
    PngSub,
    PngUp,
    PngAverage,
    PngPaeth,
    PngOptimum,
}

impl Predictor {
    /// Convert the predictor to its integer representation according to ISO
    /// 32000-2:2020, Table E.
    fn to_i32(self) -> i32 {
        match self {
            Self::None => 1,
            Self::Tiff => 2,
            Self::PngNone => 10,
            Self::PngSub => 11,
            Self::PngUp => 12,
            Self::PngAverage => 13,
            Self::PngPaeth => 14,
            Self::PngOptimum => 15,
        }
    }
}

/// Writer for a _name tree node_.
///
/// Name trees associate a large number of names with PDF objects. They are
/// lexically ordered search trees. Root nodes may directly contain all leafs,
/// however, this might degrade performance for very large numbers of
/// name-object pairs.
///
/// For each node, either the `/Kids` or `/Names` attribute must be set, but
/// never both.
pub struct NameTree<'a, T> {
    dict: Dict<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Writer<'a> for NameTree<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for NameTree<'any, T> {
    type Output = NameTree<'a, T>;
}

impl<T> NameTree<'_, T> {
    /// Start writing the `/Kids` attribute with the children of this node.
    pub fn kids(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"Kids")).array().typed()
    }

    /// Start writing the `/Names` attribute to set the immediate name-to-object
    /// mappings of this node.
    pub fn names(&mut self) -> NameTreeEntries<'_, T> {
        self.dict.insert(Name(b"Names")).start()
    }

    /// Write the `/Limits` array to set the range of names in this node. This
    /// is required for every node except the root node.
    pub fn limits(&mut self, min: Name, max: Name) -> &mut Self {
        self.dict.insert(Name(b"Limits")).array().typed().items([min, max]);
        self
    }
}

/// Writer for a _name tree names_ array.
///
/// The children must be added in ascending lexical order. Their minimum and
/// maximum keys must not exceed the `/Limits` property of the parent [`NameTree`]
/// node. This struct is created by [`NameTree::names`].
pub struct NameTreeEntries<'a, T> {
    arr: Array<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Writer<'a> for NameTreeEntries<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { arr: obj.array(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for NameTreeEntries<'any, T> {
    type Output = NameTreeEntries<'a, T>;
}

impl<T> NameTreeEntries<'_, T>
where
    T: Primitive,
{
    /// Insert a name-value pair.
    pub fn insert(&mut self, key: Str, value: T) -> &mut Self {
        self.arr.item(key);
        self.arr.item(value);
        self
    }
}

/// Writer for a _number tree node_.
///
/// Number trees associate a many integers with PDF objects. They are search
/// trees in ascending order. Root nodes may directly contain all leafs,
/// however, this might degrade performance for very large numbers of
/// integer-object pairs.
///
/// For each node, either the `/Kids` or `/Nums` attribute must be set, but
/// never both.
pub struct NumberTree<'a, T> {
    dict: Dict<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Writer<'a> for NumberTree<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for NumberTree<'any, T> {
    type Output = NumberTree<'a, T>;
}

impl<T> NumberTree<'_, T> {
    /// Start writing the `/Kids` attribute with the children of this node.
    pub fn kids(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"Kids")).array().typed()
    }

    /// Start writing the `/Nums` attribute to set the immediate
    /// number-to-object mappings of this node.
    pub fn nums(&mut self) -> NumberTreeEntries<'_, T> {
        self.dict.insert(Name(b"Nums")).start()
    }

    /// Write the `/Limits` array to set the range of numbers in this node. This
    /// is required for every node except the root node.
    pub fn limits(&mut self, min: i32, max: i32) -> &mut Self {
        self.dict.insert(Name(b"Limits")).array().typed().items([min, max]);
        self
    }
}

/// Writer for a _number tree numbers_ array.
///
/// The children must be added in ascending order. Their minimum and
/// maximum keys must not exceed the `/Limits` property of the parent [`NumberTree`]
/// node. This struct is created by [`NumberTree::nums`].
pub struct NumberTreeEntries<'a, T> {
    arr: Array<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Writer<'a> for NumberTreeEntries<'a, T> {
    fn start(obj: Obj<'a>) -> Self {
        Self { arr: obj.array(), phantom: PhantomData }
    }
}

impl<'a, 'any, T> Rewrite<'a> for NumberTreeEntries<'any, T> {
    type Output = NumberTreeEntries<'a, T>;
}

impl<T> NumberTreeEntries<'_, T>
where
    T: Primitive,
{
    /// Insert a number-value pair.
    pub fn insert(&mut self, key: i32, value: T) -> &mut Self {
        self.arr.item(key);
        self.arr.item(value);
        self
    }
}

/// Finish objects in postfix-style.
///
/// In many cases you can use writers in builder-pattern style so that they are
/// automatically dropped at the appropriate time. Sometimes though you need to
/// bind a writer to a variable and still want to regain access to the
/// [`Pdf`] in the same scope. In that case, you need to manually invoke
/// the writer's `Drop` implementation. You can of course, just write
/// `drop(array)` to finish your array, but you might find it more aesthetically
/// pleasing to write `array.finish()`. That's what this trait is for.
///
/// ```
/// # use pdf_writer::{Pdf, Ref, Finish, Name, Str};
/// # let mut pdf = Pdf::new();
/// let mut array = pdf.indirect(Ref::new(1)).array();
/// array.push().dict().pair(Name(b"Key"), Str(b"Value"));
/// array.item(2);
/// array.finish(); // instead of drop(array)
///
/// // Do more stuff with `pdf` ...
/// ```
pub trait Finish: Sized {
    /// Does nothing but move `self`, equivalent to [`drop`].
    #[inline]
    fn finish(self) {}
}

impl<T> Finish for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_objects() {
        // Test really simple objects.
        test_primitive!(true, b"true");
        test_primitive!(false, b"false");
        test_primitive!(78, b"78");
        test_primitive!(4.22, b"4.22");
        test_primitive!(1.184e-7, b"0.0000001184");
        test_primitive!(4.2e13, b"42000000000000");
        test_primitive!(Ref::new(7), b"7 0 R");
        test_primitive!(Null, b"null");

        // Test strings.
        test_primitive!(Str(b"Hello, World!"), b"(Hello, World!)");
        test_primitive!(Str(b"()"), br"(())");
        test_primitive!(Str(b")()"), br"(\)\(\))");
        test_primitive!(Str(b"()(())"), br"(()(()))");
        test_primitive!(Str(b"(()))"), br"(\(\(\)\)\))");
        test_primitive!(Str(b"\\"), br"(\\)");
        test_primitive!(Str(b"\n\ta"), br"(\n\ta)");
        test_primitive!(Str(br"\n"), br"(\\n)");
        test_primitive!(Str(b"a\x14b"), br"(a\024b)");
        test_primitive!(Str(b"\xFF\xAA"), b"<FFAA>");
        test_primitive!(Str(b"\x0A\x7F\x1F"), br"(\n\177\037)");

        // Test text strings.
        test_primitive!(TextStr("Hallo"), b"(Hallo)");
        test_primitive!(TextStr("😀!"), b"<FEFFD83DDE000021>");

        // Test names.
        test_primitive!(Name(b"Filter"), b"/Filter");
        test_primitive!(Name(b"A B"), br"/A#20B");
        test_primitive!(Name(b"~+c"), br"/~+c");
        test_primitive!(Name(b"/A-B"), br"/#2FA-B");
        test_primitive!(Name(b"<A>"), br"/#3CA#3E");
        test_primitive!(Name(b"#"), br"/#23");
        test_primitive!(Name(b"\n"), br"/#0A");
    }

    #[test]
    fn test_dates() {
        test_primitive!(Date::new(2021), b"(D:2021)");
        test_primitive!(Date::new(2021).month(30), b"(D:202112)");

        let date = Date::new(2020).month(3).day(17).hour(1).minute(2).second(3);
        test_primitive!(date, b"(D:20200317010203)");
        test_primitive!(date.utc_offset_hour(0), b"(D:20200317010203Z)");
        test_primitive!(date.utc_offset_hour(4), b"(D:20200317010203+04'00)");
        test_primitive!(
            date.utc_offset_hour(-17).utc_offset_minute(10),
            b"(D:20200317010203-17'10)"
        );
    }

    #[test]
    fn test_arrays() {
        test_obj!(|obj| obj.array(), b"[]");
        test_obj!(|obj| obj.array().item(12).item(Null), b"[12 null]");
        test_obj!(|obj| obj.array().typed().items(vec![1, 2, 3]), b"[1 2 3]");
        test_obj!(
            |obj| {
                let mut array = obj.array();
                array.push().array().typed().items(vec![1, 2]);
                array.item(3);
            },
            b"[[1 2] 3]",
        );
    }

    #[test]
    fn test_dicts() {
        test_obj!(|obj| obj.dict(), b"<<>>");
        test_obj!(
            |obj| obj.dict().pair(Name(b"Quality"), Name(b"Good")),
            b"<<\n  /Quality /Good\n>>",
        );
        test_obj!(
            |obj| {
                obj.dict().pair(Name(b"A"), 1).pair(Name(b"B"), 2);
            },
            b"<<\n  /A 1\n  /B 2\n>>",
        );
    }

    #[test]
    fn test_streams() {
        let mut w = Pdf::new();
        w.stream(Ref::new(1), &b"Hi there!"[..]).filter(Filter::Crypt);
        test!(
            w.finish(),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj",
            b"<<\n  /Length 9\n  /Filter /Crypt\n>>",
            b"stream",
            b"Hi there!",
            b"endstream",
            b"endobj\n",
            b"xref",
            b"0 2",
            b"0000000000 65535 f\r",
            b"0000000016 00000 n\r",
            b"trailer",
            b"<<\n  /Size 2\n>>",
            b"startxref\n94\n%%EOF",
        )
    }
}
