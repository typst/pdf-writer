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

    /// Start writing the `/ViewerPreferences` dictionary. Requires PDF 1.2 or
    /// later.
    pub fn viewer_preferences(&mut self) -> ViewerPreferences<'_> {
        ViewerPreferences::new(self.key(Name(b"ViewerPreferences")))
    }
}

deref!('a, Catalog<'a> => Dict<'a, IndirectGuard>, dict);

/// Writer for a _viewer preference dictionary_.
///
/// This struct is created by [`Catalog::viewer_preferences`].
pub struct ViewerPreferences<'a> {
    dict: Dict<'a>,
}

impl<'a> ViewerPreferences<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
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
    pub fn fit_window(&mut self, hide: bool) -> &mut Self {
        self.pair(Name(b"FitWindow"), hide);
        self
    }

    /// Write the `/CenterWindow` attribute to set whether the viewer should
    /// center its window on the screen.
    pub fn center_window(&mut self, hide: bool) -> &mut Self {
        self.pair(Name(b"CenterWindow"), hide);
        self
    }

    /// Write the `/NonFullScreenPageMode` attribute to set which chrome
    /// elements the viewer should show for a document which requests full
    /// screen rendering in its catalog when it is not shown in full screen
    /// mode.
    ///
    /// The function will panic if `mode` is set to [`PageMode::FullScreen`]
    /// because the specification does not allow this enum variant here.
    pub fn non_full_screen_page_mode(&mut self, mode: PageMode) -> &mut Self {
        if mode == PageMode::FullScreen {
            panic!(
                "requesting full screen view for `/NonFullScreenPageMode` is disallowed by the specification"
            );
        }

        self.pair(Name(b"NonFullScreenPageMode"), mode.to_name());
        self
    }

    /// Write the `/Direction` attribute to aid the viewer in how to lay out the
    /// pages visually. Requires PDF 1.3 or later.
    pub fn direction(&mut self, dir: Direction) -> &mut Self {
        self.pair(Name(b"Direction"), dir.to_name());
        self
    }
}

deref!('a, ViewerPreferences<'a> => Dict<'a>, dict);

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

deref!('a, Pages<'a> => Dict<'a, IndirectGuard>, dict);

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
        self.pair(Name(b"Parent"), parent);
        self
    }

    /// Write the `/MediaBox` attribute. This is the size of the physical medium
    /// the page gets printed onto.
    pub fn media_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"MediaBox"), rect);
        self
    }

    /// Write the `/CropBox` attribute.
    pub fn crop_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"CropBox"), rect);
        self
    }

    /// Write the `/BleedBox` attribute. Requires PDF 1.3 or later.
    pub fn bleed_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"BleedBox"), rect);
        self
    }

    /// Write the `/TrimBox` attribute. This is the size of the produced
    /// document after trimming is applied. Requires PDF 1.3 or later.
    pub fn trim_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"TrimBox"), rect);
        self
    }

    /// Write the `/ArtBox` attribute. This is the area that another program
    /// importing this file should use. Requires PDF 1.3 or later.
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
    /// should be displayed before advancing to the next one. Requires PDF 1.1
    /// or later.
    pub fn dur(&mut self, seconds: f32) -> &mut Self {
        self.pair(Name(b"Dur"), seconds);
        self
    }

    /// Start writing the `/Trans` dictionary. This sets a transition effect for
    /// advancing to the next page. Requires PDF 1.1 or later.
    pub fn trans(&mut self) -> Transition<'_> {
        todo!();
        Transition::new(self.key(Name(b"Trans")))
    }

    /// Start writing the `/Annots` (annotations) array.
    pub fn annots(&mut self) -> Annotations<'_> {
        Annotations::start(self.key(Name(b"Annots")))
    }
}

deref!('a, Page<'a> => Dict<'a, IndirectGuard>, dict);

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
        self.key(Name(b"XObject")).dict().typed()
    }

    /// Start writing the `/Font` dictionary.
    pub fn fonts(&mut self) -> TypedDict<'_, Ref> {
        self.key(Name(b"Font")).dict().typed()
    }
}

deref!('a, Resources<'a> => Dict<'a>, dict);

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
    /// on the left. Requires PDF 1.5 or later.
    TwoPageLeft,
    /// Only two pages are visible at a time, laid out with odd-numbered pages
    /// on the right (like in a left-bound book). Requires PDF 1.5 or later.
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

/// Predominant reading order of text. Used to aid the viewer with the spacial
/// ordering in which to display pages.
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

/// Writer for a _transition dictionary_.
///
/// This struct is created by [`Page::trans`].
pub struct Transition<'a> {
    dict: Dict<'a>,
}

