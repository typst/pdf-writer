use super::*;

/// Writer for a _document catalog_ dictionary.
pub struct Catalog<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Catalog<'a> {
    pub(crate) fn start(obj: Object<'a, Indirect>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Catalog"));
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the root page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.dict.pair("Pages", id);
        self
    }
}

/// Writer for a _page tree_ dictionary.
pub struct Pages<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Pages<'a> {
    pub(crate) fn start(obj: Object<'a, Indirect>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Pages"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair("Parent", parent);
        self
    }

    /// Write the `/Kids` and `/Count` attributes.
    pub fn kids(&mut self, kids: impl IntoIterator<Item = Ref>) -> &mut Self {
        let len = self.dict.key("Kids").array().typed().items(kids).len();
        self.dict.pair("Count", len);
        self
    }
}

/// Writer for a _page_ dictionary.
pub struct Page<'a> {
    dict: Dict<'a, Indirect>,
}

impl<'a> Page<'a> {
    pub(crate) fn start(obj: Object<'a, Indirect>) -> Self {
        let mut dict = obj.dict();
        dict.pair("Type", Name("Page"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair("Parent", parent);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.dict.pair("MediaBox", rect);
        self
    }

    /// Start writing a `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.dict.key("Resources"))
    }

    /// Write the `/Contents` attribute.
    pub fn contents(&mut self, id: Ref) -> &mut Self {
        self.dict.pair("Contents", id);
        self
    }
}

/// Writer for a _resource_ dictionary.
pub struct Resources<'a> {
    dict: Dict<'a>,
}

impl<'a> Resources<'a> {
    fn new(obj: Object<'a>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<Ref> {
        self.dict.key("Font").dict().typed()
    }
}
