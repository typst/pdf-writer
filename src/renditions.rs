use super::*;
use crate::object::TextStrLike;

/// Writer for an _rendition dictionary_.
///
/// This struct is created by [`Action::rendition`].
pub struct Rendition<'a> {
    dict: Dict<'a>,
}

writer!(Rendition: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"Rendition"));
    Self { dict }
});

impl Rendition<'_> {
    /// Write the `/S` attribute to set the rendition type.
    pub fn subtype(&mut self, kind: RenditionType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
        self
    }

    /// Write the `/N` attribute. Specify the name of the rendition for use in a
    /// user interface and for name tree lookup by JavaScript actions.
    pub fn name(&mut self, text: impl TextStrLike) -> &mut Self {
        self.pair(Name(b"N"), text);
        self
    }

    /// Start writing the `/C`, i.e. media clip, dictionary which specifies what
    /// media should be played. Only permissible for Media Renditions.
    pub fn media_clip(&mut self) -> MediaClip<'_> {
        self.insert(Name(b"C")).start()
    }

    /// Start writing the `/P`, i.e. media play parameters, dictionary which
    /// specifies how the media should be played. Only permissible for Media
    /// Renditions.
    pub fn media_play_params(&mut self) -> MediaPlayParams<'_> {
        self.insert(Name(b"P")).start()
    }
}

deref!('a, Rendition<'a> => Dict<'a>, dict);

/// Writer for an _media clip dictionary_.
///
/// This struct is created by [`Rendition::media_clip`].
///
/// ## Note on reader compatibility
///
/// Different PDF readers may have support for different media codecs and
/// container formats.
///
/// For example, [Adobe's documentation][1] states that Adobe Acrobat can play
/// videos in MP4, MOV, M4V, 3GP, and 3G2 containers using the H.264 codec.
///
/// Other readers may depend on the media libraries installed on the system. KDE
/// Okular, for example, uses the Phonon library to support a range of media
/// formats.
///
/// Yet other viewers do not support media clips at all. At the time of writing,
/// this includes the popular Pdfium library used by Google Chrome and Microsoft
/// Edge, `pdf.js` used by Firefox, mupdf, and Quartz, the PDF viewer on Apple
/// platforms.
///
/// [1]: https://helpx.adobe.com/acrobat/using/playing-video-audio-multimedia-formats.html#supported_video_audio_and_interactive_formats
pub struct MediaClip<'a> {
    dict: Dict<'a>,
}

writer!(MediaClip: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"MediaClip"));
    Self { dict }
});

impl MediaClip<'_> {
    /// Write the `/S` attribute to set the media clip type.
    pub fn subtype(&mut self, kind: MediaClipType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
        self
    }

    /// Write the `/N` attribute. Specifies the name of the media clip, for use
    /// in the user interface.
    pub fn name(&mut self, text: impl TextStrLike) -> &mut Self {
        self.pair(Name(b"N"), text);
        self
    }

    /// Start writing the `/D` dictionary specifying the media data.
    pub fn data(&mut self) -> FileSpec<'_> {
        self.insert(Name(b"D")).start()
    }

    /// Write the `/CT` attribute identifying the type of data in `/D`, i.e. the
    /// MIME type.
    pub fn data_type(&mut self, tf: Str) -> &mut Self {
        self.pair(Name(b"CT"), tf);
        self
    }

    /// Start writing the `/P`, i.e. media permissions, dictionary.
    pub fn permissions(&mut self) -> MediaPermissions<'_> {
        self.insert(Name(b"P")).start()
    }

    /// Write the `/Alt` attribute, listing alternate text descriptions which
    /// are specified as a multi-language text array. A multi-language text
    /// array shall contain pairs of strings.
    pub fn alt_texts<'b>(
        &mut self,
        texts: impl IntoIterator<Item = TextStr<'b>>,
    ) -> &mut Self {
        self.insert(Name(b"Alt")).array().items(texts);
        self
    }
}

deref!('a, MediaClip<'a> => Dict<'a>, dict);

/// Writer for an _media play parameters dictionary_.
///
/// This struct is created by [`Rendition::media_play_params`].
pub struct MediaPlayParams<'a> {
    dict: Dict<'a>,
}

writer!(MediaPlayParams: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"MediaPlayParams"));
    Self { dict }
});

impl MediaPlayParams<'_> {
    /// Write the `/C` attribute inside a `/BE` dictionary specifying whether to
    /// display a player-specific controls.
    ///
    /// This avoids implementing the "must honour" (MH) or "best effort" (BE)
    /// dictionaries for MediaPlayParams, as the required boiler-plate code
    /// would be high, and its usefulness low.
    pub fn controls(&mut self, c: bool) -> &mut Self {
        self.insert(Name(b"BE")).dict().pair(Name(b"C"), c);
        self
    }
}

deref!('a, MediaPlayParams<'a> => Dict<'a>, dict);

/// Writer for an _media permissions dictionary_.
///
/// This struct is created by [`MediaClip::permissions`].
pub struct MediaPermissions<'a> {
    dict: Dict<'a>,
}

writer!(MediaPermissions: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"MediaPermissions"));
    Self { dict }
});

impl MediaPermissions<'_> {
    /// Write the `/TF` attribute to control permissions to write a temporary file.
    pub fn temp_file(&mut self, tf: TempFileType) -> &mut Self {
        self.pair(Name(b"TF"), tf.to_str());
        self
    }
}

deref!('a, MediaPermissions<'a> => Dict<'a>, dict);

/// The circumstances under which it is acceptable to write a temporary file in
/// order to play a media clip.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub enum TempFileType {
    /// Never allowed.
    #[default]
    Never,
    /// Allowed only if the document permissions allow content extraction.
    Extract,
    /// Allowed only if the document permissions allow content extraction,
    /// including for accessibility purposes.
    Access,
    /// Always allowed.
    Always,
}

impl TempFileType {
    pub(crate) fn to_str(self) -> Str<'static> {
        match self {
            Self::Never => Str(b"TEMPNEVER"),
            Self::Extract => Str(b"TEMPEXTRACT"),
            Self::Access => Str(b"TEMPACCESS"),
            Self::Always => Str(b"TEMPALWAYS"),
        }
    }
}

/// Type of rendition objects.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RenditionType {
    /// Media Rendition.
    Media,
    /// Selector Rendition.
    Selector,
}

impl RenditionType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Media => Name(b"MR"),
            Self::Selector => Name(b"SR"),
        }
    }
}

/// Type of media clip objects.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MediaClipType {
    /// Media Clip Data.
    Data,
    /// Media Clip Section.
    Section,
}

impl MediaClipType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Data => Name(b"MCD"),
            Self::Section => Name(b"MCS"),
        }
    }
}
