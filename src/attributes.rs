use super::*;

/// Writer for an _attribute array_. PDF 1.4+
///
/// This struct is created by [`StructElement::attributes`].
pub struct AttributeArray<'a> {
    arr: Array<'a>,
}

impl<'a> Writer<'a> for AttributeArray<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { arr: obj.array() }
    }
}

impl<'a> AttributeArray<'a> {
    /// Start writing an attribute dictionary.
    pub fn attributes(&mut self) -> Attributes<'_> {
        self.arr.push().start()
    }
}

/// Writer for an _attribute dictionary_. PDF 1.4+
///
/// This struct is created by [`AttributeArray::attributes`]. It is required to set
/// the `/O` attribute by calling any of the methods.
pub struct Attributes<'a> {
    obj: Obj<'a>,
}

impl<'a> Writer<'a> for Attributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { obj }
    }
}

impl<'a> Attributes<'a> {
    /// Set the `/O` attribute to user-defined and start writing the `/P` array
    /// with user properties. PDF 1.6+
    pub fn user_properties(self) -> PropertyAttributes<'a> {
        PropertyAttributes::start(self.obj)
    }

    /// Set the `/O` attribute to `Layout` to write layout parameters.
    pub fn layout(self) -> LayoutAttributes<'a> {
        LayoutAttributes::start(self.obj)
    }

    /// Set the `/O` attribute to `List` to write list attributes.
    pub fn list(self) -> ListAttributes<'a> {
        ListAttributes::start(self.obj)
    }

    /// Set the `/O` attribute to `PrintField` to write attributes for the
    /// appearance of form fields. PDF 1.6+
    pub fn field(self) -> FieldAttributes<'a> {
        FieldAttributes::start(self.obj)
    }

    /// Set the `/O` attribute to `Table` to write table attributes.
    pub fn table(self) -> TableAttributes<'a> {
        TableAttributes::start(self.obj)
    }

    /// Write the `/O` attribute.
    pub fn custom(self, owner: AttributeOwner) -> Dict<'a> {
        let mut dict = self.obj.dict();
        dict.pair(Name(b"O"), owner.to_name());
        dict
    }
}

deref!('a, Attributes<'a> => Obj<'a>, obj);

/// Writer for an _user property attributes dictionary_. PDF 1.6+
///
/// This struct is created by [`Attributes::user_properties`].
pub struct PropertyAttributes<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for PropertyAttributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"O"), AttributeOwner::User.to_name());
        Self { dict }
    }
}

impl<'a> PropertyAttributes<'a> {
    /// Write a user property.
    pub fn properties(&mut self) -> UserProperties<'_> {
        self.dict.insert(Name(b"P")).start()
    }
}

deref!('a, PropertyAttributes<'a> => Dict<'a>, dict);

/// Writer for an _user property array_. PDF 1.6+
///
/// This struct is created by [`Attributes::user_properties`].
pub struct UserProperties<'a> {
    arr: Array<'a>,
}

impl<'a> Writer<'a> for UserProperties<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { arr: obj.array() }
    }
}

impl<'a> UserProperties<'a> {
    /// Write a user property.
    pub fn property(&mut self) -> UserProperty<'_> {
        self.arr.push().start()
    }
}

deref!('a, UserProperties<'a> => Array<'a>, arr);

/// Writer for an _user property dictionary_. PDF 1.6+
///
/// This struct is created by [`UserProperties::property`].
pub struct UserProperty<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for UserProperty<'a> {
    fn start(obj: Obj<'a>) -> Self {
        Self { dict: obj.dict() }
    }
}

impl<'a> UserProperty<'a> {
    /// Write the `/N` attribute to set the name of the property.
    pub fn name(&mut self, name: TextStr) -> &mut Self {
        self.dict.pair(Name(b"N"), name);
        self
    }

    /// Start writing the `/V` attribute to set the value of the property.
    pub fn value(&mut self) -> Obj<'_> {
        self.dict.insert(Name(b"V"))
    }

    /// Write the `/F` attribute to set the format of the property.
    pub fn format(&mut self, format: TextStr) -> &mut Self {
        self.dict.pair(Name(b"F"), format);
        self
    }

    /// Write the `/H` attribute to determine whether this property is hidden.
    pub fn hidden(&mut self, hide: bool) -> &mut Self {
        self.dict.pair(Name(b"H"), hide);
        self
    }
}

