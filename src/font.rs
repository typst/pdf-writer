use crate::buf::Buf;
use std::marker::PhantomData;

use super::*;

/// Writer for a _Type-1 font dictionary_.
///
/// This struct is created by [`Chunk::type1_font`].
pub struct Type1Font<'a> {
    dict: Dict<'a>,
}

writer!(Type1Font: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Font"));
    dict.pair(Name(b"Subtype"), Name(b"Type1"));
    Self { dict }
});

impl<'a> Type1Font<'a> {
    /// Write the `/Name` attribute, which is the name of the font in the
    /// current resource dictionary. Required in PDF 1.0, discouraged in PDF
    /// 1.1+.
    pub fn name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"Name"), name);
        self
    }

    /// Write the `/BaseFont` attribute. This is the PostScript name of the
    /// font. Required.
    ///
    /// In PDF/A files, the standard 14 fonts are unavailable, so you must
    /// embed the font data.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"BaseFont"), name);
        self
    }

    /// Write the `FirstChar` attribute, defining the first character code in
    /// the font's widths array. Required (except for standard 14 fonts
    /// before PDF 1.5).
    pub fn first_char(&mut self, first: u8) -> &mut Self {
        self.pair(Name(b"FirstChar"), i32::from(first));
        self
    }

    /// Write the `LastChar` attribute, defining the last character code in the
    /// font's widths array. Required (except for standard 14 fonts before
    /// PDF 1.5).
    pub fn last_char(&mut self, last: u8) -> &mut Self {
        self.pair(Name(b"LastChar"), i32::from(last));
        self
    }

    /// Write the `/Widths` array. Should be of length `last - first + 1`.
    /// Required (except for standard 14 fonts before PDF 1.5).
    pub fn widths(&mut self, widths: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Widths")).array().items(widths);
        self
    }

    /// Write the `/FontDescriptor` attribute. Required (except for standard 14
    /// fonts before PDF 1.5).
    pub fn font_descriptor(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontDescriptor"), id);
        self
    }

    /// Write the `/Encoding` attribute as a predefined encoding. Either this or
    /// [`encoding_custom`](Self::encoding_custom) is required.
    pub fn encoding_predefined(&mut self, encoding: Name) -> &mut Self {
        self.pair(Name(b"Encoding"), encoding);
        self
    }

    /// Start writing an `/Encoding` dictionary. Either this or
    /// [`encoding_predefined`](Self::encoding_predefined) is required.
    pub fn encoding_custom(&mut self) -> Encoding<'_> {
        self.insert(Name(b"Encoding")).start()
    }

    /// Write the `/ToUnicode` attribute. PDF 1.2+.
    ///
    /// A suitable character map can be built with [`UnicodeCmap`].
    pub fn to_unicode(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"ToUnicode"), id);
        self
    }
}

deref!('a, Type1Font<'a> => Dict<'a>, dict);

/// Writer for a _Type-3 font dictionary_.
///
/// This struct is created by [`Chunk::type3_font`].
pub struct Type3Font<'a> {
    dict: Dict<'a>,
}

writer!(Type3Font: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Font"));
    dict.pair(Name(b"Subtype"), Name(b"Type3"));
    Self { dict }
});

