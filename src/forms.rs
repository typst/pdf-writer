use super::*;

/// A form field.
pub struct Field<'a> {
    dict: Dict<'a>,
}

writer!(Field: |obj| Self { dict: obj.dict() });

impl<'a> Field<'a> {
    /// Write the `/FT` attribute to set the type of this field.
    pub fn field_type(&mut self, typ: FieldType) -> &mut Self {
        self.dict.pair(Name(b"FT"), typ.to_name());
        self
    }

    /// Write the `/Parent` attribute to set the immediate parent of this
    /// field.
    pub fn parent(&mut self, id: Ref) -> &mut Self {
        self.dict.pair(Name(b"Parent"), id);
        self
    }

    /// Start writing the `/Kids` attribute to set the immediate children of
    /// this field. These references shall refer to other [fields][Field], or
    /// [widget](crate::types::AnnotationType::Widget) [annoations](Annotation).
    pub fn children(&mut self) -> TypedArray<'_, Ref> {
        self.dict.insert(Name(b"Kids")).array().typed()
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
        self.dict.pair(Name(b"T"), name);
        self
    }

    /// Write the `/TU` attribute to set the alternative field name. This
    /// field name is used in place of the actual field name whenever the field
    /// shall be identified in the user interface (such as in error or status
    /// messages). This text is also useful when extracting the document's
    /// contents in support of accessibility to users with disabilities or for
    /// other purposes. PDF 1.3+.
    pub fn alternate_name(&mut self, alternate: TextStr) -> &mut Self {
        self.dict.pair(Name(b"TU"), alternate);
        self
    }

    /// Write the `/TM` attribute to set the mapping field name. This
    /// name shall be used when exporting interactive form field data from the
    /// document.
    pub fn mapping_name(&mut self, name: TextStr) -> &mut Self {
        self.dict.pair(Name(b"TM"), name);
        self
    }

    /// Write the `/Ff` attribute to set various characteristics of this
    /// field.
    pub fn field_flags(&mut self, flags: FieldFlags) -> &mut Self {
        self.dict.pair(Name(b"Tf"), flags.bits() as i32);
        self
    }

    /// Start writing the `/AA` dictionary to set the field's response to
    /// various trigger events.
    pub fn additional_actions(&mut self) -> AdditionalActions<'_> {
        self.dict.insert(Name(b"AA")).start()
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
        const NO_EXPORT = 1 << 3;
    }
}
