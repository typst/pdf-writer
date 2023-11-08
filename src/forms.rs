use crate::types::AnnotationType;

use super::*;

/// Writer for an _interactive forms dictionary_. PDF 1.2+.
///
/// This struct is created by [`Catalog::form`].
pub struct Form<'a> {
    dict: Dict<'a>,
}

writer!(Form: |obj| Self { dict: obj.dict() });

impl<'a> Form<'a> {
    /// Write the `/Fields` attribute to reference the root [form fields](Field)
    /// (those who have no immediate parent) of this document.
    pub fn fields(&mut self, fields: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Fields")).array().items(fields);
        self
    }

    // TODO: deprecated in PDF 2.0

    /// Write the `/NeedAppearances` attribute to set whether to construct
    /// appearance streams and appearance dictionaries for all widget
    /// annotations in this document.
    pub fn need_appearances(&mut self, need: bool) -> &mut Self {
        self.pair(Name(b"NeedAppearances"), need);
        self
    }

    /// Write the `/SigFlags` attribute to set various document-level
    /// characteristics related to signature fields.
    pub fn sig_flags(&mut self, flags: SigFlags) -> &mut Self {
        self.pair(Name(b"SigFlags"), flags.bits() as i32);
        self
    }

    /// Write the `/CO` attribute to set the field dictionaries with calculation
    /// actions, defining the calculation order in which their values will be
    /// recalculated when the value of any field changes.
    pub fn calculation_order(
        &mut self,
        actions: impl IntoIterator<Item = Ref>,
    ) -> &mut Self {
        self.insert(Name(b"CO")).array().items(actions);
        self
    }

    /// Start writing the `/DR` attribute to set the default resources
    /// that shall be used by form field appearance streams. At a minimum, this
    /// dictionary shall contain a font entry specifying the resource name and
    /// font dictionary of the default font for displaying text.
    pub fn default_resources(&mut self) -> Resources<'_> {
        self.insert(Name(b"DR")).start()
    }

    /// Write the document-wide default value for the `/DA` attribute of
    /// fields containing variable text. See
    /// [`Field::vartext_default_appearance`].
    pub fn default_appearance(&mut self, default: Str) -> &mut Self {
        self.pair(Name(b"DA"), default);
        self
    }

    /// Write the document-wide default value for the `/Q` attribute of
    /// fields containing variable text. See [`Field::vartext_quadding`].
    pub fn quadding(&mut self, default: Quadding) -> &mut Self {
        self.pair(Name(b"Q"), default as i32);
        self
    }
}

deref!('a, Form<'a> => Dict<'a>, dict);

bitflags::bitflags! {
    /// Bitflags describing various document-level characteristics related to
    /// signature fields.
    pub struct SigFlags: u32 {
        /// The document contains at least one signature field.
        const SIGNATURES_EXIST = 1;

        /// The document contains signatures that may be invalidated if the
        /// file is saved (written) in a way that alters its previous contents,
        /// as opposed to an incremental update. Merely updating the file by
        /// appending new information to the end of the previous version is
        /// safe.
        const APPEND_ONLY = 2;
    }
}

/// Writer for an _ form field dictionary_.
///
/// This struct is created by [`Chunk::form_field`].
pub struct Field<'a> {
    dict: Dict<'a>,
}

writer!(Field: |obj| Self { dict: obj.dict() });

/// Permissible on all fields.
impl<'a> Field<'a> {
    /// Write the `/FT` attribute to set the type of this field.
    pub fn field_type(&mut self, typ: FieldType) -> &mut Self {
        self.pair(Name(b"FT"), typ.to_name());
        self
    }

    /// Write the `/Parent` attribute to set the immediate parent of this
    /// field.
    pub fn parent(&mut self, id: Ref) -> &mut Self {
        self.pair(Name(b"Parent"), id);
        self
    }

    /// Write the `/Kids` attribute to set the immediate children of this field.
    /// These references shall refer to other [fields][Field], or
    /// [widget](crate::types::AnnotationType::Widget) [annotations](Annotation).
    pub fn children(&mut self, children: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Kids")).array().items(children);
        self
    }