impl<'a> Type3Font<'a> {
    /// Write the `/Name` attribute, which is the name of the font in the
    /// current resource dictionary. Always required in PDF 1.0, required if
    /// `FontName` is set in child font descriptor.
    pub fn name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"Name"), name);
        self
    }

    /// Write the `/FontBBox` attribute. Required.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.pair(Name(b"FontBBox"), bbox);
        self
    }

    /// Write the `/FontMatrix` attribute, which defines the mapping from glyph
    /// space to text space. Required.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.insert(Name(b"FontMatrix")).array().items(matrix);
        self
    }

    /// Start writing the `/CharProcs` dictionary, which maps glyph names to
    /// glyph content streams. Required.
    ///
    /// Each glyph's content stream must start with either the
    /// [`d0`](crate::Content::start_color_glyph) or
    /// [`d1`](crate::Content::start_shape_glyph) operator.
    pub fn char_procs(&mut self) -> TypedDict<'_, Ref> {
        self.insert(Name(b"CharProcs")).dict().typed()
    }

    /// Write the `/Encoding` attribute as a predefined encoding. Either this or
    /// [`encoding_custom`](Self::encoding_custom) is required.
    pub fn encoding_predefined(&mut self, encoding: Name) -> &mut Self {
        self.pair(Name(b"Encoding"), encoding);
        self
    }

    /// Start writing an `/Encoding` dictionary. Either this or
    /// [`encoding_predefined`](Self::encoding_predefined) is required.
    pub fn encoding_custom(&mut self) -> Encoding<'_> {
        self.insert(Name(b"Encoding")).start()
    }

    /// Write the `FirstChar` attribute, defining the first character code in
    /// the font's widths array. Required.
    pub fn first_char(&mut self, first: u8) -> &mut Self {
        self.pair(Name(b"FirstChar"), i32::from(first));
        self
    }

    /// Write the `LastChar` attribute, defining the last character code in the
    /// font's widths array. Required.
    pub fn last_char(&mut self, last: u8) -> &mut Self {
        self.pair(Name(b"LastChar"), i32::from(last));
        self
    }

    /// Write the `/Widths` array. Should be of length `last - first + 1`.
    /// Required.
    pub fn widths(&mut self, widths: impl IntoIterator<Item = f32>) -> &mut Self {
        self.insert(Name(b"Widths")).array().items(widths);
        self
    }

    /// Write the `/FontDescriptor` attribute. Required in Tagged PDFs.
    pub fn font_descriptor(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontDescriptor"), id);
        self
    }

    /// Start writing the `/Resources` dictionary.
    pub fn resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"Resources")).start()
    }

    /// Write the `/ToUnicode` attribute. PDF 1.2+.
    ///
    /// A suitable character map can be built with [`UnicodeCmap`].
    ///
    /// This attribute is required in some profiles of PDF/A-2, PDF/A-3, and
    /// PDF/A-4 for some fonts. When present, these standards require that no
    /// character may be mapped to `0`, `U+FEFF`, or `U+FFFE`.
    pub fn to_unicode(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"ToUnicode"), id);
        self
    }
}

deref!('a, Type3Font<'a> => Dict<'a>, dict);

/// Writer for a _simple font encoding dictionary_.
///
/// This struct is created by [`Type1Font::encoding_custom`] and
/// [`Type3Font::encoding_custom`].
pub struct Encoding<'a> {
    dict: Dict<'a>,
}

writer!(Encoding: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Encoding"));
    Self { dict }
});

impl<'a> Encoding<'a> {
    /// Write the `BaseEncoding` attribute, from which this encoding is
    /// described through differences.
    pub fn base_encoding(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"BaseEncoding"), name);
        self
    }

    /// Start writing the `/Differences` array.
    pub fn differences(&mut self) -> Differences<'_> {
        self.insert(Name(b"Differences")).start()
    }
}

deref!('a, Encoding<'a> => Dict<'a>, dict);

/// Writer for an _encoding differences array_.
///
/// This struct is created by [`Encoding::differences`].
pub struct Differences<'a> {
    array: Array<'a>,
}

writer!(Differences: |obj| Self { array: obj.array() });

impl<'a> Differences<'a> {
    /// Maps consecutive character codes starting at `start` to the given glyph
    /// names.
    pub fn consecutive<'n>(
        &mut self,
        start: u8,
        names: impl IntoIterator<Item = Name<'n>>,
    ) -> &mut Self {
        self.item(i32::from(start));
        for name in names {
            self.item(name);
        }
        self
    }
}

deref!('a, Differences<'a> => Array<'a>, array);

/// Writer for a _Type-0 (composite) font dictionary_. PDF 1.2+.
///
/// This struct is created by [`Chunk::type0_font`].
pub struct Type0Font<'a> {
    dict: Dict<'a>,
}

writer!(Type0Font: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Font"));
    dict.pair(Name(b"Subtype"), Name(b"Type0"));
    Self { dict }
});

