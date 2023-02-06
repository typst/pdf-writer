//! This example shows how to use Separation color spaces.

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

    // The names with which we can reference the color spaces in the content
    // stream. This can be any name and will be assigned to the color space
    // stream in the resource dictionary associated with the content stream.
    let metallic_name = Name(b"P8340C");
    let hot_pink_name = Name(b"AcmePink");

    // Set up the page tree. For more details see `hello.rs`.
    writer.catalog(catalog_id).pages(page_tree_id);
    writer.pages(page_tree_id).kids([page_id]).count(1);

    // Write a page.
    let mut page = writer.page(page_id);

    // Create an A4 page.
    let width = 595.0;
    let height = 842.0;
    page.media_box(Rect::new(0.0, 0.0, width, height));
    page.parent(page_tree_id);
    page.contents(content_id);

    // The resource dictionary of the page defines which color spaces its
    // content stream can reference. Because we need to write multiple color
    // spaces, we need to store a mutable reference to the resource dictionary.
    let mut resources = page.resources();
    let mut color_spaces = resources.color_spaces();

    // We insert a separation for the green metallic ink here. The first
    // argument to the `separation` method is the name of the separation that
    // the printer will use to select a colorant. RAL, PANTONE, and other
    // well-known color standards are recognized by many print shops.
    let mut metallic = color_spaces
        .insert(metallic_name)
        .start::<ColorSpace>()
        .separation(Name(b"PANTONE 8340 C"));

    // Separation color spaces rely on their (potentially proprietary) name only
    // to be recognized by a printer. In order to be rendered by a PDF viewer,
    // we need to provide an alternative color space that can be referenced by
    // name in the separation color space.
    metallic.alternate_color_space().lab(
        [0.9642, 1.0, 0.8249],
        Some([0.0, 0.0, 0.0]),
        Some([-128.0, 127.0, -128.0, 127.0]),
    );

    // In order to apply the alternate color space, we need to map from a
    // single-dimensional separation color value between `0` and `1` to a color
    // value in the alternate space. We interpolate between two values in the
    // Lab color space defined above: A neutral white and a greenish color.
    let mut metallic_func = metallic.tint_exponential();
    // The exponent is 1.0, so the interpolation is linear.
    metallic_func.n(1.0);
    metallic_func.domain([0.0, 1.0]);
    // This is a L*a*b* value, so the value with maximum luminance / white is
    // [100.0, 0.0, 0.0].
    metallic_func.c0([100.0, 0.0, 0.0]);
    metallic_func.c1([60.3922, -6.0, 12.0]);
    metallic_func.range([0.0, 100.0, -128.0, 127.0, -128.0, 127.0]);
    metallic_func.finish();

    metallic.finish();

    // Sometimes, the separation color is specific to a customer. In this case,
    // we can use a non-standard name for the separation color space.
    let mut pink = color_spaces
        .insert(hot_pink_name)
        .start::<ColorSpace>()
        .separation(Name(b"Acme Pink"));

    // We set an approximation of the sRGB color space as the alternate color
    // space here. Refer to `examples/icc_based.rs` for an example of how to use
    // ICC profiles to get an accurate sRGB color space.
    pink.alternate_color_space().srgb();

    // We use a function that interpolates between two three-dimensional RGB
    // color values: A neutral white and a hot pink.
    let mut pink_func = pink.tint_exponential();
    // The exponent is 1.0, so the interpolation is linear.
    pink_func.n(1.0);
    pink_func.domain([0.0, 1.0]);
    // In this sRGB color space, we start with full components for all colors to
    // get white.
    pink_func.c0([1.0, 1.0, 1.0]);
    // We want to interpolate to a hot pink color.
    pink_func.c1([1.0, 0.4118, 0.7059]);
    pink_func.range([0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    pink_func.finish();
    pink.finish();

    color_spaces.finish();
    resources.finish();
    page.finish();

    // Write the content stream with five stars in different colors, alternating
    // between metallic green and hot pink colors.
    let mut content = Content::new();
    // We want a total of five stars.
    let stars = 5;
    // We want to leave a margin of 50 points around the edge of the page.
    let margin = 50.0;

    for i in 0..stars {
        // Each star will be offset by 20 points to the bottom right from the
        // previous star.
        let offset = i as f32 * 20.0;

        // Save the graphics state that was in effect before we started
        // drawing the star and push it onto the graphics state stack.
        // This includes color spaces, colors, and transformations.
        content.save_state();

        // We'll use a transformation matrix to move the star to the correct
        // position. The star is defined in a coordinate system where the origin
        // is in the top left corner of the star. Because the PDF coordinates
        // start in the bottom left, we need to flip the y-axis. We also need
        // to move the star to the correct position.
        content.transform([
            1.0,
            0.0,
            0.0,
            -1.0,
            margin + offset,
            height - margin - offset,
        ]);

        // We'll set the fill color to the metallic green or hot pink color
        // space, depending on whether the number is even. Now, the
        // `set_fill_color` method will expect a single value between 0 and 1.
        if i % 2 == 0 {
            content.set_fill_color_space(metallic_name);
        } else {
            content.set_fill_color_space(hot_pink_name);
        }

        // We'll slowly ramp up the color value from 0 to 1. This example also
        // shows that separation colors will not blend ('overprint'). Instead,
        // colors will be completely replaced by the color of the top-most
        // object.
        content.set_fill_color([(i + 1) as f32 / stars as f32]);
        content.move_to(50.0, 10.0);
        content.line_to(58.8, 38.0);
        content.line_to(88.0, 37.5);
        content.line_to(64.3, 54.5);
        content.line_to(73.5, 82.5);
        content.line_to(50.0, 65.0);
        content.line_to(26.5, 82.5);
        content.line_to(35.7, 54.5);
        content.line_to(12.0, 37.5);
        content.line_to(41.2, 38.0);
        content.close_path();
        content.fill_even_odd();

        // Recover the graphics state that was in effect before we started. If
        // we did not do this, the next star's transformation would be composed
        // with the previous star's transformation, again flipping the y-axis.
        content.restore_state();
    }

    // Write the content stream.
    writer.stream(content_id, &content.finish());

    // Write the thing to a file.
    std::fs::write("target/separations.pdf", writer.finish())
}
