use std::io::{Cursor, Write};
use std::num::NonZeroU16;

use super::*;
use crate::color::SeparationInfo;

/// Writer for a _document catalog dictionary_.
///
/// This struct is created by [`Pdf::catalog`].
pub struct Catalog<'a> {
    dict: Dict<'a>,
}

writer!(Catalog: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Catalog"));
    Self { dict }
});

impl Catalog<'_> {
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

    /// Start writing the `/PageLabels` number tree. PDF 1.3+.
    pub fn page_labels(&mut self) -> NumberTree<'_, Ref> {
        self.insert(Name(b"PageLabels")).start()
    }

    /// Write the `/PageMode` attribute to set which chrome elements the viewer
    /// should show.
    pub fn page_mode(&mut self, mode: PageMode) -> &mut Self {
        self.pair(Name(b"PageMode"), mode.to_name());
        self
    }

    /// Start writing the `/ViewerPreferences` dictionary. PDF 1.2+.
    pub fn viewer_preferences(&mut self) -> ViewerPreferences<'_> {
        self.insert(Name(b"ViewerPreferences")).start()
    }

    /// Start writing the `/Names` dictionary. PDF 1.2+.
    pub fn names(&mut self) -> Names<'_> {
        self.insert(Name(b"Names")).start()
    }

    /// Write the `/Dests` attribute pointing to a
    /// [named destinations dictionary](Chunk::destinations). PDF 1.1+.
    pub fn destinations(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Dests"), id);
        self
    }

    /// Write the `/Outlines` attribute pointing to the root
    /// [outline dictionary](Outline).
    pub fn outlines(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Outlines"), id);
        self
    }

    /// Start writing the `/StructTreeRoot` attribute to specify the root of the
    /// document's structure tree. PDF 1.3+.
    ///
    /// Must be present in some PDF/A profiles like PDF/A-2a.
    pub fn struct_tree_root(&mut self) -> StructTreeRoot<'_> {
        self.insert(Name(b"StructTreeRoot")).start()
    }

    /// Start writing the `/MarkInfo` dictionary to specify this document's
    /// conformance with the tagged PDF specification. PDF 1.4+.
    ///
    /// Must be present in some PDF/A profiles like PDF/A-2a.
    pub fn mark_info(&mut self) -> MarkInfo<'_> {
        self.insert(Name(b"MarkInfo")).start()
    }

    /// Write the `/Lang` attribute to specify the language of the document as a
    /// RFC 3066 language tag. PDF 1.4+.
    ///
    /// Required in some PDF/A profiles like PDF/A-2a.
    pub fn lang(&mut self, lang: TextStr) -> &mut Self {
        self.pair(Name(b"Lang"), lang);
        self
    }

    /// Write the `/Version` attribute to override the PDF version stated in the
    /// header. PDF 1.4+.
    pub fn version(&mut self, major: u8, minor: u8) -> &mut Self {
        self.pair(Name(b"Version"), Name(format!("{major}.{minor}").as_bytes()));
        self
    }

    /// Start writing the `/AA` dictionary. This sets the additional actions for
    /// the whole document. PDF 1.4+.
    ///
    /// Note that this attribute is forbidden in PDF/A.
    pub fn additional_actions(&mut self) -> AdditionalActions<'_> {
        self.insert(Name(b"AA")).start()
    }

    /// Start writing the `/AcroForm` dictionary to specify the document wide
    /// form. PDF 1.2+.
    pub fn form(&mut self) -> Form<'_> {
        self.insert(Name(b"AcroForm")).start()
    }

    /// Write the `/Metadata` attribute to specify the document's metadata. PDF
    /// 1.4+.
    ///
    /// The reference shall point to a [metadata stream](Metadata).
    pub fn metadata(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), id);
        self
    }

    /// Start writing the `/Extensions` dictionary to specify which PDF
    /// extensions are in use in the document. PDF 1.5+.
    ///
    /// The dictionary maps a vendor name to an extension dictionary. The Adobe
    /// PDF extensions use the Name prefix `ADBE`.
    pub fn extensions(&mut self) -> TypedDict<'_, DeveloperExtension> {
        self.insert(Name(b"Extensions")).dict().typed()
    }

    /// Start writing the `/SeparationInfo` dictionary to specify which
    /// separation colors are in use on the page and how it relates to other
    /// pages in the document. PDF 1.3+.
    pub fn separation_info(&mut self) -> SeparationInfo<'_> {
        self.insert(Name(b"SeparationInfo")).start()
    }

    /// Start writing the `/OutputIntents` array to specify the output
    /// destinations for the document. PDF 1.4+.
    ///
    /// Each entry in the array is an [output intent
    /// dictionary.](writers::OutputIntent)
    ///
    /// Must be present in PDF/X documents, encouraged in PDF/A documents.
    pub fn output_intents(&mut self) -> TypedArray<'_, OutputIntent> {
        self.insert(Name(b"OutputIntents")).array().typed()
    }

    /// Start writing the `/AF` array to specify the associated files of the
    /// document. PDF 2.0+ or PDF/A-3.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.insert(Name(b"AF")).array().typed()
    }
}

deref!('a, Catalog<'a> => Dict<'a>, dict);

/// How the viewer should lay out the pages in the document.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PageLayout {
    /// Only a single page at a time.
    SinglePage,
    /// A single, continuously scrolling column of pages.
    OneColumn,
    /// Two continuously scrolling columns of pages, laid out with odd-numbered
    /// pages on the left.
    TwoColumnLeft,
    /// Two continuously scrolling columns of pages, laid out with odd-numbered
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
    /// Show the optional content group panel. PDF 1.5+.
    UseOC,
    /// Show the attachments panel. PDF 1.6+.
    UseAttachments,
}

impl PageMode {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::UseNone => Name(b"UseNone"),
            Self::UseOutlines => Name(b"UseOutlines"),
            Self::UseThumbs => Name(b"UseThumbs"),
            Self::FullScreen => Name(b"FullScreen"),
            Self::UseOC => Name(b"UseOC"),
            Self::UseAttachments => Name(b"UseAttachments"),
        }
    }
}

/// Writer for a _developer extension dictionary_. PDF 1.7+.
///
/// An array of this struct is created by [`Catalog::extensions`].
pub struct DeveloperExtension<'a> {
    dict: Dict<'a>,
}

writer!(DeveloperExtension: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"DeveloperExtensions"));
    Self { dict }
});

impl DeveloperExtension<'_> {
    /// Write the `/BaseVersion` attribute to specify the version of PDF this
    /// extension is based on. Required.
    pub fn base_version(&mut self, major: u8, minor: u8) -> &mut Self {
        self.pair(Name(b"BaseVersion"), Name(format!("{major}.{minor}").as_bytes()));
        self
    }

    /// Write the `/ExtensionLevel` attribute to specify the version of the
    /// extension. Required.
    pub fn extension_level(&mut self, level: i32) -> &mut Self {
        self.pair(Name(b"ExtensionLevel"), level);
        self
    }
}

deref!('a, DeveloperExtension<'a> => Dict<'a>, dict);

/// Writer for a _viewer preference dictionary_.
///
/// This struct is created by [`Catalog::viewer_preferences`].
pub struct ViewerPreferences<'a> {
    dict: Dict<'a>,
}

