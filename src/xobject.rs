use super::*;
use crate::types::RenderingIntent;

/// Writer for an _image XObject stream_.
///
/// This struct is created by [`Chunk::image_xobject`].
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

    /// Start writing the `/ColorSpace` attribute.
    ///
    /// Required for all images except if using the `JPXDecode` filter.
    /// If this is an image soft mask, the color space must be `DeviceGray`.
    /// Must not be `Pattern`.
    pub fn color_space(&mut self) -> ColorSpace<'_> {
        self.insert(Name(b"ColorSpace")).start()
    }

    /// Write the `/ColorSpace` attribute as a name from the resource dictionary.
    ///
    /// Required for all images except if using the `JPXDecode` filter.
    /// If this is an image soft mask, the color space must be `DeviceGray`.
    /// Must not be `Pattern`.
    pub fn color_space_name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"ColorSpace"), name);
        self
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

    /// Write the `/ImageMask` attribute to set whether this image is a clipping
    /// mask. If so, the `/BitsPerComponent` must be `1` and `/Mask` and
    /// `/ColorSpace` attributes shall be left undefined.
    pub fn image_mask(&mut self, mask: bool) -> &mut Self {
        self.pair(Name(b"ImageMask"), mask);
        self
    }

    /// Write the `/Mask` attribute to set a color key mask. The iterable color
    /// argument must contain a range of colors (minimum and maximum) for each
    /// channel that shall be masked out. PDF 1.3+.
    pub fn color_mask(&mut self, colors: impl IntoIterator<Item = i32>) -> &mut Self {
        self.insert(Name(b"Mask")).array().typed().items(colors);
        self
    }

    /// Write the `/Mask` attribute to set another image as the stencil mask of
    /// this image.
    pub fn stencil_mask(&mut self, mask: Ref) -> &mut Self {
        self.pair(Name(b"Mask"), mask);
        self
    }

    /// Write the `/Decode` attribute to set the decoding of the image sample
    /// colors to the specified color space. Must have twice the amount of
    /// elements as the color space.
    pub fn decode(&mut self, decode: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Decode")).array().typed().items(decode);
        self
    }

    /// Write the `/Interpolate` attribute.
    ///
    /// Must be false or unset for PDF/A files.
    pub fn interpolate(&mut self, interpolate: bool) -> &mut Self {
        self.pair(Name(b"Interpolate"), interpolate);
        self
    }

    /// Write the `/Alternates` attribute. PDF 1.3+.
    ///
    /// Images that may replace this image. The order is not relevant.
    ///
    /// Note that this key is forbidden in PDF/A.
    pub fn alternates(&mut self, alternates: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Alternates")).array().items(alternates);
        self
    }

    /// Start writing the `/SMask` attribute. PDF 1.4+.
    ///
    /// Must not be used if this image already is an image soft mask.
    ///
    /// Note that this key is forbidden in PDF/A-1.
    pub fn s_mask(&mut self, x_object: Ref) -> &mut Self {
        self.pair(Name(b"SMask"), x_object);
        self
    }
    ///
    /// Note that this key is forbidden in PDF/A-1.

    /// Write the `/SMaskInData` attribute. PDF 1.5+.
    ///
    /// May only be used for images that use the `JPXDecode` filter. If set to
    /// something other than `Ignore`, the `SMask` attribute must not be used.
    pub fn s_mask_in_data(&mut self, mode: SMaskInData) -> &mut Self {
        self.pair(Name(b"SMaskInData"), mode.to_int());
        self
    }

    /// Write the `/StructParent` attribute to indicate the [structure tree
    /// element][StructElement] this image belongs to. PDF 1.3+.
    pub fn struct_parent(&mut self, key: i32) -> &mut Self {
        self.pair(Name(b"StructParent"), key);
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

    /// Write the `/Metadata` attribute to specify the image's metadata. PDF
    /// 1.4+.
    ///
    /// The reference shall point to a [metadata stream](Metadata).
    pub fn metadata(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), id);
        self
    }

    /// Start writing the `/AF` array to specify the associated files of the
    /// image. PDF 2.0+ or PDF/A-3.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.insert(Name(b"AF")).array().typed()
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
/// This struct is created by [`Chunk::form_xobject`].
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

    /// Start writing the `/Resources` dictionary to specify the resources used
    /// by the XObject. This makes it independent of the parent content stream
    /// it is eventually invoked in. PDF 1.2+.
    pub fn resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"Resources")).start()
    }

    /// Start writing the `/Group` dictionary to set up transparency model
    /// parameters and let this XObject be known as a group. PDF 1.4+.
    pub fn group(&mut self) -> Group<'_> {
        self.insert(Name(b"Group")).start()
    }

    /// Write the `/StructParent` attribute to indicate the [structure tree
    /// element][StructElement] this XObject belongs to. Mutually exclusive with
    /// [`Self::struct_parents`]. PDF 1.3+.
    pub fn struct_parent(&mut self, key: i32) -> &mut Self {
        self.pair(Name(b"StructParent"), key);
        self
    }

    /// Write the `/StructParents` attribute to indicate the [structure tree
    /// elements][StructElement] the contents of this XObject may belong to.
    /// Mutually exclusive with [`Self::struct_parent`]. PDF 1.3+.
    pub fn struct_parents(&mut self, key: i32) -> &mut Self {
        self.pair(Name(b"StructParents"), key);
        self
    }

    /// Start writing the `/Ref` dictionary to identify the page from an
    /// external document that the XObject is a reference to. PDF 1.4+.
    ///
    /// Note that this key is forbidden in PDF/A.
    pub fn reference(&mut self) -> Reference<'_> {
        self.insert(Name(b"Ref")).start()
    }

    /// Write the `/Metadata` attribute to specify the XObject's metadata. PDF
    /// 1.4+.
    ///
    /// The reference shall point to a [metadata stream](Metadata).
    pub fn metadata(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), id);
        self
    }

    /// Write the `/LastModified` attribute. PDF 1.3+.
    pub fn last_modified(&mut self, last_modified: Date) -> &mut Self {
        self.pair(Name(b"LastModified"), last_modified);
        self
    }

    /// Start writing the `/AF` array to specify the associated files of the
    /// Form XObject. PDF 2.0+ or PDF/A-3.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.insert(Name(b"AF")).array().typed()
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

    /// Start writing the `/CS` attribute to set the color space.
    ///
    /// This is optional for isolated groups and required for groups where the
    /// color space cannot be derived from the parent.
    ///
    /// Required in PDF/A-2 through PDF/A-4 if there is no OutputIntent.
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
    /// separately against the backdrop instead of on top of each other.
    pub fn knockout(&mut self, knockout: bool) -> &mut Self {
        self.pair(Name(b"K"), knockout);
        self
    }
}

deref!('a, Group<'a> => Dict<'a>, dict);

/// Writer for an _external XObject reference dictionary_. PDF 1.4+.
///
/// This struct is created by [`FormXObject::reference`].
///
/// Reference XObjects are forbidden in PDF/A.
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
