use super::*;

/// Writer for an _action dictionary_.
///
/// This struct is created by [`Annotation::action`] and many keys of
/// [`AdditionalActions`].
pub struct Action<'a> {
    dict: Dict<'a>,
}

writer!(Action: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Action"));
    Self { dict }
});

impl<'a> Action<'a> {
    /// Write the `/S` attribute to set the action type.
    pub fn action_type(&mut self, kind: ActionType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
        self
    }

    /// Start writing the `/D` attribute to set the destination of this
    /// GoTo-type action.
    pub fn destination(&mut self) -> Destination<'_> {
        self.insert(Name(b"D")).start()
    }

    /// Write the `/D` attribute to set the destination of this GoTo-type action
    /// to a named destination.
    pub fn destination_named(&mut self, name: Name) -> &mut Self {
        self.pair(Name(b"D"), name);
        self
    }

    /// Start writing the `/F` attribute, depending on the [`ActionType`], setting:
    /// - `RemoteGoTo`: which file to go to
    /// - `Launch`: which application to launch
    /// - `SubmitForm`: script location of the webserver that processes the
    ///   submission
    /// - `ImportData`: the FDF file from which to import data.
    pub fn file_spec(&mut self) -> FileSpec<'_> {
        self.insert(Name(b"F")).start()
    }

    /// Write the `/NewWindow` attribute to set whether this remote GoTo action
    /// should open the referenced destination in another window.
    pub fn new_window(&mut self, new: bool) -> &mut Self {
        self.pair(Name(b"NewWindow"), new);
        self
    }

    /// Write the `/URI` attribute to set where this link action goes.
    pub fn uri(&mut self, uri: Str) -> &mut Self {
        self.pair(Name(b"URI"), uri);
        self
    }

    /// Write the `/IsMap` attribute to set if the click position of the user's
    /// cursor inside the link rectangle should be appended to the referenced
    /// URI as a query parameter.
    pub fn is_map(&mut self, map: bool) -> &mut Self {
        self.pair(Name(b"IsMap"), map);
        self
    }

    /// Write the `/JS` attribute to set the script of this action as a text
    /// string. Only permissible for JavaScript and Rendition actions.
    pub fn js_string(&mut self, script: TextStr) -> &mut Self {
        self.pair(Name(b"JS"), script);
        self
    }

    /// Write the `/JS` attribute to set the script of this action as a text
    /// stream. The indirect reference shall point to a stream containing valid
    /// ECMAScript. The stream must have `PdfDocEncoding` or be in Unicode,
    /// starting with `U+FEFF`. Only permissible for JavaScript and Rendition
    /// actions.
    pub fn js_stream(&mut self, script: Ref) -> &mut Self {
        self.pair(Name(b"JS"), script);
        self
    }

    /// Start writing the `/Fields` array to set the fields which are
    /// [include/exclude](FormActionFlags::INCLUDE_EXCLUDE) when submitting a
    /// form, resetting a form, or loading an FDF file.
    pub fn fields(&mut self) -> Fields<'_> {
        self.insert(Name(b"Fields")).start()
    }

    /// Write the `/Flags` attribute to set the various characteristics of form
    /// action.
    pub fn form_flags(&mut self, flags: FormActionFlags) -> &mut Self {
        self.pair(Name(b"Flags"), flags.bits() as i32);
        self
    }

    /// Write the `/OP` attribute to set the operation to perform when the
    /// action is triggered.
    pub fn operation(&mut self, op: RenditionOperation) -> &mut Self {
        self.pair(Name(b"OP"), op as i32);
        self
    }

    /// Write the `/AN` attribute to provide a reference to the screen
    /// annotation for the operation. Required if OP is present.
    pub fn annotation(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"AN"), id);
        self
    }

    /// Start writing the `/R` dictionary. Only permissible for the subtype
    /// `Rendition`.
    pub fn rendition(&mut self) -> Rendition<'_> {
        self.insert(Name(b"R")).start()
    }
}

deref!('a, Action<'a> => Dict<'a>, dict);

/// The operation to perform when a rendition action is triggered.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RenditionOperation {
    /// Play the rendition specified by /R, and associating it with the
    /// annotation. If a rendition is already associated with the annotation, it
    /// shall be stopped, and the new rendition shall be associated with the
    /// annotation.
    Play = 0,
    /// Stop any rendition being played in association with the annotation.
    Stop = 1,
    /// Pause any rendition being played in association with the annotation.
    Pause = 2,
    /// Resume any rendition being played in association with the annotation.
    Resume = 3,
    /// Play the rendition specified by /R, and associating it with the
    /// annotation, or resume if a rendition is already associated.
    PlayOrResume = 4,
}

