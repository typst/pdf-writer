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

/// Writer for an _additional actions dictionary_.
///
/// This struct is created by [`Annotation::additional_actions`],
/// [`Page::additional_actions`] and [`Catalog::additional_actions`].
pub struct AdditionalActions<'a> {
    dict: Dict<'a>,
}

writer!(AdditionalActions: |obj| Self { dict: obj.dict() });

/// Only permissible for [annotations](Annotation).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/E` dictionary. An action that shall be performed
    /// when the cursor enters the annotation’s active area. Only permissible
    /// for annotations. PDF 1.2+.
    pub fn curser_enter(&mut self) -> Action<'_> {
        self.insert(Name(b"E")).start()
    }

    /// Start writing the `/X` dictionary. An action that shall be performed
    /// when the cursor exits the annotation’s active area. Only permissible for
    /// annotations. PDF 1.2+.
    pub fn cursor_exit(&mut self) -> Action<'_> {
        self.insert(Name(b"X")).start()
    }

    /// Start writing the `/D` dictionary. An action that shall be performed
    /// when the mouse button is pressed inside the annotation’s active area.
    /// Only permissible for annotations. PDF 1.2+.
    pub fn mouse_press(&mut self) -> Action<'_> {
        self.insert(Name(b"D")).start()
    }

    /// Start writing the `/U` dictionary. An action that shall be performed
    /// when the mouse button is released inside the annotation’s active area.
    /// Only permissible for annotations. PDF 1.2+.
    pub fn mouse_release(&mut self) -> Action<'_> {
        self.insert(Name(b"U")).start()
    }

    /// Start writing the `/PO` dictionary. An action that shall be performed
    /// when the page containing the annotation is opened. Only permissible for
    /// annotations. PDF 1.5+.
    pub fn page_open(&mut self) -> Action<'_> {
        self.insert(Name(b"PO")).start()
    }

    /// Start writing the `/PC` dictionary. An action that shall be performed
    /// when the page containing the annotation is closed. Only permissible for
    /// annotations. PDF 1.5+.
    pub fn page_close(&mut self) -> Action<'_> {
        self.insert(Name(b"PV")).start()
    }

    /// Start writing the `/PV` dictionary. An action that shall be performed
    /// when the page containing the annotation becomes visible. Only
    /// permissible for annotations. PDF 1.5+.
    pub fn page_visible(&mut self) -> Action<'_> {
        self.insert(Name(b"PV")).start()
    }

    /// Start writing the `/PI` dictionary. An action that shall be performed
    /// when the page containing the annotation is no longer visible in the
    /// conforming reader’s user interface. Only permissible for annotations.
    /// PDF 1.5+.
    pub fn page_invisible(&mut self) -> Action<'_> {
        self.insert(Name(b"PI")).start()
    }
}

/// Only permissible for [widget](crate::types::AnnotationType::Widget)
/// [annotations](Annotation).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/Fo` dictionary. An action that shall be performed
    /// when the annotation receives the input focus.
    /// Only permissible for widget annotations. PDF 1.2+.
    pub fn focus(&mut self) -> Action<'_> {
        self.insert(Name(b"Fo")).start()
    }

    /// Start writing the `/Bl` dictionary. An action that shall be performed
    /// when the annotation loses the input focus. Only permissible for widget
    /// annotations. PDF 1.2+.
    pub fn focus_loss(&mut self) -> Action<'_> {
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
    pub fn open(&mut self) -> Action<'_> {
        self.insert(Name(b"O")).start()
    }

    /// Start writing the `/C` dictionary. This sets the action that shall
    /// be performed when the page is closed. This action applies to the page
    /// being closed and shall be executed before any other page is opened. Only
    /// permissible for [page objects](Page). PDF 1.2+.
    pub fn close(&mut self) -> Action<'_> {
        self.insert(Name(b"C")).start()
    }
}

/// Only permisible for [document catalog](Catalog).
impl<'a> AdditionalActions<'a> {
    /// Start writing the `/WC` dictionary. This sets the JavaScript action that
    /// shall be performed before closing a document. PDF 1.4+.
    pub fn before_close(&mut self) -> Action<'_> {
        self.insert(Name(b"WC")).start()
    }

    /// Start writing the `/WS` dictionary. This sets the JavaScript action that
    /// shall be performed before saving a document. PDF 1.4+.
    pub fn before_save(&mut self) -> Action<'_> {
        self.insert(Name(b"WS")).start()
    }

    /// Start writing the `/DS` dictionary. This sets the JavaScript action that
    /// shall be performed after saving a document. PDF 1.4+.
    pub fn after_save(&mut self) -> Action<'_> {
        self.insert(Name(b"DS")).start()
    }

    /// Start writing the `/WP` dictionary. This sets the JavaScript action that
    /// shall be performed before printing a document. PDF 1.4+.
    pub fn before_print(&mut self) -> Action<'_> {
        self.insert(Name(b"WP")).start()
    }

    /// Start writing the `/DP` dictionary. This sets the JavaScript action that
    /// shall be performed after printing a document. PDF 1.4+.
    pub fn after_print(&mut self) -> Action<'_> {
        self.insert(Name(b"DP")).start()
    }
}

deref!('a, AdditionalActions<'a> => Dict<'a>, dict);
