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

    /// Start writing the `/ViewerPreferences` dictionary. (1.2+)
    pub fn viewer_preferences(&mut self) -> ViewerPreferences<'_> {
        ViewerPreferences::new(self.key(Name(b"ViewerPreferences")))
    }

    /// Write the `/Dests` attribute pointing to the named attribute dictionary.
    /// (1.1+)
    pub fn destinations(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Dests"), id);
        self
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
    /// The function will panic if `mode` is set to [`PageMode::FullScreen`]
    /// because the specification does not allow this enum variant here.
    pub fn non_full_screen_page_mode(&mut self, mode: PageMode) -> &mut Self {
        if mode == PageMode::FullScreen {
            panic!("mode must not full screen");
        }

        self.pair(Name(b"NonFullScreenPageMode"), mode.to_name());
        self
    }

    /// Write the `/Direction` attribute to aid the viewer in how to lay out the
    /// pages visually. (1.3+)
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

    /// Write the `/CropBox` attribute. This is the size of the area within
    /// which content is visible.
    pub fn crop_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"CropBox"), rect);
        self
    }

    /// Write the `/BleedBox` attribute. This is the size of the area within
    /// which content is visible in a print production environment. Most
    /// production-aiding marks should be outside of this box. (1.3+)
    pub fn bleed_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"BleedBox"), rect);
        self
    }

    /// Write the `/TrimBox` attribute. This is the size of the produced
    /// document after trimming is applied. (1.3+)
    pub fn trim_box(&mut self, rect: Rect) -> &mut Self {
        self.pair(Name(b"TrimBox"), rect);
        self
    }

    /// Write the `/ArtBox` attribute. This is the area that another program
    /// importing this file should use. (1.3+)
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
    /// should be displayed before advancing to the next one. (1.1+)
    pub fn duration(&mut self, seconds: f32) -> &mut Self {
        self.pair(Name(b"Dur"), seconds);
        self
    }

    /// Start writing the `/Trans` dictionary. This sets a transition effect for
    /// advancing to the next page. (1.1+)
    pub fn transition(&mut self) -> Transition<'_> {
        Transition::new(self.key(Name(b"Trans")))
    }

    /// Start writing the `/Annots` (annotations) array.
    pub fn annotations(&mut self) -> Annotations<'_> {
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
    /// on the left. (1.5+)
    TwoPageLeft,
    /// Only two pages are visible at a time, laid out with odd-numbered pages
    /// on the right (like in a left-bound book). (1.5+)
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
    pub fn contents(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"Contents"), text);
        self
    }

    /// Write the `/NM` attribute. This uniquely identifies the anotation on the
    /// page. (1.3+)
    pub fn name(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"NM"), text);
        self
    }

    /// Write the `/M` attribute, specifying the date the annotation was last
    /// modified. (1.1+)
    pub fn modified(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"M"), date);
        self
    }

    /// Write the `/F` attribute.
    pub fn flags(&mut self, flags: AnnotationFlags) -> &mut Self {
        self.pair(Name(b"F"), flags.bits() as i32);
        self
    }

    /// Write the `/Border` attribute. This describes the look of the border
    /// around the annotation, including width and horizontal and vertical
    /// border radii. The function may also receive a dash pattern which
    /// specifies the lengths and gaps of the border segments on a dashed
    /// border. Although all PDF versions accept `/Border`, this feature
    /// specifically is only available in PDF 1.1 or later.
    pub fn border(
        &mut self,
        h_radius: f32,
        v_radius: f32,
        width: f32,
        dash_pattern: Option<impl IntoIterator<Item = f32>>,
    ) -> &mut Self {
        let mut array = self.key(Name(b"Border")).array();
        array.item(h_radius).item(v_radius).item(width);

        if let Some(pattern) = dash_pattern {
            array.obj().array().typed().items(pattern);
        }

        drop(array);
        self
    }

    /// Write the `/C` attribute forcing a transparent color. This sets the
    /// annotations background color and its popup title bar color. (1.1+)
    pub fn color_transparent(&mut self) -> &mut Self {
        self.key(Name(b"C")).array().typed::<f32>();
        self
    }

    /// Write the `/C` attribute using a grayscale color. This sets the
    /// annotations background color and its popup title bar color. (1.1+)
    pub fn color_gray(&mut self, gray: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed().item(gray);
        self
    }

    /// Write the `/C` attribute using an RGB color. This sets the annotations
    /// background color and its popup title bar color. (1.1+)
    pub fn color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed().items([r, g, b]);
        self
    }

    /// Write the `/C` attribute using a CMYK color. This sets the annotations
    /// background color and its popup title bar color. (1.1+)
    pub fn color_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed().items([c, m, y, k]);
        self
    }

    /// Start writing the `/A` dictionary. Only permissible for the subtype
    /// `Link`.
    pub fn action(&mut self) -> Action<'_> {
        Action::new(self.key(Name(b"A")))
    }

    /// Write the `/H` attribute to set what effect is used to convey that the
    /// user is pressing a link annotation. Only permissible for the subtype
    /// `Link`. (1.2+)
    pub fn highlight(&mut self, effect: HighlightEffect) -> &mut Self {
        self.pair(Name(b"H"), effect.to_name());
        self
    }

    /// Write the `/T` attribute. This is in the title bar of markup annotations
    /// and should be the name of the annotation author. (1.1+)
    pub fn author(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"T"), text);
        self
    }

    /// Write the `/Subj` attribute. This is the subject of the annotation.
    /// (1.5+)
    pub fn subject(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"Subj"), text);
        self
    }

    /// Start writing the `/BS` attribute. These are some more elaborate border
    /// settings taking precedence over `/B` for some annotation types. (1.6+)
    pub fn border_style(&mut self) -> BorderStyle<'_> {
        BorderStyle::new(self.key(Name(b"BS")))
    }

    /// Write the `/QuadPoints` attribute, specifying the region in which the link should be activated. (1.6+)
    pub fn quad_points(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> &mut Self {
        self.key(Name(b"QuadPoints"))
            .array()
            .typed()
            .items([x1, y1, x2, y2, x3, y3, x4, y4]);
        self
    }

    /// Write the `/LL` attribute. This defines the start and end point of a line annotation
    pub fn line_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> &mut Self {
        self.key(Name(b"LL")).array().typed().items([x1, y1, x2, y2]);
        self
    }

    /// Start writing the `/FS` attribute, setting which file to reference.
    pub fn file(&mut self) -> FileSpec<'_> {
        FileSpec::new(self.key(Name(b"FS")))
    }

    /// Write the `/Name` attribute. This often sets the icon for the
    /// annotation.
    pub fn custom_name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"Name"), name);
        self
    }

    /// Write the `/Name` attribute to one of the predefined icon names. Please
    /// mind the documentation to see which names are allowable for which
    /// annotation type.
    pub fn icon(&mut self, name: AnnotationName) -> &mut Self {
        self.pair(Name(b"Name"), name.to_name());
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
    /// A line. (1.3+)
    Line,
    /// A square. (1.3+)
    Square,
    /// A circle. (1.3+)
    Circle,
    /// Highlighting the text on the page. (1.3+)
    Highlight,
    /// Underline the text on the page. (1.3+)
    Underline,
    /// Squiggly underline of the text on the page. (1.4+)
    Squiggly,
    /// Strike out the text on the page. (1.3+)
    StrikeOut,
    /// A reference to another file. (1.3+)
    FileAttachment,
}