impl<'a> Transition<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

deref!('a, Transition<'a> => Dict<'a>, dict);

/// Writer for the _annotations array_ in a [`Page`].
///
/// This struct is created by [`Page::annots`].
pub struct Annotations<'a> {
    array: Array<'a>,
}

impl<'a> Annotations<'a> {
    pub(crate) fn start(obj: Obj<'a>) -> Self {
        Self { array: obj.array() }
    }

    /// Start writing a new annotation dictionary.
    pub fn add(&mut self) -> Annotation<'_> {
        Annotation::new(self.obj())
    }
}

deref!('a, Annotations<'a> => Array<'a>, array);

/// Writer for an _annotation dictionary_.
///
/// This struct is created by [`Annotations::add`].
pub struct Annotation<'a> {
    dict: Dict<'a>,
}

impl<'a> Annotation<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Annot"));
        Self { dict }
    }

    /// Write the `/Subtype` attribute to tell the viewer the type of this
    /// particular annotation.
    pub fn subtype(&mut self, kind: AnnotationType) -> &mut Self {
        self.pair(Name(b"Subtype"), kind.to_name());
        self
    }

    /// Write the `/Rect` attribute. This is the location and dimensions of the
    /// annotation on the page.
    pub fn rect(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"Rect"), rect);
        self
    }

    /// Write the `/Contents` attribute. This is the content or alt-text,
    /// depending on the [`AnnotationType`].
    pub fn contents(&mut self, text: Str) -> &mut Self {
        self.pair(Name(b"Contents"), text);
        self
    }

    /// Write the `/F` attribute.
    pub fn flags(&mut self, flags: AnnotationFlags) -> &mut Self {
        self.pair(Name(b"F"), flags.to_integer());
        self
    }

    /// Write the `/C` attribute forcing a transparent color. This sets the
    /// annotations background color and its popup title bar color. Requires PDF
    /// 1.1 or later.
    pub fn color_transparent(&mut self) -> &mut Self {
        self.key(Name(b"C")).array().typed::<f32>();
        self
    }

    /// Write the `/C` attribute using a grayscale color. This sets the
    /// annotations background color and its popup title bar color. Requires PDF
    /// 1.1 or later.
    pub fn color_gray(&mut self, gray: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed::<f32>().item(gray);
        self
    }

    /// Write the `/C` attribute using a RGB color. This sets the annotations
    /// background color and its popup title bar color. Requires PDF 1.1 or
    /// later.
    pub fn color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        let mut array = self.key(Name(b"C")).array().typed::<f32>();
        array.item(r);
        array.item(g);
        array.item(b);
        drop(array);
        self
    }

    /// Write the `/C` attribute using a CMYK color. This sets the annotations
    /// background color and its popup title bar color. Requires PDF 1.1 or
    /// later.
    pub fn color_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        let mut array = self.key(Name(b"C")).array().typed::<f32>();
        array.item(c);
        array.item(m);
        array.item(y);
        array.item(k);
        drop(array);
        self
    }

    /// Start writing the `/A` dictionary. Only permissible for the subtype
    /// `Link`.
    pub fn action(&mut self) -> Action<'_> {
        Action::new(self.key(Name(b"A")))
    }

    /// Write the `/H` attribute to set what effect is used to convey that the
    /// user is pressing a link annotation. Only permissible for the subtype
    /// `Link`. Requires PDF 1.2 or later.
    pub fn highlight(&mut self, effect: HighlightEffect) -> &mut Self {
        self.pair(Name(b"H"), effect.to_name());
        self
    }
}

deref!('a, Annotation<'a> => Dict<'a>, dict);

/// Kind of the annotation to produce.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AnnotationType {
    /// Inline text.
    Text,
    /// A link.
    Link,
    /// Text coming up in a popup. Requires PDF 1.3 or later.
    FreeText,
    /// A line. Requires PDF 1.3 or later.
    Line,
    /// A square. Requires PDF 1.3 or later.
    Square,
    /// A circle. Requires PDF 1.3 or later.
    Circle,
    /// Highlighting the text on the page. Requires PDF 1.3 or later.
    Highlight,
    /// Underline the text on the page. Requires PDF 1.3 or later.
    Underline,
    /// Squiggly underline of the text on the page. Requires PDF 1.4 or later.
    Squiggly,
    /// Strike out the text on the page. Requires PDF 1.3 or later.
    StrikeOut,
}

