use super::*;
use crate::object::TextStrLike;
use crate::types::AnnotationType;

/// Writer for an _interactive forms dictionary_. PDF 1.2+.
///
/// This struct is created by [`Catalog::form`].
pub struct Form<'a> {
    dict: Dict<'a>,
}

writer!(Form: |obj| Self { dict: obj.dict() });

impl Form<'_> {
    /// Write the `/Fields` attribute to reference the root [form fields](Field)
    /// (those who have no immediate parent) of this document.
    pub fn fields(&mut self, fields: impl IntoIterator<Item = Ref>) -> &mut Self {
        self.insert(Name(b"Fields")).array().items(fields);
        self
    }

    /// Write the `/SigFlags` attribute to set various document-level
    /// characteristics related to signature fields. PDF 1.3+.
    pub fn sig_flags(&mut self, flags: SigFlags) -> &mut Self {
        self.pair(Name(b"SigFlags"), flags.bits() as i32);
        self
    }

    /// Write the `/CO` attribute to set the field dictionaries with calculation
    /// actions, defining the calculation order in which their values will be
    /// recalculated when the value of any field changes. PDF 1.3+.
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

/// Writer for an _form field dictionary_.
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
    /// The fully qualified field name of a field is a path along its
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
    pub fn alternate_name(&mut self, alternate: impl TextStrLike) -> &mut Self {
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
        self.pair(Name(b"Ff"), flags.bits() as i32);
        self
    }

    /// Start writing the `/AA` dictionary to set the field's response to
    /// various trigger events.
    ///
    /// Note that this attribute is forbidden in PDF/A.
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
    pub fn into_annotation(mut self) -> Annotation<'a> {
        self.dict.pair(Name(b"Type"), Name(b"Annot"));
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
impl Field<'_> {
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
impl Field<'_> {
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
impl Field<'_> {
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
impl Field<'_> {
    /// Write the `/MaxLen` attribute to set the maximum length of the fields
    /// text in characters. Only permissible on text fields.
    ///
    /// The definition of a character depends on the encoding of the content of
    /// `/V`. Which is either one byte for PDFDocEncoding or 2 for UTF16-BE.
    pub fn text_max_len(&mut self, len: i32) -> &mut Self {
        self.pair(Name(b"MaxLen"), len);
        self
    }

    /// Write the `/V` attribute to set the value of this text field.
    /// Only permissible on text fields.
    pub fn text_value(&mut self, value: impl TextStrLike) -> &mut Self {
        self.pair(Name(b"V"), value);
        self
    }

    /// Start writing the `/DV` attribute to set the default value of this text
    /// field. Only permissible on text fields.
    pub fn text_default_value(&mut self, value: impl TextStrLike) -> &mut Self {
        self.pair(Name(b"DV"), value);
        self
    }
}

/// Only permissible on fields containing variable text.
impl Field<'_> {
    /// Write the `/DA` attribute containing a sequence of valid page-content
    /// graphics or text state operators that define such properties as the
    /// field's text size and colour. Only permissible on fields containing
    /// variable text.
    pub fn vartext_default_appearance(&mut self, appearance: Str) -> &mut Self {
        self.pair(Name(b"DA"), appearance);
        self
    }

    /// Write the `/Q` attribute to set the quadding (justification) that shall
    /// be used in displaying the text. Only permissible on fields containing
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
    ///
    /// Note that this is a Rich Text string, so you can use some basic XHTML
    /// and XFA attributes. That also means that untrusted input must be
    /// properly escaped.
    pub fn vartext_rich_value(&mut self, value: impl TextStrLike) -> &mut Self {
        self.pair(Name(b"RV"), value);
        self
    }
}

/// The quadding (justification) of a field containing variable text.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub enum Quadding {
    /// Left justify the text.
    #[default]
    Left = 0,
    /// Center justify the text.
    Center = 1,
    /// Right justify the text.
    Right = 2,
}

/// Only permissible on choice fields.
impl Field<'_> {
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