deref!('a, UserProperty<'a> => Dict<'a>, dict);

/// Writer for an _layout attributes dictionary_. PDF 1.4+
///
/// This struct is created by [`Attributes::layout`].
pub struct LayoutAttributes<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for LayoutAttributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"O"), AttributeOwner::Layout.to_name());
        Self { dict }
    }
}

/// General layout attributes.
impl<'a> LayoutAttributes<'a> {
    /// Write the `/Placement` attribute.
    pub fn placement(&mut self, placement: Placement) -> &mut Self {
        self.dict.pair(Name(b"Placement"), placement.to_name());
        self
    }

    /// Write the `/WritingMode` attribute to set the writing direction.
    pub fn writing_mode(&mut self, mode: WritingMode) -> &mut Self {
        self.dict.pair(Name(b"WritingMode"), mode.to_name());
        self
    }

    /// Write the `/BackgroundColor` attribute to set the background color in
    /// RGB between `0` and `1`. PDF 1.5+
    pub fn background_color(&mut self, color: [f32; 3]) -> &mut Self {
        self.dict
            .insert(Name(b"BackgroundColor"))
            .array()
            .typed()
            .items(color);
        self
    }

    /// Write the `/BorderColor` attribute.
    pub fn border_color(&mut self, color: [f32; 3]) -> &mut Self {
        self.dict.insert(Name(b"BorderColor")).array().typed().items(color);
        self
    }

    /// Write the `/BorderStyle` attribute.
    pub fn border_style(&mut self, style: LayoutBorderStyle) -> &mut Self {
        self.dict.pair(Name(b"BorderStyle"), style.to_name());
        self
    }

    /// Write the `/BorderThickness` attribute.
    pub fn border_thickness(&mut self, thickness: f32) -> &mut Self {
        self.dict.pair(Name(b"BorderThickness"), thickness);
        self
    }

    /// Write the `/Color` attribute.
    pub fn color(&mut self, color: [f32; 3]) -> &mut Self {
        self.dict.insert(Name(b"Color")).array().typed().items(color);
        self
    }

    /// Write the `/Padding` attribute.
    pub fn padding(&mut self, padding: [f32; 4]) -> &mut Self {
        self.dict.insert(Name(b"Padding")).array().typed().items(padding);
        self
    }
}

/// Placement of an element. PDF 1.4+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Placement {
    /// Stacked in the block order.
    Block,
    /// Stacked in the inline order.
    Inline,
    /// Before edge coincides with that of reference area.
    Before,
    /// Start edge coincides with that of reference area.
    Start,
    /// End edge coincides with that of reference area.
    End,
}

impl Placement {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Block => Name(b"Block"),
            Self::Inline => Name(b"Inline"),
            Self::Before => Name(b"Before"),
            Self::Start => Name(b"Start"),
            Self::End => Name(b"End"),
        }
    }
}

/// Writing direction. PDF 1.4+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum WritingMode {
    /// Horizontal writing mode, left-to-right.
    HorizontalLeftToRight,
    /// Horizontal writing mode, right-to-left.
    HorizontalRightToLeft,
    /// Vertical writing mode, right-to-left.
    VerticalRightToLeft,
}

impl WritingMode {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::HorizontalLeftToRight => Name(b"LrTb"),
            Self::HorizontalRightToLeft => Name(b"RlTb"),
            Self::VerticalRightToLeft => Name(b"TbRl"),
        }
    }
}

/// Layout border style. PDF 1.4+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LayoutBorderStyle {
    /// No border.
    None,
    /// Hidden border.
    Hidden,
    /// Solid border.
    Solid,
    /// Dashed border.
    Dashed,
    /// Dotted border.
    Dotted,
    /// Double border.
    Double,
    /// Groove border.
    Groove,
    /// Ridge border.
    Ridge,
    /// Inset border.
    Inset,
    /// Outset border.
    Outset,
}