writer!(ViewerPreferences: |obj| Self { dict: obj.dict() });

impl ViewerPreferences<'_> {
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

    /// Write the `/DisplayDocTitle` attribute to set whether the viewer should
    /// display the document's title from the `Title` entry as the window's title.
    /// PDF 1.4+
    pub fn display_doc_title(&mut self, display: bool) -> &mut Self {
        self.pair(Name(b"DisplayDocTitle"), display);
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

/// Writer for a _structure tree root dictionary_. PDF 1.3+
///
/// This struct is created by [`Catalog::struct_tree_root`].
pub struct StructTreeRoot<'a> {
    dict: Dict<'a>,
}

writer!(StructTreeRoot: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"StructTreeRoot"));
    Self { dict }
});

impl StructTreeRoot<'_> {
    /// Write the `/K` attribute to reference the immediate child of this
    /// element.
    pub fn child(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"K"), id);
        self
    }

    /// Start writing the `/K` attribute to reference the immediate children of
    /// this element.
    pub fn children(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"K")).array().typed()
    }

    /// Start writing the `/IDTree` attribute to map element identifiers to
    /// their corresponding structure element objects. Required if any elements
    /// have element identifiers.
    pub fn id_tree(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"IDTree")).start()
    }

    /// Start writing the `/ParentTree` attribute to maps structure elements to
    /// the content items they belong to. Required if any structure elements
    /// contain content items.
    pub fn parent_tree(&mut self) -> NumberTree<'_, Ref> {
        self.dict.insert(Name(b"ParentTree")).start()
    }

    /// Write the `/ParentTreeNextKey` attribute to specify the next available key
    /// for the `/ParentTree` dictionary.
    pub fn parent_tree_next_key(&mut self, key: i32) -> &mut Self {
        self.dict.pair(Name(b"ParentTreeNextKey"), key);
        self
    }

    /// Start writing the `/RoleMap` attribute to map structure element names to
    /// their approximate equivalents from the standard set of types. PDF 1.4+.
    ///
    /// For PDF 2.0 documents, note that this mapping maps to the PDF 1.7 roles.
    /// For a namespace-aware role-mapping mechanism, see
    /// [`Namespace::role_map_ns`].
    pub fn role_map(&mut self) -> RoleMap<'_> {
        self.dict.insert(Name(b"RoleMap")).start()
    }

    /// Start writing the `/ClassMap` attribute to map objects designating
    /// attribute classes to their corresponding attribute objects or arrays
    /// thereof.
    pub fn class_map(&mut self) -> ClassMap<'_> {
        self.dict.insert(Name(b"ClassMap")).start()
    }

    /// Start writing the `/Namespaces` attribute to specify the namespaces
    /// occurring in the document. Required if namespaced structure types or
    /// attributes, including the standard namespace for PDF 2.0, are used.
    /// Create these dictionaries with [`Chunk::namespace`] PDF 2.0+.
    pub fn namespaces(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"Namespaces")).array().typed()
    }

    /// Start writing the `PronunciationLexicon` attribute to specify one or
    /// multiple pronunciation lexicons for the document. PDF 2.0+.
    ///
    /// The lexicons shall be XML files conforming to the Pronunciation Lexicon
    /// Specification (PLS) Version 1.0. Each entry in the array is an indirect
    /// reference to a [`FileSpec`] dictionary for a lexicon file.
    pub fn pronunciation_lexicon(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"PronunciationLexicon")).array().typed()
    }

    /// Start writing the `/AF` attribute to specify one or multiple files
    /// associated with the entire structure tree. PDF 2.0+.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.dict.insert(Name(b"AF")).array().typed()
    }
}

deref!('a, StructTreeRoot<'a> => Dict<'a>, dict);

/// Writer for a _structure element dictionary_. PDF 1.3+
pub struct StructElement<'a> {
    dict: Dict<'a>,
}

writer!(StructElement: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"StructElem"));
    Self { dict }
});

impl StructElement<'_> {
    /// Write the `/S` attribute to specify the role of this structure element
    /// using elements from PDF 1.7 and below. Required if neither a PDF 2.0
    /// structure type is defined using [`Self::kind_2`] nor a custom type is
    /// specified using [`Self::custom_kind`].
    pub fn kind(&mut self, role: StructRole) -> &mut Self {
        self.dict.pair(Name(b"S"), role.to_name());
        self
    }

    /// Write the `/S` attribute and the `/NS` attribute to specify the role of
    /// this structure element in the PDF 2.0 namespace. Required if neither a
    /// PDF 1.7 structure type is defined using [`Self::kind`] nor a custom type
    /// is specified using [`Self::custom_kind`].
    ///
    /// The `pdf_2_ns` parameter is an indirect reference to a PDF 2.0 namespace
    /// dictionary. You can create this dictionary by using [`Chunk::namespace`]
    /// and then calling [`Namespace::pdf_2_ns`] on the returned writer.
    pub fn kind_2(&mut self, role: StructRole2, pdf_2_ns: Ref) -> &mut Self {
        self.dict.pair(Name(b"S"), Name(role.to_name_bytes(&mut [0; 6])));
        self.namespace(pdf_2_ns)
    }

    /// Write the `/S` attribute to specify the role of this structure element
    /// as a custom name. Required if no standard type is specified with
    /// [`Self::kind`].
    ///
    /// In some PDF/A profiles like PDF/A-2a, custom kinds must be mapped to
    /// their closest standard type in the role map.
    ///
    /// When using the namespaced model of PDF 2.0, using this may also require
    /// setting a namespace using [`Self::namespace`].
    pub fn custom_kind(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"S"), name);
        self
    }

    /// Write the `/P` attribute to specify the parent of this structure
    /// element. Required.
    pub fn parent(&mut self, parent: Ref) -> &mut Self {
        self.dict.pair(Name(b"P"), parent);
        self
    }

    /// Write the `/ID` attribute to specify the element identifier of this
    /// structure element.
    pub fn id(&mut self, id: Str) -> &mut Self {
        self.dict.pair(Name(b"ID"), id);
        self
    }

    /// Write the `/Ref` attribute to specify to which structure element this
    /// element refers. Used e.g. for footnotes. PDF 2.0+
    ///
    /// The parameter `refs` shall be indirect object references to other
    /// structure elements.
    pub fn refs(&mut self, refs: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.dict.insert(Name(b"Ref")).array().typed().items(refs);
        self
    }

    /// Write the `/Pg` attribute to specify the page some or all of this
    /// structure element is located on.
    pub fn page(&mut self, page: Ref) -> &mut Self {
        self.dict.pair(Name(b"Pg"), page);
        self
    }

    /// Write the `/K` attribute to reference the immediate child of this
    /// element.
    pub fn child(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"K"), id);
        self
    }

    /// Start writing the `/K` attribute to reference the immediate marked
    /// content child of this element.
    pub fn marked_content_child(&mut self) -> MarkedRef<'_> {
        self.dict.insert(Name(b"K")).start()
    }

    /// Start writing the `/K` attribute to reference the immediate object child
    /// of this element.
    pub fn object_child(&mut self) -> ObjectRef<'_> {
        self.dict.insert(Name(b"K")).start()
    }

    /// Start writing the `/K` attribute to specify the children elements and
    /// associated marked content sequences.
    pub fn children(&mut self) -> StructChildren<'_> {
        self.dict.insert(Name(b"K")).start()
    }

    /// Start writing the `/A` attribute to specify the attributes of this
    /// structure element.
    pub fn attributes(&mut self) -> TypedArray<'_, Attributes> {
        self.dict.insert(Name(b"A")).array().typed()
    }

    /// Start writing the `/C` attribute to associate the structure element with
    /// an attribute class.
    pub fn attribute_class(&mut self) -> TypedArray<'_, Name> {
        self.dict.insert(Name(b"C")).array().typed()
    }

    /// Write the `/R` attribute to specify the revision number, starting at 0.
    pub fn revision(&mut self, revision: i32) -> &mut Self {
        self.dict.pair(Name(b"R"), revision);
        self
    }

    /// Write the `/T` attribute to set a title.
    pub fn title(&mut self, title: TextStr) -> &mut Self {
        self.dict.pair(Name(b"T"), title);
        self
    }

    /// Write the `/Lang` attribute to set a language. PDF 1.4+
    pub fn lang(&mut self, lang: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Lang"), lang);
        self
    }

    /// Write the `/Alt` attribute to provide a description of the structure
    /// element.
    pub fn alt(&mut self, alt: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Alt"), alt);
        self
    }

    /// Write the `/E` attribute to set the expanded form of the abbreviation
    /// in this structure element. PDF 1.5+
    pub fn expanded(&mut self, expanded: TextStr) -> &mut Self {
        self.dict.pair(Name(b"E"), expanded);
        self
    }

    /// Write the `/ActualText` attribute to set the exact text replacement. PDF
    /// 1.4+
    pub fn actual_text(&mut self, actual_text: TextStr) -> &mut Self {
        self.dict.pair(Name(b"ActualText"), actual_text);
        self
    }

    /// Start writing the `/AF` array to specify the associated files of the
    /// element. PDF 2.0+ or PDF/A-3.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.insert(Name(b"AF")).array().typed()
    }

    /// Write the `/NS` attribute to indirectly reference a namespace dictionary
    /// for this structure element type. PDF 2.0+
    pub fn namespace(&mut self, ns: Ref) -> &mut Self {
        self.dict.pair(Name(b"NS"), ns);
        self
    }

    /// Write the `/PhoneticAlphabet` attribute to specify the phonetic alphabet
    /// used in the [StructElement::phoneme] attribute. PDF 2.0+
    pub fn phonetic_alphabet(&mut self, alphabet: PhoneticAlphabet) -> &mut Self {
        self.dict.pair(Name(b"PhoneticAlphabet"), alphabet.to_name());
        self
    }

    /// Write the `/Phoneme` attribute to specify the phonetic pronunciation of
    /// the text in the structure element. PDF 2.0+
    pub fn phoneme(&mut self, phoneme: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Phoneme"), phoneme);
        self
    }
}

