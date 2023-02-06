//! This example shows how to use ICC-based color spaces.

use pdf_writer::writers::ColorSpace;
use pdf_writer::{Content, Finish, Name, PdfWriter, Rect, Ref};

fn main() -> std::io::Result<()> {
    // Start writing.
    let mut writer = PdfWriter::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let content_id = Ref::new(4);

    // This will reference the stream containing the ICC profile. It can be used
    // with multiple `Resource` dictionaries.
    let icc_id = Ref::new(5);

    // The name with which we can reference the color space array in the content
    // stream. This can be any name and will be assigned to the color space
    // array in the resource dictionary associated with the content stream.
    let color_space_name = Name(b"sRGB");

    // Set up the page tree. For more details see `hello.rs`.
    writer.catalog(catalog_id).pages(page_tree_id);
    writer.pages(page_tree_id).kids([page_id]).count(1);

    // Write a page.
    let mut page = writer.page(page_id);

    // Create an A4 page.
    page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
    page.parent(page_tree_id);
    page.contents(content_id);

    // The resource dictionary of the page defines which color spaces its
    // content stream can reference. Because `ICCBased` color spaces are streams
    // with binary data, they cannot be defined inline. Instead, we associate
    // the designated name of our space with an indirect reference to the stream
    // that we will write later.
    page.resources()
        .color_spaces()
        .insert(color_space_name)
        .start::<ColorSpace>()
        .icc_based(icc_id);
    page.finish();

    // Write the content stream with a green rectangle and a crescent with a red
    // stroke.
    let mut content = Content::new();
    // We first need to set the color space for the `set_fill_color` / `scn`
    // operator. We'll use the name that we registered in the resource
    // dictionary above.
    content.set_fill_color_space(color_space_name);
    // Set the fill color in the current color space. Note that only the
    // `set_fill_color` and `set_stroke_color` operators will use custom color
    // spaces. The `set_fill_rgb`, `set_fill_cmyk` and `set_fill_gray` operators
    // will always use the non-calibrated Device color spaces, the same applies
    // to the stroke color operators.
    content.set_fill_color([0.0, 1.0, 0.0]);
    // Draw a green rectangle at the top of the page.
    content.rect(108.0, 734.0, 100.0, 100.0);
    // The `re` operator already closed the rectangle path, so we can just fill
    // it.
    content.fill_even_odd();

    // Fill and stroke color spaces must be set independently.
    content.set_stroke_color_space(color_space_name);
    content.set_stroke_color([1.0, 0.0, 0.0]);

    // Draw a crescent.
    // Move to the starting point of the path.
    content.move_to(208.0, 734.0);
    // Two symmetric cubic Bézier curves.
    content.cubic_to(208.0, 734.0, 208.0, 834.0, 308.0, 834.0);
    content.cubic_to(308.0, 834.0, 308.0, 734.0, 208.0, 734.0);
    // Close the path and stroke it.
    content.close_and_stroke();

    // Write the content stream.
    writer.stream(content_id, &content.finish());

    // Read the ICC profile from a file.
    let icc_data = std::fs::read("examples/sRGB_v4.icc")?;
    // Start writing the ICC profile stream. In production use, you would
    // compress the data stream with the `FlateDecode` filter. Check the
    // `image.rs` example for details.
    let mut icc_profile = writer.icc_profile(icc_id, &icc_data);

    // PDF requires metadata about the ICC profile. We provide it as entries in
    // the stream dictionary. The `n` entry is required and specifies the number
    // of components in the color space.
    icc_profile.n(3);
    // Set an alternate color space. This is optional and will be used if the
    // reader cannot interpret the ICC profile. The alternate color space must
    // have the same number of components as the ICC profile (see above). Here,
    // we use the `CalRGB` color space that approximates the sRGB ICC profile
    // and comes with `pdf-writer`.
    icc_profile.alternate().srgb();
    // Set the range of the color space. This is optional. We specified the
    // same value here that a PDF reader would use if we didn't specify it.
    icc_profile.range([0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    icc_profile.finish();

    // Write the thing to a file.
    std::fs::write("target/icc.pdf", writer.finish())
}