    /// Write the `/T` attribute to set the partial field name.
    ///
    /// The fully qualified field name of a field is a path along it's
    /// ancestor's partial field names separated by periods `.`. Therefore, a
    /// partial field name may not contain a period `.`.
    ///
    /// If two fields have the same parent and no partial field name, then they
    /// refer to two representations of the same field and should only differ
    /// in properties that specify their visual appearance. In particular, they
    /// should have the same `/FT`, `/V` and `/DV` attribute values.
    pub fn partial_name(&mut self, name: TextStr) -> &mut Self {
        self.pair(Name(b"T"), name);
        self
    }

    /// Write the `/TU` attribute to set the alternative field name. This
    /// field name is used in place of the actual field name whenever the field
    /// shall be identified in the user interface (such as in error or status
    /// messages). This text is also useful when extracting the document's
    /// contents in support of accessibility to users with disabilities or for
    /// other purposes. PDF 1.3+.
    pub fn alternate_name(&mut self, alternate: TextStr) -> &mut Self {
        self.pair(Name(b"TU"), alternate);
        self
    }

    /// Write the `/TM` attribute to set the mapping field name. This
    /// name shall be used when exporting interactive form field data from the
    /// document.
    pub fn mapping_name(&mut self, name: TextStr) -> &mut Self {
        self.pair(Name(b"TM"), name);
        self
    }

    /// Write the `/Ff` attribute to set various characteristics of this
    /// field.
    pub fn field_flags(&mut self, flags: FieldFlags) -> &mut Self {
        self.pair(Name(b"Tf"), flags.bits() as i32);
        self
    }

    /// Start writing the `/AA` dictionary to set the field's response to
    /// various trigger events.
    pub fn additional_actions(&mut self) -> AdditionalActions<'_> {
        self.insert(Name(b"AA")).start()
    }

    /// Finish writing this field as a widget annotation. This is encouraged
    /// for fields which are non-root and terminal (i.e. they have a parent and
    /// no children).
    ///
    /// While the widget annotation could be a single child to a
    /// terminal field, most readers will not correctly read the form
    /// field, if it's not merged with its annotation.
    pub fn to_annotation(self) -> Annotation<'a> {
        let mut annot = Annotation { dict: self.dict };
        annot.subtype(AnnotationType::Widget);
        annot
    }
}

deref!('a, Field<'a> => Dict<'a>, dict);

/// The type of a [`Field`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FieldType {
    /// A button field, includes push buttons, check boxes and radio buttons.
    Button,
    /// A text field, a box which a user can enter text into.
    Text,
    /// A choice field, list or combo boxes out of which the user may chose at
    /// most one.
    Choice,
    /// A signature field, fields which contain digital signatures and optional
    /// authentication data. PDF 1.3+.
    Signature,
}

impl FieldType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Button => Name(b"Btn"),
            Self::Text => Name(b"Tx"),
            Self::Choice => Name(b"Ch"),
            Self::Signature => Name(b"Sig"),
        }
    }
}

/// Only permissible on button fields.
impl<'a> Field<'a> {
    /// Write the `/Opt` array to set the export values of children of this
    /// field. Only permissible on checkbox fields, or radio button fields.
    /// PDF 1.4+.
    pub fn button_options<'b>(
        &mut self,
        options: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"Opt")).array().items(options);
        self
    }
}

/// Only permissible on check box fields.
impl<'a> Field<'a> {
    /// Write the `/V` attribute to set the state of this check box field. The
    /// state corresponds to an appearance stream in the [appearance
    /// dictionary](AppearanceCharacteristics) of this field's widget
    /// [annotation](Annotation). Only permissible on check box fields.
    pub fn checkbox_value(&mut self, state: CheckBoxState) -> &mut Self {
        self.pair(Name(b"V"), state.to_name());
        self
    }

