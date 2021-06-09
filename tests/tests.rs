use pdf_writer::{Filter, IndirectGuard, Name, Null, Obj, PdfWriter, Ref, Str};

/// Test that `buf` is the same as the result of concatenating the strings.
macro_rules! test {
    ($buf:expr, $($expected:literal),* $(,)?) => {{
        let buf = $buf;
        let string = std::str::from_utf8(&buf).unwrap();
        assert_eq!(string, concat!($($expected),*));
    }}
}

/// Test how an object is written.
macro_rules! test_obj {
    (|$obj:ident| $write:expr, $($tts:tt)*) => {{
        test!(slice_obj(|$obj| { $write; }), $($tts)*)
    }}
}

/// Test how a primitive object is written.
macro_rules! test_primitive {
    ($value:expr, $($tts:tt)*) => {
        test_obj!(|obj| obj.primitive($value), $($tts)*);
    }
}

/// Return the slice of bytes written during the execution of `f`.
fn slice<F>(f: F) -> Vec<u8>
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

/// Return the slice of bytes written for an object.
fn slice_obj<F>(f: F) -> Vec<u8>
where
    F: FnOnce(Obj<IndirectGuard>),
{
    let buf = slice(|w| f(w.indirect(Ref::new(1))));
    buf[8 .. buf.len() - 9].to_vec()
}

#[test]
fn test_minimal() {
    let w = PdfWriter::new(1, 7);
    test!(
        w.finish(Ref::new(1)),
        "%PDF-1.7\n\n",
        "xref\n0 1\n0000000000 65535 f\r\n",
        "trailer\n<<\n/Size 1\n/Root 1 0 R\n>>\n",
        "startxref\n10\n%%EOF",
    );
}

#[test]
fn test_xref_free_list_short() {
    let mut w = PdfWriter::new(1, 7);
    w.indirect(Ref::new(1)).primitive(1);
    w.indirect(Ref::new(2)).primitive(2);
    test!(
        w.finish(Ref::new(1)),
        "%PDF-1.7\n\n",
        "1 0 obj\n1\nendobj\n\n",
        "2 0 obj\n2\nendobj\n\n",
        "xref\n",
        "0 3\n",
        "0000000000 65535 f\r\n",
        "0000000010 00000 n\r\n",
        "0000000028 00000 n\r\n",
        "trailer\n",
        "<<\n/Size 3\n/Root 1 0 R\n>>\n",
        "startxref\n46\n%%EOF",
    )
}

#[test]
fn test_xref_free_list_long() {
    let mut w = PdfWriter::new(1, 4);
    w.indirect(Ref::new(1)).primitive(1);
    w.indirect(Ref::new(2)).primitive(2);
    w.indirect(Ref::new(5)).primitive(5);
    test!(
        w.finish(Ref::new(2)),
        "%PDF-1.4\n\n",
        "1 0 obj\n1\nendobj\n\n",
        "2 0 obj\n2\nendobj\n\n",
        "5 0 obj\n5\nendobj\n\n",
        "xref\n",
        "0 6\n",
        "0000000003 65535 f\r\n",
        "0000000010 00000 n\r\n",
        "0000000028 00000 n\r\n",
        "0000000004 00000 f\r\n",
        "0000000000 00000 f\r\n",
        "0000000046 00000 n\r\n",
        "trailer\n",
        "<<\n/Size 6\n/Root 2 0 R\n>>\n",
        "startxref\n64\n%%EOF",
    )
}

#[test]
#[should_panic(expected = "unfinished object")]
#[allow(unused_must_use)]
fn test_object_unused() {
    let mut w = PdfWriter::new(1, 7);
    w.indirect(Ref::new(1));
    w.finish(Ref::new(1));
}

#[test]
fn test_primitive_objects() {
    // Test really simple objects.
    test_primitive!(true, "true");
    test_primitive!(false, "false");
    test_primitive!(78, "78");
    test_primitive!(4.22, "4.22");
    test_primitive!(Ref::new(7), "7 0 R");
    test_primitive!(Null, "null");

    // Test strings.
    test_primitive!(Str(b"Hello, World!"), "(Hello, World!)");
    test_primitive!(Str(b"()"), "<2829>");
    test_primitive!(Str(br"\"), "<5C>");
    test_primitive!(Str(br"\n"), "<5C6E>");

    // Test names.
    test_primitive!(Name(b"Filter"), "/Filter");
    test_primitive!(Name(b"A B"), r"/A#20B");
    test_primitive!(Name(b"~+c"), r"/~+c");
    test_primitive!(Name(b"#"), r"/#23");
    test_primitive!(Name(b"\n"), r"/#0A");
}

#[test]
fn test_arrays() {
    test_obj!(|obj| obj.array(), "[]");
    test_obj!(|obj| obj.array().item(12).item(Null), "[12 null]");
    test_obj!(|obj| obj.array().typed().items(vec![1, 2, 3]), "[1 2 3]");
    test_obj!(
        |obj| {
            let mut array = obj.array();
            array.obj().array().typed().items(vec![1, 2]);
            array.item(3);
        },
        "[[1 2] 3]",
    );
}

#[test]
fn test_dicts() {
    test_obj!(|obj| obj.dict(), "<<\n>>");
    test_obj!(
        |obj| obj.dict().pair(Name(b"Quality"), Name(b"Good")),
        "<<\n/Quality /Good\n>>",
    );
}

#[test]
fn test_streams() {
    let mut w = PdfWriter::new(1, 7);
    w.stream(Ref::new(1), b"Hi there!").filter(Filter::Crypt);
    test!(
        w.finish(Ref::new(1)),
        "%PDF-1.7\n\n",
        "1 0 obj\n",
        "<<\n/Length 9\n/Filter /Crypt\n>>\n",
        "stream\n",
        "Hi there!\n",
        "endstream\n",
        "endobj\n\n",
        "xref\n",
        "0 2\n",
        "0000000000 65535 f\r\n",
        "0000000010 00000 n\r\n",
        "trailer\n",
        "<<\n/Size 2\n/Root 1 0 R\n>>\n",
        "startxref\n84\n%%EOF",
    )
}