impl<'a> Type0Font<'a> {
    /// Write the `/BaseFont` attribute. This is the PostScript name of the
    /// font. Required.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"BaseFont"), name);
        self
    }

    /// Write the `/Encoding` attribute as a predefined encoding. Either this or
    /// [`encoding_cmap`](Self::encoding_cmap) is required.
    pub fn encoding_predefined(&mut self, encoding: Name) -> &mut Self {
        self.pair(Name(b"Encoding"), encoding);
        self
    }

    /// Write the `/Encoding` attribute as a reference to a character map.
    /// Either this or [`encoding_predefined`](Self::encoding_predefined) is
    /// required.
    pub fn encoding_cmap(&mut self, cmap: Ref) -> &mut Self {
        self.pair(Name(b"Encoding"), cmap);
        self
    }

    /// Write the `/DescendantFonts` attribute as a one-element array containing
    /// a reference to a [`CidFont`]. Required.
    pub fn descendant_font(&mut self, cid_font: Ref) -> &mut Self {
        self.insert(Name(b"DescendantFonts")).array().item(cid_font);
        self
    }

    /// Write the `/ToUnicode` attribute.
    ///
    /// A suitable character map can be built with [`UnicodeCmap`].
    pub fn to_unicode(&mut self, cmap: Ref) -> &mut Self {
        self.pair(Name(b"ToUnicode"), cmap);
        self
    }
}

deref!('a, Type0Font<'a> => Dict<'a>, dict);

/// Writer for a _CID font dictionary_. PDF 1.2+.
///
/// This struct is created by [`Chunk::cid_font`].
pub struct CidFont<'a> {
    dict: Dict<'a>,
}

writer!(CidFont: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Font"));
    Self { dict }
});

impl<'a> CidFont<'a> {
    /// Write the `/Subtype` attribute. Required.
    pub fn subtype(&mut self, subtype: CidFontType) -> &mut Self {
        self.pair(Name(b"Subtype"), subtype.to_name());
        self
    }

    /// Write the `/BaseFont` attribute. Required.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"BaseFont"), name);
        self
    }

    /// Write the `/CIDSystemInfo` dictionary. Required.
    pub fn system_info(&mut self, info: SystemInfo) -> &mut Self {
        info.write(self.insert(Name(b"CIDSystemInfo")));
        self
    }

    /// Write the `/FontDescriptor` attribute. Required.
    pub fn font_descriptor(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontDescriptor"), id);
        self
    }

    /// Write the `/DW` attribute, specifying the default glyph width.
    pub fn default_width(&mut self, width: f32) -> &mut Self {
        self.pair(Name(b"DW"), width);
        self
    }

    /// Start writing the `/W` (widths) array.
    pub fn widths(&mut self) -> Widths<'_> {
        self.insert(Name(b"W")).start()
    }

    /// Write the `/CIDToGIDMap` attribute as a predefined name.
    ///
    /// The attribute must be present for PDF/A. The only permissible predefined
    /// name is `Identity`, otherwise [`Self::cid_to_gid_map_stream`] must be
    /// used.
    pub fn cid_to_gid_map_predefined(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"CIDToGIDMap"), name);
        self
    }

    /// Write the `/CIDToGIDMap` attribute as a reference to a stream, whose
    /// bytes directly map from CIDs to glyph indices.
    ///
    /// The attribute must be present for PDF/A.
    pub fn cid_to_gid_map_stream(&mut self, stream: Ref) -> &mut Self {
        self.pair(Name(b"CIDToGIDMap"), stream);
        self
    }
}

deref!('a, CidFont<'a> => Dict<'a>, dict);

/// The subtype of a CID font.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CidFontType {
    /// A CID font containing CFF glyph descriptions.
    Type0,
    /// A CID font containing TrueType glyph descriptions.
    Type2,
}

impl CidFontType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Type0 => Name(b"CIDFontType0"),
            Self::Type2 => Name(b"CIDFontType2"),
        }
    }
}

/// Writer for a _CID font widths array_.
///
/// This struct is created by [`CidFont::widths`].
pub struct Widths<'a> {
    array: Array<'a>,
}

writer!(Widths: |obj| Self { array: obj.array() });

