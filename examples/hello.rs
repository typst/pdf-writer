use pdf_writer::{Name, PdfWriter, Rect, Ref, TextStream};

fn main() -> std::io::Result<()> {
    // Start writing with PDF version 1.7 header. The version is not
    // semantically important to the writer, but must be present in the output
    // document.
    let mut writer = PdfWriter::new(1, 7);

    // Make the output more readable by indenting things with 2 spaces.
    writer.set_indent(2);

    // Define some indirect reference ids we'll to use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let font_id = Ref::new(4);
    let text_id = Ref::new(5);

    // Write the document catalog with a reference to the page tree.
    writer.catalog(catalog_id).pages(page_tree_id);

    // Write the page tree with a single child page.
    writer.pages(page_tree_id).kids(vec![page_id]);

    // Write a page.
    //
    // Set the size to A4 (measured in points) using `media_box` and set the
    // text object we'll write later as the page's contents.
    //
    // We also need to specify which resources the page needs, which in our case
    // is only a font that we name "F1" (the specific name doesn't matter).
    writer
        .page(page_id)
        .parent(page_tree_id)
        .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
        .contents(text_id)
        .resources()
        .fonts()
        .pair(Name(b"F1"), font_id);

    // Specify the font we want to use. Because Helvetica is one of the 14 base
    // fonts shipped with every PDF reader, we don't have to embed any font
    // data.
    writer.type1_font(font_id).base_font(Name(b"Helvetica"));

    // Write a line of text, with the font specified in the resource list
    // before, at a font size of 14.0, starting at coordinates (108.0, 734.0)
    // measured from the bottom left of the page.
    //
    // Because we haven't specified any encoding when writing the Type 1 font,
    // the standard encoding is used which happens to work with most ASCII
    // characters.
    writer.stream(
        text_id,
        &TextStream::new()
            .tf(Name(b"F1"), 14.0)
            .td(108.0, 734.0)
            .tj(b"Hello World from Rust!")
            .end(),
    );

    // Finish writing (this automatically creates the cross-reference table and
    // file trailer) and retrieve the resulting byte buffer.
    let buf: Vec<u8> = writer.end(catalog_id);

    // Write the thing to a file.
    std::fs::write("target/hello.pdf", buf)
}