impl AnnotationType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::Text => Name(b"Text"),
            Self::Link => Name(b"Link"),
            Self::Line => Name(b"Line"),
            Self::Square => Name(b"Square"),
            Self::Circle => Name(b"Circle"),
            Self::Highlight => Name(b"Highlight"),
            Self::Underline => Name(b"Underline"),
            Self::Squiggly => Name(b"Squiggly"),
            Self::StrikeOut => Name(b"StrikeOut"),
            Self::FileAttachment => Name(b"FileAttachment"),
        }
    }
}

/// Possible icons for an annotation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AnnotationIcon {
    /// Speech bubble. For use with text annotations.
    Comment,
    /// For use with text annotations.
    Key,
    /// Sticky note. For use with text annotations.
    Note,
    /// Question mark or manual. For use with text annotations.
    Help,
    /// For use with text annotations.
    NewParagraph,
    /// For use with text annotations.
    Paragraph,
    /// A plus or similar. For use with text annotations.
    Insert,
    /// Chart. For use with file attachment annotations.
    Graph,
    /// For use with file attachment annotations.
    PushPin,
    /// For use with file attachment annotations.
    Paperclip,
    /// For use with file attachment annotations.
    Tag,
}

impl AnnotationName {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::Comment => Name(b"Comment"),
            Self::Key => Name(b"Key"),
            Self::Note => Name(b"Note"),
            Self::Help => Name(b"Help"),
            Self::NewParagraph => Name(b"NewParagraph"),
            Self::Paragraph => Name(b"Paragraph"),
            Self::Insert => Name(b"Insert"),
            Self::Graph => Name(b"Graph"),
            Self::PushPin => Name(b"PushPin"),
            Self::Paperclip => Name(b"Paperclip"),
            Self::Tag => Name(b"Tag"),
        }
    }
}