impl<'a> Widths<'a> {
    /// Specifies individual widths for a range of consecutive CIDs starting at
    /// `start`.
    pub fn consecutive(
        &mut self,
        start: u16,
        widths: impl IntoIterator<Item = f32>,
    ) -> &mut Self {
        self.item(i32::from(start));
        self.push().array().items(widths);
        self
    }

    /// Specifies the same width for all CIDs between `first` and `last`.
    pub fn same(&mut self, first: u16, last: u16, width: f32) -> &mut Self {
        self.item(i32::from(first));
        self.item(i32::from(last));
        self.item(width);
        self
    }
}

deref!('a, Widths<'a> => Array<'a>, array);

/// Writer for a _font descriptor dictionary_.
///
/// This struct is created by [`Chunk::font_descriptor`].
pub struct FontDescriptor<'a> {
    dict: Dict<'a>,
}

writer!(FontDescriptor: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"FontDescriptor"));
    Self { dict }
});

impl<'a> FontDescriptor<'a> {
    /// Write the `/FontName` attribute. Required, except for Type 3 fonts.
    pub fn name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"FontName"), name);
        self
    }

    /// Write the `/FontFamily` attribute. Recommended for Type 3 fonts in
    /// Tagged PDFs. PDF 1.5+.
    pub fn family(&mut self, family: Str) -> &mut Self {
        self.pair(Name(b"FontFamily"), family);
        self
    }

    /// Write the `/FontStretch` attribute. Recommended for Type 3 fonts in
    /// Tagged PDFs. PDF 1.5+.
    pub fn stretch(&mut self, stretch: FontStretch) -> &mut Self {
        self.pair(Name(b"FontStretch"), stretch.to_name());
        self
    }

    /// Write the `/FontWeight` attribute. Should be between 100 (lightest) and
    /// 900 (heaviest), 400 is normal weight, 700 is bold. Recommended
    /// for Type 3 fonts in Tagged PDFs. PDF 1.5+.
    pub fn weight(&mut self, weight: u16) -> &mut Self {
        self.pair(Name(b"FontWeight"), i32::from(weight));
        self
    }

    /// Write the `/Flags` attribute. Required.
    pub fn flags(&mut self, flags: FontFlags) -> &mut Self {
        self.pair(Name(b"Flags"), flags.bits() as i32);
        self
    }

    /// Write the `/FontBBox` attribute. Required, except for Type 3 fonts.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.pair(Name(b"FontBBox"), bbox);
        self
    }

    /// Write the `/ItalicAngle` attribute. Required.
    pub fn italic_angle(&mut self, angle: f32) -> &mut Self {
        self.pair(Name(b"ItalicAngle"), angle);
        self
    }

    /// Write the `/Ascent` attribute. Required.
    pub fn ascent(&mut self, ascent: f32) -> &mut Self {
        self.pair(Name(b"Ascent"), ascent);
        self
    }

    /// Write the `/Descent` attribute. Required.
    pub fn descent(&mut self, descent: f32) -> &mut Self {
        self.pair(Name(b"Descent"), descent);
        self
    }

    /// Write the `/Leading` attribute.
    pub fn leading(&mut self, leading: f32) -> &mut Self {
        self.pair(Name(b"Leading"), leading);
        self
    }

    /// Write the `/CapHeight` attribute. Required for fonts with Latin
    /// characters, except for Type 3 fonts.
    pub fn cap_height(&mut self, cap_height: f32) -> &mut Self {
        self.pair(Name(b"CapHeight"), cap_height);
        self
    }

    /// Write the `/XHeight` attribute.
    pub fn x_height(&mut self, x_height: f32) -> &mut Self {
        self.pair(Name(b"XHeight"), x_height);
        self
    }

    /// Write the `/StemV` attribute. Required, except for Type 3 fonts.
    pub fn stem_v(&mut self, stem_v: f32) -> &mut Self {
        self.pair(Name(b"StemV"), stem_v);
        self
    }

    /// Write the `/StemH` attribute.
    pub fn stem_h(&mut self, stem_h: f32) -> &mut Self {
        self.pair(Name(b"StemH"), stem_h);
        self
    }

    /// Write the `/AvgWidth` attribute.
    pub fn avg_width(&mut self, avg_width: f32) -> &mut Self {
        self.pair(Name(b"AvgWidth"), avg_width);
        self
    }

    /// Write the `/MaxWidth` attribute.
    pub fn max_width(&mut self, max_width: f32) -> &mut Self {
        self.pair(Name(b"MaxWidth"), max_width);
        self
    }

    /// Write the `/MissingWidth` attribute.
    pub fn missing_width(&mut self, missing_width: f32) -> &mut Self {
        self.pair(Name(b"MissingWidth"), missing_width);
        self
    }

    /// Write the `/FontFile` attribute, referecing Type 1 font data.
    pub fn font_file(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontFile"), id);
        self
    }

    /// Write the `/FontFile2` attribute, referencing TrueType font data. PDF
    /// 1.1+.
    pub fn font_file2(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontFile2"), id);
        self
    }

    /// Write the `/FontFile3` attribute, referencing CFF font data. PDF 1.2+ or
    /// PDF 1.3+ for CID-keyed fonts.
    pub fn font_file3(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"FontFile3"), id);
        self
    }

    /// Write the `/CharSet` attribute, encoding the character names of a font
    /// subset as a string. This is only relevant for Type 1 fonts. PDF 1.1+.
    ///
    /// If present in PDF/A, this must include all characters in the subset,
    /// even if they are not used in the document.
    pub fn char_set(&mut self, names: Str) -> &mut Self {
        self.pair(Name(b"CharSet"), names);
        self
    }
}

