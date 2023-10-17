use super::*;

/// Writer for an _annotation dictionary_.
///
/// An array of this struct is created by [`Page::annotations`].
pub struct Annotation<'a> {
    dict: Dict<'a>,
}

writer!(Annotation: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Annot"));
    Self { dict }
});

impl<'a> Annotation<'a> {
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

    /// Write the `/NM` attribute. This uniquely identifies the annotation on the
    /// page. PDF 1.3+.
    pub fn name(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"NM"), text);
        self
    }

    /// Write the `/M` attribute, specifying the date the annotation was last
    /// modified. PDF 1.1+.
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
        dash_pattern: Option<&[f32]>,
    ) -> &mut Self {
        let mut array = self.insert(Name(b"Border")).array();
        array.item(h_radius);
        array.item(v_radius);
        array.item(width);

        if let Some(pattern) = dash_pattern {
            array.push().array().items(pattern);
        }

        array.finish();
        self
    }

    /// Start writing the `/BS` attribute. These are some more elaborate border
    /// settings taking precedence over `/B` for some annotation types. PDF 1.2+.
    pub fn border_style(&mut self) -> BorderStyle<'_> {
        self.insert(Name(b"BS")).start()
    }

    /// Write the `/C` attribute forcing a transparent color. This sets the
    /// annotations background color and its popup title bar color. PDF 1.1+.
    pub fn color_transparent(&mut self) -> &mut Self {
        self.insert(Name(b"C")).array();
        self
    }

    /// Write the `/C` attribute using a grayscale color. This sets the
    /// annotations background color and its popup title bar color. PDF 1.1+.
    pub fn color_gray(&mut self, gray: f32) -> &mut Self {
        self.insert(Name(b"C")).array().item(gray);
        self
    }

    /// Write the `/C` attribute using an RGB color. This sets the annotations
    /// background color and its popup title bar color. PDF 1.1+.
    pub fn color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.insert(Name(b"C")).array().items([r, g, b]);
        self
    }

    /// Write the `/C` attribute using a CMYK color. This sets the annotations
    /// background color and its popup title bar color. PDF 1.1+.
    pub fn color_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) -> &mut Self {
        self.insert(Name(b"C")).array().items([c, m, y, k]);
        self
    }

    /// Write the `/StructParent` attribute to indicate the [structure tree
    /// element][StructElement] this annotation belongs to. PDF 1.3+.
    pub fn struct_parent(&mut self, key: i32) -> &mut Self {
        self.pair(Name(b"StructParent"), key);
        self
    }

    /// Start writing the `/A` dictionary. Only permissible for the subtypes
    /// `Link` and `Widget`.
    pub fn action(&mut self) -> Action<'_> {
        self.insert(Name(b"A")).start()
    }

    /// Write the `/H` attribute to set what effect is used to convey that the
    /// user is pressing a link or widget annotation. Only permissible for the
    /// subtypes `Link` and `Widget`. PDF 1.2+.
    pub fn highlight(&mut self, effect: HighlightEffect) -> &mut Self {
        self.pair(Name(b"H"), effect.to_name());
        self
    }

    /// Write the `/T` attribute. This is in the title bar of markup annotations
    /// and should be the name of the annotation author. PDF 1.1+.
    pub fn author(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"T"), text);
        self
    }

    /// Write the `/Subj` attribute. This is the subject of the annotation.
    /// PDF 1.5+.
    pub fn subject(&mut self, text: TextStr) -> &mut Self {
        self.pair(Name(b"Subj"), text);
        self
    }

    /// Write the `/QuadPoints` attribute, specifying the region in which the
    /// link should be activated. PDF 1.6+.
    pub fn quad_points(
        &mut self,
        coordinates: impl IntoIterator<Item = f32>,
    ) -> &mut Self {
        self.insert(Name(b"QuadPoints")).array().items(coordinates);
        self
    }

    /// Write the `/L` attribute. This defines the start and end point of a
    /// line annotation
    pub fn line_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> &mut Self {
        self.insert(Name(b"L")).array().items([x1, y1, x2, y2]);
        self
    }

    /// Start writing the `/FS` attribute, setting which file to reference.
    pub fn file_spec(&mut self) -> FileSpec<'_> {
        self.insert(Name(b"FS")).start()
    }

    /// Write the `/Name` attribute. Refer to the specification to see which
    /// names are allowed for which annotation types.
    pub fn icon(&mut self, icon: AnnotationIcon) -> &mut Self {
        self.pair(Name(b"Name"), icon.to_name());
        self
    }

    /// Start writing the `/MK` dictionary. Only permissible for the subtype
    /// `Widget`.
    pub fn appearance(&mut self) -> Appearance<'_> {
        self.dict.insert(Name(b"MK")).start()
    }

    /// Write the `/Parent` attribute. Only permissible for the subtype
    /// `Widget`.
    pub fn parent(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), id);
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
    /// A line. PDF 1.3+.
    Line,
    /// A square. PDF 1.3+.
    Square,
    /// A circle. PDF 1.3+.
    Circle,
    /// Highlighting the text on the page. PDF 1.3+.
    Highlight,
    /// Underline the text on the page. PDF 1.3+.
    Underline,
    /// Squiggly underline of the text on the page. PDF 1.4+.
    Squiggly,
    /// Strike out the text on the page. PDF 1.3+.
    StrikeOut,
    /// A reference to another file. PDF 1.3+.
    FileAttachment,
    /// A widget annotation. PDF 1.2+.
    Widget,
}

