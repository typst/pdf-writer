use std::borrow::Cow;
use std::convert::TryFrom;

use super::*;

/// Writer for a stream dictionary.
pub struct Stream<'a> {
    dict: Dict<StreamGuard<'a>>,
}

impl<'a> Stream<'a> {
    pub(crate) fn start(indirect: IndirectGuard<'a>, data: Cow<'a, [u8]>) -> Self {
        let len = data.len();

        let mut dict = Dict::start(StreamGuard::start(data, indirect));
        dict.pair(
            Name(b"Length"),
            i32::try_from(len).unwrap_or_else(|_| {
                panic!("data length (is `{}`) must be <= i32::MAX", len);
            }),
        );

        Self { dict }
    }

    /// Write the `/Filter` attribute.
    pub fn filter(&mut self, filter: Filter) -> &mut Self {
        self.pair(Name(b"Filter"), filter.name());
        self
    }
}

deref!('a, Stream<'a> => Dict<StreamGuard<'a>>, dict);

/// A guard that ensures a stream is finished when it's dropped.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub struct StreamGuard<'a> {
    indirect: IndirectGuard<'a>,
    data: Cow<'a, [u8]>,
}

impl<'a> StreamGuard<'a> {
    pub(crate) fn start(data: Cow<'a, [u8]>, indirect: IndirectGuard<'a>) -> Self {
        Self { indirect, data }
    }
}

impl Drop for StreamGuard<'_> {
    fn drop(&mut self) {
        self.indirect.buf.push_bytes(b"\nstream\n");
        self.indirect.buf.push_bytes(&self.data);
        self.indirect.buf.push_bytes(b"\nendstream");
    }
}

impl Deref for StreamGuard<'_> {
    type Target = PdfWriter;

    fn deref(&self) -> &Self::Target {
        &self.indirect
    }
}

impl DerefMut for StreamGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.indirect
    }
}

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
    fn name(self) -> Name<'static> {
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
