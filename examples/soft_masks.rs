//! This example shows how to apply soft masks in arbitrary content streams.

use pdf_writer::{types::MaskType, Content, Finish, Name, Pdf, Rect, Ref};

fn main() -> std::io::Result<()> {
    // Start writing.
    let mut pdf = Pdf::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let star_group_id = Ref::new(4);
    let content_id = Ref::new(5);
    let ext_g_state_id = Ref::new(6);

    // Write the document catalog with a reference to the page tree.
    pdf.catalog(catalog_id).pages(page_tree_id);

    // The size of the page, also used for the soft-mask bounding box.
    let page_rect = Rect::new(0.0, 0.0, 595.0, 842.0);

    // Draw a star in a content stream and use it as a mask later.
    let mut star_content = Content::new();

    let offset = 150.0;

    // Move the star.
    star_content.transform([1.0, 0.0, 0.0, 1.0, offset, offset]);

    // The star is completely opaque, so the mask will fully reveal the content.
    star_content.set_fill_gray(1.0);
    star_content.move_to(50.0, 10.0);
    star_content.line_to(58.8, 38.0);
    star_content.line_to(88.0, 37.5);
    star_content.line_to(64.3, 54.5);
    star_content.line_to(73.5, 82.5);
    star_content.line_to(50.0, 65.0);
    star_content.line_to(26.5, 82.5);
    star_content.line_to(35.7, 54.5);
    star_content.line_to(12.0, 37.5);
    star_content.line_to(41.2, 38.0);
    star_content.close_path();
    star_content.fill_even_odd();
    let star_content = star_content.finish();

    let ext_g_state_name = Name(b"GS1");
    let mut star_group = pdf.form_xobject(star_group_id, &star_content);
    star_group.resources().finish();
    star_group.group().transparency();
    star_group.bbox(page_rect);
    star_group.finish();

    // External Graphics State dictionaries allow us to set more properties
    // of content streams. In this case, we use them to apply a soft mask.
    let mut ext_g_state = pdf.ext_graphics(ext_g_state_id);
    let mut soft_mask = ext_g_state.soft_mask();
    soft_mask.group(star_group_id);
    // The soft mask will show the underlying content where the mask has some
    // alpha value at that spot.
    soft_mask.subtype(MaskType::Alpha);
    soft_mask.finish();
    ext_g_state.finish();

    // Write the page tree with a single child page.
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Write a page.
    let mut page = pdf.page(page_id);

    // Set the size to A4 (measured in points) using `media_box` and set the
    // text object we'll write later as the page's contents.
    page.media_box(page_rect);
    page.parent(page_tree_id);
    page.contents(content_id);

    // We also need to specify which resources the page needs. In this case, we
    // need to make the soft mask's external graphics state available.
    page.resources().ext_g_states().pair(ext_g_state_name, ext_g_state_id);
    page.finish();

    // Write a few rectangles to the content stream.
    let mut content = Content::new();
    content.set_parameters(ext_g_state_name);
    content.set_fill_rgb(1.0, 0.0, 0.0);
    content.rect(10.0, 10.0, 200.0, 200.0);
    content.fill_nonzero();

    content.set_fill_rgb(0.0, 1.0, 0.0);
    content.rect(200.0, 10.0, 290.0, 200.0);
    content.fill_nonzero();

    content.set_fill_rgb(0.0, 0.0, 1.0);
    content.rect(10.0, 200.0, 200.0, 290.0);
    content.fill_nonzero();

    pdf.stream(content_id, &content.finish());

    // Finish writing (this automatically creates the cross-reference table and
    // file trailer) and retrieve the resulting byte buffer.
    let buf: Vec<u8> = pdf.finish();

    // Write the thing to a file.
    std::fs::write("target/soft_masks.pdf", buf)
}