deref!('a, StructElement<'a> => Dict<'a>, dict);

/// Writer for a _structure element children array_. PDF 1.3+
///
/// This struct is created by [`StructElement::children`].
pub struct StructChildren<'a> {
    arr: Array<'a>,
}

writer!(StructChildren: |obj| Self { arr: obj.array() });

impl StructChildren<'_> {
    /// Write a structure element child as an indirect object.
    pub fn struct_element(&mut self, id: Ref) -> &mut Self {
        self.arr.item(id);
        self
    }

    /// Write an integer marked content identifier.
    pub fn marked_content_id(&mut self, id: i32) -> &mut Self {
        self.arr.item(id);
        self
    }

    /// Start writing a marked content reference dictionary.
    pub fn marked_content_ref(&mut self) -> MarkedRef<'_> {
        self.arr.push().start()
    }

    /// Start writing an object reference dictionary.
    pub fn object_ref(&mut self) -> ObjectRef<'_> {
        self.arr.push().start()
    }
}

deref!('a, StructChildren<'a> => Array<'a>, arr);

/// Writer for a _marked content reference dictionary_. PDF 1.3+
///
/// This struct is created by [`StructChildren::marked_content_ref`] and
/// [`StructElement::marked_content_child`].
pub struct MarkedRef<'a> {
    dict: Dict<'a>,
}

writer!(MarkedRef: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"MCR"));
    Self { dict }
});

impl MarkedRef<'_> {
    /// Write the `/Pg` attribute to specify the page the referenced marked
    /// content sequence is located on.
    pub fn page(&mut self, page: Ref) -> &mut Self {
        self.dict.pair(Name(b"Pg"), page);
        self
    }

    /// Write the `/Stm` attribute to specify the content stream containing this
    /// marked content sequence if it was not on a page. If this content is
    /// missing, writing the page attribute here or in the associated structure
    /// element is required.
    pub fn stream(&mut self, stream: Ref) -> &mut Self {
        self.dict.pair(Name(b"Stm"), stream);
        self
    }

    /// Write the `/StmOwn` attribute to specify which object owns the content
    /// stream specified by the `/Stm` attribute.
    pub fn stream_owner(&mut self, owner: Ref) -> &mut Self {
        self.dict.pair(Name(b"StmOwn"), owner);
        self
    }

    /// Write the `/MCID` attribute to specify the integer marked content
    /// identifier. Required.
    pub fn marked_content_id(&mut self, id: i32) -> &mut Self {
        self.dict.pair(Name(b"MCID"), id);
        self
    }
}

deref!('a, MarkedRef<'a> => Dict<'a>, dict);

/// Writer for an _object reference dictionary_. PDF 1.3+
///
/// This struct is created by [`StructChildren::object_ref`].
pub struct ObjectRef<'a> {
    dict: Dict<'a>,
}

writer!(ObjectRef: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"OBJR"));
    Self { dict }
});

impl ObjectRef<'_> {
    /// Write the `/Pg` attribute to specify the page some or all of this
    /// structure element is located on.
    pub fn page(&mut self, page: Ref) -> &mut Self {
        self.dict.pair(Name(b"Pg"), page);
        self
    }

    /// Write the `/Obj` attribute to specify the object to be referenced. Required.
    pub fn object(&mut self, obj: Ref) -> &mut Self {
        self.dict.pair(Name(b"Obj"), obj);
        self
    }
}

deref!('a, ObjectRef<'a> => Dict<'a>, dict);

/// Writer for a _role map dictionary_. PDF 1.4+
///
/// This struct is created by [`StructTreeRoot::role_map`].
///
/// For PDF 2.0 documents, note that this mapping maps to the PDF 1.7 roles. For
/// a namespace-aware role-mapping mechanism, see [`Namespace::role_map_ns`].
pub struct RoleMap<'a> {
    dict: TypedDict<'a, Name<'a>>,
}

writer!(RoleMap: |obj| Self { dict: obj.dict().typed() });

impl RoleMap<'_> {
    /// Write an entry mapping a custom name to a pre-defined role.
    pub fn insert(&mut self, name: Name, role: StructRole) -> &mut Self {
        self.dict.pair(name, role.to_name());
        self
    }
}

deref!('a, RoleMap<'a> => TypedDict<'a, Name<'a>>, dict);

/// Writer for a _class map dictionary_. PDF 1.4+
///
/// This struct is created by [`StructTreeRoot::class_map`].
pub struct ClassMap<'a> {
    dict: Dict<'a>,
}

writer!(ClassMap: |obj| Self { dict: obj.dict() });