impl LayoutBorderStyle {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::None => Name(b"None"),
            Self::Hidden => Name(b"Hidden"),
            Self::Solid => Name(b"Solid"),
            Self::Dashed => Name(b"Dashed"),
            Self::Dotted => Name(b"Dotted"),
            Self::Double => Name(b"Double"),
            Self::Groove => Name(b"Groove"),
            Self::Ridge => Name(b"Ridge"),
            Self::Inset => Name(b"Inset"),
            Self::Outset => Name(b"Outset"),
        }
    }
}

/// Block level elements.
impl LayoutAttributes<'_> {
    /// Write the `/SpaceBefore` attribute.
    pub fn space_before(&mut self, space: f32) -> &mut Self {
        self.dict.pair(Name(b"SpaceBefore"), space);
        self
    }

    /// Write the `/SpaceAfter` attribute.
    pub fn space_after(&mut self, space: f32) -> &mut Self {
        self.dict.pair(Name(b"SpaceAfter"), space);
        self
    }

    /// Write the `/StartIndent` attribute.
    pub fn start_indent(&mut self, indent: f32) -> &mut Self {
        self.dict.pair(Name(b"StartIndent"), indent);
        self
    }

    /// Write the `/EndIndent` attribute.
    pub fn end_indent(&mut self, indent: f32) -> &mut Self {
        self.dict.pair(Name(b"EndIndent"), indent);
        self
    }

    /// Write the `/TextIndent` attribute.
    pub fn text_indent(&mut self, indent: f32) -> &mut Self {
        self.dict.pair(Name(b"TextIndent"), indent);
        self
    }

    /// Write the `/TextAlign` attribute.
    pub fn text_align(&mut self, align: TextAlign) -> &mut Self {
        self.dict.pair(Name(b"TextAlign"), align.to_name());
        self
    }

    /// Write the `/Width` attribute for table row groups and illustrative
    /// elements. No instrinsic height will be assumed if left empty.
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.dict.pair(Name(b"Width"), width);
        self
    }

    /// Write the `/Height` attribute for table row groups and illustrative
    /// elements. No instrinsic height will be assumed if left empty.
    pub fn height(&mut self, height: f32) -> &mut Self {
        self.dict.pair(Name(b"Height"), height);
        self
    }
}

/// The text alignment.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TextAlign {
    /// At the start of the inline advance direction.
    Start,
    /// Centered.
    Center,
    /// At the end of the inline advance direction.
    End,
    /// Justified.
    Justify,
}

impl TextAlign {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Start => Name(b"Start"),
            Self::Center => Name(b"Center"),
            Self::End => Name(b"End"),
            Self::Justify => Name(b"Justify"),
        }
    }
}

/// Illustration elements.
impl LayoutAttributes<'_> {
    /// Write the `/BBox` attribute.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.dict.pair(Name(b"BBox"), bbox);
        self
    }
}

/// Table header and data.
impl LayoutAttributes<'_> {
    /// Write the `/BlockAlign` attribute.
    pub fn block_align(&mut self, align: BlockAlign) -> &mut Self {
        self.dict.pair(Name(b"BlockAlign"), align.to_name());
        self
    }

    /// Write the `/InlineAlign` attribute.
    pub fn inline_align(&mut self, align: InlineAlign) -> &mut Self {
        self.dict.pair(Name(b"InlineAlign"), align.to_name());
        self
    }

    /// Write the `/TBorderStyle` attribute. PDF 1.5+.
    pub fn table_border_style(&mut self, style: LayoutBorderStyle) -> &mut Self {
        self.dict.pair(Name(b"TBorderStyle"), style.to_name());
        self
    }

    /// Write the `/TPadding` attribute. PDF 1.5+.
    pub fn table_padding(&mut self, padding: [f32; 4]) -> &mut Self {
        self.dict.insert(Name(b"TPadding")).array().typed().items(padding);
        self
    }
}

/// The block alignment.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BlockAlign {
    /// At the start of the block advance direction.
    Begin,
    /// Centered.
    Middle,
    /// At the end of the block advance direction.
    After,
    /// Justified.
    Justify,
}

impl BlockAlign {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Begin => Name(b"Begin"),
            Self::Middle => Name(b"Middle"),
            Self::After => Name(b"After"),
            Self::Justify => Name(b"Justify"),
        }
    }
}

