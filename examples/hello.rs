//! This example gives you a first introduction on how to use pdf-writer.

use pdf_writer::types::{ActionType, AnnotationType, BorderType};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, TextStr};

fn main() -> std::io::Result<()> {
    // Start writing.
    let mut pdf = Pdf::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let font_id = Ref::new(4);
    let content_id = Ref::new(5);
    let font_name = Name(b"F1");

    // Write the document catalog with a reference to the page tree.
    pdf.catalog(catalog_id).pages(page_tree_id);

    // Write the page tree with a single child page.
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Write a page.
    let mut page = pdf.page(page_id);

    // Set the size to A4 (measured in points) using `media_box` and set the
    // text object we'll write later as the page's contents.
    page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
    page.parent(page_tree_id);
    page.contents(content_id);

    // We also create the annotations list here that allows us to have things
    // like links or comments on the page.
    let mut annotations = page.annotations();
    let mut annotation = annotations.push();

    // Write the type, area, alt-text, and color for our link annotation.
    annotation.subtype(AnnotationType::Link);
    annotation.rect(Rect::new(215.0, 730.0, 251.0, 748.0));
    annotation.contents(TextStr("Link to the Rust project web page"));
    annotation.color_rgb(0.0, 0.0, 1.0);

    // Write an action for the annotation, telling it where to link to. Actions
    // can be associated with annotations, outline objects, and more and allow
    // creating interactive PDFs (open links, play sounds...).
    annotation
        .action()
        .action_type(ActionType::Uri)
        .uri(Str(b"https://www.rust-lang.org/"));

    // Set border and style for the link annotation.
    annotation.border_style().width(2.0).style(BorderType::Underline);

    // We have to finish all the writers that depend on the page here because
    // otherwise they would be mutably borrowed until the end of the block.
    // Finishing is handled through the writer's `Drop` implementations, so that
    // you cannot accidentally forget it. The `finish()` method from the `Finish`
    // trait is just a postfix-style version of dropping.
    annotation.finish();
    annotations.finish();

    // We also need to specify which resources the page needs, which in our case
    // is only a font that we name "F1" (the specific name doesn't matter).
    page.resources().fonts().pair(font_name, font_id);
    page.finish();

    // Specify the font we want to use. Because Helvetica is one of the 14 base
    // fonts shipped with every PDF reader, we don't have to embed any font
    // data.
    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

    // Write a line of text, with the font specified in the resource list
    // before, at a font size of 14.0, starting at coordinates (108.0, 734.0)
    // measured from the bottom left of the page.
    //
    // Because we haven't specified any encoding when writing the Type 1 font,
    // the standard encoding is used which happens to work with most ASCII
    // characters.
    let mut content = Content::new();
    content.begin_text();
    content.set_font(font_name, 14.0);
    content.next_line(108.0, 734.0);
    content.show(Str(b"Hello World from Rust!"));
    content.end_text();
    pdf.stream(content_id, &content.finish());

    // Finish writing (this automatically creates the cross-reference table and
    // file trailer) and retrieve the resulting byte buffer.
    let buf: Vec<u8> = pdf.finish();

    // Write the thing to a file.
    std::fs::write("target/hello.pdf", buf)
}
