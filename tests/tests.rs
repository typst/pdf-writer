use pdf_writer::{Name, Null, PdfWriter, Ref, Str};

/// Test that `buf` is the same as the result of concatenating the strings.
macro_rules! test {
    ($buf:expr, $($expected:literal),* $(,)?) => {{
        let buf = $buf;
        let string = std::str::from_utf8(&buf).unwrap();
        assert_eq!(string, concat!($($expected),*));
    }}
}

/// Test how a basic object is written.
macro_rules! test_obj {
    ($basic:expr, $($tts:tt)*) => {{
        let buf = with(|w| w.indirect(Ref::new(1)).obj($basic));
        test!(&buf[8 .. buf.len() - 9], $($tts)*)
    }}
}

/// Return the slice of bytes written during the execution of `f`.
fn with<F>(f: F) -> Vec<u8>
where
    F: FnOnce(&mut PdfWriter),
{
    let mut w = PdfWriter::new(1, 7);
    let start = w.len();
    f(&mut w);
    let end = w.len();
    let buf = w.finish(Ref::new(1));
    buf[start .. end].to_vec()
}

#[test]
fn test_minimal() {
    let w = PdfWriter::new(1, 7);
    test!(
        w.finish(Ref::new(1)),
        "%PDF-1.7\n\n",
        "xref\n0 1\n0000000000 65535 f\r\n",
        "trailer\n",
        "<<\n/Size 1\n/Root 1 0 R\n>>\n",
        "startxref\n10\n%%EOF",
    );
}

#[test]
fn test_basic_objects() {
    test_obj!(true, "true");
    test_obj!(false, "false");
    test_obj!(78, "78");
    test_obj!(4.22, "4.22");
    test_obj!(Str(b"hello"), "(hello)");
    test_obj!(Name(b"Filter"), "/Filter");
    test_obj!(Ref::new(7), "7 0 R");
    test_obj!(Null, "null");
}