    /// Write the `/DV` attribute to set the default state of this check box
    /// field. The state corresponds to an appearance stream in the [appearance
    /// dictionary](AppearanceCharacteristics) of this field's widget
    /// [annotation](Annotation). Only permissible on check box fields.
    pub fn checkbox_default_value(&mut self, state: CheckBoxState) -> &mut Self {
        self.pair(Name(b"DV"), state.to_name());
        self
    }
}

/// The state of a check box [`Field`].
pub enum CheckBoxState {
    /// The check box selected state `/Yes`.
    Yes,
    /// The check box unselected state `/Off`.
    Off,
}

impl CheckBoxState {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Yes => Name(b"Yes"),
            Self::Off => Name(b"Off"),
        }
    }
}

/// Only permissible on radio button fields.
impl<'a> Field<'a> {
    /// Write the `/V` attribute to set the state of this radio button field.
    /// The state corresponds to an appearance stream in the
    /// [appearance subdictionary](Appearance) of this field's widget
    /// [annotation](Annotation) and is either a custom name unique for
    /// all unique fields, or `/Off`. Only permissible on radio button fields.
    pub fn radio_value(&mut self, state: Name) -> &mut Self {
        self.pair(Name(b"V"), state);
        self
    }

    /// Write the `/DV` attribute to set the default state of this radio button
    /// field. The state corresponds to an appearance stream in the
    /// [appearance subdictionary](Appearance) of this field's widget
    /// [annotation](Annotation) and is either a custom name unique for
    /// all unique fields, or `/Off`. Only permissible on radio button fields.
    pub fn radio_default_value(&mut self, state: Name) -> &mut Self {
        self.pair(Name(b"DV"), state);
        self
    }
}

/// Only permissible on text fields.
impl<'a> Field<'a> {
    // TODO: the spec likely means the equivalent of unicode graphemes here
    //       for characters

    /// Write the `/MaxLen` attribute to set the maximum length of the field's
    /// text in characters. Only permissible on text fields.
    pub fn text_max_len(&mut self, len: i32) -> &mut Self {
        self.pair(Name(b"MaxLen"), len);
        self
    }

    /// Write the `/V` attribute to set the value of this text field.
    /// Only permissible on text fields.
    pub fn text_value(&mut self, value: TextStr) -> &mut Self {
        self.pair(Name(b"V"), value);
        self
    }

    /// Start writing the `/DV` attribute to set the default value of this text
    /// field. Only permissible on text fields.
    pub fn text_default_value(&mut self, value: TextStr) -> &mut Self {
        self.pair(Name(b"DV"), value);
        self
    }
}

/// Only permissible on fields containing variable text.
impl<'a> Field<'a> {
    /// Write the `/DA` attribute containing a sequence of valid page-content
    /// graphics or text state operators that define such properties as the
    /// field's text size and colour. Only permissible on fields containing
    /// variable text.
    pub fn vartext_default_appearance(&mut self, appearance: Str) -> &mut Self {
        self.pair(Name(b"DA"), appearance);
        self
    }

    /// Write the `/Q` attribute to set the quadding (justification) that shall
    /// be used in dispalying the text. Only permissible on fields containing
    /// variable text.
    pub fn vartext_quadding(&mut self, quadding: Quadding) -> &mut Self {
        self.pair(Name(b"Q"), quadding as i32);
        self
    }

    /// Write the `/DS` attribute to set the default style string. Only
    /// permissible on fields containing variable text. PDF 1.5+.
    pub fn vartext_default_style(&mut self, style: TextStr) -> &mut Self {
        self.pair(Name(b"DS"), style);
        self
    }

    /// Write the `/RV` attribute to set the value of this variable text field.
    /// Only permissible on fields containing variable text. PDF 1.5+.
    pub fn vartext_rich_value(&mut self, value: TextStr) -> &mut Self {
        self.pair(Name(b"RV"), value);
        self
    }
}

/// The quadding (justification) of a field containing variable text.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Quadding {
    /// Left justify the text.
    Left = 0,
    /// Center justify the text.
    Center = 1,
    /// Right justify the text.
    Right = 2,
}

