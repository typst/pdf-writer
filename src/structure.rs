use super::*;

/// Writer for a _document catalog_.
///
/// This struct is created by [`PdfWriter::catalog`].
pub struct Catalog<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> Catalog<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Catalog"));
        Self { dict }
    }

    /// Write the `/Pages` attribute pointing to the root page tree.
    pub fn pages(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Pages"), id);
        self
    }

    /// Write the `/PageLayout` attribute to determine how the viewer will
    /// display the document's pages.
    pub fn page_layout(&mut self, layout: PageLayout) -> &mut Self {
        self.pair(Name(b"PageLayout"), layout.to_name());
        self
    }

    /// Write the `/PageMode` attribute to set which chrome elements the viewer
    /// should show.
    pub fn page_mode(&mut self, mode: PageMode) -> &mut Self {
        self.pair(Name(b"PageMode"), mode.to_name());
        self
    }

    /// Write the `/Outlines` attribute pointing to the root
    /// [outline dictionary](Outline).
    pub fn outlines(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Outlines"), id);
        self
    }

    /// Write the `/Dests` attribute pointing to a
    /// [named destinations dictionary](Destinations). PDF 1.1+.
    pub fn destinations(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Dests"), id);
        self
    }

    /// Start writing the `/ViewerPreferences` dictionary. PDF 1.2+.
    pub fn viewer_preferences(&mut self) -> ViewerPreferences<'_> {
        ViewerPreferences::new(self.key(Name(b"ViewerPreferences")))
    }
}

deref!('a, Catalog<'a> => Dict<IndirectGuard<'a>>, dict);

/// Writer for a _page tree_.
///
/// This struct is created by [`PdfWriter::pages`].
pub struct Pages<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> Pages<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pages"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/Kids` and `/Count` attributes.
    pub fn kids(&mut self, kids: impl IntoIterator<Item = Ref>) -> &mut Self {
        let len = self.key(Name(b"Kids")).array().typed().items(kids).len();
        self.pair(Name(b"Count"), len);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.key(Name(b"Resources")))
    }
}

deref!('a, Pages<'a> => Dict<IndirectGuard<'a>>, dict);

/// Writer for a _page_.
///
/// This struct is created by [`PdfWriter::page`].
pub struct Page<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> Page<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Page"));
        Self { dict }
    }

    /// Write the `/Parent` attribute.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/MediaBox` attribute. This is the size of the physical medium
    /// the page gets printed onto.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Write the `/CropBox` attribute. This is the size of the area within
    /// which content is visible.
    pub fn crop_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"CropBox"), rect);
        self
    }

    /// Write the `/BleedBox` attribute. This is the size of the area within
    /// which content is visible in a print production environment. Most
    /// production-aiding marks should be outside of this box. PDF 1.3+.
    pub fn bleed_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"BleedBox"), rect);
        self
    }

    /// Write the `/TrimBox` attribute. This is the size of the produced
    /// document after trimming is applied. PDF 1.3+.
    pub fn trim_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"TrimBox"), rect);
        self
    }

    /// Write the `/ArtBox` attribute. This is the area that another program
    /// importing this file should use. PDF 1.3+.
    pub fn art_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"ArtBox"), rect);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.key(Name(b"Resources")))
    }

    /// Write the `/Contents` attribute.
    pub fn contents(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Contents"), id);
        self
    }

    /// Write the `/Dur` attribute. This is the amount of seconds the page
    /// should be displayed before advancing to the next one. PDF 1.1+.
    pub fn duration(&mut self, seconds: f32) -> &mut Self {
        self.pair(Name(b"Dur"), seconds);
        self
    }

    /// Start writing the `/Trans` dictionary. This sets a transition effect for
    /// advancing to the next page. PDF 1.1+.
    pub fn transition(&mut self) -> Transition<'_> {
        Transition::new(self.key(Name(b"Trans")))
    }

    /// Start writing the `/Annots` (annotations) array.
    pub fn annotations(&mut self) -> Annotations<'_> {
        Annotations::start(self.key(Name(b"Annots")))
    }
}

