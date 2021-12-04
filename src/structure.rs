use super::*;

/// Writer for a _document catalog dictionary_.
///
/// This struct is created by [`PdfWriter::catalog`].
pub struct Catalog<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for Catalog<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Catalog"));
        Self { dict }
    }
}

impl<'a> Catalog<'a> {
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
        self.insert(Name(b"ViewerPreferences")).start()
    }

    /// Write the `/PageLabels` attribute to specify the page labels. PDF 1.3+.
    pub fn page_labels(&mut self) -> Dict<'_> {
        self.insert(Name(b"PageLabels")).start()
    }

    /// Write the `/Lang` attribute to specify the language of the document as a
    /// RFC 3066 language tag. PDF 1.4+.
    pub fn lang(&mut self, lang: TextStr) -> &mut Self {
        self.pair(Name(b"Lang"), lang);
        self
    }
}

deref!('a, Catalog<'a> => Dict<'a>, dict);

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
    pub(crate) fn to_name(self) -> Name<'static> {
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
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::UseNone => Name(b"UseNone"),
            Self::UseOutlines => Name(b"UseOutlines"),
            Self::UseThumbs => Name(b"UseThumbs"),
            Self::FullScreen => Name(b"FullScreen"),
        }
    }
}

/// Writer for a _viewer preference dictionary_.
///
/// This struct is created by [`Catalog::viewer_preferences`].
pub struct ViewerPreferences<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for ViewerPreferences<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

impl<'a> ViewerPreferences<'a> {
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

deref!('a, ViewerPreferences<'a> => Dict<'a>, dict);

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
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::L2R => Name(b"L2R"),
            Self::R2L => Name(b"R2L"),
        }
    }
}

/// Writer for a _page label dictionary_.
pub struct PageLabel<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for PageLabel<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"PageLabel"));
        Self { dict }
    }
}

impl<'a> PageLabel<'a> {
    /// Write the `/S` attribute to set the page label's numbering style.
    ///
    /// If this attribute is omitted, only the prefix will be used, there will
    /// be no page number.
    pub fn style(&mut self, style: NumberingStyle) -> &mut Self {
        self.pair(Name(b"S"), style.to_name());
        self
    }

    /// Write the `/P` attribute to set the page label's prefix.
    pub fn prefix(&mut self, prefix: TextStr) -> &mut Self {
        self.pair(Name(b"P"), prefix);
        self
    }

    /// Write the `/St` attribute to set the page label's offset.
    ///
    /// This must be greater or equal to `1` if set.
    pub fn offset(&mut self, offset: i32) -> &mut Self {
        self.pair(Name(b"St"), offset);
        self
    }
}

deref!('a, PageLabel<'a> => Dict<'a>, dict);

/// The numbering style of a page label.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum NumberingStyle {
    /// Arabic numerals.
    Arabic,
    /// Lowercase Roman numerals.
    LowerRoman,
    /// Uppercase Roman numerals.
    UpperRoman,
    /// Lowercase letters (a-z, then aa-zz, ...).
    LowerAlpha,
    /// Uppercase letters (A-Z, then AA-ZZ, ...).
    UpperAlpha,
}

impl NumberingStyle {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            NumberingStyle::Arabic => Name(b"D"),
            NumberingStyle::LowerRoman => Name(b"r"),
            NumberingStyle::UpperRoman => Name(b"R"),
            NumberingStyle::LowerAlpha => Name(b"a"),
            NumberingStyle::UpperAlpha => Name(b"A"),
        }
    }
}

/// Writer for a _document information dictionary_.
///
/// This struct is created by [`PdfWriter::document_info`].
pub struct DocumentInfo<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for DocumentInfo<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

impl<'a> DocumentInfo<'a> {
    /// Write the `/Title` attribute to set the document's title. PDF 1.1+.
    pub fn title(&mut self, title: TextStr) -> &mut Self {
        self.pair(Name(b"Title"), title);
        self
    }

    /// Write the `/Author` attribute to set the document's author.
    pub fn author(&mut self, author: TextStr) -> &mut Self {
        self.pair(Name(b"Author"), author);
        self
    }

    /// Write the `/Subject` attribute to set the document's subject. PDF 1.1+.
    pub fn subject(&mut self, subject: TextStr) -> &mut Self {
        self.pair(Name(b"Subject"), subject);
        self
    }

    /// Write the `/Keywords` attribute to set terms associated to the document.
    /// PDF 1.1+.
    pub fn keywords(&mut self, keywords: TextStr) -> &mut Self {
        self.pair(Name(b"Keywords"), keywords);
        self
    }

    /// Write the `/Creator` attribute to set the name of the product that
    /// converted or wrote the file that this PDF has been converted from.
    pub fn creator(&mut self, creator: TextStr) -> &mut Self {
        self.pair(Name(b"Creator"), creator);
        self
    }

    /// Write the `/Producer` attribute to set the name of the product that
    /// converted or wrote this PDF.
    pub fn producer(&mut self, producer: TextStr) -> &mut Self {
        self.pair(Name(b"Producer"), producer);
        self
    }

    /// Write the `/CreationDate` attribute to set the date the document was
    /// created.
    pub fn creation_date(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"CreationDate"), date);
        self
    }

    /// Write the `/ModDate` attribute to set the date the document was last
    /// modified.
    ///
    /// Required if `/PieceInfo` is set in the document catalog.
    pub fn modified_date(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"ModDate"), date);
        self
    }

    /// Write the `/Trapped` attribute to set whether the document is fully or
    /// partially trapped. PDF 1.3+.
    pub fn trapped(&mut self, trapped: TrappingStatus) -> &mut Self {
        self.pair(Name(b"Trapped"), trapped.to_name());
        self
    }
}