impl ClassMap<'_> {
    /// Start writing an attributes dictionary for a class name.
    pub fn single(&mut self, name: Name) -> Attributes<'_> {
        self.dict.insert(name).start()
    }

    /// Start writing an array of attribute dictionaries for a class name.
    pub fn multiple(&mut self, name: Name) -> TypedArray<'_, Attributes> {
        self.dict.insert(name).array().typed()
    }
}

deref!('a, ClassMap<'a> => Dict<'a>, dict);

/// Role the structure element fulfills in the document for PDF 1.7 and below.
///
/// These are the predefined standard roles in PDF 1.7 and below, matching the
/// `https://www.iso.org/pdf/ssn` namespace. The writer may write their own
/// roles and then provide a mapping with [`StructTreeRoot::role_map`], or, if
/// writing PDF 2.0, with [`Namespace::role_map_ns`]. PDF 1.4+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StructRole {
    /// The whole document.
    Document,
    /// A part of a document that may contain multiple articles or sections.
    Part,
    /// An article with largely self-contained content.
    Art,
    /// Section of a larger document.
    Sect,
    /// Generic subdivision.
    Div,
    /// A paragraph-level quote.
    BlockQuote,
    /// An image or figure caption.
    Caption,
    /// Table of contents.
    TOC,
    /// Item in the table of contents.
    TOCI,
    /// Index of the key terms in the document.
    Index,
    /// Element only present for grouping purposes that shall not be exported.
    NonStruct,
    /// Element present only for use by the writer and associated products.
    Private,
    /// A paragraph
    P,
    /// First-level heading.
    H1,
    /// Second-level heading.
    H2,
    /// Third-level heading.
    H3,
    /// Fourth-level heading.
    H4,
    /// Fifth-level heading.
    H5,
    /// Sixth-level heading.
    H6,
    /// A list.
    L,
    /// A list item.
    LI,
    /// Label for a list item.
    Lbl,
    /// Description of the list item.
    LBody,
    /// A table.
    Table,
    /// A table row.
    TR,
    /// A table header cell.
    TH,
    /// A table data cell.
    TD,
    /// A table header row group.
    THead,
    /// A table data row group.
    TBody,
    /// A table footer row group.
    TFoot,
    /// A generic inline element.
    Span,
    /// An inline quotation.
    Quote,
    /// A foot- or endnote.
    Note,
    /// A reference to elsewhere in the document.
    Reference,
    /// A reference to an external document.
    BibEntry,
    /// Computer code.
    Code,
    /// A link.
    Link,
    /// An association between an annotation and the content it belongs to. PDF
    /// 1.5+
    Annot,
    /// Ruby annotation for CJK text. PDF 1.5+
    Ruby,
    /// Warichu annotation for CJK text. PDF 1.5+
    Warichu,
    /// Base text of a Ruby annotation. PDF 1.5+
    RB,
    /// Annotation text of a Ruby annotation. PDF 1.5+
    RT,
    /// Punctuation of a Ruby annotation. PDF 1.5+
    RP,
    /// Text of a Warichu annotation. PDF 1.5+
    WT,
    /// Punctuation of a Warichu annotation. PDF 1.5+
    WP,
    /// Item of graphical content.
    Figure,
    /// Mathematical formula.
    Formula,
    /// Form widget.
    Form,
}

impl StructRole {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Document => Name(b"Document"),
            Self::Part => Name(b"Part"),
            Self::Art => Name(b"Art"),
            Self::Sect => Name(b"Sect"),
            Self::Div => Name(b"Div"),
            Self::BlockQuote => Name(b"BlockQuote"),
            Self::Caption => Name(b"Caption"),
            Self::TOC => Name(b"TOC"),
            Self::TOCI => Name(b"TOCI"),
            Self::Index => Name(b"Index"),
            Self::NonStruct => Name(b"NonStruct"),
            Self::Private => Name(b"Private"),
            Self::P => Name(b"P"),
            Self::H1 => Name(b"H1"),
            Self::H2 => Name(b"H2"),
            Self::H3 => Name(b"H3"),
            Self::H4 => Name(b"H4"),
            Self::H5 => Name(b"H5"),
            Self::H6 => Name(b"H6"),
            Self::L => Name(b"L"),
            Self::LI => Name(b"LI"),
            Self::Lbl => Name(b"Lbl"),
            Self::LBody => Name(b"LBody"),
            Self::Table => Name(b"Table"),
            Self::TR => Name(b"TR"),
            Self::TH => Name(b"TH"),
            Self::TD => Name(b"TD"),
            Self::THead => Name(b"THead"),
            Self::TBody => Name(b"TBody"),
            Self::TFoot => Name(b"TFoot"),
            Self::Span => Name(b"Span"),
            Self::Quote => Name(b"Quote"),
            Self::Note => Name(b"Note"),
            Self::Reference => Name(b"Reference"),
            Self::BibEntry => Name(b"BibEntry"),
            Self::Code => Name(b"Code"),
            Self::Link => Name(b"Link"),
            Self::Annot => Name(b"Annot"),
            Self::Ruby => Name(b"Ruby"),
            Self::Warichu => Name(b"Warichu"),
            Self::RB => Name(b"RB"),
            Self::RT => Name(b"RT"),
            Self::RP => Name(b"RP"),
            Self::WT => Name(b"WT"),
            Self::WP => Name(b"WP"),
            Self::Figure => Name(b"Figure"),
            Self::Formula => Name(b"Formula"),
            Self::Form => Name(b"Form"),
        }
    }

    /// Return the corresponding PDF 2.0 [`StructRole2`] for this role or
    /// `None`.
    pub fn into_pdf_2_0(self) -> Option<StructRole2> {
        match self {
            Self::Document => Some(StructRole2::Document),
            Self::Part => Some(StructRole2::Part),
            Self::Art => None,
            Self::Sect => Some(StructRole2::Sect),
            Self::Div => Some(StructRole2::Div),
            Self::BlockQuote => None,
            Self::Caption => Some(StructRole2::Caption),
            Self::TOC => None,
            Self::TOCI => None,
            Self::Index => None,
            Self::NonStruct => Some(StructRole2::NonStruct),
            Self::Private => None,
            Self::P => Some(StructRole2::P),
            Self::H1 => Some(StructRole2::Heading(NonZeroU16::new(1).unwrap())),
            Self::H2 => Some(StructRole2::Heading(NonZeroU16::new(2).unwrap())),
            Self::H3 => Some(StructRole2::Heading(NonZeroU16::new(3).unwrap())),
            Self::H4 => Some(StructRole2::Heading(NonZeroU16::new(4).unwrap())),
            Self::H5 => Some(StructRole2::Heading(NonZeroU16::new(5).unwrap())),
            Self::H6 => Some(StructRole2::Heading(NonZeroU16::new(6).unwrap())),
            Self::L => Some(StructRole2::L),
            Self::LI => Some(StructRole2::LI),
            Self::Lbl => Some(StructRole2::Lbl),
            Self::LBody => Some(StructRole2::LBody),
            Self::Table => Some(StructRole2::Table),
            Self::TR => Some(StructRole2::TR),
            Self::TH => Some(StructRole2::TH),
            Self::TD => Some(StructRole2::TD),
            Self::THead => Some(StructRole2::THead),
            Self::TBody => Some(StructRole2::TBody),
            Self::TFoot => Some(StructRole2::TFoot),
            Self::Span => Some(StructRole2::Span),
            Self::Quote => Some(StructRole2::Em),
            Self::Note => Some(StructRole2::FENote),
            Self::Reference => Some(StructRole2::Link),
            Self::BibEntry => None,
            Self::Code => Some(StructRole2::Strong),
            Self::Link => Some(StructRole2::Link),
            Self::Annot => Some(StructRole2::Annot),
            Self::Ruby => Some(StructRole2::Ruby),
            Self::Warichu => Some(StructRole2::Warichu),
            Self::RB => Some(StructRole2::RB),
            Self::RT => Some(StructRole2::RT),
            Self::RP => Some(StructRole2::WP),
            Self::WT => Some(StructRole2::WT),
            Self::WP => Some(StructRole2::WP),
            Self::Figure => Some(StructRole2::Figure),
            Self::Formula => Some(StructRole2::Formula),
            Self::Form => Some(StructRole2::Form),
        }
    }

    /// Return the type of the structure element.
    pub fn role_type(self) -> StructRoleType {
        match self {
            Self::Document
            | Self::Part
            | Self::Art
            | Self::Sect
            | Self::Div
            | Self::BlockQuote
            | Self::Caption
            | Self::TOC
            | Self::TOCI
            | Self::Index
            | Self::NonStruct
            | Self::Private => StructRoleType::Grouping,
            Self::P | Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6 => {
                StructRoleType::BlockLevel(BlockLevelRoleSubtype::ParagraphLike)
            }
            Self::L | Self::LI | Self::Lbl | Self::LBody => {
                StructRoleType::BlockLevel(BlockLevelRoleSubtype::List)
            }
            Self::Table
            | Self::TR
            | Self::TH
            | Self::TD
            | Self::THead
            | Self::TBody
            | Self::TFoot => StructRoleType::BlockLevel(BlockLevelRoleSubtype::Table),
            Self::Span
            | Self::Quote
            | Self::Note
            | Self::Reference
            | Self::BibEntry
            | Self::Code
            | Self::Ruby
            | Self::Warichu => {
                StructRoleType::InlineLevel(InlineLevelRoleSubtype::Generic)
            }
            Self::Link => StructRoleType::InlineLevel(InlineLevelRoleSubtype::Link),
            Self::Annot => {
                StructRoleType::InlineLevel(InlineLevelRoleSubtype::Annotation)
            }
            Self::RB | Self::RT | Self::RP | Self::WT | Self::WP => {
                StructRoleType::InlineLevel(InlineLevelRoleSubtype::RubyWarichu)
            }
            Self::Figure | Self::Formula | Self::Form => StructRoleType::Illustration,
        }
    }
}