impl ChoiceOptions<'_> {
    /// Add an option with the given value.
    pub fn option(&mut self, value: impl TextStrLike) -> &mut Self {
        self.array.item(value);
        self
    }

    /// Add options with the given values.
    pub fn options(
        &mut self,
        values: impl IntoIterator<Item = impl TextStrLike>,
    ) -> &mut Self {
        self.array.items(values);
        self
    }

    /// Add an option with the given value and export value.
    pub fn export(
        &mut self,
        value: impl TextStrLike,
        export_value: TextStr,
    ) -> &mut Self {
        {
            let mut array = self.array.push().array();
            array.item(export_value);
            array.item(value);
        }
        self
    }

    /// Add options with the given pairs of value and export value.
    pub fn exports<'b>(
        &mut self,
        values: impl IntoIterator<Item = (impl TextStrLike, TextStr<'b>)>,
    ) -> &mut Self {
        for (value, export) in values {
            self.export(value, export);
        }
        self
    }
}

deref!('a, ChoiceOptions<'a> => Array<'a>, array);

/// Only permissible on signature fields.
impl Field<'_> {
    /// Write the `/Lock` attribute which points to a [signature field lock
    /// dictionary](SignatureFieldLock), containing a set of form fields that shall
    /// be locked when this signature field is signed. PDF 1.5+.
    pub fn signature_field_lock(&mut self, field_lock: Ref) -> &mut Self {
        self.pair(Name(b"Lock"), field_lock);
        self
    }

    /// Write the `/SV` attribute which points to a [seed value
    /// dictionary](SignatureSeedValue), containing constraints for the
    /// properties of a signature that is applied to this field. PDF 1.5+.
    pub fn signature_seed_value(&mut self, seed_value: Ref) -> &mut Self {
        self.pair(Name(b"SV"), seed_value);
        self
    }
}

/// Writer for a _signature field lock dictionary_.
///
/// This struct is created by [`Chunk::signature_field_lock`].
pub struct SignatureFieldLock<'a> {
    dict: Dict<'a>,
}

writer!(SignatureFieldLock: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"SigFieldLock"));
    Self { dict }
});

impl SignatureFieldLock<'_> {
    /// Write the `/Action` attribute which, in conjunction with
    /// the [`/Fields`](SignatureFieldLock::fields) array, indicates the
    /// set of fields that should be locked when the signature field is signed.
    /// Required.
    pub fn action(&mut self, action: SignatureLockAction) -> &mut Self {
        self.dict.pair(Name(b"Action"), action.to_name());
        self
    }

    /// Write the `/Fields` array which contains field names,
    /// indicating the set of fields that should be locked when the
    /// signature field is signed.
    /// The behavior of this array depends on the value of
    /// the [`/Action`](SignatureFieldLock::action) attribute.
    /// Required if action is not [`SignatureLockAction::All`].
    pub fn fields<'b>(
        &mut self,
        fields: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.dict.insert(Name(b"Fields")).array().items(fields);
        self
    }

    /// Write the `/P` attribute to set the changes allowed without
    /// invalidating the signature.
    pub fn access_permissions(&mut self, p: AccessPermissions) -> &mut Self {
        self.dict.pair(Name(b"P"), p.to_int());
        self
    }
}

deref!('a, SignatureFieldLock<'a> => Dict<'a>, dict);

/// In conjunction with the [`/Fields`](SignatureFieldLock::fields) array,
/// indicates the set of fields that should be locked when the signature
/// field is signed.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SignatureLockAction {
    /// All fields in the document.
    All,
    /// All fields specified in the [`/Fields`](SignatureFieldLock::fields) array.
    Include,
    /// All fields except those specified in the
    /// [`/Fields`](SignatureFieldLock::fields) array.
    Exclude,
}

impl SignatureLockAction {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::All => Name(b"All"),
            Self::Include => Name(b"Include"),
            Self::Exclude => Name(b"Exclude"),
        }
    }
}

/// Writer for a _seed value dictionary_.
///
/// This struct is created by [`Chunk::signature_seed_value`].
pub struct SignatureSeedValue<'a> {
    dict: Dict<'a>,
}

writer!(SignatureSeedValue: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"SV"));
    Self { dict }
});