impl AnnotationType {
    pub(crate) fn to_name(self) -> Name<'static> {
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
            Self::Widget => Name(b"Widget"),
        }
    }
}

/// Possible icons for an annotation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AnnotationIcon<'a> {
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
    /// A custom icon name.
    Custom(Name<'a>),
}

impl<'a> AnnotationIcon<'a> {
    pub(crate) fn to_name(self) -> Name<'a> {
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
            Self::Custom(name) => name,
        }
    }
}

bitflags::bitflags! {
    /// Bitflags describing various characteristics of annotations.
    pub struct AnnotationFlags: u32 {
        /// This will hide the annotation if the viewer does not recognize its
        /// subtype. Otherwise, it will be rendered as specified in its appearance
        /// stream.
        const INVISIBLE = 1 << 0;
        /// This hides the annotation from view and disallows interaction. PDF 1.2+.
        const HIDDEN = 1 << 1;
        /// Print the annotation. If not set, it will be always hidden on print.
        /// PDF 1.2+.
        const PRINT = 1 << 2;
        /// Do not zoom the annotation appearance if the document is zoomed in.
        /// PDF 1.3+.
        const NO_ZOOM = 1 << 3;
        /// Do not rotate the annotation appearance if the document is zoomed in.
        /// PDF 1.3+.
        const NO_ROTATE = 1 << 4;
        /// Do not view the annotation on screen. It may still show on print.
        /// PDF 1.3+.
        const NO_VIEW = 1 << 5;
        /// Do not allow interactions. PDF 1.3+.
        const READ_ONLY = 1 << 6;
        /// Do not allow the user to delete or reposition the annotation. Contents
        /// may still be changed. PDF 1.4+.
        const LOCKED = 1 << 7;
        /// Invert the interpretation of the `no_view` flag for certain events.
        /// PDF 1.5+.
        const TOGGLE_NO_VIEW = 1 << 8;
        /// Do not allow content changes. PDF 1.7+.
        const LOCKED_CONTENTS = 1 << 9;
    }
}

/// Writer for an _appearance dictionary_.
///
/// This struct is created by [`Annotation::appearance`].
pub struct Appearance<'a> {
    dict: Dict<'a>,
}

writer!(Appearance: |obj| Self { dict: obj.dict() });

impl<'a> Appearance<'a> {
    /// Write the `/R` attribute. This is the number of degrees the widget
    /// annotation should be rotated by counterclockwise relative to its page
    /// when displayed. This should be a multiple of 90.
    pub fn rotate(&mut self, degrees: i32) -> &mut Self {
        self.pair(Name(b"R"), degrees);
        self
    }

    /// Write the `/BC` attribute forcing a transparent color. This sets the
    /// widget annotation's border color.
    pub fn border_color_transparent(&mut self) -> &mut Self {
        self.insert(Name(b"BC")).array();
        self
    }

    /// Write the `/BC` attribute using a grayscale color. This sets the
    /// widget annotation's border color.
    pub fn border_color_gray(&mut self, gray: f32) -> &mut Self {
        self.insert(Name(b"BC")).array().item(gray);
        self
    }

