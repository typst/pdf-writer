use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::num::NonZeroI32;

use super::*;
use crate::chunk::WriteSettings;

/// A primitive PDF object.
pub trait Primitive {
    /// Whether the primitive object starts with one of the PDF delimiter characters.
    const STARTS_WITH_DELIMITER: bool;

    /// Write the object into a buffer.
    fn write(self, buf: &mut Buf);
}

impl<T: Primitive> Primitive for &T
where
    T: Copy,
{
    const STARTS_WITH_DELIMITER: bool = T::STARTS_WITH_DELIMITER;

    #[inline]
    fn write(self, buf: &mut Buf) {
        (*self).write(buf);
    }
}

impl Primitive for bool {
    const STARTS_WITH_DELIMITER: bool = false;

    #[inline]
    fn write(self, buf: &mut Buf) {
        if self {
            buf.extend(b"true");
        } else {
            buf.extend(b"false");
        }
    }
}

impl Primitive for i32 {
    const STARTS_WITH_DELIMITER: bool = false;

    #[inline]
    fn write(self, buf: &mut Buf) {
        buf.push_int(self);
    }
}

impl Primitive for f32 {
    const STARTS_WITH_DELIMITER: bool = false;

    #[inline]
    fn write(self, buf: &mut Buf) {
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
    const STARTS_WITH_DELIMITER: bool = true;

    fn write(self, buf: &mut Buf) {
        buf.limits.register_str_len(self.0.len());

        // We use:
        // - Literal strings for ASCII with nice escape sequences to make it
        //   also be represented fully in visible ASCII. We also escape
        //   parentheses because they are delimiters.
        // - Hex strings for anything non-ASCII.
        if self.0.iter().all(|b| b.is_ascii()) {
            buf.reserve(self.0.len());
            buf.inner.push(b'(');

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
///
/// The natural language is inherited from the document catalog's
/// [`/Lang` key](crate::Catalog::lang). If you need to specify another language
/// or if the string contains multiple natural languages, see
/// [`TextStrWithLang`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextStr<'a>(pub &'a str);

impl Primitive for TextStr<'_> {
    const STARTS_WITH_DELIMITER: bool = true;

    fn write(self, buf: &mut Buf) {
        buf.limits.register_str_len(self.0.len());

        // ASCII and PDFDocEncoding match for 32 up to 126.
        if self.0.bytes().all(|b| matches!(b, 32..=126)) {
            Str(self.0.as_bytes()).write(buf);
        } else {
            buf.reserve(6 + 4 * self.0.len());
            write_utf16be_text_str_header(buf);
            for value in self.0.encode_utf16() {
                buf.push_hex_u16(value);
            }
            write_utf16be_text_str_footer(buf);
        }
    }
}

/// An identifier for the natural language in a section of a
/// [`TextStrWithLang`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LanguageIdentifier {
    /// A two-byte ISO 639 language code.
    lang: [u8; 2],
    /// A two-byte ISO 3166 country code.
    region: Option<[u8; 2]>,
}

impl LanguageIdentifier {
    /// Create a new language identifier.
    ///
    /// Language and region codes are not checked for validity.
    pub fn new(lang: [u8; 2], region: Option<[u8; 2]>) -> Self {
        Self { lang, region }
    }

    /// Create a new language identifier from a language, with an unset region.
    ///
    /// The method returns `Some` if the argument has two alphanumeric ASCII
    /// bytes.
    pub fn from_lang(lang: &str) -> Option<Self> {
        let lang = Self::str_to_code(lang)?;
        Some(Self::new(lang, None))
    }

    /// Create a new language identifier from a language and a region
    ///
    /// The method returns `Some` if both arguments have two alphanumeric ASCII
    /// bytes.
    pub fn from_lang_region(lang: &str, region: &str) -> Option<Self> {
        let lang = Self::str_to_code(lang)?;
        let region = Self::str_to_code(region)?;
        Some(Self::new(lang, Some(region)))
    }

    /// Returns the length of the language identifier. It does not include the
    /// enclosing escape bytes.
    fn len(self) -> usize {
        if self.region.is_some() {
            4
        } else {
            2
        }
    }

    fn str_to_code(string: &str) -> Option<[u8; 2]> {
        if string.chars().all(|c| c.is_ascii_alphanumeric()) {
            string.as_bytes().try_into().ok()
        } else {
            None
        }
    }
}

/// A text string with a natural language specified.
///
///
/// This is written as a string containing either bare ASCII (if possible) or a
/// byte order mark followed by UTF-16-BE bytes. Both forms are interspersed by
/// the requisite ASCII language escape sequences.
///
/// For a text string with an undefined natural language, see [`TextStr`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextStrWithLang<'a, 'b>(pub &'b [(LanguageIdentifier, &'a str)]);

impl<'a, 'b> Primitive for TextStrWithLang<'a, 'b> {
    const STARTS_WITH_DELIMITER: bool = true;

    fn write(self, buf: &mut Buf) {
        let mut len = 0;
        let mut buf_len = 6;

        for (lang, text) in self.0 {
            // Each language tag is enclosed in two escape characters, plus the
            // two-letter language code and optional two-letter region code.
            len += text.len() + lang.len() + 2;
            // Each hexadecimal character is four bytes long, plus four bytes
            // for the escape sequence. The language tag is encoded in
            // hexadecimal, so each byte becomes two hexadecimal characters.
            buf_len += 4 * text.len() + lang.len() * 2 + 4;
        }

        buf.limits.register_str_len(len);

        // Escape sequences for languages may only appear in Unicode-encoded
        // text strings, see clause 7.9.2.2 of ISO 32000-1:2008.
        buf.reserve(buf_len);
        write_utf16be_text_str_header(buf);

        for (lang, text) in self.0 {
            write_utf16be_lang_code(*lang, buf);
            for value in text.encode_utf16() {
                buf.push_hex_u16(value);
            }
        }

        write_utf16be_text_str_footer(buf);
    }
}

fn write_utf16be_text_str_header(buf: &mut Buf) {
    buf.push(b'<');
    buf.push_hex(254);
    buf.push_hex(255);
}

fn write_utf16be_text_str_footer(buf: &mut Buf) {
    buf.push(b'>');
}

fn write_utf16be_lang_code(lang: LanguageIdentifier, buf: &mut Buf) {
    // The escape character U+001B encloses the language tag. It must not
    // otherwise appear in a text string and the spec offers no opportunity to
    // escape it. In the future, `pdf-writer` may offer a constructor for
    // [`TextStrWithLang`] and [`TextStr`] that either checks for it or replaces
    // it with the object replacement character U+FFFD.
    buf.push_hex_u16(0x001B);
    buf.push_hex_u16(u16::from(lang.lang[0]));
    buf.push_hex_u16(u16::from(lang.lang[1]));
    if let Some(region) = lang.region {
        buf.push_hex_u16(u16::from(region[0]));
        buf.push_hex_u16(u16::from(region[1]));
    }
    buf.push_hex_u16(0x001B);
}

/// A trait for types that can be used everywhere a text string is expected.
/// This includes both [`TextStr`] and [`TextStrWithLang`].
///
/// Methods that accept an implementor of this trait expect strings in natural
/// language for which a language specification makes sense, often for use in
/// the UI or with AT.
pub trait TextStrLike: Primitive {}

impl<'a> TextStrLike for TextStr<'a> {}
impl<'a, 'b> TextStrLike for TextStrWithLang<'a, 'b> {}

/// A name object.
///
/// Written as `/Thing`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Name<'a>(pub &'a [u8]);

impl Primitive for Name<'_> {
    const STARTS_WITH_DELIMITER: bool = true;

    fn write(self, buf: &mut Buf) {
        buf.limits.register_name_len(self.0.len());

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

#[inline]
pub(crate) fn is_delimiter_character(byte: u8) -> bool {
    matches!(byte, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'/' | b'%')
}

/// The null object.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;

impl Primitive for Null {
    const STARTS_WITH_DELIMITER: bool = false;

    #[inline]
    fn write(self, buf: &mut Buf) {
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
    const STARTS_WITH_DELIMITER: bool = false;

    #[inline]
    fn write(self, buf: &mut Buf) {
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
    const STARTS_WITH_DELIMITER: bool = true;

    #[inline]
    fn write(self, buf: &mut Buf) {
        buf.push(b'[');
        buf.push_val(self.x1);
        buf.push(b' ');
        buf.push_val(self.y1);
        buf.push(b' ');
        buf.push_val(self.x2);
        buf.push(b' ');
        buf.push_val(self.y2);
        buf.push(b']');

        buf.limits.register_array_len(4);
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
    const STARTS_WITH_DELIMITER: bool = true;

    fn write(self, buf: &mut Buf) {
        buf.extend(b"(D:");

        (|| {
            write!(buf.inner, "{:04}", self.year).unwrap();
            write!(buf.inner, "{:02}", self.month?).unwrap();
            write!(buf.inner, "{:02}", self.day?).unwrap();
            write!(buf.inner, "{:02}", self.hour?).unwrap();
            write!(buf.inner, "{:02}", self.minute?).unwrap();
            write!(buf.inner, "{:02}", self.second?).unwrap();
            let utc_offset_hour = self.utc_offset_hour?;
            if utc_offset_hour == 0 && self.utc_offset_minute == 0 {
                buf.push(b'Z');
            } else {
                write!(
                    buf.inner,
                    "{:+03}'{:02}",
                    utc_offset_hour, self.utc_offset_minute
                )
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
    buf: &'a mut Buf,
    indirect: bool,
    indent: u8,
    write_settings: WriteSettings,
    needs_padding: bool,
}

impl<'a> Obj<'a> {
    /// Start a new direct object.
    #[inline]
    pub(crate) fn direct(
        buf: &'a mut Buf,
        indent: u8,
        write_settings: WriteSettings,
        needs_padding: bool,
    ) -> Self {
        Self {
            buf,
            indirect: false,
            indent,
            write_settings,
            needs_padding,
        }
    }

    /// Start a new indirect object.
    #[inline]
    pub(crate) fn indirect(
        buf: &'a mut Buf,
        id: Ref,
        write_settings: WriteSettings,
    ) -> Self {
        buf.push_int(id.get());
        buf.extend(b" 0 obj\n");
        Self {
            buf,
            indirect: true,
            indent: 0,
            write_settings,
            needs_padding: false,
        }
    }

    /// Write a primitive object.
    #[inline]
    pub fn primitive<T: Primitive>(self, value: T) {
        // Normally, we need to separate different PDF objects by a whitespace. he key to the
        // optimizations applied here are explained in 7.2.3 in the PDF reference:
        // > The delimiter characters (, ), <, >, [, ], /, and % are special. They
        // > delimit syntactic entities such as arrays, names, and comments. Any of these
        // > delimiters terminates the entity preceding it and is not included in the entity.
        // Therefore, if either the previous byte is a delimiter character or the current token
        // starts with one, we don't need to add a whitespace for padding.

        let ends_with_delimiter =
            self.buf.last().copied().is_some_and(is_delimiter_character);

        if self.needs_padding && !T::STARTS_WITH_DELIMITER && !ends_with_delimiter {
            self.buf.extend(b" ");
        }

        value.write(self.buf);

        if self.indirect {
            self.buf.extend(b"\nendobj\n");

            if self.write_settings.pretty {
                self.buf.extend(b"\n");
            }
        }
    }

    // Note: Arrays and dictionaries always start with a delimiter, so we don't need to do any case
    // distinction, unlike in `primitive`.

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
    buf: &'a mut Buf,
    indirect: bool,
    indent: u8,
    settings: WriteSettings,
    len: i32,
}

writer!(Array: |obj| {
    obj.buf.push(b'[');
    Self {
        buf: obj.buf,
        indirect: obj.indirect,
        indent: obj.indent,
        settings: obj.write_settings,
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
        let needs_padding = if self.len != 0 {
            if self.settings.pretty {
                self.buf.push(b' ');
                false
            } else {
                true
            }
        } else {
            false
        };

        self.len += 1;

        Obj::direct(self.buf, self.indent, self.settings, needs_padding)
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
        self.buf.limits.register_array_len(self.len() as usize);
        self.buf.push(b']');
        if self.indirect {
            self.buf.extend(b"\nendobj\n");

            if self.settings.pretty {
                self.buf.extend(b"\n");
            }
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

impl<'a, T> Rewrite<'a> for TypedArray<'_, T> {
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
    pub fn push<'b>(&'b mut self) -> <T as Rewrite<'b>>::Output
    where
        T: Writer<'a> + Rewrite<'b>,
    {
        <T as Rewrite>::Output::start(self.array.push())
    }
}

/// Writer for a dictionary.
pub struct Dict<'a> {
    buf: &'a mut Buf,
    indirect: bool,
    indent: u8,
    settings: WriteSettings,
    len: i32,
}

writer!(Dict: |obj| {
    obj.buf.extend(b"<<");
    Self {
        buf: obj.buf,
        indirect: obj.indirect,
        indent: obj.indent.saturating_add(2),
        settings: obj.write_settings,
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

        // Keys always start with a delimiter since they are names, so we never need
        // padding unless `pretty` is activated.
        if self.settings.pretty {
            self.buf.push(b'\n');

            for _ in 0..self.indent {
                self.buf.push(b' ');
            }
        }

        self.buf.push_val(key);

        let needs_padding = if self.settings.pretty {
            self.buf.push(b' ');
            false
        } else {
            true
        };

        Obj::direct(self.buf, self.indent, self.settings, needs_padding)
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
        self.buf.limits.register_dict_entries(self.len as usize);

        if self.len != 0 && self.settings.pretty {
            self.buf.push(b'\n');
            for _ in 0..self.indent - 2 {
                self.buf.push(b' ');
            }
        }

        self.buf.extend(b">>");

        if self.indirect {
            self.buf.extend(b"\nendobj\n");

            if self.settings.pretty {
                self.buf.extend(b"\n");
            }
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

impl<'a, T> Rewrite<'a> for TypedDict<'_, T> {
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
    pub fn insert<'b>(&'b mut self, key: Name) -> <T as Rewrite<'b>>::Output
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
        let dict_len = self.dict.len as usize;
        self.dict.buf.limits.register_dict_entries(dict_len);

        if self.dict.settings.pretty {
            self.dict.buf.extend(b"\n");
        }

        self.dict.buf.extend(b">>");
        self.dict.buf.extend(b"\nstream\n");
        self.dict.buf.extend(self.data.as_ref());
        self.dict.buf.extend(b"\nendstream");
        self.dict.buf.extend(b"\nendobj\n");

        if self.dict.settings.pretty {
            self.dict.buf.extend(b"\n");
        }
    }
}

deref!('a, Stream<'a> => Dict<'a>, dict);

/// A compression filter for a stream.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Filter {
    AsciiHexDecode,
    Ascii85Decode,
    /// Lempel-Ziv-Welch (LZW) compression.
    ///
    /// Note that this filter is forbidden in PDF/A.
    LzwDecode,
    FlateDecode,
    RunLengthDecode,
    CcittFaxDecode,
    Jbig2Decode,
    /// Decodes JPEG/JFIF files with a SOF0, SOF1, or (PDF 1.3+) SOF2 marker.
    ///
    /// See ISO 32000-1:2008, Section 7.4.8 and Adobe Technical Note #5116.
    DctDecode,
    /// Decodes JPEG2000 files with a JPX baseline marker.
    ///
    /// Note that additional restrictions are imposed by PDF/A and PDF/X.
    JpxDecode,
    /// Encrypt the stream.
    ///
    /// Note that this filter is restricted in PDF/A.
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
            panic!("`Colors` must be greater than 0");
        }

        self.pair(Name(b"Colors"), colors);
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
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum Predictor {
    /// No prediction.
    #[default]
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

impl<'a, T> Rewrite<'a> for NameTree<'_, T> {
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

impl<'a, T> Rewrite<'a> for NameTreeEntries<'_, T> {
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

impl<'a, T> Rewrite<'a> for NumberTree<'_, T> {
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

impl<'a, T> Rewrite<'a> for NumberTreeEntries<'_, T> {
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

    #[test]
    fn test_arrays_no_pretty() {
        test_obj_no_pretty!(|obj| obj.array(), b"[]");
        test_obj_no_pretty!(
            |obj| obj
                .array()
                .item(12)
                .item(Name(b"Hi"))
                .item(Name(b"Hi2"))
                .item(false)
                .item(TextStr("A string"))
                .item(Null)
                .item(23.40),
            b"[12/Hi/Hi2 false(A String)null 23.4]"
        );
    }

    #[test]
    fn test_dicts_no_pretty() {
        test_obj_no_pretty!(|obj| obj.dict(), b"<<>>");
        test_obj_no_pretty!(
            |obj| obj
                .dict()
                .pair(Name(b"Key1"), 12)
                .pair(Name(b"Key2"), Name(b"Hi"))
                .pair(Name(b"Key3"), false)
                .pair(Name(b"Key4"), TextStr("A string"))
                .pair(Name(b"Key5"), Null)
                .pair(Name(b"Key6"), 23.40),
            b"<</Key1 12/Key2/Hi/Key3 false/Key4(A string)/Key5 null/Key6 23.4>>"
        );
    }
}