bitflags::bitflags! {
    /// Bitflags describing various characteristics of fonts.
    pub struct AnnotationFlags: u32 {
        /// This will hide the annotation if the viewer does not recognize its
        /// subtype. Otherwise, it will be rendered as specified in its apprearance
        /// stream.
        const INVISIBLE = 1 << 0;
        /// This hides the annotation from view and disallows interaction. (1.2+)
        const HIDDEN = 1 << 1;
        /// Print the annotation. If not set, it will be always hidden on print.
        /// (1.2+)
        const PRINT = 1 << 2;
        /// Do not zoom the annotation appearance if the document is zoomed in.
        /// (1.3+)
        const NO_ZOOM = 1 << 3;
        /// Do not rotate the annotation appearance if the document is zoomed in.
        /// (1.3+)
        const NO_ROTATE = 1 << 4;
        /// Do not view the annotation on screen. It may still show on print.
        /// (1.3+)
        const NO_VIEW = 1 << 5;
        /// Do not allow interactions. (1.3+)
        const READ_ONLY = 1 << 6;
        /// Do not allow the user to delete or reposition the annotation. Contents
        /// may still be changed. (1.4+)
        const LOCKED = 1 << 7;
        /// Invert the interpretation of the `no_view` flag for certain events.
        /// (1.5+)
        const TOGGLE_NO_VIEW = 1 << 8;
        /// Do not allow content changes. (1.7+)
        const LOCKED_CONTENTS = 1 << 9;
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

/// Writer for a _file specification dictionary_.
///
/// This struct is created by [`Annotation::file`] and [`Action::file`].
pub struct FileSpec<'a> {
    dict: Dict<'a>,
}

impl<'a> FileSpec<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Filespec"));
        Self { dict }
    }

    /// Write the `/FS` attribute to set the file system this entry relates to.
    /// If you set the `system` argument to `Name(b"URL")`, this becomes an URL
    /// specification.
    pub fn file_system(&mut self, system: Name) -> &mut Self {
        self.pair(Name(b"FS"), system);
        self
    }

    /// Write the `/F` attribute to set the file path. Directories are indicated
    /// by `/`, independent of the platform.
    pub fn file(&mut self, path: Str) -> &mut Self {
        self.pair(Name(b"F"), path);
        self
    }

    /// Write the `/UF` attribute to set a Unicode-compatible path. Directories
    /// are indicated by `/`, independent of the platform. (1.7+)
    pub fn unic_file(&mut self, path: TextStr) -> &mut Self {
        self.pair(Name(b"UF"), path);
        self
    }

    /// Write the `/V` attribute to indicate whether to cache the file.
    pub fn volatile(&mut self, no_cache: bool) -> &mut Self {
        self.pair(Name(b"V"), no_cache);
        self
    }

    /// Write the `/Desc` attribute to set a file description. (1.6+)
    pub fn description(&mut self, desc: TextStr) -> &mut Self {
        self.pair(Name(b"Desc"), desc);
        self
    }
}

deref!('a, FileSpec<'a> => Dict<'a>, dict);

/// Writer for an _border style dictionary_.
///
/// This struct is created by [`Annotation::border_style`].
pub struct BorderStyle<'a> {
    dict: Dict<'a>,
}

impl<'a> BorderStyle<'a> {
    pub(crate) fn new(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Border"));
        Self { dict }
    }

    /// Write the `/W` attribute. This is the width of the border in points.
    pub fn width(&mut self, points: f32) -> &mut Self {
        self.pair(Name(b"W"), points);
        self
    }

    /// Write the `/S` attribute.
    pub fn style(&mut self, style: BorderType) -> &mut Self {
        self.pair(Name(b"S"), style.to_name());
        self
    }

    /// Write the `/D` attribute to set the repeating lengths of dashes and gaps
    /// inbetween.
    pub fn dashes(&mut self, dash_pattern: impl IntoIterator<Item = f32>) -> &mut Self {
        self.key(Name(b"D")).array().typed().items(dash_pattern);
        self
    }
}

deref!('a, BorderStyle<'a> => Dict<'a>, dict);

/// The kind of line to draw on the border.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BorderType {
    /// A solid line.
    Solid,
    /// A dashed line, dash pattern may be specified further elsewhere.
    Dashed,
    /// A line with a 3D effect.
    Beveled,
    /// A line that makes the rectangle appear depressed.
    Inset,
    /// A single line at the bottom of the border rectangle.
    Underline,
}

