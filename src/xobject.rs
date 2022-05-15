use super::*;
use crate::types::RenderingIntent;

/// Writer for an _image XObject stream_.
///
/// This struct is created by [`PdfWriter::image_xobject`].
pub struct ImageXObject<'a> {
    stream: Stream<'a>,
}

impl<'a> ImageXObject<'a> {
    /// Create a new image stream writer.
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
    ///
    /// Required for all images except if using the `JPXDecode` filter.
    /// If this is an image soft mask, the color space must be `DeviceGray`.
    /// Must not be `Pattern`.
    pub fn color_space(&mut self) -> ColorSpace<'_> {
        self.insert(Name(b"ColorSpace")).start()
    }

    /// Write the `/BitsPerComponent` attribute. Required.
    ///
    /// Required for all images except if using the `JPXDecode` filter.
    pub fn bits_per_component(&mut self, bits: i32) -> &mut Self {
        self.pair(Name(b"BitsPerComponent"), bits);
        self
    }

    /// Write the `/Intent` attribute. PDF 1.1+.
    pub fn intent(&mut self, intent: RenderingIntent) -> &mut Self {
        self.pair(Name(b"Intent"), intent.to_name());
        self
    }

    /// Write the `/Interpolate` attribute.
    pub fn interpolate(&mut self, interpolate: bool) -> &mut Self {
        self.pair(Name(b"Interpolate"), interpolate);
        self
    }

    /// Write the `/Alternates` attribute. PDF 1.3+.
    ///
    /// Images that may replace this image. The order is not relevant.
    pub fn alternates(&mut self, alternates: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Alternates")).array().items(alternates);
        self
    }

    /// Start writing the `/SMask` attribute. PDF 1.4+.
    ///
    /// Must not be used if this image already is an image soft mask.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.pair(Name(b"SMask"), x_object);
        self
    }

    /// Write the `/SMaskInData` attribute. PDF 1.5+.
    ///
    /// May only be used for images that use the `JPXDecode` filter. If set to
    /// something other than `Ignore`, the `SMask` attribute must not be used.
    pub fn s_mask_in_data(&mut self, mode: SMaskInData) -> &mut Self {
        self.pair(Name(b"SMaskInData"), mode.to_int());
        self
    }

    /// Write the `/Matte` attribute for image soft masks. PDF 1.4+.
    ///
    /// This shall be the matte color of the parent image encoded in its color
    /// space.
    pub fn matte(&mut self, color: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Matte")).array().items(color);
        self
    }
}

deref!('a, ImageXObject<'a> => Stream<'a>, stream);

/// What to do with in-data mask information in `JPXDecode` images.
pub enum SMaskInData {
    /// Discard the mask data.
    Ignore,
    /// Use the mask data.
    Use,
    /// Use the mask data on the image whose backdrop has been pre-blended with
    /// a matte color.
    Preblended,
}

impl SMaskInData {
    pub(crate) fn to_int(&self) -> i32 {
        match self {
            Self::Ignore => 0,
            Self::Use => 1,
            Self::Preblended => 2,
        }
    }
}

/// Writer for an _form XObject stream_. PDF 1.1+.
///
/// This struct is created by [`PdfWriter::form_xobject`].
///
/// Note that these have nothing to do with forms that have fields to fill out.
/// Rather, they are a way to encapsulate and reuse content across the file.
pub struct FormXObject<'a> {
    stream: Stream<'a>,
}