deref!('a, Page<'a> => Dict<IndirectGuard<'a>>, dict);

/// Writer for a _resource dictionary_.
///
/// This struct is created by [`Pages::resources`], [`Page::resources`] and
/// [`TilingStream::resources`].
pub struct Resources<'a> {
    dict: Dict<&'a mut PdfWriter>,
}

impl<'a> Resources<'a> {
    pub(crate) fn new(obj: Obj<&'a mut PdfWriter>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start writing the `/XObject` dictionary.
    pub fn x_objects(&mut self) -> TypedDict<Ref, &mut PdfWriter> {
        self.key(Name(b"XObject")).dict().typed()
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<Ref, &mut PdfWriter> {
        self.key(Name(b"Font")).dict().typed()
    }

    /// Start writing the `/ColorSpace` dictionary. PDF 1.1+.
    pub fn color_spaces(&mut self) -> ColorSpaces<'_> {
        ColorSpaces::new(self.key(Name(b"ColorSpace")))
    }

    /// Start writing the `/Pattern` dictionary. PDF 1.2+.
    pub fn patterns(&mut self) -> TypedDict<Ref, &mut PdfWriter> {
        self.key(Name(b"Pattern")).dict().typed()
    }

    /// Start writing the `/Shading` dictionary. PDF 1.3+.
    pub fn shadings(&mut self) -> TypedDict<Ref, &mut PdfWriter> {
        self.key(Name(b"Shading")).dict().typed()
    }
}

deref!('a, Resources<'a> => Dict<&'a mut PdfWriter>, dict);

/// How the viewer should lay out the pages in the document.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PageLayout {
    /// Only a single page at a time.
    SinglePage,
    /// A single, continously scrolling column of pages.
    OneColumn,
    /// Two continously scrolling columns of pages, laid out with odd-numbered
    /// pages on the left.
    TwoColumnLeft,
    /// Two continously scrolling columns of pages, laid out with odd-numbered
    /// pages on the right (like in a left-bound book).
    TwoColumnRight,
    /// Only two pages are visible at a time, laid out with odd-numbered pages
    /// on the left. PDF 1.5+.
    TwoPageLeft,
    /// Only two pages are visible at a time, laid out with odd-numbered pages
    /// on the right (like in a left-bound book). PDF 1.5+.
    TwoPageRight,
}

impl PageLayout {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::SinglePage => Name(b"SinglePage"),
            Self::OneColumn => Name(b"OneColumn"),
            Self::TwoColumnLeft => Name(b"TwoColumnLeft"),
            Self::TwoColumnRight => Name(b"TwoColumnRight"),
            Self::TwoPageLeft => Name(b"TwoPageLeft"),
            Self::TwoPageRight => Name(b"TwoPageRight"),
        }
    }
}

/// Elements of the viewer chrome that should be visible when opening the
/// document.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PageMode {
    /// Neither the document outline panel nor a panel with page preview images
    /// are visible.
    UseNone,
    /// The document outline panel is visible.
    UseOutlines,
    /// A panel with page preview images is visible.
    UseThumbs,
    /// Show the document page in full screen mode, with no chrome.
    FullScreen,
}

impl PageMode {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::UseNone => Name(b"UseNone"),
            Self::UseOutlines => Name(b"UseOutlines"),
            Self::UseThumbs => Name(b"UseThumbs"),
            Self::FullScreen => Name(b"FullScreen"),
        }
    }
}

/// Writer for an _outline dictionary_.
///
/// This struct is created by [`PdfWriter::outline`].
pub struct Outline<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> Outline<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Outlines"));
        Self { dict }
    }

    /// Write the `/First` attribute which points to the first
    /// [item](OutlineItem) in the document's outline.
    pub fn first(&mut self, item: Ref) -> &mut Self {
        self.pair(Name(b"First"), item);
        self
    }

    /// Write the `/Last` attribute which points to the last [item](OutlineItem)
    /// in the document's outline.
    pub fn last(&mut self, item: Ref) -> &mut Self {
        self.pair(Name(b"Last"), item);
        self
    }

    /// Write the `/Count` attribute. This tells the viewer how many outline
    /// elements (at all levels) are currently visible.
    ///
    /// Panics if `count` is negative.
    pub fn count(&mut self, count: i32) -> &mut Self {
        assert!(count >= 0, "visible outline count must not be negative");
        self.pair(Name(b"Count"), count);
        self
    }
}