/// Additional `FontDescriptor` attributes for CIDFonts.
impl FontDescriptor<'_> {
    /// Write the `/Style` attribute. Optional.
    ///
    /// The class and subclass values should be extracted from the
    /// `sFamilyClass` field of the OS/2 table. The `panose` array should be
    /// extracted from the `panose` field of the OS/2 table.
    pub fn style(&mut self, class: u8, subclass: u8, panose: [u8; 10]) -> &mut Self {
        let mut bytes = [0; 12];
        bytes[0] = class;
        bytes[1] = subclass;
        bytes[2..].copy_from_slice(&panose);
        self.insert(Name(b"Style")).dict().pair(Name(b"Panose"), Str(&bytes));
        self
    }

    /// Start writing the `/FD` attribute. Optional.
    ///
    /// Overrides the global font descriptor for specific glyphs.
    pub fn descriptor_override(&mut self) -> FontDescriptorOverride<'_> {
        self.insert(Name(b"FD")).start()
    }

    /// Write the `/CIDSet` attribute.
    ///
    /// If present in PDF/A, this must include all characters in the subset,
    /// even if they are not used in the document.
    pub fn cid_set(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"CIDSet"), id);
        self
    }
}

deref!('a, FontDescriptor<'a> => Dict<'a>, dict);

/// Writer for a _font descriptor override dictionary_.
///
/// This struct is created by [`FontDescriptor::descriptor_override`].
///
/// The font descriptor dictionaries within can only contain metrics data.
pub struct FontDescriptorOverride<'a> {
    dict: Dict<'a>,
}

writer!(FontDescriptorOverride: |obj| {
    Self { dict: obj.dict() }
});

impl FontDescriptorOverride<'_> {
    /// Start writing a `FontDescriptor` dictionary for a custom character
    /// class.
    pub fn custom_class(&mut self, class: Name) -> FontDescriptor<'_> {
        self.insert(class).start()
    }

    /// Write a class override referencing a `FontDescriptor` dictionary
    /// indirectly.
    pub fn custom_class_ref(&mut self, class: Name, id: Ref) -> &mut Self {
        self.pair(class, id);
        self
    }

    /// Start writing a `FontDescriptor` dictionary for a predefined CJK class.
    pub fn cjk_class(&mut self, class: CjkClass) -> FontDescriptor<'_> {
        self.insert(class.to_name()).start()
    }

    /// Write a class override for a predefined CJK class referencing a
    /// `FontDescriptor` dictionary indirectly.
    pub fn cjk_class_ref(&mut self, class: CjkClass, id: Ref) -> &mut Self {
        self.pair(class.to_name(), id);
        self
    }
}

