use super::*;
use crate::types::{ColorSpace, RenderingIntent};

/// Writer for an _image XObject_.
///
/// This struct is created by [`PdfWriter::image`].
pub struct ImageStream<'a> {
    stream: Stream<'a>,
}

impl<'a> ImageStream<'a> {
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"XObject"));
        stream.pair(Name(b"Subtype"), Name(b"Image"));
        Self { stream }
    }

    /// Write the `/Width` attribute.
    pub fn width(&mut self, width: i32) -> &mut Self {
        self.pair(Name(b"Width"), width);
        self
    }

    /// Write the `/Height` attribute.
    pub fn height(&mut self, height: i32) -> &mut Self {
        self.pair(Name(b"Height"), height);
        self
    }

    /// Write the `/ColorSpace` attribute.
    pub fn color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.pair(Name(b"ColorSpace"), space.to_name());
        self
    }

    /// Write the `/BitsPerComponent` attribute.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        self.pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/SMask` attribute.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.pair(Name(b"SMask"), x_object);
        self
    }

    /// Write the `/Intent` attribute. PDF 1.1+.
    pub fn intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.pair(Name(b"Intent"), intent.to_name());
        self
    }
}

deref!('a, ImageStream<'a> => Stream<'a>, stream);
