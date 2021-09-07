use std::io::Write;

use super::Primitive;

/// Additional methods for byte buffers.
pub trait BufExt {
    fn push_val<T: Primitive>(&mut self, value: T);
    fn push_bytes(&mut self, bytes: &[u8]);
    fn push_int(&mut self, value: i32);
    fn push_float(&mut self, value: f32);
    fn push_hex(&mut self, value: u8);
    fn push_hex_u16(&mut self, value: u16);
}

impl BufExt for Vec<u8> {
    #[inline]
    fn push_val<T: Primitive>(&mut self, value: T) {
        value.write(self);
    }

    #[inline]
    fn push_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    #[inline]
    fn push_int(&mut self, value: i32) {
        write!(self, "{}", value).unwrap();
    }

    #[inline]
    fn push_float(&mut self, value: f32) {
        write!(self, "{}", value).unwrap();
    }

    #[inline]
    fn push_hex(&mut self, value: u8) {
        fn hex(b: u8) -> u8 {
            if b < 10 { b'0' + b } else { b'A' + (b - 10) }
        }

        self.push(hex(value >> 4));
        self.push(hex(value & 0xF));
    }

    #[inline]
    fn push_hex_u16(&mut self, value: u16) {
        self.push_hex((value >> 8) as u8);
        self.push_hex(value as u8);
    }
}
