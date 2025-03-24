//! This example shows how you can track PDF limits of your chunks.

use pdf_writer::{Chunk, Content, Limits, Name, Ref};

fn main() {
    let mut limits = Limits::new();

    let mut content = Content::new();
    content.transform([-3.4, 0.0, 0.0, 3.1, 100.0, 100.0]);
    content.line_to(15.0, -26.1);
    let buf = content.finish();
    // This will have the limits:
    // - Max real number: 26.1 (for negative values we use their absolute value)
    // - Max int number 100 (even though above 100.0 is a float number, it will be coerced into an
    //   integer, and thus counts towards the int limit)
    limits.merge(buf.limits());

    let mut chunk = Chunk::new();
    chunk.stream(Ref::new(1), &buf.into_vec());
    chunk.type3_font(Ref::new(2)).name(Name(b"A_long_font_name"));
    // This will update the limit for the maximum name and dictionary length.
    limits.merge(chunk.limits());

    // This is what the final PDF will look like.
    assert_eq!(
        chunk.as_bytes(),
        b"1 0 obj
<<
  /Length 34
>>
stream
-3.4 0 0 3.1 100 100 cm
15 -26.1 l
endstream
endobj

2 0 obj
<<
  /Type /Font
  /Subtype /Type3
  /Name /A_long_font_name
>>
endobj

"
    );

    // And the limits should match, as well!
    assert_eq!(limits.int(), 100);
    assert_eq!(limits.real(), 26.1);
    assert_eq!(limits.name_len(), 16);
    assert_eq!(limits.dict_entries(), 3);
}