impl BorderType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::Solid => Name(b"S"),
            Self::Dashed => Name(b"D"),
            Self::Beveled => Name(b"B"),
            Self::Inset => Name(b"I"),
            Self::Underline => Name(b"U"),
        }
    }
}

/// Writer for a _destination array_.
///
/// This struct is created by [`Destinations::push`].
pub struct Destination<'a> {
    array: Array<'a>,
}

impl<'a> Destination<'a> {
    pub(crate) fn start(obj: Obj<'a>, page: Ref) -> Self {
        let mut array = obj.array();
        array.item(page);
        Self { array }
    }

    /// Write the `/XYZ` command which skips to the specified coordinated. Leave
    /// `zoom` on `0` if you do not want to change the zoom level.
    pub fn xyz(mut self, left: f32, top: f32, zoom: f32) {
        self.item(Name(b"XYZ"));
        self.item(left);
        self.item(top);
        self.item(zoom);
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
    /// content on screen. (1.1+)
    pub fn fit_bounding_box(mut self) {
        self.item(Name(b"FitB"));
    }

    /// Write the `/FitBH` command which fits the referenced page's content to
    /// the screen width and skips to the specified offset. (1.1+)
    pub fn fit_bounding_box_horizontal(mut self, top: f32) {
        self.item(Name(b"FitBH"));
        self.item(top);
    }

    /// Write the `/FitBV` command which fits the referenced page's content to
    /// the screen height and skips to the specified offset. (1.1+)
    pub fn fit_bounding_box_vertical(mut self, left: f32) {
        self.item(Name(b"FitBV"));
        self.item(left);
    }
}

deref!('a, Destination<'a> => Array<'a>, array);

/// Writer for a _named destinations dictionary_.
///
/// This struct is created by [`PdfWriter::destinations`].
pub struct Destinations<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Destinations<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Start adding another named destination.
    pub fn add(&mut self, name: Name, page: Ref) -> Destination<'_> {
        Destination::start(self.key(name), page)
    }
}

deref!('a, Destinations<'a> => Dict<'a, IndirectGuard>, dict);

/// Writer for an _outline dictionary_.
///
/// This struct is created by [`PdfWriter::outline`].
pub struct Outline<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Outline<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Outlines"));
        Self { dict }
    }

    /// Write the `/First` attribute which points to the first item in the
    /// document's outline.
    pub fn first(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"First"), outline);
        self
    }

    /// Write the `/Last` attribute which points to the last item in the
    /// document's outline.
    pub fn last(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"Last"), outline);
        self
    }

    /// Write the `/Count` attribute. This tells the viewer how many outline
    /// elements (at all levels) are currently visible. The value must not be
    /// negative, otherwise, the function will panic.
    pub fn count(&mut self, items: i32) -> &mut Self {
        if items < 0 {
            panic!("negative values not allowed for `/Count` attribute")
        }

        self.pair(Name(b"Count"), items);
        self
    }
}

deref!('a, Outline<'a> => Dict<'a, IndirectGuard>, dict);

/// Writer for an _outline item dictionary_.
///
/// This struct is created by [`PdfWriter::outline_item`].
pub struct OutlineItem<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> OutlineItem<'a> {
    pub(crate) fn start(obj: Obj<'a, IndirectGuard>) -> Self {
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

    /// Write the `/First` attribute which points to the first item in the
    /// item's children.
    pub fn first(&mut self, outline: Ref) -> &mut Self {
        self.pair(Name(b"First"), outline);
        self
    }

    /// Write the `/Last` attribute which points to the last item in the
    /// item's children.
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

    /// Write the `/C` attribute using an RGB color. This sets the color in which
    /// the outline item's title should be rendered. (1.4+)
    pub fn color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.key(Name(b"C")).array().typed().items([r, g, b]);
        self
    }

    /// Write the `/F` attribute. (1.4+)
    pub fn flags(&mut self, flags: OutlineItemFlags) -> &mut Self {
        self.pair(Name(b"F"), flags.bits() as i32);
        self
    }
}

deref!('a, OutlineItem<'a> => Dict<'a, IndirectGuard>, dict);

bitflags::bitflags! {
    /// Bitflags describing the appearance of an outline item.
    pub struct OutlineItemFlags: u32 {
        /// This renders the outline item italicized.
        const ITALIC = 1 << 0;
        /// This renders the outline item emboldened.
        const BOLD = 1 << 1;
    }
}
