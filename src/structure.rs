use super::*;

/// Writer for a _document catalog_ dictionary.
pub struct Catalog<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Catalog<'a> {
    pub(crate) fn start(any: Any<'a, Indirect>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Catalog"));
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the root page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"Pages"), id);
        self
    }
}

/// Writer for a _page tree_ dictionary.
pub struct Pages<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Pages<'a> {
    pub(crate) fn start(any: Any<'a, Indirect>) -> Self {
        let mut dict = any.dict();
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
}

/// Writer for a _page_ dictionary.
pub struct Page<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Page<'a> {
    pub(crate) fn start(any: Any<'a, Indirect>) -> Self {
        let mut dict = any.dict();
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

    /// Start writing a `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.dict.key(Name(b"Resources")))
    }

    /// Write the `/Contents` attribute.
    pub fn contents(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"Contents"), id);
        self
    }
}

/// Writer for a _resource_ dictionary.
pub struct Resources<'a> {
    dict: Dict<'a>,
}

impl<'a> Resources<'a> {
    fn new(any: Any<'a>) -> Self {
        Self { dict: any.dict() }
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<Ref> {
        self.dict.key(Name(b"Font")).dict().typed()
    }
}