impl SignatureSeedValue<'_> {
    /// Write the `/Ff` attribute to set which entries of this signature
    /// seed value dictionary are mandatory constraints.
    pub fn constraint_flags(&mut self, flags: SignatureSeedValueFlags) -> &mut Self {
        self.pair(Name(b"Ff"), flags.bits() as i32);
        self
    }

    /// Write the `/Filter` attribute to set which signature handler
    /// shall be used to sign the signature field.
    /// If [`SignatureSeedValueFlags::FILTER`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags),
    /// this is a required constraint.
    pub fn filter(&mut self, signature_handler: Name) -> &mut Self {
        self.pair(Name(b"Filter"), signature_handler);
        self
    }

    /// Write the `/Filter` attribute to set which encodings
    /// to use when signing. One of these must be used if
    /// [`SignatureSeedValueFlags::SUB_FILTER`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags).
    pub fn sub_filter<'b>(
        &mut self,
        encodings: impl IntoIterator<Item = Name<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"SubFilter")).array().items(encodings);
        self
    }

    /// Write the `/DigestMethod` attribute to set which digest algorithms
    /// to use while signing. One of these must be used if
    /// [`SignatureSeedValueFlags::DIGEST_METHOD`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags).
    /// PDF 1.7+.
    pub fn digest_method(
        &mut self,
        digest_method: impl IntoIterator<Item = SignatureDigestMethod>,
    ) -> &mut Self {
        self.insert(Name(b"DigestMethod"))
            .array()
            .items(digest_method.into_iter().map(SignatureDigestMethod::to_name));
        self
    }

    /// Write the `/V` attribute to set the minimum required capability
    /// of the signature field seed value dictionary parser.
    /// If [`SignatureSeedValueFlags::VERSION`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags),
    /// this is a required constraint.
    pub fn version(&mut self, version: SeedValueParserVersion) -> &mut Self {
        self.pair(Name(b"V"), version.to_int());
        self
    }

    /// Start writing the `/Cert` dictionary to set various characteristics
    /// of the certificate that shall be used when signing.
    pub fn certificate(&mut self) -> CertificateSeedValue<'_> {
        self.insert(Name(b"Cert")).start()
    }

    /// Write the `/Reasons` attribute to set the possible reasons
    /// for signing a document.
    /// If [`SignatureSeedValueFlags::REASONS`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags),
    /// one of these reasons must be used when signing.
    pub fn reasons<'b>(
        &mut self,
        reasons: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"Reasons")).array().items(reasons);
        self
    }

    /// Start writing the `/MDP` dictionary to set which changes
    /// to the document will invalidate the signature. PDF 1.6+.
    pub fn modification_detection_prevention(
        &mut self,
    ) -> SignatureModificationDetectionPrevention<'_> {
        self.insert(Name(b"MDP")).start()
    }

    /// Start writing the `/TimeStamp` dictionary to set a
    /// timestamping server and whether a timestamp is required
    /// while signing. PDF 1.6+.
    pub fn timestamp(&mut self) -> SignatureTimeStamp<'_> {
        self.insert(Name(b"TimeStamp")).start()
    }

    /// Write the `/LegalAttestation` attribute to set the possible
    /// legal attestations.
    /// If [`SignatureSeedValueFlags::LEGAL_ATTESTATION`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags),
    /// one of these must be used when signing.
    pub fn legal_attestation<'b>(
        &mut self,
        legal_attestations: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"LegalAttestation"))
            .array()
            .items(legal_attestations);
        self
    }

    /// Write the `/AddRevInfo` attribute to set whether revocation
    /// checking shall be carried out.
    /// If [`SignatureSeedValueFlags::ADD_REV_INFO`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags)
    /// and revocation checking fails, then signing shall fail.
    pub fn add_rev_info(&mut self, check_revocation: bool) -> &mut Self {
        self.pair(Name(b"AddRevInfo"), check_revocation);
        self
    }

    /// Write the `/LockDocument` attribute to set the author's intent for
    /// whether the signing dialog should allow the user to lock the document
    /// at the time of signing.
    /// Unless this is set to [`LockingIntent::Auto`], the user cannot
    /// override this if [`SignatureSeedValueFlags::LOCK_DOCUMENT`] is present
    /// on [constraint flags](SignatureSeedValue::constraint_flags).
    /// PDF 2.0+.
    pub fn lock_document(&mut self, intent: LockingIntent) -> &mut Self {
        self.pair(Name(b"LockDocument"), intent.to_name());
        self
    }

    /// Write the `/AppearanceFilter` attribute to set the appearance that
    /// shall be used when signing the signature field.
    /// If [`SignatureSeedValueFlags::APPEARANCE_FILTER`] is present on
    /// [constraint flags](SignatureSeedValue::constraint_flags),
    /// the appearance must be available and used to sign the document.
    /// PDF 2.0+.
    pub fn appearance_filter(&mut self, signature_appearance: TextStr<'_>) -> &mut Self {
        self.pair(Name(b"LockDocument"), signature_appearance);
        self
    }
}

