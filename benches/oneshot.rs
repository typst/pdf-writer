use std::io::Write;

use pdf_writer::{Content, PdfWriter, Rect, Ref};

fn bench_alloc() -> Vec<u8> {
    Vec::with_capacity(16)
}

fn bench_static() -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.extend(b"q\n10 i\nQ");
    v
}

fn bench_chunks() -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.extend(b"q\n");
    v.extend(itoa::Buffer::new().format(10).as_bytes());
    v.extend(b" i\n");
    v.extend(b"Q\n");
    v.pop();
    v
}

fn bench_byte_by_byte() -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.push(b'q');
    v.push(b'\n');
    v.extend(itoa::Buffer::new().format(10).as_bytes());
    v.push(b' ');
    v.push(b'i');
    v.push(b'\n');
    v.push(b'Q');
    v.push(b'\n');
    v.pop();
    v
}

fn bench_std_format() -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    write!(v, "q\n{} i\nQ\n", 10).unwrap();
    v
}

fn bench_content() -> Vec<u8> {
    let mut c = Content::with_capacity(16);
    c.save_state();
    c.set_flatness(10);
    c.restore_state();
    c.finish()
}

fn bench_new() -> PdfWriter {
    PdfWriter::new()
}

fn bench_full() -> Vec<u8> {
    let mut w = PdfWriter::new();
    w.catalog(Ref::new(1));
    w.page(Ref::new(2)).media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
    w.stream(Ref::new(3), &b"ABCDEFG"[..]);
    w.finish(Ref::new(1))
}

#[rustfmt::skip]
iai::main!(
    bench_alloc,
    bench_static,
    bench_chunks,
    bench_byte_by_byte,
    bench_std_format,
    bench_content,
    bench_new,
    bench_full,
);
