use super::*;

/// Writer for a _file specification dictionary_.
///
/// This struct is created by [`Annotation::file_spec`],
/// [`Reference::file_spec`], [`MediaClip::data`], and [`Action::file_spec`].
pub struct FileSpec<'a> {
    dict: Dict<'a>,
}

writer!(FileSpec: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Filespec"));
    Self { dict }
});

impl FileSpec<'_> {
    /// Write the `/FS` attribute to set the file system this entry relates to.
    /// If you set the `system` argument to `Name(b"URL")`, this becomes an URL
    /// specification.
    pub fn file_system(&mut self, system: Name) -> &mut Self {
        self.pair(Name(b"FS"), system);
        self
    }

    /// Write the `/F` attribute to set the file path. Directories are indicated
    /// by `/`, independent of the platform.
    pub fn path(&mut self, path: Str) -> &mut Self {
        self.pair(Name(b"F"), path);
        self
    }

    /// Write the `/UF` attribute to set a Unicode-compatible path. Directories
    /// are indicated by `/`, independent of the platform. PDF 1.7+.
    pub fn unic_file(&mut self, path: TextStr) -> &mut Self {
        self.pair(Name(b"UF"), path);
        self
    }

    /// Write the `/V` attribute to indicate whether _not_ to cache the file.
    pub fn volatile(&mut self, dont_cache: bool) -> &mut Self {
        self.pair(Name(b"V"), dont_cache);
        self
    }

    /// Write the `/Desc` attribute to set a file description. PDF 1.6+.
    pub fn description(&mut self, desc: TextStr) -> &mut Self {
        self.pair(Name(b"Desc"), desc);
        self
    }

    /// Write the `/EF` attribute to reference an [embedded file](EmbeddedFile).
    /// PDF 1.3+.
    ///
    /// This only sets an embedded file for the `F` attribute corresponding to
    /// the [`path`](Self::path) method. If you want to set the same embedded
    /// file for the `UF` attribute, also call [`Self::embedded_file_with_unicode`]
    /// instead.
    ///
    /// Note that this key is forbidden in PDF/A-1 and restricted in PDF/A-2 and
    /// PDF/A-4.
    pub fn embedded_file(&mut self, id: Ref) -> &mut Self {
        self.insert(Name(b"EF")).dict().pair(Name(b"F"), id);
        self
    }

    /// Write the `/EF` attribute to reference an [embedded file](EmbeddedFile)
    /// for the legacy and Unicode-compatible file path. PDF 1.7+.
    ///
    /// Note that this key is forbidden in PDF/A-1 and restricted in PDF/A-2 an
    /// PDF/A-4.
    pub fn embedded_file_with_unicode(&mut self, id: Ref) -> &mut Self {
        self.insert(Name(b"EF"))
            .dict()
            .pair(Name(b"F"), id)
            .pair(Name(b"UF"), id);
        self
    }

    /// How this file relates to the PDF document it is embedded in.
    /// PDF/A-3 and PDF/A-4f.
    pub fn association_kind(&mut self, kind: AssociationKind) -> &mut Self {
        self.pair(Name(b"AFRelationship"), kind.to_name());
        self
    }
}

deref!('a, FileSpec<'a> => Dict<'a>, dict);

/// Writer for an _embedded file stream_.
///
/// This struct is created by [`Chunk::embedded_file`].
pub struct EmbeddedFile<'a> {
    stream: Stream<'a>,
}

impl<'a> EmbeddedFile<'a> {
    /// Create a new embedded file writer.
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"EmbeddedFile"));
        Self { stream }
    }

    /// Write the `/Subtype` attribute to set the file type.
    ///
    /// This can either be a MIME type or a name prefixed by a first class PDF
    /// prefix. Note that special characters must be encoded as described in
    /// section 7.3.5 of the PDF 1.7 specification, e.g. `image/svg+xml` would
    /// become `Name(b"image#2Fsvg+xml")`.
    pub fn subtype(&mut self, subtype: Name) -> &mut Self {
        self.pair(Name(b"Subtype"), subtype);
        self
    }

    /// Start writing the `/Params` dictionary.
    pub fn params(&mut self) -> EmbeddingParams<'_> {
        self.insert(Name(b"Params")).start()
    }
}

deref!('a, EmbeddedFile<'a> => Stream<'a>, stream);

/// Writer for an _embedded file parameter dictionary_.
///
/// This struct is created by [`EmbeddedFile::params`].
pub struct EmbeddingParams<'a> {
    dict: Dict<'a>,
}

writer!(EmbeddingParams: |obj| Self { dict: obj.dict() });

impl EmbeddingParams<'_> {
    /// Write the `/Size` attribute to set the uncompressed file size in bytes.
    pub fn size(&mut self, size: i32) -> &mut Self {
        self.pair(Name(b"Size"), size);
        self
    }

    /// Write the `/CreationDate` attribute to set the file creation date.
    pub fn creation_date(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"CreationDate"), date);
        self
    }

    /// Write the `/ModDate` attribute to set the file modification date.
    pub fn modification_date(&mut self, date: Date) -> &mut Self {
        self.pair(Name(b"ModDate"), date);
        self
    }

    /// Write the `/CheckSum` attribute to set the file checksum.
    ///
    /// The checksum shall be a 16-byte MD5 string.
    pub fn checksum(&mut self, checksum: Str) -> &mut Self {
        self.pair(Name(b"CheckSum"), checksum);
        self
    }
}

deref!('a, EmbeddingParams<'a> => Dict<'a>, dict);

/// How an embedded file relates to the PDF document it is embedded in.
/// PDF 1.7 with PDF/A-3, PDF 2.0+ (including PDF/A-4f).
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub enum AssociationKind {
    /// The PDF document was created from this source file.
    Source,
    /// This file was used to derive a visual presentation in the PDF.
    Data,
    /// An alternative representation of this document.
    Alternative,
    /// Additional resources for this document.
    Supplement,
    /// An encrypted file. PDF 2.0+.
    EncryptedPayload,
    /// Data associated with the `AcroForm`. PDF 2.0+.
    FormData,
    /// A machine-readable schema. PDF 2.0+.
    Schema,
    /// There is no clear relationship or it is not known.
    #[default]
    Unspecified,
}

impl AssociationKind {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Source => Name(b"Source"),
            Self::Data => Name(b"Data"),
            Self::Alternative => Name(b"Alternative"),
            Self::Supplement => Name(b"Supplement"),
            Self::EncryptedPayload => Name(b"EncryptedPayload"),
            Self::FormData => Name(b"FormData"),
            Self::Schema => Name(b"Schema"),
            Self::Unspecified => Name(b"Unspecified"),
        }
    }
}