/// Type of the PDF 1.7 [structure element](StructRole) in the document,
/// determining layout, permitted attributes, and nesting.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StructRoleType {
    /// Elements used solely to group other elements together.
    Grouping,
    /// Elements laid out across the block axis, also known as BLSE.
    BlockLevel(BlockLevelRoleSubtype),
    /// Elements laid out across the inline axis, also known as ILSE.
    InlineLevel(InlineLevelRoleSubtype),
    /// Elements whose contents consist of one or more graphics objects.
    Illustration,
}

/// Subtypes of block-level structure roles, determining the layout and
/// permitted attributes of the element.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BlockLevelRoleSubtype {
    /// Block-level elements containing predominantly text content.
    ParagraphLike,
    /// List-related elements, such as lists and list items.
    List,
    /// Table-related elements, such as tables and table rows.
    Table,
}

/// Subtypes of inline-level PDF 1.7 structure roles, determining the layout and
/// permitted attributes of the element.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InlineLevelRoleSubtype {
    /// Generic inline elements, such as spans, quotes, and code.
    Generic,
    /// Links.
    Link,
    /// Superimposed annotations.
    Annotation,
    /// Ruby and Warichu annotations, which are used for CJK text.
    RubyWarichu,
}

impl TryFrom<StructRole> for StructRole2 {
    type Error = ();

    fn try_from(value: StructRole) -> Result<Self, Self::Error> {
        value.into_pdf_2_0().ok_or(())
    }
}

/// PDF 2.0 roles the structure element fulfills in the document.
///
/// These are the predefined standard roles in PDF 2.0, matching the
/// `https://www.iso.org/pdf2/ssn` namespace. The writer may write their own
/// types and then provide a mapping using [`Namespace::role_map_ns`]. PDF 2.0+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StructRole2 {
    /// The whole document.
    Document,
    /// An incomplete fragment of another document.
    DocumentFragment,
    /// A part of a document that may contain multiple articles or sections.
    Part,
    /// Section of a larger document.
    Sect,
    /// Generic subdivision.
    Div,
    /// Content distinct from other content within the parent, such as callouts,
    /// sidebars, commentary, or background information.
    Aside,
    /// Element only present for grouping purposes that shall not be exported.
    NonStruct,
    /// A paragraph
    P,
    /// Heading with a specific level.
    Heading(NonZeroU16),
    /// Strongly structured heading.
    StructuredHeading,
    /// A title of a document.
    Title,
    /// A foot- or endnote.
    FENote,
    /// A subdivision within a block level element.
    Sub,
    /// Label for a list item.
    Lbl,
    /// A generic inline element.
    Span,
    /// An emphasized inline element.
    Em,
    /// An inline element with heightened (strong) importance.
    Strong,
    /// A link.
    Link,
    /// An association between an annotation and the content it belongs to. PDF
    /// 1.5+
    Annot,
    /// Form widget.
    Form,
    /// Ruby annotation for CJK text. PDF 1.5+
    Ruby,
    /// Base text of a Ruby annotation. PDF 1.5+
    RB,
    /// Annotation text of a Ruby annotation. PDF 1.5+
    RT,
    /// Warichu annotation for CJK text. PDF 1.5+
    Warichu,
    /// Text of a Warichu annotation. PDF 1.5+
    WT,
    /// Punctuation of a Warichu annotation. PDF 1.5+
    WP,
    /// A list.
    L,
    /// A list item.
    LI,
    /// Description of the list item.
    LBody,
    /// A table.
    Table,
    /// A table row.
    TR,
    /// A table header cell.
    TH,
    /// A table data cell.
    TD,
    /// A table header row group.
    THead,
    /// A table data row group.
    TBody,
    /// A table footer row group.
    TFoot,
    /// An image or figure caption.
    Caption,
    /// Item of graphical content.
    Figure,
    /// Mathematical formula.
    Formula,
    /// An artifact not part of the logical content of the document.
    Artifact,
}