/// The inline alignment.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InlineAlign {
    /// At the start of the inline advance direction.
    Start,
    /// Centered.
    Center,
    /// At the end of the inline advance direction.
    End,
}

impl InlineAlign {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Start => Name(b"Start"),
            Self::Center => Name(b"Center"),
            Self::End => Name(b"End"),
        }
    }
}

/// Inline elements.
impl LayoutAttributes<'_> {
    /// Write the `/LineHeight` attribute.
    pub fn line_height(&mut self, height: f32) -> &mut Self {
        self.dict.pair(Name(b"LineHeight"), height);
        self
    }

    /// Write the `/BaselineShift` attribute.
    pub fn baseline_shift(&mut self, shift: f32) -> &mut Self {
        self.dict.pair(Name(b"BaselineShift"), shift);
        self
    }

    /// Write the `/TextDecorationType` attribute. PDF 1.5+.
    pub fn text_decoration_type(&mut self, decoration: TextDecorationType) -> &mut Self {
        self.dict.pair(Name(b"TextDecorationType"), decoration.to_name());
        self
    }

    /// Write the `/TextDecorationColor` attribute in RGB. PDF 1.5+.
    pub fn text_decoration_color(&mut self, color: [f32; 3]) -> &mut Self {
        self.dict
            .insert(Name(b"TextDecorationColor"))
            .array()
            .typed()
            .items(color);
        self
    }

    /// Write the `/TextDecorationThickness` attribute. PDF 1.5+.
    pub fn text_decoration_thickness(&mut self, thickness: f32) -> &mut Self {
        self.dict.pair(Name(b"TextDecorationThickness"), thickness);
        self
    }
}

/// The text decoration type (over- and underlines).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TextDecorationType {
    /// No decoration.
    None,
    /// Underlined.
    Underline,
    /// Line over the text.
    Overline,
    /// Strike the text.
    LineThrough,
}

impl TextDecorationType {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::None => Name(b"None"),
            Self::Underline => Name(b"Underline"),
            Self::Overline => Name(b"Overline"),
            Self::LineThrough => Name(b"LineThrough"),
        }
    }
}

/// Grouping elements.
impl<'a> LayoutAttributes<'a> {
    /// Write the `/ColumnCount` attribute. PDF 1.6+.
    pub fn column_count(&mut self, count: i32) -> &mut Self {
        self.dict.pair(Name(b"ColumnCount"), count);
        self
    }

    /// Start writing the `/ColumnWidths` array. The last number in the array is
    /// used for all extra columns. PDF 1.6+.
    pub fn column_widths(&mut self) -> TypedArray<'_, f32> {
        self.dict.insert(Name(b"ColumnWidths")).array().typed()
    }

    /// Start writing the `/ColumnGap` array. The last number in the array is used
    /// for all extra columns. PDF 1.6+.
    pub fn column_gap(&mut self) -> TypedArray<'_, f32> {
        self.dict.insert(Name(b"ColumnGap")).array().typed()
    }
}

/// Vertical Text.
impl LayoutAttributes<'_> {
    /// Write the `/GlyphOrientationVertical` attribute as an angle between 0 and 360. PDF 1.5+.
    pub fn glyph_orientation_vertical(&mut self, angle: f32) -> &mut Self {
        self.dict.pair(Name(b"GlyphOrientationVertical"), angle);
        self
    }
}

/// Ruby annotations.
impl LayoutAttributes<'_> {
    /// Write the `/RubyAlign` attribute. PDF 1.5+.
    pub fn ruby_align(&mut self, align: RubyAlign) -> &mut Self {
        self.dict.pair(Name(b"RubyAlign"), align.to_name());
        self
    }

    /// Write the `/RubyPosition` attribute. PDF 1.5+.
    pub fn ruby_position(&mut self, position: RubyPosition) -> &mut Self {
        self.dict.pair(Name(b"RubyPosition"), position.to_name());
        self
    }
}

