use std::borrow::Cow;
use std::convert::TryFrom;

use super::*;

/// Writer for a stream dictionary.
pub struct Stream<'a> {
    dict: Dict<'a, StreamGuard<'a>>,
}

impl<'a> Stream<'a> {
    pub(crate) fn start(
        w: &'a mut PdfWriter,
        data: Cow<'a, [u8]>,
        indirect: IndirectGuard,
    ) -> Self {
        let len = data.len();
        let stream = StreamGuard::new(data, indirect);

        let mut dict = Dict::start(w, stream);
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

deref!('a, Stream<'a> => Dict<'a, StreamGuard<'a>>, dict);

/// A guard that finishes a stream when released.
///
/// This is an implementation detail that you shouldn't need to worry about.
pub struct StreamGuard<'a> {
    indirect: IndirectGuard,
    data: Cow<'a, [u8]>,
}

impl<'a> StreamGuard<'a> {
    pub(crate) fn new(data: Cow<'a, [u8]>, indirect: IndirectGuard) -> Self {
        Self { indirect, data }
    }
}

impl Guard for StreamGuard<'_> {
    fn finish(&self, w: &mut PdfWriter) {
        w.buf.push_bytes(b"\nstream\n");
        w.buf.push_bytes(&self.data);
        w.buf.push_bytes(b"\nendstream");
        self.indirect.finish(w);
    }
}

/// A compression filter.
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
