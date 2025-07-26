use std::ops::Deref;

use super::Primitive;

/// A buffer of arbitrary PDF content.
#[derive(Debug, Clone, PartialEq)]
pub struct Buf {
    pub(crate) inner: Vec<u8>,
    pub(crate) limits: Limits,
}

impl Buf {
    pub(crate) fn new() -> Self {
        Self { inner: Vec::new(), limits: Limits::new() }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            limits: Limits::new(),
        }
    }

    /// Get the underlying bytes of the buffer.
    pub fn into_vec(self) -> Vec<u8> {
        self.inner
    }

    /// Get the underlying bytes of the buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    /// Return the limits of the buffer.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    #[inline]
    pub(crate) fn push_val<T: Primitive>(&mut self, value: T) {
        value.write(self);
    }

    #[inline]
    pub(crate) fn push_int(&mut self, value: i32) {
        self.limits.register_int(value);
        self.extend(itoa::Buffer::new().format(value).as_bytes());
    }

    #[inline]
    pub(crate) fn push_float(&mut self, value: f32) {
        // Don't write the decimal point if we don't need it.
        // Also, integer formatting is way faster.
        if value as i32 as f32 == value {
            self.push_int(value as i32);
        } else {
            self.push_decimal(value);
        }
    }

    /// Like `push_float`, but forces the decimal point.
    #[inline]
    pub(crate) fn push_decimal(&mut self, value: f32) {
        self.limits.register_real(value);

        if value == 0.0 || (value.abs() > 1e-6 && value.abs() < 1e12) {
            self.extend(ryu::Buffer::new().format(value).as_bytes());
        } else {
            #[inline(never)]
            fn write_extreme(buf: &mut Buf, value: f32) {
                use std::io::Write;
                write!(buf.inner, "{value}").unwrap();
            }

            write_extreme(self, value);
        }
    }

    #[inline]
    pub(crate) fn extend_buf(&mut self, other: &Buf) {
        self.limits.merge(&other.limits);
        self.inner.extend(&other.inner);
    }

    #[inline]
    pub(crate) fn push(&mut self, b: u8) {
        self.inner.push(b);
    }

    #[inline]
    pub(crate) fn push_hex(&mut self, value: u8) {
        fn hex(b: u8) -> u8 {
            if b < 10 {
                b'0' + b
            } else {
                b'A' + (b - 10)
            }
        }

        self.push(hex(value >> 4));
        self.push(hex(value & 0xF));
    }

    #[inline]
    pub(crate) fn push_hex_u16(&mut self, value: u16) {
        self.push_hex((value >> 8) as u8);
        self.push_hex(value as u8);
    }

    #[inline]
    pub(crate) fn push_octal(&mut self, value: u8) {
        fn octal(b: u8) -> u8 {
            b'0' + b
        }

        self.push(octal(value >> 6));
        self.push(octal((value >> 3) & 7));
        self.push(octal(value & 7));
    }

    #[inline]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }
}

impl Deref for Buf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Extend<u8> for Buf {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}

impl<'a> Extend<&'a u8> for Buf {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}

/// Tracks the limits of data types used in a buffer.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Limits {
    int: i32,
    real: f32,
    name_len: usize,
    str_len: usize,
    array_len: usize,
    dict_entries: usize,
}

impl Limits {
    /// Create a new `Limits` struct with all values initialized to zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the absolute value of the largest positive/negative integer number.
    pub fn int(&self) -> i32 {
        self.int
    }

    /// Get the absolute value of the largest positive/negative real number.
    pub fn real(&self) -> f32 {
        self.real
    }

    /// Get the maximum length of any used name.
    pub fn name_len(&self) -> usize {
        self.name_len
    }

    /// Get the maximum length of any used string.
    pub fn str_len(&self) -> usize {
        self.str_len
    }

    /// Get the maximum length of any used array.
    pub fn array_len(&self) -> usize {
        self.array_len
    }

    /// Get the maximum number of entries in any dictionary.
    pub fn dict_entries(&self) -> usize {
        self.dict_entries
    }

    pub(crate) fn register_int(&mut self, val: i32) {
        self.int = self.int.max(val.abs());
    }

    pub(crate) fn register_real(&mut self, val: f32) {
        self.real = self.real.max(val.abs());
    }

    pub(crate) fn register_name_len(&mut self, len: usize) {
        self.name_len = self.name_len.max(len);
    }

    pub(crate) fn register_str_len(&mut self, len: usize) {
        self.str_len = self.str_len.max(len);
    }

    pub(crate) fn register_array_len(&mut self, len: usize) {
        self.array_len = self.array_len.max(len);
    }

    pub(crate) fn register_dict_entries(&mut self, len: usize) {
        self.dict_entries = self.dict_entries.max(len);
    }

    /// Merge two `Limits` with each other, taking the maximum
    /// of each field from both.
    pub fn merge(&mut self, other: &Limits) {
        self.register_int(other.int);
        self.register_real(other.real);
        self.register_name_len(other.name_len);
        self.register_str_len(other.str_len);
        self.register_array_len(other.array_len);
        self.register_dict_entries(other.dict_entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, Content, Finish, Name, Rect, Ref, Str, TextStr};

    #[test]
    fn test_content_limits() {
        let mut limits = Limits::default();

        let mut content = Content::new();
        content.cubic_to(14.3, 16.2, 22.6, 30.9, 50.1, 40.0);
        content.show(Str(b"Some text"));
        content.set_font(Name(b"NotoSans"), 10.0);
        let buf = content.finish();
        limits.merge(buf.limits());

        let mut content = Content::new();
        content.line_to(55.0, -75.3);
        content.set_font(Name(b"Noto"), 10.0);
        content
            .show_positioned()
            .items()
            .show(Str(b"A"))
            .show(Str(b"B"))
            .adjust(32.0);
        content
            .marked_content_point_with_properties(Name(b"Hi"))
            .properties()
            .actual_text(TextStr("text"));
        let buf = content.finish();
        limits.merge(buf.limits());

        assert_eq!(
            limits,
            Limits {
                int: 55,
                real: 75.3,
                name_len: 10,
                str_len: 9,
                array_len: 3,
                dict_entries: 1,
            }
        )
    }

    #[test]
    fn test_chunk_limits() {
        let mut limits = Limits::default();

        let mut chunk = Chunk::new();
        let mut x_object = chunk.form_xobject(Ref::new(1), &[]);
        x_object.bbox(Rect::new(4.0, 6.0, 22.1, 31.0));
        x_object.finish();

        limits.merge(chunk.limits());

        assert_eq!(
            limits,
            Limits {
                int: 31,
                real: 22.1,
                name_len: 7,
                str_len: 0,
                array_len: 4,
                dict_entries: 4,
            }
        )
    }
}