deref!('a, SignatureSeedValue<'a> => Dict<'a>, dict);

bitflags::bitflags! {
    /// Bitflags describing the whether a specific entry in a
    /// [seed value dictionary](SignatureSeedValue) is required (1)
    /// or optional (0).
    pub struct SignatureSeedValueFlags: u32 {
        /// The `/Filter` entry is a mandatory constraint.
        const FILTER = 1 << 0;
        /// The `/SubFilter` entry is a mandatory constraint.
        const SUB_FILTER = 1 << 1;
        /// The `/V` entry is a mandatory constraint.
        const VERSION = 1 << 2;
        /// The `/Reasons` entry is a mandatory constraint.
        const REASONS = 1 << 3;
        /// The `/LegalAttestation` entry is a mandatory constraint. PDF 1.6+.
        const LEGAL_ATTESTATION = 1 << 4;
        /// The `/AddRevInfo` entry is a mandatory constraint. PDF 1.7+.
        const ADD_REV_INFO = 1 << 5;
        /// The `/DigestMethod` entry is a mandatory constraint. PDF 1.7+.
        const DIGEST_METHOD = 1 << 6;
        /// The `/LockDocument` entry is a mandatory constraint. PDF 2.0+.
        const LOCK_DOCUMENT = 1 << 7;
        /// The `/AppearanceFilter` entry is a mandatory constraint. PDF 2.0+.
        const APPEARANCE_FILTER = 1 << 8;
    }
}

/// The digest algorithm of a signature.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SignatureDigestMethod {
    /// The SHA-1 digest algorithm. Deprecated in PDF 2.0.
    SHA1,
    /// The SHA-256 digest algorithm.
    SHA256,
    /// The SHA-384 digest algorithm.
    SHA384,
    /// The SHA-512 digest algorithm.
    SHA512,
    /// The RIPEMD-160 digest algorithm.
    RIPEMD160,
}

impl SignatureDigestMethod {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::SHA1 => Name(b"SHA1"),
            Self::SHA256 => Name(b"SHA256"),
            Self::SHA384 => Name(b"SHA384"),
            Self::SHA512 => Name(b"SHA512"),
            Self::RIPEMD160 => Name(b"RIPEMD160"),
        }
    }
}

/// The capability of a signature field seed value
/// dictionary parser.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SeedValueParserVersion {
    /// The parser can recognize all seed value dictionary entries
    /// in a PDF 1.5 file.
    Pdf15,
    /// The parser can recognize all seed value dictionary entries
    /// specified for PDF 1.7 or earlier.
    Pdf17,
    /// The parser can recognize all seed value dictionary entries
    /// specified for PDF 2.0 or earlier.
    Pdf20,
}

impl SeedValueParserVersion {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Pdf15 => 1,
            Self::Pdf17 => 2,
            Self::Pdf20 => 3,
        }
    }
}

/// Writer for the `/MDP` dictionary in the
/// [_signature seed value dictionary_](SignatureSeedValue).
///
/// This struct is created by [`SignatureSeedValue::modification_detection_prevention`].
pub struct SignatureModificationDetectionPrevention<'a> {
    dict: Dict<'a>,
}

writer!(SignatureModificationDetectionPrevention: |obj| Self { dict: obj.dict() });

impl SignatureModificationDetectionPrevention<'_> {
    /// Write the `/P` attribute to set the type of signature of
    /// this field and the changes allowed without invalidating the signature.
    pub fn signature_type(&mut self, p: MdpSignatureType) -> &mut Self {
        self.dict.pair(Name(b"P"), p.to_int());
        self
    }
}

deref!('a, SignatureModificationDetectionPrevention<'a> => Dict<'a>, dict);

/// The access permissions granted for the document
/// without invalidating the signature.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AccessPermissions {
    /// No changes to the document are permitted.
    NoChanges,
    /// Permitted changes shall be filling in forms, instantiating page
    ///templates, and signing.
    FillAndSign,
    /// Permitted changes are the same as for [`AccessPermissions::FillAndSign`],
    /// as well as annotation creation, deletion, and modification.
    Annotations,
}

impl AccessPermissions {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::NoChanges => 1,
            Self::FillAndSign => 2,
            Self::Annotations => 3,
        }
    }
}