/// Writer for a _fields array_.
///
/// This struct is created by [`Action::fields`].
pub struct Fields<'a> {
    array: Array<'a>,
}

writer!(Fields: |obj| Self { array: obj.array() });

impl<'a> Fields<'a> {
    /// The indirect reference to the field.
    pub fn id(&mut self, id: Ref) -> &mut Self {
        self.array.item(id);
        self
    }

    /// The fully qualified name of the field. PDF 1.3+.
    pub fn name(&mut self, name: TextStr) -> &mut Self {
        self.array.item(name);
        self
    }
}

deref!('a, Fields<'a> => Array<'a>, array);

/// What kind of action to perform when clicking a link annotation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ActionType {
    /// Go to a destination in the document.
    GoTo,
    /// Go to a destination in another document.
    RemoteGoTo,
    /// Launch an application.
    Launch,
    /// Open a URI.
    Uri,
    /// Set an annotation's hidden flag. PDF 1.2+.
    SubmitForm,
    /// Set form fields to their default values. PDF 1.2+.
    ResetForm,
    /// Import form field values from a file. PDF 1.2+.
    ImportData,
    /// Execute a JavaScript action. PDF 1.2+.
    ///
    /// See Adobe's
    /// [JavaScript for Acrobat API Reference](https://opensource.adobe.com/dc-acrobat-sdk-docs/acrobatsdk/pdfs/acrobatsdk_jsapiref.pdf)
    /// and ISO 21757.
    JavaScript,
    /// A rendition action to control the playing of multimedia content. PDF 1.5+.
    Rendition,
}

impl ActionType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::GoTo => Name(b"GoTo"),
            Self::RemoteGoTo => Name(b"GoToR"),
            Self::Launch => Name(b"Launch"),
            Self::Uri => Name(b"URI"),
            Self::SubmitForm => Name(b"SubmitForm"),
            Self::ResetForm => Name(b"ResetForm"),
            Self::ImportData => Name(b"ImportData"),
            Self::JavaScript => Name(b"JavaScript"),
            Self::Rendition => Name(b"Rendition"),
        }
    }
}

bitflags::bitflags! {
    /// A set of flags specifying various characteristics of an [`Action`].
    pub struct FormActionFlags: u32 {
        /// Whether to include (unset) or exclude (set) the values in the
        /// `/Fields` attribute on form submission or reset. This Flag has very
        /// specific interacitons with other flags and fields, read the PDF 1.7
        /// spec for more info.
        const INCLUDE_EXCLUDE = 1;
        /// Fields shall be submitted regardless of if they have a value or
        /// not, otherwise they are excluded.
        const INCLUDE_NO_VALUE_FIELDS = 2;
        /// Export the fields as HTML instead of submitting as FDF. Ignored if
        /// `SUBMIT_PDF` or `XFDF` are set.
        const EXPORT_FORMAT = 1 << 3;
        /// Field name should be submitted using an HTTP GET request, otherwise
        /// POST. Should only be if `EXPORT_FORMAT` is also set.
        const GET_METHOD = 1 << 4;
        /// Include the coordinates of the mouse when submit was pressed. Should
        /// only be if `EXPORT_FORMAT` is also set.
        const SUBMIT_COORDINATES = 1 << 5;
        /// Submit field names and values as XFDF instead of submitting an FDF.
        /// Should not be set if `SUBMIT_PDF` is set. PDF1.4+.
        const XFDF = 1 << 6;
        /// Include all updates done to the PDF document in the submission FDF
        /// file. Should only be used when `XFDF` and `EXPORT_FORMAT` are not
        /// set. PDF 1.4+.
        const INCLUDE_APPEND_SAVES = 1 << 7;
        /// Include all markup annotations of the PDF dcoument in the submission
        /// FDF file. Should only be used when `XFDF` and `EXPORT_FORMAT` are
        /// not set. PDF 1.4+.
        const INCLUDE_ANNOTATIONS = 1 << 8;
        /// Submit the PDF file instead of an FDF file. All other flags other
        /// than `GET_METHOD` are ignored if this is set. PDF 1.4+.
        const SUBMIT_PDF = 1 << 9;
        /// Convert fields which represent dates into the
        /// [canonical date format](crate::types::Date). The interpretation of
        /// a form field as a date is is not specified in the field but the
        /// JavaScript code that processes it. PDF 1.4+.
        const CANONICAL_FORMAT = 1 << 10;
        /// Include only the markup annotations made by the current user (the
        /// `/T` entry of the annotation) as determined by the remote server
        /// the form will be submitted to. Should only be used when `XFDF` and
        /// `EXPORT_FORMAT` are not set and `INCLUDE_ANNOTATIONS` is set. PDF
        /// 1.4+.
        const EXCLUDE_NON_USER_ANNOTS = 1 << 11;
        /// Include the F entry in the FDF file.
        /// Should only be used when `XFDF` and `EXPORT_FORMAT` are not set.
        /// PDF 1.4+
        const EXCLUDE_F_KEY = 1 << 12;
        /// Include the PDF file as a stream in the FDF file that will be submitted.
        /// Should only be used when `XFDF` and `EXPORT_FORMAT` are not set.
        /// PDF 1.5+.
        const EMBED_FORM = 1 << 14;
    }
}