/// The alignment of a ruby annotation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RubyAlign {
    /// At the start of the inline advance direction.
    Start,
    /// Centered.
    Center,
    /// At the end of the inline advance direction.
    End,
    /// Justified.
    Justify,
    /// Distribute along the full width of the line with additional space.
    Distribute,
}

impl RubyAlign {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Start => Name(b"Start"),
            Self::Center => Name(b"Center"),
            Self::End => Name(b"End"),
            Self::Justify => Name(b"Justify"),
            Self::Distribute => Name(b"Distribute"),
        }
    }
}

/// The position of a ruby annotation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RubyPosition {
    /// Before edge of the element.
    Before,
    /// After edge of the element.
    After,
    /// Render as a Warichu.
    Warichu,
    /// Render in-line.
    Inline,
}

impl RubyPosition {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Before => Name(b"Before"),
            Self::After => Name(b"After"),
            Self::Warichu => Name(b"Warichu"),
            Self::Inline => Name(b"Inline"),
        }
    }
}

deref!('a, LayoutAttributes<'a> => Dict<'a>, dict);

/// Writer for an _list attributes dictionary_. PDF 1.4+
///
/// This struct is created by [`Attributes::list`].
pub struct ListAttributes<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for ListAttributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"O"), AttributeOwner::List.to_name());
        Self { dict }
    }
}

impl<'a> ListAttributes<'a> {
    /// Write the `/ListNumbering` attribute.
    pub fn list_numbering(&mut self, numbering: ListNumbering) -> &mut Self {
        self.dict.pair(Name(b"ListNumbering"), numbering.to_name());
        self
    }
}

deref!('a, ListAttributes<'a> => Dict<'a>, dict);

/// The list numbering type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ListNumbering {
    /// No numbering.
    None,
    /// Solid circular bullets.
    Disc,
    /// Open circular bullets.
    Circle,
    /// Solid square bullets.
    Square,
    /// Decimal numbers.
    Decimal,
    /// Lowercase Roman numerals.
    LowerRoman,
    /// Uppercase Roman numerals.
    UpperRoman,
    /// Lowercase letters.
    LowerAlpha,
    /// Uppercase letters.
    UpperAlpha,
}

impl ListNumbering {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::None => Name(b"None"),
            Self::Disc => Name(b"Disc"),
            Self::Circle => Name(b"Circle"),
            Self::Square => Name(b"Square"),
            Self::Decimal => Name(b"Decimal"),
            Self::LowerRoman => Name(b"LowerRoman"),
            Self::UpperRoman => Name(b"UpperRoman"),
            Self::LowerAlpha => Name(b"LowerAlpha"),
            Self::UpperAlpha => Name(b"UpperAlpha"),
        }
    }
}

/// Writer for an _PrintFields attributes dictionary_. PDF 1.6+
///
/// This struct is created by [`Attributes::field`].
pub struct FieldAttributes<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for FieldAttributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"O"), AttributeOwner::PrintField.to_name());
        Self { dict }
    }
}

impl<'a> FieldAttributes<'a> {
    /// Write the `/Role` attribute to determine the kind of form control.
    pub fn role(&mut self, role: FieldRole) -> &mut Self {
        self.dict.pair(Name(b"Role"), role.to_name());
        self
    }

    /// Write the `/checked` attribute to set whether a radio button or checkbox
    /// is checked.
    pub fn checked(&mut self, checked: FieldCheckState) -> &mut Self {
        self.dict.pair(Name(b"checked"), checked.to_name());
        self
    }

    /// Write the `/Desc` attribute to set the description of the form control.
    pub fn desc(&mut self, desc: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Desc"), desc);
        self
    }
}

deref!('a, FieldAttributes<'a> => Dict<'a>, dict);

/// The kind of form control.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FieldRole {
    /// A button.
    Button,
    /// A checkbox.
    CheckBox,
    /// A radio button.
    RadioButton,
    /// A text field.
    TextField,
}

impl FieldRole {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Button => Name(b"pb"),
            Self::CheckBox => Name(b"cb"),
            Self::RadioButton => Name(b"rb"),
            Self::TextField => Name(b"tv"),
        }
    }
}

/// The checked state of a check box or radio button.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FieldCheckState {
    /// The check box or radio button is unchecked.
    Unchecked,
    /// The check box or radio button is checked.
    Checked,
    /// The check box or radio button is in a quantum superstate.
    Neutral,
}