/// Only permissible on choice fields.
impl<'a> Field<'a> {
    /// Start writing the `/Opt` array to set the options that shall be
    /// presented to the user.
    pub fn choice_options(&mut self) -> ChoiceOptions<'_> {
        self.insert(Name(b"Opt")).start()
    }

    /// Write the `/TI` attribute to set the index in the
    /// [`Field::choice_options`] array of the first visible option for
    /// scrollable lists.
    pub fn choice_top_index(&mut self, index: i32) -> &mut Self {
        self.pair(Name(b"TI"), index);
        self
    }

    /// Write the `/I` array to set the indices of the currently selected
    /// options. The integers in this array must be sorted in ascending order
    /// and correspond to 0-based indices in the [`Field::choice_options`]
    /// array.
    ///
    /// This entry shall be used for choice fields which allow multiple
    /// selections ([`FieldFlags::MULTI_SELECT`]). This means when two or more
    /// elements in the [`Field::choice_options`] array have different names
    /// but export the same value or when the value fo the choice field is an
    /// array. This entry should not be used for choice fields that do not allow
    /// multiple selections. PDF 1.4+.
    pub fn choice_indices(
        &mut self,
        indices: impl IntoIterator<Item = i32>,
    ) -> &mut Self {
        self.insert(Name(b"I")).array().items(indices);
        self
    }

    /// Write the `/V` attribute to set the currently selected values
    /// of this choice field. Should be one of the values given in
    /// [`Self::choice_options`] or `None` if no choice is selected. Only
    /// permissible on choice fields.
    pub fn choice_value(&mut self, option: Option<TextStr>) -> &mut Self {
        match option {
            Some(value) => self.pair(Name(b"V"), value),
            None => self.pair(Name(b"V"), Null),
        };
        self
    }

    /// Write the `/V` attribute to set the currently selected values of this
    /// choice field. See also [`Self::choice_value`], for a single or no value.
    /// Only permissible on choice fields.
    pub fn choice_values<'b>(
        &mut self,
        options: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"V")).array().items(options);
        self
    }

    /// Write the `/DV` attribute to set the default selected value
    /// of this choice field. Should be one of the values given in
    /// [`Self::choice_options`] or `None` if no choice is selected. Only
    /// permissible on choice fields.
    pub fn choice_default_value(&mut self, option: Option<TextStr>) -> &mut Self {
        match option {
            Some(value) => self.pair(Name(b"DV"), value),
            None => self.pair(Name(b"DV"), Null),
        };
        self
    }

    /// Write the `/DV` attribute to set the default selected values of this
    /// choice field. See also [`Self::choice_default_value`], for a single or
    /// no value. Only permissible on choice fields.
    pub fn choice_default_values<'b>(
        &mut self,
        options: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"DV")).array().items(options);
        self
    }
}

/// Writer for a _choice options array_.
///
/// This struct is created by [`Field::choice_options`].
pub struct ChoiceOptions<'a> {
    array: Array<'a>,
}

writer!(ChoiceOptions: |obj| Self { array: obj.array() });

impl<'a> ChoiceOptions<'a> {
    /// Add an option with the given value.
    pub fn option(&mut self, value: TextStr) -> &mut Self {
        self.array.item(value);
        self
    }

    /// Add an option with the given value and export value.
    pub fn export(&mut self, value: TextStr, export_value: TextStr) -> &mut Self {
        self.array.push().array().items([export_value, value]);
        self
    }
}

