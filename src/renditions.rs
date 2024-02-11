use super::*;

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

impl<'a> Rendition<'a> {
    /// Write the `/S` attribute to set the rendition type.
    pub fn rendition_type(&mut self, kind: RenditionType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
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
pub struct MediaClip<'a> {
    dict: Dict<'a>,
}

writer!(MediaClip: |obj| {
    let mut dict = obj.dict();
    dict.pair(Name(b"Type"), Name(b"MediaClip"));
    Self { dict }
});

impl<'a> MediaClip<'a> {
    /// Write the `/S` attribute to set the media clip type.
    pub fn media_clip_type(&mut self, kind: MediaClipType) -> &mut Self {
        self.pair(Name(b"S"), kind.to_name());
        self
    }

    /// Start writing the `/D` dictionary specifying the media data.
    pub fn data(&mut self) -> FileSpec<'_> {
        self.insert(Name(b"D")).start()
    }

    /// Writing the `/D` dictionary including the file specification as a link,
    /// i.e. an URL, to a given path.
    pub fn data_url(&mut self, path: Str) -> &mut Self {
        self.data()
            .file_system(Name(b"URL"))
            .path(path);
        self
    }

    /// Writing the `/D` dictionary including the file specification as
    /// embedded file referenced by a given id.
    pub fn data_embedded(&mut self, id: Ref) -> &mut Self {
        self.data()
            .path(Str(b"<embedded file>"))
            .embedded_file(id);
        self
    }

    /// Write the `/CT` attribute identifying the type of data in D, i.e. the
    /// MIME type.
    pub fn data_type(&mut self, tf: Str) -> &mut Self {
        self.pair(Name(b"CT"), tf);
        self
    }

    /// Write the `/TF` attribute inside the `/P` dictionary controlling the
    /// permissions to write a temporary file.
    ///
    /// The media permissions dictionary has a single entry. Thus, skip
    /// implementing a full MediaPermissions dictionary object.
    pub fn temp_file(&mut self, tf: Str) -> &mut Self {
        self.insert(Name(b"P")).dict()
                               .pair(Name(b"Type"), Name(b"MediaPermissions"))
                               .pair(Name(b"TF"), tf);
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

impl<'a> MediaPlayParams<'a> {
    /// Write the `/C` attribute inside a `/BE` dictionary specifying whether to
    /// display a player-specific controls.
    ///
    /// This avoids implementing the "must honour" (MH) or "best effort" (BE)
    /// dictionaries for MediaPlayParams, as the required boiler-plate code
    /// would be high, and its usefulness low.
    pub fn controls (&mut self, c: bool) -> &mut Self {
        self.insert(Name(b"BE")).dict().pair(Name(b"C"), c);
        self
    }
}

deref!('a, MediaPlayParams<'a> => Dict<'a>, dict);


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