deref!('a, FontDescriptorOverride<'a> => Dict<'a>, dict);

/// Glyph classes for CJK fonts as defined in ISO 32000-1:2008.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum CjkClass {
    Alphabetic,
    AlphaNum,
    Dingbats,
    DingbatsRot,
    Generic,
    GenericRot,
    Hangul,
    Hanja,
    Hanzi,
    HRoman,
    HRomanRot,
    HKana,
    HKanaRot,
    HojoKanji,
    Kana,
    Kanji,
    Proportional,
    ProportionalRot,
    Ruby,
}

impl CjkClass {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Alphabetic => Name(b"Alphabetic"),
            Self::AlphaNum => Name(b"AlphaNum"),
            Self::Dingbats => Name(b"Dingbats"),
            Self::DingbatsRot => Name(b"DingbatsRot"),
            Self::Generic => Name(b"Generic"),
            Self::GenericRot => Name(b"GenericRot"),
            Self::Hangul => Name(b"Hangul"),
            Self::Hanja => Name(b"Hanja"),
            Self::Hanzi => Name(b"Hanzi"),
            Self::HRoman => Name(b"HRoman"),
            Self::HRomanRot => Name(b"HRomanRot"),
            Self::HKana => Name(b"HKana"),
            Self::HKanaRot => Name(b"HKanaRot"),
            Self::HojoKanji => Name(b"HojoKanji"),
            Self::Kana => Name(b"Kana"),
            Self::Kanji => Name(b"Kanji"),
            Self::Proportional => Name(b"Proportional"),
            Self::ProportionalRot => Name(b"ProportionalRot"),
            Self::Ruby => Name(b"Ruby"),
        }
    }
}

/// The width of a font's glyphs.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretch {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::UltraCondensed => Name(b"UltraCondensed"),
            Self::ExtraCondensed => Name(b"ExtraCondensed"),
            Self::Condensed => Name(b"Condensed"),
            Self::SemiCondensed => Name(b"SemiCondensed"),
            Self::Normal => Name(b"Normal"),
            Self::SemiExpanded => Name(b"SemiExpanded"),
            Self::Expanded => Name(b"Expanded"),
            Self::ExtraExpanded => Name(b"ExtraExpanded"),
            Self::UltraExpanded => Name(b"UltraExpanded"),
        }
    }
}

bitflags::bitflags! {
    /// Bitflags describing various characteristics of fonts.
    pub struct FontFlags: u32 {
        /// All glyphs have the same width.
        const FIXED_PITCH = 1 << 0;
        /// Glyphs have short strokes at their stems.
        const SERIF = 1 << 1;
        /// The font contains glyphs not in the Adobe standard Latin character
        /// set.
        const SYMBOLIC = 1 << 2;
        /// The glyphs resemeble cursive handwritiwng.
        const SCRIPT = 1 << 3;
        /// The font only uses glyphs in the Adobe standard Latin character set.
        const NON_SYMBOLIC = 1 << 5;
        /// The glyphs are slanted to the right.
        const ITALIC = 1 << 6;
        /// The font does not contain lowercase letters.
        const ALL_CAP = 1 << 16;
        /// The font's lowercase letters are similar to the uppercase ones, but
        /// smaller.
        const SMALL_CAP = 1 << 17;
        /// Ensures that bold glyphs are painted with more pixels than normal
        /// glyphs even at very small sizes.
        const FORCE_BOLD = 1 << 18;
    }
}

/// Writer for a _character map stream_.
///
/// This struct is created by [`Chunk::cmap`].
pub struct Cmap<'a> {
    stream: Stream<'a>,
}