deref!('a, Outline<'a> => Dict<IndirectGuard<'a>>, dict);

/// Writer for an _outline item dictionary_.
///
/// This struct is created by [`PdfWriter::outline_item`].
pub struct OutlineItem<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> OutlineItem<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Outlines"));
        Self { dict }
    }

    /// Write the `/Title` attribute.
    pub fn title(&mut self, title: TextStr) -> &mut Self {
        self.pair(Name(b"Title"), title);
        self
    }

    /// Write the `/Parent` attribute which points to the item's parent or the
    /// top-level outline dictionary.
    pub fn parent(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), outline);
        self
    }

    /// Write the `/Prev` attribute which points to the previous item on the
    /// item's level.
    pub fn prev(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"Prev"), outline);
        self
    }

    /// Write the `/Next` attribute which points to the next item on the item's
    /// level.
    pub fn next(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"Next"), outline);
        self
    }

    /// Write the `/First` attribute which points to the item's first child.
    pub fn first(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"First"), outline);
        self
    }

    /// Write the `/Last` attribute which points to the item's last child.
    pub fn last(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"Last"), outline);
        self
    }

    /// Write the `/Count` attribute. This tells the viewer how many outline
    /// element children are currently visible. If the item is collapsed, this
    /// number shall be negative indicating how many elements you would be able
    /// to see if it was open.
    pub fn count(&mut self, items: i32) -> &mut Self {
        self.pair(Name(b"Count"), items);
        self
    }

    /// Start writing the `/Dest` attribute to set the destination of this
    /// outline item.
    pub fn dest_direct(&mut self, page: Ref) -> Destination<'_> {
        Destination::start(self.key(Name(b"Dest")), page)
    }

    /// Write the `/Dest` attribute to set the destination of this
    /// outline item to a named destination.
    pub fn dest_name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"Dest"), name);
        self
    }

    /// Write the `/C` attribute using an RGB color. This sets the color in
    /// which the outline item's title should be rendered. PDF 1.4+.
    pub fn color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed().items([r, g, b]);
        self
    }

    /// Write the `/F` attribute. PDF 1.4+.
    pub fn flags(&mut self, flags: OutlineItemFlags) -> &mut Self {
        self.pair(Name(b"F"), flags.bits() as i32);
        self
    }
}

deref!('a, OutlineItem<'a> => Dict<IndirectGuard<'a>>, dict);

bitflags::bitflags! {
    /// Bitflags describing the appearance of an outline item.
    pub struct OutlineItemFlags: u32 {
        /// This renders the outline item italicized.
        const ITALIC = 1 << 0;
        /// This renders the outline item emboldened.
        const BOLD = 1 << 1;
    }
}

/// Writer for a _named destinations dictionary_.
///
/// This struct is created by [`PdfWriter::destinations`].
pub struct Destinations<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> Destinations<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start adding another named destination.
    pub fn insert(&mut self, name: Name, page: Ref) -> Destination<'_> {
        Destination::start(self.key(name), page)
    }
}

deref!('a, Destinations<'a> => Dict<IndirectGuard<'a>>, dict);

/// Writer for a _destination array_.
///
/// This struct is created by [`Destinations::insert`] and [`Action::dest_direct`].
pub struct Destination<'a> {
    array: Array<&'a mut PdfWriter>,
}

impl<'a> Destination<'a> {
    pub(crate) fn start(obj: Obj<&'a mut PdfWriter>, page: Ref) -> Self {
        let mut array = obj.array();
        array.item(page);
        Self { array }
    }

    /// Write the `/XYZ` command which skips to the specified coordinated.
    pub fn xyz(mut self, left: f32, top: f32, zoom: Option<f32>) {
        self.item(Name(b"XYZ"));
        self.item(left);
        self.item(top);
        self.item(zoom.unwrap_or_default());
    }

    /// Write the `/Fit` command which fits all of the referenced page on
    /// screen.
    pub fn fit(mut self) {
        self.item(Name(b"Fit"));
    }

