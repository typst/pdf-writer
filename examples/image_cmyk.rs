//! This example shows you how to embed raster images into a PDF.

use jpeg_decoder::{Decoder as JpegDecoder, ImageInfo, PixelFormat};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref};

fn main() -> std::io::Result<()> {
    // Start writing.
    let mut pdf = Pdf::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let image_id = Ref::new(4);
    let icc_ref = Ref::new(5);
    let content_id = Ref::new(6);
    let image_name = Name(b"Im1");

    // Set up the page tree. For more details see `hello.rs`.
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Specify one A4 page and map the image name "Im1" to the id of the
    // embedded image stream.
    let mut page = pdf.page(page_id);
    let a4 = Rect::new(0.0, 0.0, 595.0, 842.0);
    page.media_box(a4);
    page.parent(page_tree_id);
    page.contents(content_id);
    page.resources().x_objects().pair(image_name, image_id);
    page.finish();

    // Decode the image.
    let input_image = std::fs::read("examples/tiger-cmyk.jpg").unwrap();

    // CMYK JPEGs and PDFs have different conventions for what 0 and 255 mean:
    // They are the reverse of each other. In most cases, we can handle this
    // using a `Decode` array. But if maximum compatibility is important, you
    // should invert the color values before embedding the image.
    //
    // We check if this is a CMYK file. If so, we invert the colors and write a
    // new buffer. If not, we save ourselves some work and just use the original
    // buffer.
    let mut decoder = JpegDecoder::new(&*input_image);
    // We need to call this for the decoder to read the image info.
    decoder.read_info().unwrap();
    let icc_profile = decoder.icc_profile();
    let ImageInfo { width, height, pixel_format, .. } = decoder.info().unwrap();
    let bits_per_component =
        if matches!(pixel_format, PixelFormat::L16) { 16 } else { 8 };

    let image_data = if matches!(pixel_format, PixelFormat::CMYK32) {
        let mut pixels = decoder.decode().unwrap();
        pixels = pixels.into_iter().map(|c| 255 - c).collect();

        let mut w = Vec::new();
        let encoder = jpeg_encoder::Encoder::new(&mut w, 80);
        encoder
            .encode(&pixels, width, height, jpeg_encoder::ColorType::Cmyk)
            .unwrap();
        w
    } else {
        input_image
    };

    // Now, there are multiple considerations:
    // - Writing an XObject with just the raw samples would work, but lead to
    //   huge file sizes since the image would be embedded without any
    //   compression.
    // - We can encode the samples with a filter. However, which filter is best
    //   depends on the file format. For example, for JPEGs you should use
    //   DCT-Decode and for PNGs you should use Deflate.
    let (filter, encoded) = (Filter::DctDecode, image_data);

    // Write the stream for the image we want to embed.
    let mut image = pdf.image_xobject(image_id, &encoded);
    image.filter(filter);
    image.width(width as i32);
    image.height(height as i32);

    // The image needs some color management. We can either use the ICC profile
    // if the file has any or default to the appropriate device color space.
    if icc_profile.is_some() {
        image.color_space().icc_based(icc_ref);
    } else {
        match pixel_format {
            PixelFormat::L8 | PixelFormat::L16 => image.color_space().device_gray(),
            PixelFormat::CMYK32 => image.color_space().device_cmyk(),
            PixelFormat::RGB24 => image.color_space().device_rgb(),
        };
    }

    image.bits_per_component(bits_per_component);
    image.intent(pdf_writer::types::RenderingIntent::RelativeColorimetric);

    image.finish();

    // Size the image at 1pt per pixel.
    let w = 1024.0;
    let h = 670.0;

    // Center the image on the page.
    let x = (a4.x2 - w) / 2.0;
    let y = (a4.y2 - h) / 2.0;

    if let Some(icc_profile) = icc_profile {
        let mut icc = pdf.icc_profile(icc_ref, &icc_profile);
        match pixel_format {
            PixelFormat::L8 | PixelFormat::L16 => {
                icc.alternate().device_gray();
                icc.n(1);
            }
            PixelFormat::CMYK32 => {
                icc.alternate().device_cmyk();
                icc.n(4);
            }
            PixelFormat::RGB24 => {
                icc.alternate().device_rgb();
                icc.n(3);
            }
        };
    }

    // Place and size the image in a content stream.
    //
    // By default, PDF XObjects always have a size of 1x1 user units (and 1 user
    // unit is one 1pt if you don't change that). To position and size them, you
    // have to change the current transformation matrix, which is structured as
    // [scale_x, skew_x, skew_y, scale_y, translate_x, translate_y]. Also,
    // remember that the PDF coordinate system starts at the bottom left! When
    // you have other elements after the image, it's also important to save &
    // restore the state so that they are not affected by the transformation.
    let mut content = Content::new();
    content.save_state();
    content.transform([w, 0.0, 0.0, h, x, y]);
    content.x_object(image_name);
    content.restore_state();
    pdf.stream(content_id, &content.finish());

    // Write the thing to a file.
    std::fs::write("target/image-cmyk.pdf", pdf.finish())
}