    /// Write the `/BC` attribute using an RGB color. This sets the widget
    /// annotation's border color.
    pub fn border_color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.insert(Name(b"BC")).array().items([r, g, b]);
        self
    }

    /// Write the `/BC` attribute using an RGB color. This sets the widget
    /// annotation's border color.
    pub fn border_color_cymk(&mut self, c: f32, y: f32, m: f32, k: f32) -> &mut Self {
        self.insert(Name(b"BC")).array().items([c, y, m, k]);
        self
    }

    /// Write the `/BG` attribute forcing a transparent color. This sets the
    /// widget annotation's background color.
    pub fn background_color_transparent(&mut self) -> &mut Self {
        self.insert(Name(b"BG")).array();
        self
    }

    /// Write the `/BG` attribute using a grayscale color. This sets the
    /// widget annotation's backround color.
    pub fn background_color_gray(&mut self, gray: f32) -> &mut Self {
        self.insert(Name(b"BG")).array().item(gray);
        self
    }

    /// Write the `/BG` attribute using an RGB color. This sets the widget
    /// annotation's backround color.
    pub fn background_color_rgb(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.insert(Name(b"BG")).array().items([r, g, b]);
        self
    }

    /// Write the `/BG` attribute using an RGB color. This sets the widget
    /// annotation's backround color.
    pub fn background_color_cymk(&mut self, c: f32, y: f32, m: f32, k: f32) -> &mut Self {
        self.insert(Name(b"BG")).array().items([c, y, m, k]);
        self
    }

    /// Write the `/CA` attribute. This sets the widget annotation's normal
    /// caption. Only permissible for button fields.
    pub fn normal_caption(&mut self, caption: TextStr) -> &mut Self {
        self.pair(Name(b"CA"), caption);
        self
    }

    /// Write the `/RC` attribute. This sets the widget annotation's rollover
    /// (hover) caption. Only permissible for push button fields.
    pub fn rollover_caption(&mut self, caption: TextStr) -> &mut Self {
        self.pair(Name(b"RC"), caption);
        self
    }

    /// Write the `/AC` attribute. This sets the widget annotation's alternate
    /// (down) caption. Only permissible for push button fields.
    pub fn alterante_caption(&mut self, caption: TextStr) -> &mut Self {
        self.pair(Name(b"AC"), caption);
        self
    }

    /// Write the `/I` attribute. This sets the widget annotation's normal icon
    /// as a reference to a [`FormXObject`]. Only permissible for push button
    /// fields.
    pub fn normal_icon(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"I"), id);
        self
    }

    /// Write the `/RI` attribute. This sets the widget annotation's rollover
    /// (hover) icon as a reference to a [`FormXObject`]. Only permissible for
    /// push button fields.
    pub fn rollover_icon(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"RI"), id);
        self
    }

    /// Write the `/IX` attribute. This sets the widget annotation's alternate
    /// (down) icon as a reference to a [`FormXObject`]. Only permissible for
    /// push button fields.
    pub fn alternate_icon(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"IX"), id);
        self
    }

    /// Start writing the `/IF` dictonary. This sets the widget annotation's
    /// icon display characteristics. Only permissible for push button fields.
    pub fn icon_fit(&mut self) -> IconFit<'_> {
        self.insert(Name(b"IF")).start()
    }

    /// Write the `/TP` attribute. This sets the widget annotation's caption
    /// position relative to the annotation's icon. Only permissible for push
    /// button fields.
    pub fn text_position(&mut self, position: TextPosition) -> &mut Self {
        self.pair(Name(b"TP"), position as i32);
        self
    }
}

deref!('a, Appearance<'a> => Dict<'a>, dict);

/// The position the text of the widget annotation's caption relative to its
/// icon.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TextPosition {
    /// Hide icon, show only caption.
    CaptionOnly = 0,
    /// Hide caption, show only icon.
    IconOnly = 1,
    /// The caption should be placed below the icon.
    Below = 2,
    /// The caption should be placed above the icon.
    Above = 3,
    /// The caption should be placed to the right of the icon.
    Right = 4,
    /// The caption should be placed to the left of the icon.
    Left = 5,
    /// The caption should be placed overlaid directly on the icon.
    Overlaid = 6,
}

/// Writer for an _icon fit dictionary_.
///
/// This struct is created by [`Appearance::icon_fit`].
pub struct IconFit<'a> {
    dict: Dict<'a>,
}