impl StructRole2 {
    pub(crate) fn to_name_bytes(self, buf: &mut [u8; 6]) -> &[u8] {
        match self {
            Self::Document => b"Document",
            Self::DocumentFragment => b"DocumentFragment",
            Self::Part => b"Part",
            Self::Sect => b"Sect",
            Self::Div => b"Div",
            Self::Aside => b"Aside",
            Self::NonStruct => b"NonStruct",
            Self::P => b"P",
            Self::Heading(level) => {
                let mut cursor = Cursor::new(buf.as_mut_slice());
                write!(&mut cursor, "H{}", level.get()).unwrap();
                let pos = cursor.position() as usize;
                &buf[..pos]
            }
            Self::StructuredHeading => b"H",
            Self::Title => b"Title",
            Self::FENote => b"FENote",
            Self::Sub => b"Sub",
            Self::Lbl => b"Lbl",
            Self::Span => b"Span",
            Self::Em => b"Em",
            Self::Strong => b"Strong",
            Self::Link => b"Link",
            Self::Annot => b"Annot",
            Self::Form => b"Form",
            Self::Ruby => b"Ruby",
            Self::RB => b"RB",
            Self::RT => b"RT",
            Self::Warichu => b"Warichu",
            Self::WT => b"WT",
            Self::WP => b"WP",
            Self::L => b"L",
            Self::LI => b"LI",
            Self::LBody => b"LBody",
            Self::Table => b"Table",
            Self::TR => b"TR",
            Self::TH => b"TH",
            Self::TD => b"TD",
            Self::THead => b"THead",
            Self::TBody => b"TBody",
            Self::TFoot => b"TFoot",
            Self::Caption => b"Caption",
            Self::Figure => b"Figure",
            Self::Formula => b"Formula",
            Self::Artifact => b"Artifact",
        }
    }

    /// Return the corresponding PDF 1.7 [`StructRole`] for this role or `None`.
    pub fn into_pdf_1_7(self) -> Option<StructRole> {
        match self {
            Self::Document => Some(StructRole::Document),
            Self::DocumentFragment => None,
            Self::Part => Some(StructRole::Part),
            Self::Sect => Some(StructRole::Sect),
            Self::Div => Some(StructRole::Div),
            Self::Aside => None,
            Self::NonStruct => Some(StructRole::NonStruct),
            Self::P => Some(StructRole::P),
            Self::Heading(n) if n.get() == 1 => Some(StructRole::H1),
            Self::Heading(n) if n.get() == 2 => Some(StructRole::H2),
            Self::Heading(n) if n.get() == 3 => Some(StructRole::H3),
            Self::Heading(n) if n.get() == 4 => Some(StructRole::H4),
            Self::Heading(n) if n.get() == 5 => Some(StructRole::H5),
            Self::Heading(n) if n.get() == 6 => Some(StructRole::H6),
            Self::Heading(_) => None,
            Self::StructuredHeading => None,
            Self::Title => None,
            Self::FENote => None,
            Self::Sub => None,
            Self::Lbl => Some(StructRole::Lbl),
            Self::Span => Some(StructRole::Span),
            Self::Em => None,
            Self::Strong => None,
            Self::Link => Some(StructRole::Link),
            Self::Annot => Some(StructRole::Annot),
            Self::Form => Some(StructRole::Form),
            Self::Ruby => Some(StructRole::Ruby),
            Self::RB => Some(StructRole::RB),
            Self::RT => Some(StructRole::RT),
            Self::Warichu => Some(StructRole::Warichu),
            Self::WT => Some(StructRole::WT),
            Self::WP => Some(StructRole::WP),
            Self::L => Some(StructRole::L),
            Self::LI => Some(StructRole::LI),
            Self::LBody => Some(StructRole::LBody),
            Self::Table => Some(StructRole::Table),
            Self::TR => Some(StructRole::TR),
            Self::TH => Some(StructRole::TH),
            Self::TD => Some(StructRole::TD),
            Self::THead => Some(StructRole::THead),
            Self::TBody => Some(StructRole::TBody),
            Self::TFoot => Some(StructRole::TFoot),
            Self::Caption => Some(StructRole::Caption),
            Self::Figure => Some(StructRole::Figure),
            Self::Formula => Some(StructRole::Formula),
            Self::Artifact => None,
        }
    }

    /// Return the closest equivalent role in the PDF 1.7 namespace.
    ///
    /// Returns `None` if the role exactly matches a PDF 1.7 role (see
    /// [`Self::into_pdf_1_7`]).
    ///
    /// There are three parameters governing the role mapping:
    ///
    /// - `map_hn_to_h6`: Are headings with levels higher than 6 are mapped to
    ///   [`StructRole::H6`] (`true`) or [`StructRole::P`] (`false`)
    /// - `map_title_to_h1`: Is the `Title` role mapped to [`StructRole::H1`]
    ///   (`true`) or to [`StructRole::P`] (`false`)
    /// - `map_sub_to_span`: Is the `Sub` role mapped to [`StructRole::Span`]
    ///   (`true`) or to [`StructRole::Div`] (`false`)
    pub fn role_mapped_1_7(
        self,
        map_hn_to_h6: bool,
        map_title_to_h1: bool,
        map_sub_to_span: bool,
    ) -> Option<StructRole> {
        match self {
            Self::Document => None,
            Self::DocumentFragment => Some(StructRole::Div),
            Self::Part => None,
            Self::Sect => None,
            Self::Div => None,
            Self::Aside => Some(StructRole::Div),
            Self::NonStruct => None,
            Self::P => None,
            Self::Heading(n) if (1u16..=6).contains(&n.get()) => None,
            Self::Heading(_) => {
                Some(if map_hn_to_h6 { StructRole::H6 } else { StructRole::P })
            }
            Self::StructuredHeading => Some(StructRole::P),
            Self::Title => {
                Some(if map_title_to_h1 { StructRole::H1 } else { StructRole::P })
            }
            Self::FENote => Some(StructRole::Note),
            Self::Sub => {
                Some(if map_sub_to_span { StructRole::Span } else { StructRole::Div })
            }
            Self::Lbl => None,
            Self::Span => None,
            Self::Em => Some(StructRole::Span),
            Self::Strong => Some(StructRole::Span),
            Self::Link => None,
            Self::Annot => None,
            Self::Form => None,
            Self::Ruby => None,
            Self::RB => None,
            Self::RT => None,
            Self::Warichu => None,
            Self::WT => None,
            Self::WP => None,
            Self::L => None,
            Self::LI => None,
            Self::LBody => None,
            Self::Table => None,
            Self::TR => None,
            Self::TH => None,
            Self::TD => None,
            Self::THead => None,
            Self::TBody => None,
            Self::TFoot => None,
            Self::Caption => None,
            Self::Figure => None,
            Self::Formula => None,
            Self::Artifact => Some(StructRole::Private),
        }
    }

    /// Return the type of the structure element.
    pub fn role_type(self) -> StructRoleType2 {
        match self {
            Self::Document | Self::DocumentFragment => StructRoleType2::Document,
            Self::Part | Self::Sect | Self::Div | Self::Aside | Self::NonStruct => {
                StructRoleType2::Grouping
            }
            Self::P
            | Self::Heading(_)
            | Self::StructuredHeading
            | Self::Title
            | Self::FENote => StructRoleType2::BlockLevel,
            Self::Sub => StructRoleType2::SubBlockLevel,
            Self::Lbl
            | Self::Span
            | Self::Em
            | Self::Strong
            | Self::Link
            | Self::Annot
            | Self::Form => {
                StructRoleType2::InlineLevel(InlineLevelRoleSubtype2::Generic)
            }
            Self::Ruby | Self::RB | Self::RT | Self::Warichu | Self::WT | Self::WP => {
                StructRoleType2::InlineLevel(InlineLevelRoleSubtype2::RubyWarichu)
            }
            Self::L | Self::LI | Self::LBody => StructRoleType2::List,
            Self::Table
            | Self::TR
            | Self::TH
            | Self::TD
            | Self::THead
            | Self::TBody
            | Self::TFoot => StructRoleType2::Table,
            Self::Caption => StructRoleType2::Caption,
            Self::Figure => StructRoleType2::Figure,
            Self::Formula => StructRoleType2::Formula,
            Self::Artifact => StructRoleType2::Artifact,
        }
    }
}