/// The signature type to be used when signing a signature field.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MdpSignatureType {
    /// The signature shall be an approval signature.
    ApprovalSignature,
    /// The signature shell be a certification signature
    /// with the given access permissions.
    CertificationSignature(AccessPermissions),
}

impl MdpSignatureType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::ApprovalSignature => 0,
            Self::CertificationSignature(permission) => permission.to_int(),
        }
    }
}

/// Writer for the `/TimeStamp` dictionary in the
/// [_signature seed value dictionary_](SignatureSeedValue).
///
/// This struct is created by [`SignatureSeedValue::timestamp`].
pub struct SignatureTimeStamp<'a> {
    dict: Dict<'a>,
}

writer!(SignatureTimeStamp: |obj| Self { dict: obj.dict() });

impl SignatureTimeStamp<'_> {
    /// Write the `/URL` attribute to set the URL of the timestamping
    /// server to use when signing.
    pub fn url(&mut self, url: Str<'_>) -> &mut Self {
        self.dict.pair(Name(b"URL"), url);
        self
    }
    /// Write the `/Ff` attribute to set whether the signature
    /// must have a timestamp.
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.dict.pair(Name(b"Ff"), required as i32);
        self
    }
}

deref!('a, SignatureTimeStamp<'a> => Dict<'a>, dict);

/// The author's intent for whether the signing dialog
/// should allow the user to lock the document at the time
/// of signing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LockingIntent {
    /// Document should be locked at the time of signing.
    Lock,
    /// Document should not be locked at the time of signing.
    DoNotLock,
    /// The consuming application decides.
    Auto,
}

impl LockingIntent {
    pub(crate) fn to_name<'b>(self) -> Name<'b> {
        match self {
            Self::Lock => Name(b"true"),
            Self::DoNotLock => Name(b"false"),
            Self::Auto => Name(b"auto"),
        }
    }
}

/// Writer for a _certificate seed value dictionary_.
///
/// This struct is created by [`SignatureSeedValue::certificate`].
pub struct CertificateSeedValue<'a> {
    dict: Dict<'a>,
}

writer!(CertificateSeedValue: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"SVCert"));
    Self { dict }
});