deref!('a, DocumentInfo<'a> => Dict<'a>, dict);

/// Whether a document has been adjusted with traps to account for colorant
/// misregistration during the printing process.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TrappingStatus {
    /// The document is fully trapped.
    Trapped,
    /// The document has not been trapped.
    NotTrapped,
    /// The document is partially trapped or the trapping status is unknown.
    Unknown,
}

impl TrappingStatus {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            TrappingStatus::Trapped => Name(b"True"),
            TrappingStatus::NotTrapped => Name(b"False"),
            TrappingStatus::Unknown => Name(b"Unknown"),
        }
    }
}

/// Writer for a _page tree dictionary_.
///
/// This struct is created by [`PdfWriter::pages`].
pub struct Pages<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for Pages<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pages"));
        Self { dict }
    }
}

impl<'a> Pages<'a> {
    /// Write the `/Parent` attribute. Required except in root node.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/Kids` attributes, listing the immediate children of this
    /// node in the page tree. Required.
    pub fn kids(&mut self, kids: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Kids")).array().items(kids);
        self
    }

    /// Write the `/Count` attribute, specifying how many descendants this node
    /// in the page tree has. This may be different to the length of `/Kids`
    /// when the tree has multiple layers. Required.
    pub fn count(&mut self, count: i32) -> &mut Self {
        self.pair(Name(b"Count"), count);
        self
    }

    /// Write the `/MediaBox` attribute.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"Resources")).start()
    }
}

deref!('a, Pages<'a> => Dict<'a>, dict);

/// Writer for a _page dictionary_.
///
/// This struct is created by [`PdfWriter::page`].
pub struct Page<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for Page<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Page"));
        Self { dict }
    }
}

impl<'a> Page<'a> {
    /// Write the `/Parent` attribute. Required.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/LastModified` attribute. PDF 1.3+.
    pub fn last_modified(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"LastModified"), date);
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
        self.insert(Name(b"Resources")).start()
    }

    /// Write the `/Contents` attribute as reference to a single content stream.
    ///
    /// Such a content stream can be created using the [`Content`] builder and
    /// written to the file using [`PdfWriter::stream`].
    pub fn contents(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Contents"), id);
        self
    }

    /// Write the `/Contents` attribute as an array.
    pub fn contents_array(&mut self, ids: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Contents")).array().items(ids);
        self
    }

    /// Write the `/Rotate` attribute. This is the number of degrees the page
    /// should be rotated clockwise when displayed. This should be a multiple
    /// of 90.
    pub fn rotate(&mut self, degrees: i32) -> &mut Self {
        self.pair(Name(b"Rotate"), degrees);
        self
    }

    /// Start writing the `/Group` dictionary to set the transparency settings
    /// for the page. PDF 1.4+.
    pub fn group(&mut self) -> Group<'_> {
        self.insert(Name(b"Group")).start()
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
        self.insert(Name(b"Trans")).start()
    }

    /// Start writing the `/Annots` (annotations) array.
    pub fn annotations(&mut self) -> Annotations<'_> {
        self.insert(Name(b"Annots")).start()
    }

    /// Write the `/Tabs` attribute. This specifies the order in which the
    /// annotations should be focussed by hitting tab. PDF 1.5+.
    pub fn tab_order(&mut self, order: TabOrder) -> &mut Self {
        self.pair(Name(b"Tabs"), order.to_name());
        self
    }
}

deref!('a, Page<'a> => Dict<'a>, dict);

/// Writer for an _outline dictionary_.
///
/// This struct is created by [`PdfWriter::outline`].
pub struct Outline<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for Outline<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Outlines"));
        Self { dict }
    }
}

impl<'a> Outline<'a> {
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

deref!('a, Outline<'a> => Dict<'a>, dict);

/// Writer for an _outline item dictionary_.
///
/// This struct is created by [`PdfWriter::outline_item`].
pub struct OutlineItem<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for OutlineItem<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Outlines"));
        Self { dict }
    }
}

impl<'a> OutlineItem<'a> {
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
    pub fn dest_direct(&mut self) -> Destination<'_> {
        self.insert(Name(b"Dest")).start()
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
        self.insert(Name(b"C")).array().items([r, g, b]);
        self
    }

    /// Write the `/F` attribute. PDF 1.4+.
    pub fn flags(&mut self, flags: OutlineItemFlags) -> &mut Self {
        self.pair(Name(b"F"), flags.bits() as i32);
        self
    }
}

deref!('a, OutlineItem<'a> => Dict<'a>, dict);

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
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for Destinations<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

impl<'a> Destinations<'a> {
    /// Start adding another named destination.
    pub fn insert(&mut self, name: Name) -> Destination<'_> {
        self.dict.insert(name).start()
    }
}

deref!('a, Destinations<'a> => Dict<'a>, dict);

/// Writer for a _destination array_.
///
/// This struct is created by [`Destinations::insert`] and
/// [`Action::destination_direct`].
pub struct Destination<'a> {
    array: Array<'a>,
}

impl<'a> Writer<'a> for Destination<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { array: obj.array() }
    }
}

impl<'a> Destination<'a> {
    /// The target page. Required.
    pub fn page(mut self, page: Ref) -> Self {
        self.item(page);
        self
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
        self.array.items([rect.x1, rect.y1, rect.x2, rect.y2]);
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

deref!('a, Destination<'a> => Array<'a>, array);

/// What order to tab through the annotations on a page.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum TabOrder {
    RowOrder,
    ColumnOrder,
    StructureOrder,
}

impl TabOrder {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::RowOrder => Name(b"R"),
            Self::ColumnOrder => Name(b"C"),
            Self::StructureOrder => Name(b"S"),
        }
    }
}