/// Writer for an _additional actions dictionary_.
///
/// This struct is created by [`Annotation::additional_actions`],
/// [`Field::additional_actions`], [`Page::additional_actions`] and
/// [`Catalog::additional_actions`].
pub struct AdditionalActions<'a> {
    dict: Dict<'a>,
}

writer!(AdditionalActions: |obj| Self { dict: obj.dict() });

/// Only permissible for [annotations](Annotation).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/E` dictionary. An action that shall be performed
    /// when the cursor enters the annotation's active area. Only permissible
    /// for annotations. PDF 1.2+.
    pub fn annot_curser_enter(&mut self) -> Action<'_> {
        self.insert(Name(b"E")).start()
    }

    /// Start writing the `/X` dictionary. An action that shall be performed
    /// when the cursor exits the annotation's active area. Only permissible for
    /// annotations. PDF 1.2+.
    pub fn annot_cursor_exit(&mut self) -> Action<'_> {
        self.insert(Name(b"X")).start()
    }

    /// Start writing the `/D` dictionary. This sets the action action
    /// that shall be performed when the mouse button is pressed inside the
    /// annotation's active area. Only permissible for annotations. PDF 1.2+.
    pub fn annot_mouse_press(&mut self) -> Action<'_> {
        self.insert(Name(b"D")).start()
    }

    /// Start writing the `/U` dictionary. This sets the action action that
    /// shall be performed when the mouse button is released inside the
    /// annotation's active area. Only permissible for annotations. PDF 1.2+.
    pub fn annot_mouse_release(&mut self) -> Action<'_> {
        self.insert(Name(b"U")).start()
    }

    /// Start writing the `/PO` dictionary. This sets the action action that
    /// shall be performed when the page containing the annotation is opened.
    /// Only permissible for annotations. PDF 1.5+.
    pub fn annot_page_open(&mut self) -> Action<'_> {
        self.insert(Name(b"PO")).start()
    }

    /// Start writing the `/PC` dictionary. This sets the action action that
    /// shall be performed when the page containing the annotation is closed.
    /// Only permissible for annotations. PDF 1.5+.
    pub fn annot_page_close(&mut self) -> Action<'_> {
        self.insert(Name(b"PV")).start()
    }

    /// Start writing the `/PV` dictionary. This sets the action action that
    /// shall be performed when the page containing the annotation becomes
    /// visible. Only permissible for annotations. PDF 1.5+.
    pub fn annot_page_visible(&mut self) -> Action<'_> {
        self.insert(Name(b"PV")).start()
    }

    /// Start writing the `/PI` dictionary. This sets the action action that
    /// shall be performed when the page containing the annotation is no longer
    /// visible in the conforming reader's user interface. Only permissible for
    /// annotations. PDF 1.5+.
    pub fn annot_page_invisible(&mut self) -> Action<'_> {
        self.insert(Name(b"PI")).start()
    }
}