impl CertificateSeedValue<'_> {
    /// Write the `/Ff` attribute to set which entries of this certificate
    /// seed value dictionary are mandatory constraints.
    pub fn constraint_flags(&mut self, flags: CertificateSeedValueFlags) -> &mut Self {
        self.pair(Name(b"Ff"), flags.bits() as i32);
        self
    }

    /// Write the `/Subject` attribute to set which DER-encoded X.509v3
    /// certificates are acceptable for signing.
    /// If [`CertificateSeedValueFlags::SUBJECT`] is present on
    /// [constraint flags](CertificateSeedValue::constraint_flags),
    /// this is a required constraint.
    pub fn subject<'b>(
        &mut self,
        certificates: impl IntoIterator<Item = Str<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"Subject")).array().items(certificates);
        self
    }

    /// Write the `/SignaturePolicyOID` attribute to set the OID of
    /// the signature policy to use when signing. PDF 2.0+.
    pub fn signature_policy_oid(&mut self, oid: Str<'_>) -> &mut Self {
        self.pair(Name(b"SignaturePolicyOID"), oid);
        self
    }

    /// Write the `/SignaturePolicyHashValue` attribute to set the
    /// computed hash value of the signature policy, according to the
    /// hash algorithm specified by the
    /// [`/SignaturePolicyHashAlgorithm`](CertificateSeedValue::signature_policy_hash_algorithm)
    /// attribute. PDF 2.0+.
    pub fn signature_policy_hash_value(&mut self, hash_value: Str<'_>) -> &mut Self {
        self.pair(Name(b"SignaturePolicyHashValue"), hash_value);
        self
    }

    /// Write the `/SignaturePolicyHashAlgorithm` attribute to set the
    /// hash function used to compute the value of the
    /// [`/SignaturePolicyHashValue`](CertificateSeedValue::signature_policy_hash_value)
    /// attribute. PDF 2.0+.
    pub fn signature_policy_hash_algorithm(
        &mut self,
        hash_algorithm: Name<'_>,
    ) -> &mut Self {
        self.pair(Name(b"SignaturePolicyHashAlgorithm"), hash_algorithm);
        self
    }

    /// Write the `/SignaturePolicyCommitmentType` attribute to set the
    /// commitment types that may be used within the signature policy,
    /// if the [`/SignaturePolicyOID`](CertificateSeedValue::signature_policy_oid)
    /// attribute is present.
    /// An empty string may be used to indicate that all commitments defined
    /// by the signature policy may be used. PDF 2.0+.
    pub fn signature_policy_commitment_type<'b>(
        &mut self,
        commitment_types: impl IntoIterator<Item = Str<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"SignaturePolicyHashAlgorithm"))
            .array()
            .items(commitment_types);
        self
    }

    /// Start writing the `/SubjectDN` array of dictionaries, each
    /// specifying a Subject Distinguished Name (DN) that shall be
    /// present within the certificate for it to be acceptable for
    /// signing.
    /// Attribute names shall contain characters in the set `a-z`,
    /// `A-Z` `0-9` and `PERIOD`.
    /// If [`CertificateSeedValueFlags::SUBJECT_DN`] is present on
    /// [constraint flags](CertificateSeedValue::constraint_flags),
    /// this is a required constraint.
    /// PDF 1.7+.
    pub fn subject_dn(&mut self) -> TypedArray<'_, Dict<'_>> {
        self.insert(Name(b"SubjectDN")).array().typed()
    }

    /// Write the `/KeyUsage` attribute to set which key-usage extensions
    /// shall be present in the signing certificate.
    /// If [`CertificateSeedValueFlags::KEY_USAGE`] is present on
    /// [constraint flags](CertificateSeedValue::constraint_flags),
    /// this is a required constraint.
    /// PDF 1.7+.
    pub fn key_usage(
        &mut self,
        key_usages: impl IntoIterator<Item = CertificateKeyUsage>,
    ) -> &mut Self {
        let mut array = self.insert(Name(b"KeyUsage")).array();
        key_usages.into_iter().for_each(|usage| {
            array.item(Str(&usage.into_string()));
        });
        array.finish();
        self
    }

    /// Write the `/Issuer` attribute to set which issuers' DER-encoded
    /// X.509v3 certificates are acceptable for signing.
    /// If [`CertificateSeedValueFlags::ISSUER`] is present on
    /// [constraint flags](CertificateSeedValue::constraint_flags),
    /// this is a required constraint.
    pub fn issuer<'b>(
        &mut self,
        issuers: impl IntoIterator<Item = Str<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"Issuer")).array().items(issuers);
        self
    }

    /// Write the `/OID` attribute to set which certificate policies'
    /// Object Identifiers (OIDs) shall be present in the signing certificate.
    /// If [`CertificateSeedValueFlags::OID`] is present on
    /// [constraint flags](CertificateSeedValue::constraint_flags),
    /// this is a required constraint.
    pub fn oid<'b>(
        &mut self,
        oids: impl IntoIterator<Item = Str<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"OID")).array().items(oids);
        self
    }

    /// Write the `/URL` attribute to set a URL whose use is defined
    /// by the [`/URLType`](CertificateSeedValue::url_type) attribute.
    pub fn url(&mut self, url: Str<'_>) -> &mut Self {
        self.pair(Name(b"URL"), url);
        self
    }

    /// Write the `/URLType` attribute to set the use of
    /// the [`/URL`](CertificateSeedValue::url) attribute.
    /// PDF 1.7+.
    pub fn url_type(&mut self, url_type: CertificateUrlType) -> &mut Self {
        self.pair(Name(b"URLType"), url_type.to_name());
        self
    }
}

