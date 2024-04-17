//! This example shows how to create forms accepted by the most popular readers.

use pdf_writer::types::{
    ActionType, AnnotationFlags, BorderType, FieldFlags, FieldType, FormActionFlags,
};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, TextStr};

fn main() -> std::io::Result<()> {
    let mut pdf = Pdf::new();

    // Let's set up our primary font, we'll have to reference it a few times.
    let text_font_id = Ref::new(1);
    let text_font_name = Name(b"F1");

    // Here we'll set up our Dingbat font, this is used for symbols such as the
    // ticks in checkboxes.
    let symbol_font_id = Ref::new(2);
    let symbol_font_name = Name(b"F2");

    // One of the most common form field types is the text field. Let's add that
    // and look at some of the basics of PDF form fields.
    let text_field_id = Ref::new(4);

    // We start by writing a form field dictionary with an id which we later
    // need for referencing it.
    let mut field = pdf.form_field(text_field_id);

    // While the `/T` attribute is optional according to the spec, you should
    // include it, most readers will only render widget annotations with both
    // partial name and field type. Next, we set it's value and default value:
    // - The value is used to store what the user has put into the field.
    // - The default value is used when resetting the form.
    field
        .partial_name(TextStr("text"))
        .field_type(FieldType::Text)
        .text_value(TextStr("Hello"))
        .text_default_value(TextStr("Who reset me"));

    // Our field is a terminal field because it has no children, so it's merged
    // with its widget annotation. The widget annotation is what declares the
    // appearance and position in the document, whereas the field defines its
    // semantic behavior for the document-wide form. The appearance is more
    // relevant to button fields, we'll see how to cofigure it below.
    let mut annot = field.to_annotation();
    annot.rect(Rect::new(108.0, 730.0, 208.0, 748.0));

    // We can pass some fairly simple appearances here, common things such
    // as the border color and style. This will give out field a purple
    // underline, keep in mind that this may be drowned out by the viewer's
    // form highlighting.
    annot.border_style().style(BorderType::Underline);
    annot.appearance_characteristics().border_color_rgb(0.0, 0.0, 0.5);

    // TODO: test whether this is actually needed depending on the
    // appearance stream

    // The reader will usually provide a default appearance and automatically
    // highlight form fields. The appearance is relevant for printing however.
    // While we don't provide an explicit appearnce here, if we did we likely
    // want this flag to be set.
    annot.flags(AnnotationFlags::PRINT);
    annot.finish();

    // A good form has radio buttons. Radio buttons are checkboxes which turn
    // off when another checkbox is turned on. A group of radio button widget
    // annotations shares a single radio button field as parent.
    let radio_group_id = Ref::new(5);

    // The FormXObjects for our checkboxes need bounding boxes, in this case
    // these are the same size as out rectangles, but within their coordinate
    // system.
    let bbox = Rect::new(0.0, 0.0, 30.0, 18.0);

    // We define our three radio buttons, they all have a different appearance
    // streams, but if they shared the same appearance stream and used the
    // RADIOS_IN_UNISON flag, then two buttons could refer to the same choice.
    // This is not widely supported, so we'll simply showcase some normal radio
    // buttons here.
    //
    // NOTE: A reader like Okular will also use on-state name in the default
    //       appearance.
    let radios = [
        (Ref::new(6), Rect::new(108.0, 710.0, 138.0, 728.0), b"ch1"),
        (Ref::new(7), Rect::new(140.0, 710.0, 170.0, 728.0), b"ch2"),
        (Ref::new(8), Rect::new(172.0, 710.0, 202.0, 728.0), b"ch3"),
    ];
    // First, we define the radio group parent. The children of this field will
    // be our actual buttons. We can define most of the radio related properties
    // here.
    let mut field = pdf.form_field(radio_group_id);

    // We set some flags to get the exact behavior we want.
    // - FieldFlags::NO_TOGGLE_OFF means that once a button is selected it
    //   cannot be manually turned off without turning another button on.
    // - FieldFlags::RADIOS_IN_UNISON ensures that if we have buttons which use
    //   the same appearance on-state, they'll be toggled in unison with the
    //   others (although we don't use this here).
    // Finally we define the children of this field, the widget annotations
    // which again define appearance and postion of the individual buttons.
    //
    // NOTE: by the time of writing this, RADIOS_IN_UNISON does not work
    //       correctly pdf.js (firefox), okular or evince.
    field
        .partial_name(TextStr("radio"))
        .field_type(FieldType::Button)
        .field_flags(
            FieldFlags::RADIO
                | FieldFlags::NO_TOGGLE_TO_OFF
                | FieldFlags::RADIOS_IN_UNISON,
        )
        .children(radios.map(|(id, _, _)| id));
    field.finish();

    // For buttons appearances are more relevant when printing as they're
    // usually not as easy to find as text fields if they have no appearance.
    let radio_on_appearance_id = Ref::new(9);
    let radio_off_appearance_id = Ref::new(10);

    // Here we prepare our appearances, the on appearance is a tick and the off
    // appearance is empty.
    let mut content = Content::new();
    content.save_state();
    content.begin_text();
    content.set_fill_gray(0.0);
    content.set_font(symbol_font_name, 14.0);
    // The character 4 is a tick in this font.
    content.show(Str(b"4"));
    content.end_text();
    content.restore_state();

    let on_stream = content.finish();
    let mut on_appearance = pdf.form_xobject(radio_on_appearance_id, &on_stream);

    on_appearance.bbox(bbox);

    // We use the symbol font to display the tick, so we need to add it to the
    // resources of the appearance stream.
    on_appearance
        .resources()
        .fonts()
        .pair(symbol_font_name, symbol_font_id);

    on_appearance.finish();

    // Our off appearance is empty, we haven't ticked the box.
    pdf.form_xobject(radio_off_appearance_id, &Content::new().finish())
        .bbox(bbox);

    // Now we'll write a widget annotation for each button.
    for (id, rect, state) in radios {
        // While we create a field here we could directly create widget
        // annotation too.
        let mut field = pdf.form_field(id);

        // Each button shares the single parent.
        field.parent(radio_group_id);

        let mut annot = field.to_annotation();
        annot.rect(rect).flags(AnnotationFlags::PRINT);

        // This is the state the button starts off with. `/Off` is the off state
        // and is the same for all radio buttons. The `on` state gets its own
        // name to distinguish different buttons.
        annot.appearance_state(Name(b"Off"));

        // Finally we set the appearance dictionary to contain a normal
        // appearance sub dictionary mapping both on and off state to the
        // respective FormXObject.
        {
            let mut appearance = annot.appearance();
            appearance.normal().streams().pairs([
                (Name(state), radio_on_appearance_id),
                (Name(b"Off"), radio_off_appearance_id),
            ]);
        }
    }

    // Let's add a dropdown menu and allow the user to chose from preconfigrued
    // options while allowing them to add their own custom option too.
    let dropdown_id = Ref::new(11);
    let mut field = pdf.form_field(dropdown_id);

    // Choice fields come in two types, list and combo boxes. A combo box is
    // also known as a dropdown menu, a list box is like a permanently expanded
    // drop down menu. The edit flag allows the user to insert their own custom
    // option.
    // NOTE: at the time of writing this pdf.js (Firefox) does not allow
    //       editing of the box
    field
        .partial_name(TextStr("choice"))
        .field_type(FieldType::Choice)
        .field_flags(FieldFlags::COMBO | FieldFlags::EDIT);

    // Here we define the options the user will be presented with.
    field.choice_options().options([
        TextStr("male"),
        TextStr("female"),
        TextStr("non-binary"),
        TextStr("prefer not to say"),
    ]);

    let mut annot = field.to_annotation();
    annot
        .rect(Rect::new(108.0, 690.0, 208.0, 708.0))
        .flags(AnnotationFlags::PRINT);
    annot.finish();

    // PDFs can also have push buttons, buttons which retain no state when
    // pressed. We'll use that to demonstrate form actions. Actions can be
    // activated on many events, like a change in the input of a field, or
    // simply the mous cursor moving over the annotation.
    let button_id = Ref::new(12);
    let mut field = pdf.form_field(button_id);

    // We set the push button field, otherwise it's interpreted to be a check
    // box.
    field
        .partial_name(TextStr("button"))
        .field_type(FieldType::Button)
        .field_flags(FieldFlags::PUSHBUTTON);

    let mut annot = field.to_annotation();
    annot
        .rect(Rect::new(108.0, 670.0, 138.0, 688.0))
        .flags(AnnotationFlags::PRINT);

    // We can quickly give it some basic appearance characteristics like
    // background and border color.
    annot.appearance_characteristics().border_color_gray(0.5);

    // Finally, we set the action that is taken when the button is pushed.
    // It should reset fields in the form, but we must tell it which fields.
    // By setting the `FormActionFlags::INCLUDE_EXCLUDE` flag, we tell it to
    // exclude all fields in the we specify and by specifying no fields we
    // ensure all fields are reset.
    annot
        .action()
        .form_flags(FormActionFlags::INCLUDE_EXCLUDE)
        .action_type(ActionType::ResetForm)
        .fields();
    annot.finish();

    // The PDF catalog contains the form dictionary, telling the reader that
    // this document contains interactive form fields.
    let catalog_id = Ref::new(13);
    let page_tree_id = Ref::new(14);
    let mut cat = pdf.catalog(catalog_id);
    cat.pages(page_tree_id);

    // We write all root fields in to the form field dictionary. Root fields are
    // those which have no parent.
    cat.form()
        .fields([text_field_id, radio_group_id, dropdown_id, button_id]);
    cat.finish();

    // First we create a page which should contain the form fields and write
    // its resources.
    let page_id = Ref::new(15);
    let mut page = pdf.page(page_id);
    page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
        .parent(page_tree_id)
        .resources()
        .fonts()
        .pair(text_font_name, text_font_id);

    // Now we write each widget annotations refereence into the annotations
    // array. Those are our terminal fields, those with no children.
    page.annotations([
        text_field_id,
        radios[0].0,
        radios[1].0,
        radios[2].0,
        dropdown_id,
        button_id,
    ]);
    page.finish();

    // Finally we write the font and page tree.
    pdf.type1_font(text_font_id).base_font(Name(b"Helvetica"));
    pdf.type1_font(symbol_font_id).base_font(Name(b"ZapfDingbats"));
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    std::fs::write("target/forms.pdf", pdf.finish())
}
