//! This example show how to write document info.

use pdf_writer::{Date, Finish, Pdf, Ref, TextStr};

fn main() -> std::io::Result<()> {
    // Start writing.
    let mut pdf = Pdf::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let document_info_id = Ref::new(4); // a ref to a dict

    {
        let mut document_info = pdf.document_info(document_info_id);
        document_info.author(TextStr("The author"));
        document_info.creation_date(Date::new(2026).month(10));
        document_info.producer(TextStr("Producer"));
        // Fill the other fields in `document_info` if needed
    }

    // Write the document catalog with a reference to the page tree.
    pdf.catalog(catalog_id).pages(page_tree_id);

    // Write the page tree with a single child page.
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Write a page.
    let page = pdf.page(page_id);
    page.finish();

    // Finish writing (this automatically creates the cross-reference table and
    // file trailer) and retrieve the resulting byte buffer.
    let buf: Vec<u8> = pdf.finish();

    // Write the thing to a file.
    std::fs::write("target/document_info.pdf", buf)
}