impl<'a> Cmap<'a> {
    /// Create a new character map writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"CMap"));
        Self { stream }
    }

    /// Write the `/CMapName` attribute. Required.
    pub fn name(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"CMapName"), name);
        self
    }

    /// Write the `/CIDSystemInfo` attribute. Required.
    pub fn system_info(&mut self, info: SystemInfo) -> &mut Self {
        info.write(self.insert(Name(b"CIDSystemInfo")));
        self
    }

    /// Write the `/WMode` attribute. Optional.
    ///
    /// This describes whether the CMap applies to a font with horizontal or
    /// vertical writing mode. The default is whatever is specified in the CMap.
    ///
    /// This is required in PDF/A and must match the writing mode of the
    /// embedded CMap.
    pub fn writing_mode(&mut self, mode: WMode) -> &mut Self {
        self.pair(Name(b"WMode"), mode.to_int());
        self
    }

    /// Write the `/UseCMap` attribute using a stream reference. Optional.
    ///
    /// This allows specifying a base CMap to extend.
    ///
    /// Note that this attribute is restricted in PDF/A and may only be used
    /// with the well-known CMap names from the PDF standard. Use
    /// [`Self::use_cmap_predefined`] to specify a predefined name.
    pub fn use_cmap_stream(&mut self, cmap: Ref) -> &mut Self {
        self.pair(Name(b"UseCMap"), cmap);
        self
    }

    /// Write the `/UseCMap` attribute using a predefined name. Optional.
    ///
    /// This allows specifying a base CMap to extend.
    ///
    /// Note that this attribute is restricted in PDF/A.
    pub fn use_cmap_predefined(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"UseCMap"), name);
        self
    }
}

deref!('a, Cmap<'a> => Stream<'a>, stream);

/// The writing mode of a character map.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum WMode {
    Horizontal,
    Vertical,
}

impl WMode {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Horizontal => 0,
            Self::Vertical => 1,
        }
    }
}

/// A builder for a `/ToUnicode` character map stream.
pub struct UnicodeCmap<G = u16> {
    buf: Buf,
    mappings: Buf,
    count: i32,
    glyph_id: PhantomData<G>,
}

