//! This example demonstrates how to link/embed videos.

use pdf_writer::types::{ActionType, AnnotationType, MediaClipType, RenditionType};
use pdf_writer::{Content, Finish, Pdf, Rect, Ref, Str, Name, Filter};
use image::ColorType;


fn get_bbox(page: &Rect, mut w: f32, mut h: f32) -> Rect {
    // Limit the width and height of the object to the page size, retaining the
    // aspect ratio.
    if w > (page.x2 - page.x1) {
        let f = (page.x2 - page.x1)/w;
        w *= f;
        h *= f;
    }
    if h > (page.y2 - page.y1) {
        let f = (page.y2 - page.y1)/h;
        w *= f;
        h *= f;
    }

    // Return a bounding box for the object centered on the page.
    Rect::new((page.x2 - w)/2.0,
              (page.y2 - h)/2.0,
              (page.x2 + w)/2.0,
              (page.y2 + h)/2.0)
}


fn main() -> std::io::Result<()> {
    let embedded = true;

    // Start writing.
    let mut pdf = Pdf::new();

    // Define some indirect reference ids we'll use.
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let annotation_id = Ref::new(4);
    let video_file_id = Ref::new(5);
    let form_xobject_id = Ref::new(6);
    let image_id = Ref::new(7);
    let image_name = Name(b"Im1");

    // Set up the page tree. For more details see `hello.rs`.
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Specify one A4 landscape page.
    let mut page = pdf.page(page_id);
    let a4_landscape = Rect::new(0.0, 0.0, 842.0, 595.0);
    page.media_box(a4_landscape);
    page.parent(page_tree_id);
    page.annotations([annotation_id]);
    page.finish();

    // Decode the image.
    // Image extracte from video file using ffmpeg:
    //   ffmpeg -i bear-1280x720.mp4 -vf "select=eq(n\,0)" -q:v 3 bear-1280x720.jpg
    let data = std::fs::read("examples/bear-1280x720.jpg").unwrap();
    let dynamic = image::load_from_memory(&data).unwrap();
    assert!(dynamic.color() == ColorType::Rgb8);

    // Write the stream for the image we want to embed.
    let mut image = pdf.image_xobject(image_id, &data);
    image.filter(Filter::DctDecode);
    image.width(dynamic.width() as i32);
    image.height(dynamic.height() as i32);
    image.color_space().device_rgb();
    image.bits_per_component(8);
    image.finish();

    // Get a centered and fitted bounding box for the screen annotation and image.
    let bbox = get_bbox(&a4_landscape,
                        dynamic.width() as f32,
                        dynamic.height() as f32);

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
    content.transform([(bbox.x2 - bbox.x1),
                       0.0,
                       0.0,
                       (bbox.y2 - bbox.y1),
                       bbox.x1,
                       bbox.y1]);
    content.x_object(image_name);
    content.restore_state();
    let content_data = content.finish();

    // Create a form XObject with the image for the appearance stream in the
    // screen annotation.
    let mut form_xobject = pdf.form_xobject(form_xobject_id, &content_data);
    form_xobject.bbox(bbox);
    form_xobject.resources().x_objects().pair(image_name, image_id);
    form_xobject.finish();

    // Video file
    // Downloaded from the Chromium sources at:
    //     https://github.com/chromium/chromium/blob/main/media/test/data/bear-1280x720.mp4
    let file_name = "examples/bear-1280x720.mp4";

    if embedded {
        // Read video file and add to pdf as embedded file.
        let data = std::fs::read(file_name).unwrap();
        pdf.embedded_file(video_file_id, &data);
    }

    // Create a screen annotation and set the appearance stream.
    let mut annotation = pdf.annotation(annotation_id);
    annotation.subtype(AnnotationType::Screen);
    annotation.rect(bbox);
    annotation.page(page_id);
    annotation.appearance().normal().stream(form_xobject_id);

    // Write a rendition action for the screen annotation.
    let mut action = annotation.action();
    action.action_type(ActionType::Rendition);
    action.operation(0);
    action.annotation(annotation_id);

    // Write a media rendition for the action.
    let mut rendition = action.rendition();
    rendition.rendition_type(RenditionType::Media);

    // Write the media clip data for the media rendition.
    let mut media_clip = rendition.media_clip();
    media_clip.media_clip_type(MediaClipType::Data);
    if embedded {
        media_clip.data_embedded(video_file_id);
    } else {
        // Get the absolute path to the video file.
        let file_path = std::fs::canonicalize(file_name)?;
        // FIXME: Is there a more elegant way to assemble the URL?
        let file_url = &[b"file://", file_path.as_os_str().as_encoded_bytes()].concat();
        media_clip.data_url(Str(file_url));
    }
    media_clip.data_type(Str(b"video/mp4"));
    media_clip.permissions().temp_file(Str(b"TEMPACCESS"));
    media_clip.finish();

    // Add controls for the media player.
    rendition.media_play_params().controls(true);

    // Finish off a few things.
    rendition.finish();
    action.finish();
    annotation.finish();

    // Write the thing to a file.
    std::fs::write("target/video.pdf", pdf.finish())
}