impl FieldCheckState {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Unchecked => Name(b"off"),
            Self::Checked => Name(b"on"),
            Self::Neutral => Name(b"neutral"),
        }
    }
}

/// Writer for an _table attributes dictionary_. PDF 1.4+
///
/// This struct is created by [`Attributes::table`].
pub struct TableAttributes<'a> {
    dict: Dict<'a>,
}

impl<'a> Writer<'a> for TableAttributes<'a> {
    fn start(obj: Obj<'a>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"O"), AttributeOwner::Table.to_name());
        Self { dict }
    }
}

impl<'a> TableAttributes<'a> {
    /// Write the `/RowSpan` attribute to set the number of rows that shall be
    /// spanned by this cell.
    pub fn row_span(&mut self, row_span: i32) -> &mut Self {
        self.dict.pair(Name(b"RowSpan"), row_span);
        self
    }

    /// Write the `/ColSpan` attribute to set the number of columns that shall
    /// be spanned by this cell.
    pub fn col_span(&mut self, col_span: i32) -> &mut Self {
        self.dict.pair(Name(b"ColSpan"), col_span);
        self
    }

    /// Write the `/Headers` attribute to refer to the header cells of the
    /// table. PDF 1.6+.
    pub fn headers(&mut self) -> TypedArray<'_, Str> {
        self.dict.insert(Name(b"Headers")).array().typed()
    }

    /// Write the `/Scope` attribute to define whether a table header cell
    /// refers to its row or column.
    pub fn scope(&mut self, scope: TableHeadingScope) -> &mut Self {
        self.dict.pair(Name(b"Scope"), scope.to_name());
        self
    }

    /// Write the `/Summary` attribute to set the summary of the table. PDF
    /// 1.7+.
    pub fn summary(&mut self, summary: TextStr) -> &mut Self {
        self.dict.pair(Name(b"Summary"), summary);
        self
    }
}

deref!('a, TableAttributes<'a> => Dict<'a>, dict);

/// The scope of a table header cell.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TableHeadingScope {
    /// The header cell refers to the row.
    Row,
    /// The header cell refers to the column.
    Column,
    /// The header cell refers to both the row and the column.
    Both,
}

impl TableHeadingScope {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Row => Name(b"Row"),
            Self::Column => Name(b"Column"),
            Self::Both => Name(b"Both"),
        }
    }
}

/// Owner of the attribute dictionary. PDF 1.4+.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AttributeOwner {
    /// General layout attributes.
    Layout,
    /// List attributes.
    List,
    /// Attributes governing the print out behavior of form fields. PDF 1.7+.
    PrintField,
    /// Table attributes.
    Table,
    /// Hints for conversion to XML 1.0.
    Xml,
    /// Hints for conversion to HTML 3.2.
    Html3_2,
    /// Hints for conversion to HTML 4.01.
    Html4,
    /// Hints for conversion to OEB 1.0.
    Oeb,
    /// Hints for conversion to RTF 1.05.
    Rtf1_05,
    /// Hints for conversion to CSS 1.
    Css1,
    /// Hints for conversion to CSS 2.
    Css2,
    /// User-defined attributes. Requires to set the `/UserProperties` attribute
    /// of the [`MarkInfo`] dictionary to true. PDF 1.6+
    User,
}

impl AttributeOwner {
    pub(crate) fn to_name(self) -> Name<'static> {
        match self {
            Self::Layout => Name(b"Layout"),
            Self::List => Name(b"List"),
            Self::PrintField => Name(b"PrintField"),
            Self::Table => Name(b"Table"),
            Self::Xml => Name(b"XML-1.00"),
            Self::Html3_2 => Name(b"HTML-3.20"),
            Self::Html4 => Name(b"HTML-4.01"),
            Self::Oeb => Name(b"OEB-1.00"),
            Self::Rtf1_05 => Name(b"RTF-1.05"),
            Self::Css1 => Name(b"CSS-1.00"),
            Self::Css2 => Name(b"CSS-2.00"),
            Self::User => Name(b"UserDefined"),
        }
    }
}