impl<G> UnicodeCmap<G>
where
    G: GlyphId,
{
    /// Create a new, empty unicode character map for a horizontal writing mode
    /// font.
    pub fn new(name: Name, info: SystemInfo) -> Self {
        Self::with_writing_mode(name, info, WMode::Horizontal)
    }

    /// Create a new, empty unicode character map while specifying the writing
    /// mode.
    pub fn with_writing_mode(name: Name, info: SystemInfo, mode: WMode) -> Self {
        // https://www.adobe.com/content/dam/acom/en/devnet/font/pdfs/5014.CIDFont_Spec.pdf

        let mut buf = Buf::new();

        // Static header.
        buf.extend_slice(b"%!PS-Adobe-3.0 Resource-CMap\n");
        buf.extend_slice(b"%%DocumentNeededResources: procset CIDInit\n");
        buf.extend_slice(b"%%IncludeResource: procset CIDInit\n");

        // Dynamic header.
        buf.extend_slice(b"%%BeginResource: CMap ");
        buf.extend_slice(name.0);
        buf.push(b'\n');
        buf.extend_slice(b"%%Title: (");
        buf.extend_slice(name.0);
        buf.push(b' ');
        buf.extend_slice(info.registry.0);
        buf.push(b' ');
        buf.extend_slice(info.ordering.0);
        buf.push(b' ');
        buf.push_int(info.supplement);
        buf.extend_slice(b")\n");
        buf.extend_slice(b"%%Version: 1\n");
        buf.extend_slice(b"%%EndComments\n");

        // General body.
        buf.extend_slice(b"/CIDInit /ProcSet findresource begin\n");
        buf.extend_slice(b"12 dict begin\n");
        buf.extend_slice(b"begincmap\n");
        buf.extend_slice(b"/CIDSystemInfo 3 dict dup begin\n");
        buf.extend_slice(b"    /Registry ");
        buf.push_val(info.registry);
        buf.extend_slice(b" def\n");
        buf.extend_slice(b"    /Ordering ");
        buf.push_val(info.ordering);
        buf.extend_slice(b" def\n");
        buf.extend_slice(b"    /Supplement ");
        buf.push_val(info.supplement);
        buf.extend_slice(b" def\n");
        buf.extend_slice(b"end def\n");
        buf.extend_slice(b"/CMapName ");
        buf.push_val(name);
        buf.extend_slice(b" def\n");
        buf.extend_slice(b"/CMapVersion 1 def\n");
        buf.extend_slice(b"/CMapType 0 def\n");
        buf.extend_slice(b"/WMode ");
        buf.push_int(mode.to_int());
        buf.extend_slice(b" def\n");

        // We just cover the whole unicode codespace.
        buf.extend_slice(b"1 begincodespacerange\n");
        buf.push(b'<');
        G::MIN.push(&mut buf);
        buf.extend_slice(b"> <");
        G::MAX.push(&mut buf);
        buf.extend_slice(b">\n");
        buf.extend_slice(b"endcodespacerange\n");

        Self {
            buf,
            mappings: Buf::new(),
            count: 0,
            glyph_id: PhantomData,
        }
    }

    /// Add a mapping from a glyph ID to a codepoint.
    pub fn pair(&mut self, glyph: G, codepoint: char) {
        self.pair_with_multiple(glyph, [codepoint]);
    }

    /// Add a mapping from a glyph ID to multiple codepoints.
    pub fn pair_with_multiple(
        &mut self,
        glyph: G,
        codepoints: impl IntoIterator<Item = char>,
    ) {
        self.mappings.push(b'<');
        glyph.push(&mut self.mappings);
        self.mappings.extend_slice(b"> <");

        for c in codepoints {
            for &mut part in c.encode_utf16(&mut [0; 2]) {
                self.mappings.push_hex_u16(part);
            }
        }

        self.mappings.extend_slice(b">\n");
        self.count += 1;

        // At most 100 lines per range.
        if self.count >= 100 {
            self.flush_range();
        }
    }

    /// Finish building the character map.
    pub fn finish(mut self) -> Buf {
        // Flush the in-progress range.
        self.flush_range();

        // End of body.
        self.buf.extend_slice(b"endcmap\n");
        self.buf
            .extend_slice(b"CMapName currentdict /CMap defineresource pop\n");
        self.buf.extend_slice(b"end\n");
        self.buf.extend_slice(b"end\n");
        self.buf.extend_slice(b"%%EndResource\n");
        self.buf.extend_slice(b"%%EOF");

        self.buf
    }

    fn flush_range(&mut self) {
        if self.count > 0 {
            self.buf.push_int(self.count);
            self.buf.extend_slice(b" beginbfchar\n");
            self.buf.extend_slice(&self.mappings);
            self.buf.extend_slice(b"endbfchar\n");
        }

        self.count = 0;
        self.mappings.inner.clear();
    }
}

/// Type3 fonts require (in Acrobat at least) IDs in CMaps to be encoded with
/// one byte only, whereas other font types use two bytes.
///
/// This trait provides an abstraction to support both.
pub trait GlyphId: private::Sealed {}

impl GlyphId for u8 {}

impl GlyphId for u16 {}

/// Module to seal the `GlyphId` trait.
mod private {
    use crate::buf::Buf;

    pub trait Sealed {
        const MIN: Self;
        const MAX: Self;
        fn push(self, buf: &mut Buf);
    }

    impl Sealed for u8 {
        const MIN: Self = u8::MIN;
        const MAX: Self = u8::MAX;

        fn push(self, buf: &mut Buf) {
            buf.push_hex(self);
        }
    }

    impl Sealed for u16 {
        const MIN: Self = u16::MIN;
        const MAX: Self = u16::MAX;

        fn push(self, buf: &mut Buf) {
            buf.push_hex_u16(self);
        }
    }
}

/// Specifics about a character collection.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemInfo<'a> {
    /// The issuer of the collection.
    pub registry: Str<'a>,
    /// A unique name of the collection within the registry.
    pub ordering: Str<'a>,
    /// The supplement number (i.e. the version).
    pub supplement: i32,
}

impl SystemInfo<'_> {
    fn write(&self, obj: Obj<'_>) {
        obj.dict()
            .pair(Name(b"Registry"), self.registry)
            .pair(Name(b"Ordering"), self.ordering)
            .pair(Name(b"Supplement"), self.supplement);
    }
}