deref!('a, CertificateSeedValue<'a> => Dict<'a>, dict);

bitflags::bitflags! {
    /// Bitflags describing the whether a specific entry in a
    /// [certificate seed value dictionary](CertificateSeedValue)
    /// is required to be present in the signing certificate (1)
    /// or optional (0).
    pub struct CertificateSeedValueFlags: u32 {
        /// The `/Subject` entry is a mandatory constraint.
        const SUBJECT = 1 << 0;
        /// The `/Issuer` entry is a mandatory constraint.
        const ISSUER = 1 << 1;
        /// The `/OID` entry is a mandatory constraint.
        const OID = 1 << 2;
        /// The `/SubjectDN` entry is a mandatory constraint.
        const SUBJECT_DN = 1 << 3;

        // Bit 5 (1 << 4) is reserved

        /// The `/KeyUsage` entry is a mandatory constraint.
        const KEY_USAGE = 1 << 5;
        /// The `/URL` entry is a mandatory constraint.
        const URL = 1 << 6;
    }
}

/// The key-usage extensions that shall or shall not be present in a
/// certificate.
///
/// For each field of this struct, its value is interpreted as follows:
/// `Some(true)` if it shall be set;
/// `Some(false)` if it shall not be set; or
/// `None` if it does not matter.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct CertificateKeyUsage {
    /// Whether the `digitalSignature` key-usage extension shall be set.
    pub digital_signature: Option<bool>,
    /// Whether the `non-Repudiation` key-usage extension shall be set.
    pub non_repudiation: Option<bool>,
    /// Whether the `keyEncipherment` key-usage extension shall be set.
    pub key_encipherment: Option<bool>,
    /// Whether the `dataEncipherment` key-usage extension shall be set.
    pub data_encipherment: Option<bool>,
    /// Whether the `keyAgreement` key-usage extension shall be set.
    pub key_agreement: Option<bool>,
    /// Whether the `KeyCertSign` key-usage extension shall be set.
    pub key_cert_sign: Option<bool>,
    /// Whether the `cRLSign` key-usage extension shall be set.
    pub crl_sign: Option<bool>,
    /// Whether the `encipherOnly` key-usage extension shall be set.
    pub encipher_only: Option<bool>,
    /// Whether the `decipherOnly` key-usage extension shall be set.
    pub decipher_only: Option<bool>,
}

impl CertificateKeyUsage {
    fn to_byte(value: Option<bool>) -> u8 {
        match value {
            Some(true) => b'1',
            Some(false) => b'0',
            None => b'X',
        }
    }
    pub(crate) fn into_string(&self) -> [u8; 9] {
        [
            Self::to_byte(self.digital_signature),
            Self::to_byte(self.non_repudiation),
            Self::to_byte(self.key_encipherment),
            Self::to_byte(self.data_encipherment),
            Self::to_byte(self.key_agreement),
            Self::to_byte(self.key_cert_sign),
            Self::to_byte(self.crl_sign),
            Self::to_byte(self.encipher_only),
            Self::to_byte(self.decipher_only),
        ]
    }
}

/// The usage of the [URL entry](CertificateSeedValue::url).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CertificateUrlType<'a> {
    /// The URL references content that shall be displayed in a web
    /// browser to allow enrolling for a new credential if a matching
    /// credential is not found.
    Browser,
    /// An implementation-specific third-party extension.
    Custom(Name<'a>),
}

impl<'a> CertificateUrlType<'a> {
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::Browser => Name(b"Browser"),
            Self::Custom(name) => name,
        }
    }
}

bitflags::bitflags! {
    /// Bitflags describing various characteristics of a form field.
    pub struct FieldFlags: u32 {
        /// The user may not change the value of the field. Any associated
        /// widget annotations will not interact with the user; that is, they
        /// will not respond to mouse clicks or change their appearance in
        /// response to mouse motions. This flag is useful for fields whose
        /// values are computed or imported from a database.
        const READ_ONLY = 1 << 0;
        /// The field shall have a value at the time it is exported by a
        /// [submit-form](crate::types::ActionType::SubmitForm) [`Action`].
        const REQUIRED = 1 << 1;
        /// The field shall not be exported by a
        /// [submit-form](crate::types::ActionType::SubmitForm) [`Action`].
        const NO_EXPORT = 1 << 2;
        /// The entered text shall not be spell-checked, can be used for text
        /// and choice fields. PDF 1.4+.
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
        /// vertically (for multi-line) to accommodate more text. Once the field
        /// is full, no further text shall be accepted for interactive form
        /// filling; for non-interactive form filling, the filler should take
        /// care not to add more character than will visibly fit in the defined
        /// area. PDF 1.4+.
        const DO_NOT_SCROLL = 1 << 23;
        /// The field shall be automatically divided into as many equally
        /// spaced positions or _combs_ as the value of [`Field::max_len`]
        /// and the text is laid out into these combs. May only be set if
        /// the [`Field::max_len`] property is set and if the [`MULTILINE`],
        /// [`PASSWORD`] and [`FILE_SELECT`] flags are clear. PDF 1.5+.
        const COMB = 1 << 24;
        /// The value of this field shall be a rich text string. If the field
        /// has a value, the [`TextField::rich_text_value`] shall specify the
        /// rich text string. PDF 1.5+.
        const RICH_TEXT = 1 << 25;

        // Choice field specific flags

        /// The field is a combo box if set, else it's a list box. A combo box
        /// is often referred to as a dropdown menu.
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
