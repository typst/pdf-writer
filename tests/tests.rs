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
macro_rules! test_primitive {
    ($basic:expr, $($tts:tt)*) => {{
        let buf = with(|w| w.indirect(Ref::new(1)).primitive($basic));
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
fn test_primitive_objects() {
    test_primitive!(true, "true");
    test_primitive!(false, "false");
    test_primitive!(78, "78");
    test_primitive!(4.22, "4.22");
    test_primitive!(Str(b"hello"), "(hello)");
    test_primitive!(Name(b"Filter"), "/Filter");
    test_primitive!(Ref::new(7), "7 0 R");
    test_primitive!(Null, "null");
}