impl TryFrom<StructRole2> for StructRole {
    type Error = ();

    fn try_from(value: StructRole2) -> Result<Self, Self::Error> {
        value.into_pdf_1_7().ok_or(())
    }
}

/// Type of the PDF 2.0 [structure element](StructRole2) in the document,
/// determining layout, permitted attributes, and nesting.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StructRoleType2 {
    /// Elements representing the whole document or a fragment of it.
    Document,
    /// Elements used solely to group other elements together.
    Grouping,
    /// Elements laid out across the block axis, also known as BLSE.
    BlockLevel,
    /// Elements that appear as sub-divisions of a block-level element.
    SubBlockLevel,
    /// Elements laid out across the inline axis, also known as ILSE.
    InlineLevel(InlineLevelRoleSubtype2),
    /// Elements related to lists.
    List,
    /// Elements related to tables.
    Table,
    /// Figure captions.
    Caption,
    /// Figures, such as images and illustrations.
    Figure,
    /// Mathematical formulas.
    Formula,
    /// Artifacts not part of the logical content of the document.
    Artifact,
}

/// Subtypes of inline-level structure roles for PDF 2.0, determining the layout and
/// permitted attributes of the element.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InlineLevelRoleSubtype2 {
    /// Generic inline element.
    Generic,
    /// Ruby or Warichu annotation.
    RubyWarichu,
}

/// Which phonetic alphabet to use for the `/Phonetic` key in the
/// [`StructElement`] dictionary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PhoneticAlphabet<'a> {
    /// The International Phonetic Alphabet.
    Ipa,
    /// The Extended Speech Assessment Methods Phonetic Alphabet (X-SAMPA).
    XSampa,
    /// The Pinyin romanization system for Chinese.
    Pinyin,
    /// The Wade-Giles romanization system for Chinese.
    WadeGiles,
    /// A custom phonetic alphabet.
    Custom(Name<'a>),
}

impl<'a> PhoneticAlphabet<'a> {
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::Ipa => Name(b"ipa"),
            Self::XSampa => Name(b"x-sampa"),
            Self::Pinyin => Name(b"zh-Latn-pinyin"),
            Self::WadeGiles => Name(b"zh-Latn-wadegile"),
            Self::Custom(name) => name,
        }
    }
}

/// Writer for a _namespace dictionary_. PDF 2.0+
///
/// This struct is created by [`Chunk::namespace`].
pub struct Namespace<'a> {
    dict: Dict<'a>,
}

writer!(Namespace: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Namespace"));
    Self { dict }
});

impl Namespace<'_> {
    /// Write the `/NS` attribute to specify the identifier (URI) of the namespace.
    pub fn ns(&mut self, identifier: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Name"), identifier);
        self
    }

    /// Start writing the `/Schema` attribute to point to a schema definition
    /// for the namespace. Optional.
    pub fn schema(&mut self) -> FileSpec<'_> {
        self.dict.insert(Name(b"Schema")).start()
    }

    /// Start writing the `/RoleMapNS` attribute to map structure elements to
    /// elements in another namespace. Optional.
    ///
    /// For a mechanism to define role mappings compatible with PDF 1.3 and
    /// above, see [`StructTreeRoot::role_map`].
    pub fn role_map_ns(&mut self) -> NamespaceRoleMap<'_> {
        self.dict.insert(Name(b"RoleMapNS")).start()
    }

    /// Write the namespace dictionary for the _standard structure namespace for
    /// PDF 2.0_.
    pub fn pdf_2_ns(mut self) {
        self.ns(TextStr("https://www.iso.org/pdf2/ssn"));
    }

    /// Write the namespace dictionary for the _standard structure namespace for
    /// PDF 1.7_.
    pub fn pdf_1_7_ns(mut self) {
        self.ns(TextStr("https://www.iso.org/pdf/ssn"));
    }

    /// Write the namespace dictionary for MathML 3.0.
    pub fn mathml_3_0_ns(mut self) {
        self.ns(TextStr("https://www.w3.org/1998/Math/MathML"));
    }
}

deref!('a, Namespace<'a> => Dict<'a>, dict);

/// Writer for a _namespace role map dictionary_. PDF 2.0+
///
/// This struct is created by [`Namespace::role_map_ns`].
pub struct NamespaceRoleMap<'a> {
    dict: Dict<'a>,
}

writer!(NamespaceRoleMap: |obj| {
    Self { dict: obj.dict() }
});

impl NamespaceRoleMap<'_> {
    /// Write an entry mapping a custom structure type to an element in the
    /// standard (PDF 1.7) namespace.
    pub fn to_pdf_1_7(&mut self, name: Name, role: StructRole) -> &mut Self {
        self.dict.pair(name, role.to_name());
        self
    }

    /// Write an entry mapping a custom structure type to an element in the
    /// namespace referenced by the namespace dictionary the value of the
    /// `ns_ref` parameter points to.
    pub fn to_namespace(&mut self, name: Name, role: Name, ns_ref: Ref) -> &mut Self {
        let mut dict = self.dict.insert(name).dict();
        dict.pair(Name(b"Role"), role);
        dict.pair(Name(b"NS"), ns_ref);
        dict.finish();
        self
    }
}

deref!('a, NamespaceRoleMap<'a> => Dict<'a>, dict);

/// Writer for a _mark information dictionary_. PDF 1.4+
///
/// This struct is created by [`Catalog::mark_info`].
pub struct MarkInfo<'a> {
    dict: Dict<'a>,
}

writer!(MarkInfo: |obj| Self { dict: obj.dict() });

impl MarkInfo<'_> {
    /// Write the `/Marked` attribute to indicate whether the document conforms
    /// to the Tagged PDF specification.
    ///
    /// Must be `true` in some PDF/A profiles like PDF/A-2a.
    pub fn marked(&mut self, conformant: bool) -> &mut Self {
        self.pair(Name(b"Marked"), conformant);
        self
    }

    /// Write the `/UserProperties` attribute to indicate whether the document
    /// contains structure elements with user properties. PDF 1.6+.
    pub fn user_properties(&mut self, present: bool) -> &mut Self {
        self.pair(Name(b"UserProperties"), present);
        self
    }

    /// Write the `/Suspects` attribute to indicate whether the document
    /// contains tag suspects. PDF 1.6+.
    pub fn suspects(&mut self, present: bool) -> &mut Self {
        self.pair(Name(b"Suspects"), present);
        self
    }
}

deref!('a, MarkInfo<'a> => Dict<'a>, dict);

/// Predominant reading order of text.
///
/// Used to aid the viewer with the special ordering in which to display pages.
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

writer!(PageLabel: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"PageLabel"));
    Self { dict }
});

impl PageLabel<'_> {
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
/// This struct is created by [`Pdf::document_info`].
pub struct DocumentInfo<'a> {
    dict: Dict<'a>,
}

writer!(DocumentInfo: |obj| Self { dict: obj.dict() });