impl AnnotationType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::Text => Name(b"Text"),
            Self::Link => Name(b"Link"),
            Self::FreeText => Name(b"FreeText"),
            Self::Line => Name(b"Line"),
            Self::Square => Name(b"Square"),
            Self::Circle => Name(b"Circle"),
            Self::Highlight => Name(b"Highlight"),
            Self::Underline => Name(b"Underline"),
            Self::Squiggly => Name(b"Squiggly"),
            Self::StrikeOut => Name(b"StrikeOut"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct AnnotationFlags {
    /// This will hide the annotation if the viewer does not recognize its
    /// subtype. Otherwise, it will be rendered as specified in its apprearance
    /// stream.
    pub invisible: bool,
    /// This hides the annotation from view and disallows interaction. Requires
    /// PDF 1.2 or later.
    pub hidden: bool,
    /// Print the annotation. If not set, it will be always hidden on print.
    /// Requires PDF 1.2 or later.
    pub print: bool,
    /// Do not zoom the annotation appearance if the document is zoomed in.
    /// Requires PDF 1.3 or later.
    pub no_zoom: bool,
    /// Do not rotate the annotation appearance if the document is zoomed in.
    /// Requires PDF 1.3 or later.
    pub no_rotate: bool,
    /// Do not view the annotation on screen. It may still show on print.
    /// Requires PDF 1.3 or later.
    pub no_view: bool,
    /// Do not allow interactions. Requires PDF 1.3 or later.
    pub read_only: bool,
    /// Do not allow the user to delete or reposition the annotation. Contents
    /// may still be changed. Requires PDF 1.4 or later.
    pub locked: bool,
    /// Invert the interpretation of the `no_view` flag for certain events.
    /// Requires PDF 1.5 or later.
    pub toggle_no_view: bool,
    /// Do not allow content changes. Requires PDF 1.7 or later.
    pub locked_contents: bool,
}

impl AnnotationFlags {
    fn to_integer(self) -> i32 {
        let mut res = 0;

        if self.invisible {
            res |= 0x1;
        }
        if self.hidden {
            res |= 0x01;
        }
        if self.print {
            res |= 0x001;
        }
        if self.no_zoom {
            res |= 0x0001;
        }
        if self.no_rotate {
            res |= 0x00001;
        }
        if self.no_view {
            res |= 0x000001;
        }
        if self.read_only {
            res |= 0x0000001;
        }
        if self.locked {
            res |= 0x00000001;
        }
        if self.toggle_no_view {
            res |= 0x000000001;
        }
        if self.locked_contents {
            res |= 0x0000000001;
        }

        res
    }
}

/// Highlighting effect.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum HighlightEffect {
    /// No effect.
    None,
    /// Invert the colors inside of the annotation rect.
    Invert,
    /// Invert the colors on the annotation border.
    Outline,
    /// Make the annotation rect's area appear depressed.
    Push,
}

impl HighlightEffect {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::None => Name(b"N"),
            Self::Invert => Name(b"I"),
            Self::Outline => Name(b"O"),
            Self::Push => Name(b"P"),
        }
    }
}

/// Writer for an _action dictionary_.
///
/// This struct is created by [`Annotation::action`].
pub struct Action<'a> {
    dict: Dict<'a>,
}

impl<'a> Action<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Action"));
        Self { dict }
    }

    /// Write the `/S` attribute to set the action type.
    pub fn action_type(&mut self, kind: ActionType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
        self
    }
}

deref!('a, Action<'a> => Dict<'a>, dict);

pub enum ActionType {
    // Go to a destination in the document.
    GoTo,
    // Launch an application.
    Launch,
    // Begin reading an article thread.
    Thread,
    // Open a URI.
    Uri,
}

impl ActionType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::GoTo => Name(b"GoTo"),
            Self::Launch => Name(b"Launch"),
            Self::Thread => Name(b"Thread"),
            Self::Uri => Name(b"Uri"),
        }
    }
}

// TODO: 7.11.3, 12.6.4.2, 12.6.4.5-7, 12.5.2 Border, 12.5.4, 12.4.4.1, 12.3.2.2-3, and maybe 12.3.3
