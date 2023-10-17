use super::*;

/// Writer for an _action dictionary_.
///
/// This struct is created by [`Annotation::action`].
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

    /// Start writing the `/F` attribute, setting which file to go to or which
    /// application to launch.
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
}

deref!('a, Action<'a> => Dict<'a>, dict);

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
}

impl ActionType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::GoTo => Name(b"GoTo"),
            Self::RemoteGoTo => Name(b"GoToR"),
            Self::Launch => Name(b"Launch"),
            Self::Uri => Name(b"URI"),
        }
    }
}