impl DocumentInfo<'_> {
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

/// Whether a document has been adjusted with traps.
///
/// Those account for colorant misregistration during the printing process.
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
/// This struct is created by [`Chunk::pages`].
pub struct Pages<'a> {
    dict: Dict<'a>,
}

writer!(Pages: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Pages"));
    Self { dict }
});

impl Pages<'_> {
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
/// This struct is created by [`Chunk::page`].
pub struct Page<'a> {
    dict: Dict<'a>,
}

writer!(Page: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Page"));
    Self { dict }
});

impl Page<'_> {
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
    /// written to the file using [`Chunk::stream`].
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
    ///
    /// Required for pages with transparency in PDF/A if no output intent is
    /// present.
    pub fn group(&mut self) -> Group<'_> {
        self.insert(Name(b"Group")).start()
    }

    /// Write the `/Thumb` attribute to set an [image][ImageXObject] as the page
    /// thumbnail. Must be RGB, Grayscale, or an indexed color space based on
    /// the former two.
    pub fn thumbnail(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Thumb"), id);
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
        self.insert(Name(b"Trans")).start()
    }

    /// Write the `/Annots` (annotations) array.
    pub fn annotations(&mut self, ids: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Annots")).array().items(ids);
        self
    }

    /// Write the `/StructParents` attribute to indicate the [structure tree
    /// elements][StructElement] the contents of this XObject may belong to. PDF 1.3+.
    pub fn struct_parents(&mut self, key: i32) -> &mut Self {
        self.pair(Name(b"StructParents"), key);
        self
    }

    /// Write the `/Tabs` attribute. This specifies the order in which the
    /// annotations should be focused by hitting tab. PDF 1.5+.
    pub fn tab_order(&mut self, order: TabOrder) -> &mut Self {
        self.pair(Name(b"Tabs"), order.to_name());
        self
    }

    /// Write the `/UserUnit` attribute. This sets how large a user space unit
    /// is in printer's points (1/72 inch). This defaults to `1.0`. PDF 1.6+.
    pub fn user_unit(&mut self, value: f32) -> &mut Self {
        self.pair(Name(b"UserUnit"), value);
        self
    }

    /// Start writing the `/AA` dictionary. This sets the actions to perform
    /// when a page is opened or closed. PDF 1.2+.
    ///
    /// Note that this attribute is forbidden in PDF/A.
    pub fn additional_actions(&mut self) -> AdditionalActions<'_> {
        self.insert(Name(b"AA")).start()
    }

    /// Write the `/Metadata` attribute to specify the page's metadata. PDF
    /// 1.4+.
    ///
    /// The reference shall point to a [metadata stream](Metadata).
    ///
    /// Required in PDF/A.
    pub fn metadata(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Metadata"), id);
        self
    }

    /// Start writing the `/AF` array to specify the associated files of the
    /// page. PDF 2.0+ or PDF/A-3.
    pub fn associated_files(&mut self) -> TypedArray<'_, FileSpec> {
        self.insert(Name(b"AF")).array().typed()
    }
}

deref!('a, Page<'a> => Dict<'a>, dict);

/// Writer for an _outline dictionary_.
///
/// This struct is created by [`Chunk::outline`].
pub struct Outline<'a> {
    dict: Dict<'a>,
}

writer!(Outline: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Outlines"));
    Self { dict }
});

impl Outline<'_> {
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
/// This struct is created by [`Chunk::outline_item`].
pub struct OutlineItem<'a> {
    dict: Dict<'a>,
}

writer!(OutlineItem: |obj| Self { dict: obj.dict() });

impl OutlineItem<'_> {
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
    pub fn dest(&mut self) -> Destination<'_> {
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

/// Writer for a _names dictionary_.
///
/// This dictionary can map various objects to names using name trees. This
/// struct is created by [`Catalog::names`].
pub struct Names<'a> {
    dict: Dict<'a>,
}

writer!(Names: |obj| Self { dict: obj.dict() });

impl Names<'_> {
    /// Start writing the `/Dests` attribute to provide associations for
    /// [destinations](Destination).
    pub fn destinations(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"Dests")).start()
    }

    /// Start writing the `/AP` attribute to provide associations for appearance
    /// streams. PDF 1.3+.
    pub fn appearances(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"AP")).start()
    }

    /// Start writing the `/JavaScript` attribute to provide associations for
    /// JavaScript actions. PDF 1.3+.
    pub fn javascript(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"JavaScript")).start()
    }

    /// Start writing the `/Pages` attribute to name [pages](Page). PDF 1.3+.
    pub fn pages(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"Pages")).start()
    }

    /// Start writing the `/Template` attribute to name [pages](Pages) outside
    /// of the page tree as templates for interactive forms. PDF 1.3+.
    pub fn templates(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"Templates")).start()
    }

    /// Start writing the `/IDS` attribute to map identifiers to Web Capture
    /// content sets. PDF 1.3+.
    pub fn capture_ids(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"IDS")).start()
    }

    /// Start writing the `/URLS` attribute to map URLs to Web Capture content
    /// sets. PDF 1.3+.
    pub fn capture_urls(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"URLS")).start()
    }

    /// Start writing the `/EmbeddedFiles` attribute to name [embedded
    /// files](EmbeddedFile). PDF 1.4+.
    ///
    /// Note that this key is forbidden in PDF/A-1, and restricted in PDF/A-2
    /// and PDF/A-4.
    pub fn embedded_files(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"EmbeddedFiles")).start()
    }

    /// Start writing the `/AlternatePresentations` attribute to name alternate
    /// presentations. PDF 1.4+.
    ///
    /// Note that this key is forbidden in PDF/A.
    pub fn alternate_presentations(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"AlternatePresentations")).start()
    }

    /// Start writing the `/Renditions` attribute to name renditions. The names
    /// must conform to Unicode. PDF 1.5+.
    pub fn renditions(&mut self) -> NameTree<'_, Ref> {
        self.dict.insert(Name(b"Renditions")).start()
    }
}

deref!('a, Names<'a> => Dict<'a>, dict);

/// Writer for a _destination array_.
///
/// A dictionary mapping to this struct is created by
/// [`Chunk::destinations`]. This struct is also created by
/// [`Action::destination`].
pub struct Destination<'a> {
    array: Array<'a>,
}

writer!(Destination: |obj| Self { array: obj.array() });

impl Destination<'_> {
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
pub enum TabOrder {
    /// Navigate the annotations horizontally, then vertically.
    RowOrder,
    /// Navigate the annotations vertically, then horizontally.
    ColumnOrder,
    /// Navigate the annotations in the order they appear in the structure tree.
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

/// Writer for a _metadata stream_. PDF 1.4+.
///
/// This struct is created by [`Chunk::metadata`].
pub struct Metadata<'a> {
    stream: Stream<'a>,
}

impl<'a> Metadata<'a> {
    /// Create a new metadata stream writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"Metadata"));
        stream.pair(Name(b"Subtype"), Name(b"XML"));
        Self { stream }
    }
}

deref!('a, Metadata<'a> => Stream<'a>, stream);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_heading_name() {
        let mut buf = [0; 6];
        let name = Name(StructRole2::Heading(NonZeroU16::MAX).to_name_bytes(&mut buf));
        assert_eq!(Name(b"H65535"), name);
    }
}
