use super::*;

/// Writer for a _document catalog_.
///
/// This struct is created by [`PdfWriter::catalog`].
pub struct Catalog<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Catalog<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Catalog"));
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the root page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"Pages"), id);
        self
    }
}

/// Writer for a _page tree_.
///
/// This struct is created by [`PdfWriter::pages`].
pub struct Pages<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Pages<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pages"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/Kids` and `/Count` attributes.
    pub fn kids(&mut self, kids: impl IntoIterator<Item = Ref>) -> &mut Self {
        let len = self.dict.key(Name(b"Kids")).array().typed().items(kids).len();
        self.dict.pair(Name(b"Count"), len);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.dict.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.dict.key(Name(b"Resources")))
    }
}

/// Writer for a _page_.
///
/// This struct is created by [`PdfWriter::page`].
pub struct Page<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Page<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Page"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.dict.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.dict.key(Name(b"Resources")))
    }

    /// Write the `/Contents` attribute.
    pub fn contents(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"Contents"), id);
        self
    }
}

/// Writer for a _resource dictionary_.
///
/// This struct is created by [`Pages::resources`] and [`Page::resources`].
pub struct Resources<'a> {
    dict: Dict<'a>,
}

impl<'a> Resources<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start writing the `/XObject` dictionary.
    pub fn x_objects(&mut self) -> TypedDict<'_, Ref> {
        self.dict.key(Name(b"XObject")).dict().typed()
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<'_, Ref> {
        self.dict.key(Name(b"Font")).dict().typed()
    }
}