    /// Write the `/FitH` command which fits the referenced page to the screen
    /// width and skips to the specified offset.
    pub fn fit_horizontal(mut self, top: f32) {
        self.item(Name(b"FitH"));
        self.item(top);
    }

    /// Write the `/FitV` command which fits the referenced page to the screen
    /// height and skips to the specified offset.
    pub fn fit_vertical(mut self, left: f32) {
        self.item(Name(b"FitV"));
        self.item(left);
    }

    /// Write the `/FitR` command which fits the rectangle argument on the
    /// screen.
    pub fn fit_rect(mut self, rect: Rect) {
        self.item(Name(b"FitR"));
        self.item(rect.x1);
        self.item(rect.y1);
        self.item(rect.x2);
        self.item(rect.y2);
    }

    /// Write the `/FitB` command which fits all of the referenced page's
    /// content on screen. PDF 1.1+.
    pub fn fit_bounding_box(mut self) {
        self.item(Name(b"FitB"));
    }

    /// Write the `/FitBH` command which fits the referenced page's content to
    /// the screen width and skips to the specified offset. PDF 1.1+.
    pub fn fit_bounding_box_horizontal(mut self, top: f32) {
        self.item(Name(b"FitBH"));
        self.item(top);
    }

    /// Write the `/FitBV` command which fits the referenced page's content to
    /// the screen height and skips to the specified offset. PDF 1.1+.
    pub fn fit_bounding_box_vertical(mut self, left: f32) {
        self.item(Name(b"FitBV"));
        self.item(left);
    }
}

deref!('a, Destination<'a> => Array<&'a mut PdfWriter>, array);

/// Writer for a _viewer preference dictionary_.
///
/// This struct is created by [`Catalog::viewer_preferences`].
pub struct ViewerPreferences<'a> {
    dict: Dict<&'a mut PdfWriter>,
}

impl<'a> ViewerPreferences<'a> {
    pub(crate) fn new(obj: Obj<&'a mut PdfWriter>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write the `/HideToolbar` attribute to set whether the viewer should hide
    /// its toolbars while the document is open.
    pub fn hide_toolbar(&mut self, hide: bool) -> &mut Self {
        self.pair(Name(b"HideToolbar"), hide);
        self
    }

    /// Write the `/HideMenubar` attribute to set whether the viewer should hide
    /// its menu bar while the document is open.
    pub fn hide_menubar(&mut self, hide: bool) -> &mut Self {
        self.pair(Name(b"HideMenubar"), hide);
        self
    }

    /// Write the `/FitWindow` attribute to set whether the viewer should resize
    /// its window to the size of the first page.
    pub fn fit_window(&mut self, fit: bool) -> &mut Self {
        self.pair(Name(b"FitWindow"), fit);
        self
    }

    /// Write the `/CenterWindow` attribute to set whether the viewer should
    /// center its window on the screen.
    pub fn center_window(&mut self, center: bool) -> &mut Self {
        self.pair(Name(b"CenterWindow"), center);
        self
    }

    /// Write the `/NonFullScreenPageMode` attribute to set which chrome
    /// elements the viewer should show for a document which requests full
    /// screen rendering in its catalog when it is not shown in full screen
    /// mode.
    ///
    /// Panics if `mode` is [`PageMode::FullScreen`].
    pub fn non_full_screen_page_mode(&mut self, mode: PageMode) -> &mut Self {
        assert!(mode != PageMode::FullScreen, "mode must not full screen");
        self.pair(Name(b"NonFullScreenPageMode"), mode.to_name());
        self
    }

    /// Write the `/Direction` attribute to aid the viewer in how to lay out the
    /// pages visually. PDF 1.3+.
    pub fn direction(&mut self, dir: Direction) -> &mut Self {
        self.pair(Name(b"Direction"), dir.to_name());
        self
    }
}

deref!('a, ViewerPreferences<'a> => Dict<&'a mut PdfWriter>, dict);

/// Predominant reading order of text.
///
/// Used to aid the viewer with the spacial ordering in which to display pages.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    /// Left-to-right.
    L2R,
    /// Right-to-left as well as vertical writing systems.
    R2L,
}

impl Direction {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::L2R => Name(b"L2R"),
            Self::R2L => Name(b"R2L"),
        }
    }
}
