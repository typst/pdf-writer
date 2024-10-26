use std::ops::{Deref, DerefMut};
use super::Primitive;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Buf {
    buf: Vec<u8>
}

impl Deref for Buf {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl DerefMut for Buf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl Buf {
    pub(crate) fn new() -> Self {
        Self {
            buf: Vec::new()
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity)
        }
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.buf
    }

    #[inline]
    pub(crate) fn push_val<T: Primitive>(&mut self, value: T) {
        value.write(self);
    }

    #[inline]
    pub(crate) fn push_int(&mut self, value: i32) {
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
        if value == 0.0 || (value.abs() > 1e-6 && value.abs() < 1e12) {
            self.extend(ryu::Buffer::new().format(value).as_bytes());
        } else {
            #[inline(never)]
            fn write_extreme(buf: &mut Buf, value: f32) {
                use std::io::Write;
                write!(buf, "{}", value).unwrap();
            }

            write_extreme(self, value);
        }
    }

    #[inline]
    pub(crate) fn extend(&mut self, other: &[u8]) {
        self.buf.extend(other);
    }

    #[inline]
    pub(crate) fn push(&mut self, b: u8) {
        self.buf.push(b);
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
}