writer!(IconFit: |obj| Self { dict: obj.dict() });

impl<'a> IconFit<'a> {
    /// Write the `/SW` attribute. This sets under which circumstances the icon
    /// of the widget annotation should be scaled.
    pub fn scale(&mut self, value: IconScale) -> &mut Self {
        self.pair(Name(b"SW"), value.to_name());
        self
    }

    /// Write the `/S` attribute. This sets the scaling type of this annoation.
    pub fn scale_type(&mut self, value: IconScaleType) -> &mut Self {
        self.pair(Name(b"S"), value.to_name());
        self
    }

    /// Write the `/A` attribute. This sets the widget annotation's leftover
    /// space if proportional scaling is applied given as fractions between
    /// `0.0` and `1.0`.
    pub fn leftover_space(&mut self, x: f32, y: f32) -> &mut Self {
        self.insert(Name(b"A")).array().items([x, y]);
        self
    }

    /// Wrtite the `/FB` attribute. This sets whether the border line width
    /// should be ignored when scaling the icon to fit the annotation bounds.
    /// PDF 1.5+.
    pub fn fit_bounds(&mut self, fit: bool) -> &mut Self {
        self.pair(Name(b"FB"), fit);
        self
    }
}

deref!('a, IconFit<'a> => Dict<'a>, dict);

/// How the icon in a push button field should be scaled.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum IconScale {
    /// Always scale the icon.
    Always,
    /// Scale the icon only when the icon is bigger than the annotation
    /// rectangle.
    Bigger,
    /// Scale the icon only when the icon is smaller than the annotation
    /// rectangle.
    Smaller,
    /// Never scale the icon.
    Never,
}

impl IconScale {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Always => Name(b"A"),
            Self::Bigger => Name(b"B"),
            Self::Smaller => Name(b"S"),
            Self::Never => Name(b"N"),
        }
    }
}

/// How the icon in a push button field should be scaled.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum IconScaleType {
    /// Scale the icon to fill the annotation rectangle exactly, without regard
    /// to its original aspect ratio (ratio of width to height).
    Anamorphic,
    /// Scale the icon to fit the width or height of the annotation rectangle
    /// while maintaining the icon’s original aspect ratio. If the required
    /// horizontal and vertical scaling factors are different, use the smaller
    /// of the two, centering the icon within the annotation rectangle in the
    /// other dimension.
    Proportional,
}

impl IconScaleType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Anamorphic => Name(b"A"),
            Self::Proportional => Name(b"P"),
        }
    }
}

/// Highlighting effect applied when a user holds the mouse button over an
/// annotation.
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
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::None => Name(b"N"),
            Self::Invert => Name(b"I"),
            Self::Outline => Name(b"O"),
            Self::Push => Name(b"P"),
        }
    }
}

/// Writer for an _border style dictionary_.
///
/// This struct is created by [`Annotation::border_style`].
pub struct BorderStyle<'a> {
    dict: Dict<'a>,
}

writer!(BorderStyle: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Border"));
    Self { dict }
});

impl<'a> BorderStyle<'a> {
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
    /// in between.
    pub fn dashes(&mut self, dash_pattern: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"D")).array().items(dash_pattern);
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
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Solid => Name(b"S"),
            Self::Dashed => Name(b"D"),
            Self::Beveled => Name(b"B"),
            Self::Inset => Name(b"I"),
            Self::Underline => Name(b"U"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotations() {
        test!(
            crate::tests::slice(|w| {
                let mut page = w.page(Ref::new(1));
                let mut annots = page.annotations();
                annots.push().rect(Rect::new(0.0, 0.0, 1.0, 1.0));
                annots.push().rect(Rect::new(1.0, 1.0, 0.0, 0.0));
                annots.finish();
                page.bleed_box(Rect::new(-100.0, -100.0, 100.0, 100.0));
            }),
            b"1 0 obj",
            b"<<",
            b"  /Type /Page",
            b"  /Annots [<<",
            b"    /Type /Annot",
            b"    /Rect [0 0 1 1]",
            b"  >> <<",
            b"    /Type /Annot",
            b"    /Rect [1 1 0 0]",
            b"  >>]",
            b"  /BleedBox [-100 -100 100 100]",
            b">>",
            b"endobj\n\n",
        );
    }
}