impl<'a> FormXObject<'a> {
    /// Create a new form stream writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"XObject"));
        stream.pair(Name(b"Subtype"), Name(b"Form"));
        Self { stream }
    }

    /// Write the `/BBox` attribute. Required.
    ///
    /// This clips the form xobject to coordinates in its coordinate system.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/Matrix` attribute to map form space to user space.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.insert(Name(b"Matrix")).array().items(matrix);
        self
    }

    /// Start writing the `/Resources` dictionary to specify the resources used by the
    /// XObject. This makes it independant of the parent content stream it is
    /// eventually invoked in. PDF 1.2+.
    pub fn resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"Resources")).start()
    }

    /// Start writing the `/Group` dictionary to set up transparency model parameters and
    /// let this XObject be known as a group. PDF 1.4+.
    pub fn group(&mut self) -> Group<'_> {
        self.insert(Name(b"Group")).start()
    }

    /// Start writing the `/Ref` dictionary to identify the page from an external document
    /// that the XObject is a reference to. PDF 1.4+.
    pub fn reference(&mut self) -> Reference<'_> {
        self.insert(Name(b"Ref")).start()
    }

    /// Write the `/Metadata` attribute. PDF 1.4+.
    pub fn metadata(&mut self, meta: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), meta);
        self
    }

    /// Write the `/LastModified` attribute. PDF 1.3+.
    pub fn last_modified(&mut self, last_modified: Date) -> &mut Self {
        self.pair(Name(b"LastModified"), last_modified);
        self
    }
}

deref!('a, FormXObject<'a> => Stream<'a>, stream);

/// Writer for a _group XObject dictionary_. PDF 1.4+.
///
/// This struct is created by [`FormXObject::group`] and [`Page::group`].
pub struct Group<'a> {
    dict: Dict<'a>,
}

writer!(Group: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Group"));
    Self { dict }
});

impl<'a> Group<'a> {
    /// Set the `/S` attribute to `/Transparency`. Required to set the remaining
    /// transparency parameters.
    pub fn transparency(&mut self) -> &mut Self {
        self.pair(Name(b"S"), Name(b"Transparency"));
        self
    }

    /// Set the `/CS` attribute to set the color space.
    ///
    /// This is optional for isolated groups and required for groups where the
    /// color space cannot be derived from the parent.
    pub fn color_space(&mut self) -> ColorSpace<'_> {
        self.insert(Name(b"CS")).start()
    }

    /// Set the `/I` attribute to indicate whether the group is isolated.
    ///
    /// If it is true, the group will initially be composited against a clear
    /// backdrop. If it is false, the group will be composited against the
    /// backdrop of the parent group.
    pub fn isolated(&mut self, isolated: bool) -> &mut Self {
        self.pair(Name(b"I"), isolated);
        self
    }

    /// Set the `/K` attribute to indicate whether the group is a knockout
    /// group.
    ///
    /// Within a knockout group, the group children are all composited
    /// seperately against the backdrop instead of on top of each other.
    pub fn knockout(&mut self, knockout: bool) -> &mut Self {
        self.pair(Name(b"K"), knockout);
        self
    }
}

deref!('a, Group<'a> => Dict<'a>, dict);

/// Writer for an _external XObject reference dictionary_. PDF 1.4+.
///
/// This struct is created by [`FormXObject::reference`].
pub struct Reference<'a> {
    dict: Dict<'a>,
}

writer!(Reference: |obj| Self { dict: obj.dict() });

impl<'a> Reference<'a> {
    /// Start writing the `/F` attribute to set a file specification dictionary.
    /// Required.
    pub fn file_spec(&mut self) -> FileSpec<'_> {
        self.insert(Name(b"F")).start()
    }

    /// Write the `/Page` attribute to set the page number. Setting the
    /// attribute through either this function or [`Self::page_label`] is
    /// required. Page indices start at 0.
    pub fn page_number(&mut self, page: i32) -> &mut Self {
        self.pair(Name(b"Page"), page);
        self
    }

    /// Write the `/Page` attribute to set the page label. Setting the attribute
    /// through either this function or [`Self::page_number`] is required.
    pub fn page_label(&mut self, label: TextStr) -> &mut Self {
        self.pair(Name(b"Page"), label);
        self
    }

    /// Write the `/ID` attribute to set the file identifier.
    pub fn id(&mut self, id: [Str; 2]) -> &mut Self {
        self.insert(Name(b"ID")).array().items(id);
        self
    }
}

deref!('a, Reference<'a> => Dict<'a>, dict);