deref!('a, ChoiceOptions<'a> => Array<'a>, array);

bitflags::bitflags! {
    /// Bitflags describing various characteristics of a form field.
    pub struct FieldFlags: u32 {
        /// The user may not change the value of the field. Any associated
        /// widget annotations will not interact with the user; that is, they
        /// will not respond to mouse clicks or change their appearance in
        /// response to mouse motions. This flag is useful for fields whose
        /// values are computed or imported from a database.
        const READ_ONLY = 1;
        /// The field shall have a value at the time it is exported by a
        /// [submit-form](crate::types::ActionType::SubmitForm)[`Action`].
        const REQUIRED = 2;
        /// The field shall not be exported by a
        /// [submit-form](crate::types::ActionType::SubmitForm)[`Action`].
        const NO_EXPORT = 1 << 2;
        /// The entered text shall not be spell-checked, can be used for text
        /// and choice fields.
        const DO_NOT_SPELL_CHECK = 1 << 22;

        // Button specific flags

        /// Exactly one radio button shall be selected at all times; selecting
        /// the currently selected button has no effect. If unset, clicking
        /// the selected button deselects it, leaving no button selected. Only
        /// permissible for radio buttons.
        const NO_TOGGLE_TO_OFF = 1 << 14;
        /// The field is a set of radio buttons; if clear, the field is a check
        /// box. This flag may be set only if the `PUSHBUTTON` flag is unset.
        const RADIO = 1 << 15;
        /// The field is a push button that does not retain a permanent
        /// value.
        const PUSHBUTTON = 1 << 16;
        /// A group of radio buttons within a radio button field that use the
        /// same value for the on state will turn on and off in unison; that
        /// is if one is checked, they are all checked. If unset, the buttons
        /// are mutually exclusive (the same behavior as HTML radio buttons).
        /// PDF 1.5+.
        const RADIOS_IN_UNISON = 1 << 25;

        // Text field specific flags

        /// The text may contain multiple lines of text, otherwise the text is
        /// restricted to one line.
        const MULTILINE = 1 << 12;
        /// The text contains a password and should not be echoed visibly to
        /// the screen.
        const PASSWORD = 1 << 13;
        /// The entered text represents a path to a file who's contents shall be
        /// submitted as the value of the field. PDF 1.4+.
        const FILE_SELECT = 1 << 20;
        /// The field shall not scroll horizontally (for single-line) or
        /// vertically (for multi-line) to accomodate more text. Once the field
        /// is full, no further text shall be accepted for interactive form
        /// filling; for non-interactive form filling, the filler should take
        /// care not to add more character than will visibly fit in the defined
        /// area. PDF 1.4+.
        const DO_NOT_SCROLL = 1 << 23;
        /// The field shall eb automatically divided into as many equally
        /// spaced postions or _combs_ as the value of [`Field::max_len`]
        /// and the text is layed out into these combs. May only be set if
        /// the [`Field::max_len`] property is set and if the [`MULTILINE`],
        /// [`PASSWORD`] and [`FILE_SELECT`] flags are clear. PDF 1.5+.
        const COMB = 1 << 24;
        /// The value of this field shall be a rich text string. If the field
        /// has a value, the [`TextField::rich_text_value`] shall specify the
        /// rich text string. PDF 1.5+.
        const RICH_TEXT = 1 << 25;

        // Choice field specific flags

        /// The field is a combo box if set, else it's a list box.
        const COMBO = 1 << 17;
        /// The combo box shall include an editable text box as well as a
        /// drop-down list. Shall only be used if [`COMBO`] is set.
        const EDIT = 1 << 18;
        /// The field’s option items shall be sorted alphabetically. This
        /// flag is intended for use by writers, not by readers.
        const SORT = 1 << 19;
        /// More than one option of the choice field may be selected
        /// simultaneously. PDF 1.4+.
        const MULTI_SELECT = 1 << 21;
        /// The new value shall be committed as soon as a selection is made
        /// (commonly with the mouse). In this case, supplying a value for
        /// a field involves three actions: selecting the field for fill-in,
        /// selecting a choice for the fill-in value, and leaving that field,
        /// which finalizes or "commits" the data choice and triggers any
        /// actions associated with the entry or changing of this data.
        ///
        /// If set, processing does not wait for leaving the field action to
        /// occur, but immediately proceeds to the third step. PDF 1.5+.
        const COMMIT_ON_SEL_CHANGE = 1 << 26;
    }
}
