use super::*;
use crate::types::{ColorSpace, RenderingIntent};

/// Writer for an _image XObject stream_.
///
/// This struct is created by [`PdfWriter::image`].
pub struct Image<'a> {
    stream: Stream<'a>,
}

impl<'a> Image<'a> {
    /// Create a new image stream writer.
    pub fn new(mut stream: Stream<'a>) -> Self {
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

    /// Write the `/SMask` attribute. PDF 1.4+.
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

deref!('a, Image<'a> => Stream<'a>, stream);

/// Writer for an _form XObject stream_. PDF 1.1+.
///
/// This struct is created by [`PdfWriter::form_xobject`].
pub struct FormXObject<'a> {
    stream: Stream<'a>,
}

impl<'a> FormXObject<'a> {
    /// Create a new form stream writer.
    pub fn new(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"XObject"));
        stream.pair(Name(b"Subtype"), Name(b"Form"));
        stream.pair(Name(b"FormType"), 1);
        Self { stream }
    }

    /// Write the `/BBox` attribute. Required.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/Matrix` attribute to map form space to user space.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }

    /// Start writing the `/Resources` dictionary to specify the resources used by the
    /// XObject. This makes it independant of the parent content stream it is
    /// eventually invoked in. PDF 1.2+.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.key(Name(b"Resources")))
    }

    /// Start writing the `/Group` dictionary to set up transparency model parameters and
    /// let this XObject be known as a group. PDF 1.4+.
    pub fn group(&mut self) -> Group<'_> {
        Group::new(self.key(Name(b"Group")))
    }

    /// Start writing the `/Ref` dictionary to identify the page from an external document
    /// that the XObject is a reference to. PDF 1.4+.
    pub fn reference(&mut self) -> Reference<'_> {
        Reference::new(self.key(Name(b"Ref")))
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

/// Writer for an _group XObject dictionary_. PDF 1.4+.
///
/// This struct is created by [`FormXObject::group`] and [`Page::group`].
pub struct Group<'a> {
    dict: Dict<'a>,
}

impl<'a> Group<'a> {
    /// Create a new group dictionary writer.
    pub fn new(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Group"));
        Self { dict }
    }

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
    pub fn color_space(&mut self, space: ColorSpace) -> &mut Self {
        self.pair(Name(b"CS"), space.to_name());
        self
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

impl<'a> Reference<'a> {
    /// Create a new reference dictionary writer.
    pub fn new(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write the `/F` attribute to set the file path. Directories are indicated
    /// by `/`, independent of the platform. Required.
    pub fn file(&mut self, file: Str) -> &mut Self {
        self.pair(Name(b"F"), file);
        self
    }

    /// Write the `/Page` attribute to set the page number. Setting the
    /// attribute through either this function or [`Self::page_label`] is
    /// required.
    pub fn page_no(&mut self, page: i32) -> &mut Self {
        self.pair(Name(b"Page"), page);
        self
    }

    /// Write the `/Page` attribute to set the page label. Setting the attribute
    /// through either this function or [`Self::page_no`] is required.
    pub fn page_label(&mut self, label: TextStr) -> &mut Self {
        self.pair(Name(b"Page"), label);
        self
    }

    /// Write the `/ID` attribute to set the file identifier.
    pub fn id(&mut self, id: [Str; 2]) -> &mut Self {
        self.key(Name(b"ID")).array().typed().items(id);
        self
    }
}

deref!('a, Reference<'a> => Dict<'a>, dict);
