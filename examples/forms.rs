//! This example shows how to create forms accepted by the most popular readers.

use pdf_writer::types::{
    ActionType, AnnotationFlags, FieldFlags, FieldType, FormActionFlags,
};
use pdf_writer::{Content, Name, Pdf, Rect, Ref, Str, TextStr};

fn main() -> std::io::Result<()> {
    let mut pdf = Pdf::new();

    // let's set up our font, we'll need to reference it a few times
    let font_id = Ref::new(1);
    let font_name = Name(b"F1");

    // let's define a text field
    let text_id = Ref::new(4);
    {
        // we start writing a form field object
        let mut field = pdf.form_field(text_id);

        // we set its field type and it's default value, the value is what the
        // reader will write to when saving the file and we can prefill it, the
        // default_value is what the field will be set to if we reset it using
        // a reset form action, see the button example below
        field
            .partial_name(TextStr("text"))
            .field_type(FieldType::Text)
            .text_value(TextStr("Hello"))
            .text_default_value(TextStr("Who reset me"));

        // because this is a terminal field, we must finish it as an annotation
        // to provide it's location, size and optionally appearance, most
        // readers highlight form fields, so the appearance is optional, the
        // created annotation type is already set to widget, so we don't write
        // it ourselves
        let mut annot = field.to_annotation();
        annot.rect(Rect::new(108.0, 730.0, 208.0, 748.0));

        // TODO: test whether this is actually needed depending on the
        // appearance stream

        // we ensure that the annoation is printed too
        annot.flags(AnnotationFlags::PRINT);
    }

    // a good form has radio buttons, radio button fields are related by their
    // common parent, each set of buttons needs a unique parent
    let radio_id = Ref::new(5);
    let radio_ids = [Ref::new(6), Ref::new(7), Ref::new(8)];
    {
        // this field doens't hold much of a value itself, it simply groups it's
        // child annotations together so the reader knows they are connected
        // without this, we'd simply have checkboxes
        let mut field = pdf.form_field(radio_id);
        field
            .partial_name(TextStr("radio"))
            .field_type(FieldType::Button)
            .field_flags(
                FieldFlags::RADIO
                    | FieldFlags::NO_TOGGLE_TO_OFF
                    | FieldFlags::RADIOS_IN_UNISON,
            )
            .children(radio_ids);
    }

    // we place all three buttons side by side so the user can tell they are
    // one set of radio buttons, but in theory they can be anywhere even on
    // different pages, two of those will have the same appearance, this means
    // they will be turned on and off in unison because we used RADIOS_IN_UNISON
    let radios = [
        (Rect::new(108.0, 710.0, 138.0, 728.0), b"ch1"),
        (Rect::new(140.0, 710.0, 170.0, 728.0), b"ch2"),
        (Rect::new(172.0, 710.0, 202.0, 728.0), b"ch1"),
    ];

    let radio_on_appearance_id = Ref::new(9);
    let radio_off_appearance_id = Ref::new(10);

    // each actual button now get's its own annotation
    for (id, (rect, state)) in radio_ids.into_iter().zip(radios) {
        let mut field = pdf.form_field(id);
        field.parent(radio_id);

        let mut annot = field.to_annotation();
        annot.rect(rect);

        // we state which appearance state each button has by default
        annot.appearance_state(Name(b"Off"));

        // most readers need appearances to distinguish the actual radio buttons
        // the normal appearance should contain an appearance stream for each
        // state of the button, each button should have a unique name for it's
        // on state and it's `/Off` state, if the on-state names are not unique
        // they will shate their state, we see this here with ch1 being in there
        // twice
        {
            let mut appearance = annot.appearance();
            appearance.normal().streams().pairs([
                (Name(state), radio_on_appearance_id),
                (Name(b"Off"), radio_off_appearance_id),
            ]);
        }

        annot.flags(AnnotationFlags::PRINT);
    }

    let mut content = Content::new();
    content.save_state();
    content.begin_text();
    content.set_font(font_name, 14.0);
    content.show(Str(b"y"));
    content.end_text();
    content.restore_state();
    pdf.form_xobject(radio_on_appearance_id, &content.finish());

    let mut content = Content::new();
    content.save_state();
    content.begin_text();
    content.set_font(font_name, 14.0);
    content.show(Str(b"n"));
    content.end_text();
    content.restore_state();
    pdf.form_xobject(radio_off_appearance_id, &content.finish());

    // let's add a dropdown menu and allow the user to chose a custom option
    // too
    let dropdown_id = Ref::new(11);
    {
        let mut field = pdf.form_field(dropdown_id);

        // this is a choice field, the user is presented with human readable
        // options to choose from, by setting the combo and edit flag we make
        // this a drop down menu with a custom option respectively
        //
        // you could leave out the field and make it a bit larger to make it a
        // list box, which is like a permanently open drop down menu
        field
            .partial_name(TextStr("choice"))
            .field_type(FieldType::Choice)
            .field_flags(FieldFlags::COMBO | FieldFlags::EDIT);

        // our predefined options go here
        field.choice_options().options([
            TextStr("male"),
            TextStr("female"),
            TextStr("non-binary"),
            TextStr("prefer not to say"),
        ]);

        let mut annot = field.to_annotation();
        annot.rect(Rect::new(108.0, 690.0, 208.0, 708.0));
        annot.flags(AnnotationFlags::PRINT);
    }

    // let's add a push button to our form to reset all fields easily, this is
    // achieved by giving it a form action
    let button_id = Ref::new(12);
    {
        let mut field = pdf.form_field(button_id);

        // it's a push button, it doesn't retain any state after it was pushed
        field
            .partial_name(TextStr("button"))
            .field_type(FieldType::Button)
            .field_flags(FieldFlags::PUSHBUTTON);

        let mut annot = field.to_annotation();
        annot.rect(Rect::new(108.0, 670.0, 138.0, 688.0));

        // we give it a gray appearance to distinguish it from the other fields
        annot.appearance_characteristics().border_color_gray(0.5);

        // finally we set the action that is taken when the button is pushed
        // it should reset fields in the form, by setting the `INCLUDE_EXCLUDE`
        // flag we tell it to exclude all fields in the we specify and by
        // specifying no fields we ensure all fields are reset
        annot
            .action()
            .form_flags(FormActionFlags::INCLUDE_EXCLUDE)
            .action_type(ActionType::ResetForm)
            .fields();

        annot.flags(AnnotationFlags::PRINT);
    }

    // finally we write the pdf catalog so the pdf reader knows there are
    // fields
    let catalog_id = Ref::new(13);
    let page_tree_id = Ref::new(14);
    {
        let mut cat = pdf.catalog(catalog_id);
        cat.pages(page_tree_id);

        // we write all root fields in to the document catalogue, i.e. those
        // without parent
        cat.form().fields([text_id, radio_id, dropdown_id, button_id]);
    }

    // we write each terminal field into it's repsective page, of which we only
    // have one
    let page_id = Ref::new(15);
    {
        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0)).parent(page_tree_id);

        page.resources().fonts().pair(font_name, font_id);

        // we create a norm a4 page and attach our annotations, the annotations
        // are terminal all the fields, i.e. those without children of their
        // own
        page.annotations([
            text_id,
            radio_ids[0],
            radio_ids[1],
            radio_ids[2],
            dropdown_id,
            button_id,
        ]);
    }

    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    std::fs::write("target/forms.pdf", pdf.finish())
}