/// Only permissible for [widget](crate::types::AnnotationType::Widget)
/// [annotations](Annotation).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/Fo` dictionary. This sets the action that shall be
    /// performed when the annotation receives the input focus. Only permissible
    /// for widget annotations. PDF 1.2+.
    pub fn widget_focus(&mut self) -> Action<'_> {
        self.insert(Name(b"Fo")).start()
    }

    /// Start writing the `/Bl` dictionary. This sets the action that shall be
    /// performed when the annotation loses the input focus. Only permissible
    /// for widget annotations. PDF 1.2+.
    pub fn widget_focus_loss(&mut self) -> Action<'_> {
        self.insert(Name(b"Bl")).start()
    }
}

/// Only permissible for [page objects](Page).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/O` dictionary. This sets the action that shall be
    /// performed when the page is opened. This action is independent of any
    /// that may be defined by the open action entry in the
    /// [document catalog](Catalog) and shall be executed after such an action.
    /// Only permissible for [page objects](Page). PDF 1.2+.
    pub fn page_open(&mut self) -> Action<'_> {
        self.insert(Name(b"O")).start()
    }

    /// Start writing the `/C` dictionary. This sets the action that shall
    /// be performed when the page is closed. This action applies to the page
    /// being closed and shall be executed before any other page is opened. Only
    /// permissible for [page objects](Page). PDF 1.2+.
    pub fn page_close(&mut self) -> Action<'_> {
        self.insert(Name(b"C")).start()
    }
}

/// Only permisible for form fields.
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/K` dictionary. This sets the JavaScript action that
    /// shall be performed when the user modifies a character in a text field
    /// or combo box or modifies the selection in a scrollable list box. This
    /// action may check the added text for validity and reject or modify it.
    /// Only permissible for form fields. PDF 1.3+.
    pub fn form_calculate_partial(&mut self) -> Action<'_> {
        self.insert(Name(b"K")).start()
    }

    /// Start writing the `/F` dictionary. This sets the JavaScript action
    /// that shall be performed before the field is formatted to display its
    /// value. This action may modify the field's value before formatting. Only
    /// permissible for form fields. PDF 1.3+.
    pub fn form_format(&mut self) -> Action<'_> {
        self.insert(Name(b"F")).start()
    }

    /// Start writing the `/V` dictionary. This sets the JavaScript action that
    /// shall be performed when the field's value is changed. This action may
    /// check the new value for validity. Only permissible for form fields.
    /// PDF 1.3+.
    pub fn form_validate(&mut self) -> Action<'_> {
        self.insert(Name(b"V")).start()
    }

    /// Start writing the `/C` dictionary. This sets the JavaScript action that
    /// shall be performed to recalculate the value of this field when that
    /// of another field changes. The order in which the document's fields are
    /// recalculated shall be defined by the `/CO` entry in the interactive form
    /// dictionary. Only permissible for form fields. PDF 1.3+.
    pub fn form_calculate(&mut self) -> Action<'_> {
        self.insert(Name(b"C")).start()
    }
}

/// Only permisible for [document catalog](Catalog).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/WC` dictionary. This sets the JavaScript action
    /// that shall be performed before closing a document. Only permissible for
    /// the [document catalog](Catalog) PDF 1.4+.
    pub fn cat_before_close(&mut self) -> Action<'_> {
        self.insert(Name(b"WC")).start()
    }

    /// Start writing the `/WS` dictionary. This sets the JavaScript action
    /// that shall be performed before saving a document. Only permissible for
    /// the [document catalog](Catalog) PDF 1.4+.
    pub fn cat_before_save(&mut self) -> Action<'_> {
        self.insert(Name(b"WS")).start()
    }

    /// Start writing the `/DS` dictionary. This sets the JavaScript action
    /// that shall be performed after saving a document. Only permissible for
    /// the [document catalog](Catalog) PDF 1.4+.
    pub fn cat_after_save(&mut self) -> Action<'_> {
        self.insert(Name(b"DS")).start()
    }

    /// Start writing the `/WP` dictionary. This sets the JavaScript action
    /// that shall be performed before printing a document. Only permissible for
    /// the [document catalog](Catalog) PDF 1.4+.
    pub fn cat_before_print(&mut self) -> Action<'_> {
        self.insert(Name(b"WP")).start()
    }

    /// Start writing the `/DP` dictionary. This sets the JavaScript action
    /// that shall be performed after printing a document. Only permissible for
    /// the [document catalog](Catalog) PDF 1.4+.
    pub fn cat_after_print(&mut self) -> Action<'_> {
        self.insert(Name(b"DP")).start()
    }
}

deref!('a, AdditionalActions<'a> => Dict<'a>, dict);
